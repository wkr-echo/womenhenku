// Translation Agent
//
// 段落级双语翻译。按 <p>, <ul>, <ol> 切分 cleaned_html，
// 有界并发（默认 3），支持重试失败段落、清除翻译。
// 结果持久化到 agent_runs 表。

use crate::agent::client::{AiClient, AiClientError, AiStreamEvent};
use crate::agent::prompt::PromptManager;
use crate::agent::state::AgentSlot;
use crate::db::model::NewAgentRun;
use crate::db::repository::AgentRunRepository;
use crate::db::DbPool;
use scraper::ElementRef;
use scraper::{Html, Selector};
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock, Semaphore};

/// 翻译段落
#[derive(Debug, Clone)]
pub struct TranslationSegment {
    pub index: usize,
    pub source: String,
    pub translated: Option<String>,
    pub status: SegmentStatus,
}

#[derive(Debug, Clone, PartialEq)]
pub enum SegmentStatus {
    Pending,
    Success,
    Failed(String),
}

/// 解析后的翻译段落结果（从数据库 output_text 还原）
#[derive(Debug, Clone)]
pub struct ParsedSegment {
    pub index: usize,
    pub source: String,
    pub translation: Option<String>,
    pub is_failed: bool,
}

/// Translation Agent 配置
#[derive(Debug, Clone)]
pub struct TranslationConfig {
    pub target_language: String,
    pub concurrency_degree: usize, // 1~5
}

impl Default for TranslationConfig {
    fn default() -> Self {
        Self {
            target_language: "中文".to_string(),
            concurrency_degree: 3,
        }
    }
}

/// Translation Agent 错误
#[derive(Debug, thiserror::Error)]
pub enum TranslationError {
    #[error("AI 客户端错误: {0}")]
    Client(#[from] AiClientError),
    #[error("数据库错误: {0}")]
    Database(String),
    #[error("Provider 未配置")]
    NoProvider,
    #[error("文章内容为空")]
    EmptyContent,
    #[error("任务已取消")]
    Cancelled,
    #[error("段落切分失败: {0}")]
    SegmentParse(String),
    #[error("没有失败段落需要重试")]
    NoFailedSegments,
    #[error("未找到之前的翻译结果")]
    NoPreviousTranslation,
}

/// Translation Agent
pub struct TranslationAgent {
    pool: DbPool,
    #[allow(dead_code)]
    client: AiClient,
    prompt_manager: Arc<PromptManager>,
    /// 每个 entry 的 slot
    slots: Arc<RwLock<HashMap<i64, AgentSlot>>>,
    /// 取消标志
    cancel_flags: Arc<Mutex<HashMap<i64, Arc<AtomicBool>>>>,
}

impl TranslationAgent {
    pub fn new(pool: DbPool, prompt_manager: Arc<PromptManager>) -> Self {
        Self {
            pool,
            client: AiClient::new(),
            prompt_manager,
            slots: Arc::new(RwLock::new(HashMap::new())),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 对文章进行段落级双语翻译
    #[allow(clippy::too_many_arguments)]
    pub async fn translate_entry(
        &self,
        entry_id: i64,
        config: TranslationConfig,
        provider_id: i64,
        base_url: &str,
        api_key: &str,
        model: &str,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), TranslationError> {
        // 立即包装为 Arc<dyn Fn>，方便跨线程共享
        let on_event: Arc<dyn Fn(AiStreamEvent) + Send + Sync> = Arc::new(on_event);

        let run_repo = AgentRunRepository::new(self.pool.clone());

        // 1. 创建 AgentRun
        let run = run_repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "translation".to_string(),
                target_language: config.target_language.clone(),
                detail_level: None,
            })
            .map_err(|e| TranslationError::Database(e.to_string()))?;

        let run_id = run.id;

        // 2. 获取 slot
        {
            let mut slots = self.slots.write().await;
            let slot = slots.entry(entry_id).or_insert_with(AgentSlot::new);
            if !slot.try_acquire(run_id) {
                (on_event)(AiStreamEvent {
                    entry_id: 0,
                    task_id: run_id,
                    content: String::new(),
                    is_done: true,
                    agent_type: "translation".to_string(),
                    error: Some("已有翻译任务在进行中".to_string()),
                });
                return Ok(());
            }
        }

        // 3. 标记 running
        run_repo
            .mark_started(run_id)
            .map_err(|e| TranslationError::Database(e.to_string()))?;

        // 4. 取消标志
        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(entry_id, cancel_flag.clone());
        }

