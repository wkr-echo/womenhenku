// Summary Agent
//
// 对文章内容生成 AI 摘要。
// 输入：cleaned_markdown
// 输出：流式文本 → 存入 agent_runs 表
// 行为：串行执行、latest-only 队列、可取消

use crate::agent::client::{AiClient, AiClientError, AiStreamEvent};
use crate::agent::prompt::PromptManager;
use crate::agent::state::AgentSlot;
use crate::db::model::NewAgentRun;
use crate::db::repository::AgentRunRepository;
use crate::db::DbPool;
use std::collections::HashMap;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use tokio::sync::{Mutex, RwLock};

/// Summary Agent 配置
#[derive(Debug, Clone)]
pub struct SummaryConfig {
    pub target_language: String,
    pub detail_level: String,
}

impl Default for SummaryConfig {
    fn default() -> Self {
        Self {
            target_language: "中文".to_string(),
            detail_level: "balanced".to_string(),
        }
    }
}

/// Summary Agent 错误
#[derive(Debug, thiserror::Error)]
pub enum SummaryError {
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
    #[error("状态错误: {0}")]
    State(String),
}

/// 累积流式内容
struct Accumulator {
    full_content: String,
}

/// Summary Agent
pub struct SummaryAgent {
    pool: DbPool,
    client: AiClient,
    prompt_manager: Arc<PromptManager>,
    /// 每个 entry 的 slot 管理
    slots: Arc<RwLock<HashMap<i64, AgentSlot>>>,
    /// 取消标志：entry_id → Arc<AtomicBool>
    cancel_flags: Arc<Mutex<HashMap<i64, Arc<AtomicBool>>>>,
}