        // 5. 获取内容并切分段落
        let segments = match self.get_entry_segments(entry_id) {
            Ok(s) => s,
            Err(e) => {
                Self::cleanup(
                    &self.slots,
                    &self.cancel_flags,
                    &run_repo,
                    run_id,
                    entry_id,
                    &e,
                    &on_event,
                )
                .await;
                return Err(TranslationError::Database(e));
            }
        };

        if segments.is_empty() {
            Self::cleanup(
                &self.slots,
                &self.cancel_flags,
                &run_repo,
                run_id,
                entry_id,
                "文章内容为空",
                &on_event,
            )
            .await;
            return Err(TranslationError::EmptyContent);
        }

        // 6. 有界并发翻译
        let semaphore = Arc::new(Semaphore::new(config.concurrency_degree));
        let total_segments = segments.len();
        let results: Arc<Mutex<Vec<TranslationSegment>>> = Arc::new(Mutex::new(Vec::new()));
        let mut handles = Vec::new();
        let agent = Arc::new(TranslationAgentInner {
            client: AiClient::new(),
            prompt_manager: self.prompt_manager.clone(),
        });

        for (i, seg_text) in segments.iter().enumerate() {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }

            let permit = semaphore
                .clone()
                .acquire_owned()
                .await
                .unwrap_or_else(|_| {
                    panic!("Semaphore closed");
                });

            let cancel = cancel_flag.clone();
            let results_inner = results.clone();
            let seg = seg_text.clone();
            let agent = agent.clone();
            let lang = config.target_language.clone();
            let url = base_url.to_string();
            let key = api_key.to_string();
            let mdl = model.to_string();
            let on_event_clone = on_event.clone();
            let run_id_inner = run_id;

            let handle = tokio::spawn(async move {
                let translated = agent
                    .translate_segment(
                        &url, &key, &mdl, &lang, &seg, run_id_inner, i, total_segments,
                        entry_id, on_event_clone, &cancel,
                    )
                    .await;

                let result = match translated {
                    Ok(text) => TranslationSegment {
                        index: i,
                        source: seg,
                        translated: Some(text),
                        status: SegmentStatus::Success,
                    },
                    Err(e) => TranslationSegment {
                        index: i,
                        source: seg,
                        translated: None,
                        status: SegmentStatus::Failed(e),
                    },
                };
                let mut r = results_inner.lock().await;
                r.push(result);
                drop(permit);
            });
            handles.push(handle);
        }

        // 等待所有段落翻译完成
        for h in handles {
            let _ = h.await;
        }

        // 7. 收集结果
        let results_vec = {
            let r = results.lock().await;
            r.clone()
        };

        // 统计成功/失败
        let success_count = results_vec
            .iter()
            .filter(|s| s.status == SegmentStatus::Success)
            .count();
        let fail_count = results_vec
            .iter()
            .filter(|s| matches!(s.status, SegmentStatus::Failed(_)))
            .count();

        // 构建输出文本（段落编号 + 原文 + 译文）
        let mut output = String::new();
        for seg in &results_vec {
            if let Some(ref trans) = seg.translated {
                output.push_str(&format!(
                    "[{}]\n原文: {}\n译文: {}\n\n",
                    seg.index + 1,
                    seg.source,
                    trans
                ));
            } else {
                output.push_str(&format!(
                    "[{}] (翻译失败)\n原文: {}\n\n",
                    seg.index + 1,
                    seg.source
                ));
            }
        }

        if fail_count == 0 {
            run_repo
                .mark_completed(run_id, &output, None, None)
                .map_err(|e| TranslationError::Database(e.to_string()))?;

            (on_event)(AiStreamEvent {
                    entry_id: 0,
                task_id: run_id,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: None,
            });
        } else {
            let err_msg = format!(
                "翻译完成，{} 段成功，{} 段失败",
                success_count, fail_count
            );
            run_repo
                .mark_completed(run_id, &output, None, None)
                .map_err(|e| TranslationError::Database(e.to_string()))?;

            (on_event)(AiStreamEvent {
                    entry_id: 0,
                task_id: run_id,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: Some(err_msg),
            });
        }