impl SummaryAgent {
    pub fn new(pool: DbPool, prompt_manager: Arc<PromptManager>) -> Self {
        Self {
            pool,
            client: AiClient::new(),
            prompt_manager,
            slots: Arc::new(RwLock::new(HashMap::new())),
            cancel_flags: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    /// 为指定文章生成摘要（异步入口）
    /// 通过 on_event 回调将流式事件推送给前端
    #[allow(clippy::too_many_arguments)]
    pub async fn generate_summary(
        &self,
        entry_id: i64,
        config: SummaryConfig,
        provider_id: i64,
        base_url: &str,
        api_key: &str,
        model: &str,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), SummaryError> {
        let run_repo = AgentRunRepository::new(self.pool.clone());

        // 1. 创建 AgentRun 记录
        let run = run_repo
            .create(&NewAgentRun {
                entry_id,
                provider_id,
                task_kind: "summary".to_string(),
                target_language: config.target_language.clone(),
                detail_level: Some(config.detail_level.clone()),
            })
            .map_err(|e| SummaryError::Database(e.to_string()))?;

        let run_id = run.id;

        // 2. 尝试获取 slot（串行执行）
        {
            let mut slots = self.slots.write().await;
            let slot = slots.entry(entry_id).or_insert_with(AgentSlot::new);
            if !slot.try_acquire(run_id) {
                on_event(AiStreamEvent {
                    entry_id: 0,
                    task_id: run_id,
                    content: String::new(),
                    is_done: true,
                    agent_type: "summary".to_string(),
                    error: Some("已有任务在进行中，请等待完成".to_string()),
                });
                return Ok(());
            }
        }

        // 3. 标记为 running
        run_repo
            .mark_started(run_id)
            .map_err(|e| SummaryError::Database(e.to_string()))?;

        // 4. 注册取消标志
        let cancel_flag = Arc::new(AtomicBool::new(false));
        {
            let mut flags = self.cancel_flags.lock().await;
            flags.insert(entry_id, cancel_flag.clone());
        }

        // 5. 获取文章内容
        let content = match self.get_entry_markdown(entry_id) {
            Ok(c) => c,
            Err(e) => {
                Self::cleanup(&self.slots, &self.cancel_flags, entry_id, run_id).await;
                return Err(SummaryError::Database(e));
            }
        };

        if content.trim().is_empty() {
            Self::cleanup(&self.slots, &self.cancel_flags, entry_id, run_id).await;
            return Err(SummaryError::EmptyContent);
        }

        // 6. 检查是否被取消
        if cancel_flag.load(Ordering::SeqCst) {
            Self::handle_cancel(&run_repo, run_id, &on_event).await;
            Self::cleanup(&self.slots, &self.cancel_flags, entry_id, run_id).await;
            return Err(SummaryError::Cancelled);
        }

        // 7. 渲染 Prompt
        let mut vars = std::collections::HashMap::new();
        vars.insert("target_language".to_string(), config.target_language.clone());
        vars.insert("detail_level".to_string(), config.detail_level.clone());
        vars.insert("content".to_string(), content);

        let (system_prompt, user_prompt) = self
            .prompt_manager
            .render("summary", &vars)
            .unwrap_or_else(|_| {
                let (sys, user) = crate::agent::prompt::builtin_summary_prompt();
                let user = user
                    .replace("{{target_language}}", &config.target_language)
                    .replace("{{detail_level}}", &config.detail_level)
                    .replace("{{content}}", &vars["content"]);
                (sys, user)
            });

        // 8. 发送流式请求
        let acc = Arc::new(std::sync::Mutex::new(Accumulator {
            full_content: String::new(),
        }));
        let acc_clone = acc.clone();

        let result = self
            .client
            .stream_chat(
                base_url,
                api_key,
                model,
                &system_prompt,
                &user_prompt,
                |delta| {
                    let mut acc = acc_clone.lock().unwrap();
                    acc.full_content.push_str(delta);
                    on_event(AiStreamEvent {
                    entry_id,
                        task_id: run_id,
                        content: delta.to_string(),
                        is_done: false,
                        agent_type: "summary".to_string(),
                        error: None,
                    });
                },
                |_usage, _error| {},
            )
            .await;

        // 9. 处理结果
        match result {
            Ok(()) => {
                let final_text = {
                    let acc_inner = acc.lock().unwrap();
                    acc_inner.full_content.clone()
                };

                run_repo
                    .mark_completed(run_id, &final_text, None, None)
                    .map_err(|e| SummaryError::Database(e.to_string()))?;

                on_event(AiStreamEvent {
                    entry_id: 0,
                    task_id: run_id,
                    content: String::new(),
                    is_done: true,
                    agent_type: "summary".to_string(),
                    error: None,
                });
            }
            Err(e) => {
                run_repo
                    .mark_failed(run_id, &e.to_string())
                    .map_err(|db_err| SummaryError::Database(db_err.to_string()))?;

                on_event(AiStreamEvent {
                    entry_id: 0,
                    task_id: run_id,
                    content: String::new(),
                    is_done: true,
                    agent_type: "summary".to_string(),
                    error: Some(e.to_string()),
                });
            }
        }

        Self::cleanup(&self.slots, &self.cancel_flags, entry_id, run_id).await;
        Ok(())
    }

    /// 取消指定文章的摘要任务
    pub async fn cancel_summary(&self, entry_id: i64) -> Result<(), SummaryError> {
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

    // ---- 内部辅助方法 ----

    async fn cleanup(
        slots: &Arc<RwLock<HashMap<i64, AgentSlot>>>,
        cancel_flags: &Arc<Mutex<HashMap<i64, Arc<AtomicBool>>>>,
        entry_id: i64,
        run_id: i64,
    ) {
        let mut s = slots.write().await;
        if let Some(slot) = s.get_mut(&entry_id) {
            if slot.active == Some(run_id) {
                slot.complete();
            }
        }
        let mut f = cancel_flags.lock().await;
        f.remove(&entry_id);
    }

    async fn handle_cancel(
        run_repo: &AgentRunRepository,
        run_id: i64,
        on_event: &impl Fn(AiStreamEvent),
    ) {
        let _ = run_repo.mark_cancelled(run_id);
        on_event(AiStreamEvent {
                    entry_id: 0,
            task_id: run_id,
            content: String::new(),
            is_done: true,
            agent_type: "summary".to_string(),
            error: Some("用户取消".to_string()),
        });
    }

    /// 获取文章的 cleaned_markdown（优先）或降级内容
    fn get_entry_markdown(&self, entry_id: i64) -> Result<String, String> {
        let repo = crate::db::repository::ContentRepository::new(self.pool.clone());
        let content = repo
            .find_by_entry_id(entry_id)
            .map_err(|e| format!("读取文章内容失败: {}", e))?
            .ok_or_else(|| format!("Entry_id={} 的文章内容不存在", entry_id))?;

        if let Some(ref md) = content.cleaned_markdown {
            if !md.is_empty() {
                return Ok(md.clone());
            }
        }
        if let Some(ref html) = content.cleaned_html {
            if !html.is_empty() {
                return Ok(crate::reader::pipeline::to_markdown(html));
            }
        }
        if !content.raw_html.is_empty() {
            return Ok(crate::reader::pipeline::to_markdown(&content.raw_html));
        }
        Err("文章内容为空".to_string())
    }
}