        Self::cleanup(
            &self.slots,
            &self.cancel_flags,
            &run_repo,
            run_id,
            entry_id,
            "",
            &on_event,
        )
        .await;
        Ok(())
    }

    /// 取消翻译任务
    pub async fn cancel_translation(&self, entry_id: i64) -> Result<(), TranslationError> {
        let flags = self.cancel_flags.lock().await;
        if let Some(flag) = flags.get(&entry_id) {
            flag.store(true, Ordering::SeqCst);
        }
        drop(flags);
        let mut slots = self.slots.write().await;
        if let Some(slot) = slots.get_mut(&entry_id) {
            slot.cancel();
        }
        Ok(())
    }

    /// 重试翻译中失败的段落
    /// 1. 从数据库读取上次翻译结果
    /// 2. 解析出失败段落的索引
    /// 3. 重新获取原文段落
    /// 4. 只重新翻译失败的段落
    /// 5. 合并结果并保存为新 AgentRun
    #[allow(clippy::too_many_arguments)]
    pub async fn retry_failed_segments(
        &self,
        entry_id: i64,
        config: TranslationConfig,
        provider_id: i64,
        base_url: &str,
        api_key: &str,
        model: &str,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), TranslationError> {
        let on_event: Arc<dyn Fn(AiStreamEvent) + Send + Sync> = Arc::new(on_event);
        let run_repo = AgentRunRepository::new(self.pool.clone());

        // 1. 读取上次翻译结果
        let previous_run = run_repo
            .find_latest_by_entry_and_task(entry_id, "translation")
            .map_err(|e| TranslationError::Database(e.to_string()))?
            .ok_or(TranslationError::NoPreviousTranslation)?;

        let output_text = previous_run
            .output_text
            .ok_or(TranslationError::NoPreviousTranslation)?;

        // 2. 解析出成功/失败的段落
        let parsed_segments = parse_translation_output(&output_text);
        let failed_indices: Vec<usize> = parsed_segments
            .iter()
            .filter(|s| s.is_failed)
            .map(|s| s.index)
            .collect();

        if failed_indices.is_empty() {
            (on_event)(AiStreamEvent {
                entry_id: 0,
                task_id: 0,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: Some("没有失败的段落需要重试".to_string()),
            });
            return Err(TranslationError::NoFailedSegments);
        }

        // 3. 获取原文段落
        let all_segments = self
            .get_entry_segments(entry_id)
            .map_err(TranslationError::Database)?;

        // 4. 创建新的 AgentRun
        let run = run_repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "translation".to_string(),
                target_language: config.target_language.clone(),
                detail_level: None,
            })
            .map_err(|e| TranslationError::Database(e.to_string()))?;

        let run_id = run.id;

        // 5. 获取 slot
        {
            let mut slots = self.slots.write().await;
            let slot = slots.entry(entry_id).or_insert_with(AgentSlot::new);
            if !slot.try_acquire(run_id) {
                (on_event)(AiStreamEvent {
                    entry_id: 0,
                    task_id: run_id,
                    content: String::new(),
                    is_done: true,
                    agent_type: "translation".to_string(),
                    error: Some("已有翻译任务在进行中".to_string()),
                });
                return Ok(());
            }
        }

        // 6. 标记 running
        run_repo
            .mark_started(run_id)
            .map_err(|e| TranslationError::Database(e.to_string()))?;

        // 7. 取消标志
        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(entry_id, cancel_flag.clone());
        }

        // 8. 只对失败的段落进行重试
        let total_failed = failed_indices.len();
        let semaphore = Arc::new(Semaphore::new(config.concurrency_degree.min(total_failed)));
        let retry_results: Arc<Mutex<HashMap<usize, String>>> = Arc::new(Mutex::new(HashMap::new()));
        let mut handles = Vec::new();
        let agent = Arc::new(TranslationAgentInner {
            client: AiClient::new(),
            prompt_manager: self.prompt_manager.clone(),
        });

        for (retry_idx, seg_idx) in failed_indices.iter().enumerate() {
            if cancel_flag.load(Ordering::SeqCst) {
                break;
            }

            let permit = semaphore.clone().acquire_owned().await.unwrap_or_else(|_| {
                panic!("Semaphore closed");
            });

            let cancel = cancel_flag.clone();
            let retry_results_inner = retry_results.clone();
            let seg_text = all_segments[*seg_idx].clone();
            let agent = agent.clone();
            let lang = config.target_language.clone();
            let url = base_url.to_string();
            let key = api_key.to_string();
            let mdl = model.to_string();
            let on_event_clone = on_event.clone();
            let run_id_inner = run_id;
            let cur_seg_idx = *seg_idx;

            let handle = tokio::spawn(async move {
                let translated = agent
                    .translate_segment(
                        &url, &key, &mdl, &lang, &seg_text, run_id_inner,
                        retry_idx, total_failed, entry_id, on_event_clone, &cancel,
                    )
                    .await;

                if let Ok(text) = translated {
                    let mut r = retry_results_inner.lock().await;
                    r.insert(cur_seg_idx, text);
                }
                drop(permit);
            });
            handles.push(handle);
        }

        // 等待所有重试完成
        for h in handles {
            let _ = h.await;
        }

        // 9. 合并结果：已有成功的 + 重试成功的
        let retry_map = {
            let r = retry_results.lock().await;
            r.clone()
        };

        let mut output = String::new();
        let mut retry_success = 0u32;
        let mut retry_fail = 0u32;

        for seg in &parsed_segments {
            if seg.is_failed {
                if let Some(new_translation) = retry_map.get(&seg.index) {
                    output.push_str(&format!(
                        "[{}]\n原文: {}\n译文: {}\n\n",
                        seg.index + 1,
                        seg.source,
                        new_translation
                    ));
                    retry_success += 1;
                } else {
                    output.push_str(&format!(
                        "[{}] (翻译失败)\n原文: {}\n\n",
                        seg.index + 1,
                        seg.source
                    ));
                    retry_fail += 1;
                }
            } else if let Some(ref trans) = seg.translation {
                output.push_str(&format!(
                    "[{}]\n原文: {}\n译文: {}\n\n",
                    seg.index + 1,
                    seg.source,
                    trans
                ));
            }
        }

        // 10. 保存结果
        run_repo
            .mark_completed(run_id, &output, None, None)
            .map_err(|e| TranslationError::Database(e.to_string()))?;

        if retry_fail == 0 {
            (on_event)(AiStreamEvent {
                entry_id: 0,
                task_id: run_id,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: None,
            });
        } else {
            (on_event)(AiStreamEvent {
                entry_id: 0,
                task_id: run_id,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: Some(format!(
                    "重试完成，{} 段成功，{} 段失败",
                    retry_success, retry_fail
                )),
            });
        }

        // 11. 清理
        let mut s = self.slots.write().await;
        if let Some(slot) = s.get_mut(&entry_id) {
            if slot.active == Some(run_id) {
                slot.complete();
            }
        }
        let mut f = self.cancel_flags.lock().await;
        f.remove(&entry_id);

        Ok(())
    }

    // ---- 内部辅助 ----

    async fn cleanup(
        slots: &Arc<RwLock<HashMap<i64, AgentSlot>>>,
        cancel_flags: &Arc<Mutex<HashMap<i64, Arc<AtomicBool>>>>,
        run_repo: &AgentRunRepository,
        run_id: i64,
        entry_id: i64,
        error_msg: &str,
        on_event: &Arc<dyn Fn(AiStreamEvent) + Send + Sync>,
    ) {
        if !error_msg.is_empty() {
            let _ = run_repo.mark_failed(run_id, error_msg);
            (on_event)(AiStreamEvent {
                    entry_id: 0,
                task_id: run_id,
                content: String::new(),
                is_done: true,
                agent_type: "translation".to_string(),
                error: Some(error_msg.to_string()),
            });
        }
        let mut s = slots.write().await;
        if let Some(slot) = s.get_mut(&entry_id) {
            if slot.active == Some(run_id) {
                slot.complete();
            }
        }
        let mut f = cancel_flags.lock().await;
        f.remove(&entry_id);
    }

    /// 获取文章内容并按段落切分
    /// 首段为合成的标题+作者信息，用于 AI 翻译时保持上下文对齐
    fn get_entry_segments(&self, entry_id: i64) -> Result<Vec<String>, String> {
        let content_repo = crate::db::repository::ContentRepository::new(self.pool.clone());
        let entry_repo = crate::db::repository::EntryRepository::new(self.pool.clone());

        // 获取文章元数据（标题、作者）
        let entry = entry_repo
            .find_by_id(entry_id)
            .map_err(|e| format!("读取文章信息失败: {}", e))?
            .ok_or_else(|| format!("Entry_id={} 的文章不存在", entry_id))?;

        // 获取文章内容
        let content = content_repo
            .find_by_entry_id(entry_id)
            .map_err(|e| format!("读取文章内容失败: {}", e))?
            .ok_or_else(|| format!("Entry_id={} 的文章内容不存在", entry_id))?;

        // 优先使用 cleaned_html
        let html = content
            .cleaned_html
            .filter(|h| !h.is_empty())
            .or_else(|| {
                if !content.raw_html.is_empty() {
                    Some(content.raw_html.clone())
                } else {
                    None
                }
            })
            .ok_or_else(|| "文章内容为空".to_string())?;

        let mut segments = split_html_into_segments(&html);

        // 合成前置段落：标题 + 作者，帮助 AI 理解文章上下文
        let mut preface = String::new();
        if !entry.title.is_empty() {
            preface.push_str(&format!("标题: {}", entry.title));
        }
        if !entry.author.is_empty() {
            if !preface.is_empty() {
                preface.push('\n');
            }
            preface.push_str(&format!("作者: {}", entry.author));
        }
        if !preface.is_empty() {
            segments.insert(0, preface);
        }

        Ok(segments)
    }
}

/// 内部翻译 agent（Arc 包裹以在 tokio::spawn 中共享）
struct TranslationAgentInner {
    client: AiClient,
    prompt_manager: Arc<PromptManager>,
}

impl TranslationAgentInner {
     #[allow(clippy::too_many_arguments)]
    async fn translate_segment(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
        target_language: &str,
        segment: &str,
        run_id: i64,
        seg_index: usize,
        total: usize,
        entry_id: i64,
        on_event: Arc<dyn Fn(AiStreamEvent) + Send + Sync>,
        _cancel: &AtomicBool,
    ) -> Result<String, String> {
        let mut vars = HashMap::new();
        vars.insert("target_language".to_string(), target_language.to_string());
        vars.insert("content".to_string(), segment.to_string());

        let (system_prompt, user_prompt) = self
            .prompt_manager
            .render("translation", &vars)
            .unwrap_or_else(|_| {
                let (sys, user) = crate::agent::prompt::builtin_translation_prompt();
                let user = user
                    .replace("{{target_language}}", target_language)
                    .replace("{{content}}", segment);
                (sys, user)
            });

        let result_text = Arc::new(std::sync::Mutex::new(String::new()));
        let result_clone = result_text.clone();

        let result = self
            .client
            .stream_chat(
                base_url,
                api_key,
                model,
                &system_prompt,
                &user_prompt,
                |delta| {
                    let mut text = result_clone.lock().unwrap();
                    text.push_str(delta);
                    on_event(AiStreamEvent {
                    entry_id,
                        task_id: run_id,
                        content: format!(
                            "[{}/{}] {}",
                            seg_index + 1,
                            total,
                            delta
                        ),
                        is_done: false,
                        agent_type: "translation".to_string(),
                        error: None,
                    });
                },
                |_usage, _error| {},
            )
            .await;

        match result {
            Ok(()) => {
                let text = result_text.lock().unwrap();
                Ok(text.clone())
            }
            Err(e) => Err(format!("段落 {} 翻译失败: {}", seg_index + 1, e)),
        }
    }
}

/// 提取元素的纯文本内容（递归遍历子节点）
fn element_text(elem: &ElementRef) -> String {
    let mut text = String::new();
    for child in elem.children() {
        match child.value() {
            scraper::node::Node::Text(t) => text.push_str(&t.text),
            _ => {
                if let Some(child_elem) = ElementRef::wrap(child) {
                    let child_text = element_text(&child_elem);
                    if !child_text.is_empty() {
                        if !text.is_empty() && !text.ends_with(' ') {
                            text.push(' ');
                        }
                        text.push_str(&child_text);
                    }
                }
            }
        }
    }
    text
}

/// 将 HTML 按 <p>, <ul>, <ol> 切分为段落列表
fn split_html_into_segments(html: &str) -> Vec<String> {
    let document = Html::parse_fragment(html);
    let mut segments = Vec::new();

    // 选择器：p, ul, ol
    let selectors = [
        Selector::parse("p").unwrap(),
        Selector::parse("ul").unwrap(),
        Selector::parse("ol").unwrap(),
        Selector::parse("h1").unwrap(),
        Selector::parse("h2").unwrap(),
        Selector::parse("h3").unwrap(),
        Selector::parse("h4").unwrap(),
        Selector::parse("h5").unwrap(),
        Selector::parse("h6").unwrap(),
        Selector::parse("blockquote").unwrap(),
        Selector::parse("pre").unwrap(),
    ];

    for selector in &selectors {
        for element in document.select(selector) {
            let segment_text = element_text(&element);
            let trimmed = segment_text.trim().to_string();
            if !trimmed.is_empty() {
                segments.push(trimmed);
            }
        }
    }

    // 如果切分结果为空，将整个 html 作为一段
    if segments.is_empty() {
        let root = document.root_element();
        let full_text = element_text(&root);
        let trimmed = full_text.trim().to_string();
        if !trimmed.is_empty() {
            segments.push(trimmed);
        }
    }

    segments
}

/// 从翻译输出文本中解析出段落结果
/// 格式: [N]\n原文: xxx\n译文: xxx\n\n 或 [N] (翻译失败)\n原文: xxx\n\n
pub fn parse_translation_output(text: &str) -> Vec<ParsedSegment> {
    let mut segments = Vec::new();
    let blocks: Vec<&str> = text.split("\n\n").collect();

    for block in &blocks {
        let block = block.trim();
        if block.is_empty() {
            continue;
        }

        let lines: Vec<&str> = block.lines().collect();
        if lines.is_empty() {
            continue;
        }

        // 解析行号: [1] 或 [1] (翻译失败)
        let first_line = lines[0];
        let index_match = first_line.trim_start_matches('[').split([']', ' ']).next();

        let index = match index_match.and_then(|s| s.parse::<usize>().ok()) {
            Some(i) => i.saturating_sub(1), // 转为 0-based
            None => continue,
        };

        let is_failed = first_line.contains("翻译失败");

        let mut source = String::new();
        let mut translation = None;

        for line in &lines {
            if let Some(src) = line.strip_prefix("原文: ") {
                source = src.to_string();
            } else if let Some(tgt) = line.strip_prefix("译文: ") {
                translation = Some(tgt.to_string());
            }
        }

        segments.push(ParsedSegment {
            index,
            source,
            translation,
            is_failed,
        });
    }

    segments
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_split_html_into_segments_basic() {
        let html = r#"<p>第一段内容</p><p>第二段内容</p><ul><li>列表项1</li><li>列表项2</li></ul>"#;
        let segments = split_html_into_segments(html);
        assert_eq!(segments.len(), 3);
        assert!(segments[0].contains("第一段"));
        assert!(segments[1].contains("第二段"));
        assert!(segments[2].contains("列表项1"));
    }

    #[test]
    fn test_split_html_empty() {
        let html = "<div></div>";
        let segments = split_html_into_segments(html);
        // 空内容应返回一段空文本或空列表
        assert!(segments.is_empty() || segments[0].is_empty());
    }

    #[test]
    fn test_split_html_headings() {
        let html = "<h1>标题</h1><p>正文</p><h2>二级标题</h2>";
        let segments = split_html_into_segments(html);
        assert_eq!(segments.len(), 3);
    }

    #[test]
    fn test_parse_translation_output_all_success() {
        let text = "[1]\n原文: Hello\n译文: 你好\n\n[2]\n原文: World\n译文: 世界\n\n";
        let segments = parse_translation_output(text);
        assert_eq!(segments.len(), 2);
        assert!(!segments[0].is_failed);
        assert_eq!(segments[0].source, "Hello");
        assert_eq!(segments[0].translation.as_deref(), Some("你好"));
        assert!(!segments[1].is_failed);
    }

    #[test]
    fn test_parse_translation_output_with_failures() {
        let text = "[1]\n原文: Hello\n译文: 你好\n\n[2] (翻译失败)\n原文: World\n\n[3]\n原文: Foo\n译文: 条\n\n";
        let segments = parse_translation_output(text);
        assert_eq!(segments.len(), 3);
        assert!(!segments[0].is_failed);
        assert!(segments[1].is_failed);
        assert_eq!(segments[1].source, "World");
        assert!(segments[1].translation.is_none());
        assert!(!segments[2].is_failed);
    }

    #[test]
    fn test_parse_translation_output_empty() {
        let segments = parse_translation_output("");
        assert!(segments.is_empty());
    }
}
