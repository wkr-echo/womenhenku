// Agent Service 层
//
// 对外暴露的统一入口。Tauri Command 通过此 service 调用 Agent 功能。
// 管理 Provider 查找、Agent 实例创建和事件分发。

use crate::agent::client::{AiClient, AiStreamEvent};
use crate::agent::prompt::PromptManager;
use crate::agent::state::AgentSlot;
use crate::agent::summary::{SummaryAgent, SummaryConfig, SummaryError};
use crate::agent::translation::{TranslationAgent, TranslationConfig, TranslationError};
use crate::db::repository::ProviderRepository;
use crate::db::DbPool;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// Agent Service 错误
#[derive(Debug, thiserror::Error)]
pub enum AgentServiceError {
    #[error("Summary 错误: {0}")]
    Summary(#[from] SummaryError),
    #[error("Translation 错误: {0}")]
    Translation(#[from] TranslationError),
    #[error("Provider 未找到")]
    ProviderNotFound,
    #[error("没有默认 Provider")]
    NoDefaultProvider,
    #[error("数据库错误: {0}")]
    Database(String),
}

/// Agent Service
pub struct AgentService {
    pool: DbPool,
    summary_agent: SummaryAgent,
    translation_agent: TranslationAgent,
    ai_client: AiClient,
    /// Slot 共享管理
    slots: Arc<RwLock<HashMap<(i64, String), AgentSlot>>>, // key: (entry_id, task_kind)
}

impl AgentService {
    pub fn new(pool: DbPool, prompt_manager: Arc<PromptManager>) -> Self {
        Self {
            summary_agent: SummaryAgent::new(pool.clone(), prompt_manager.clone()),
            translation_agent: TranslationAgent::new(pool.clone(), prompt_manager),
            ai_client: AiClient::new(),
            pool,
            slots: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 获取默认 Provider 的连接信息
    pub fn get_default_provider(&self) -> Result<(i64, String, String, String), AgentServiceError> {
        let repo = ProviderRepository::new(self.pool.clone());
        let provider = repo
            .find_default()
            .map_err(|e| AgentServiceError::Database(e.to_string()))?
            .ok_or(AgentServiceError::NoDefaultProvider)?;

        // 获取默认模型
        let models = repo
            .list_models(provider.id)
            .map_err(|e| AgentServiceError::Database(e.to_string()))?;

        let default_model = models
            .iter()
            .find(|m| m.is_default)
            .map(|m| m.model_name.clone())
            .or_else(|| models.first().map(|m| m.model_name.clone()))
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        Ok((
            provider.id,
            provider.base_url,
            provider.api_key_ref, // 简化：实际应解密
            default_model,
        ))
    }

    /// 获取指定 ID 的 Provider
    pub fn get_provider_by_id(
        &self,
        provider_id: i64,
    ) -> Result<(String, String, String), AgentServiceError> {
        let repo = ProviderRepository::new(self.pool.clone());
        let provider = repo
            .find_by_id(provider_id)
            .map_err(|e| AgentServiceError::Database(e.to_string()))?
            .ok_or(AgentServiceError::ProviderNotFound)?;

        let models = repo
            .list_models(provider_id)
            .map_err(|e| AgentServiceError::Database(e.to_string()))?;

        let default_model = models
            .iter()
            .find(|m| m.is_default)
            .map(|m| m.model_name.clone())
            .or_else(|| models.first().map(|m| m.model_name.clone()))
            .unwrap_or_else(|| "gpt-3.5-turbo".to_string());

        Ok((
            provider.base_url,
            provider.api_key_ref,
            default_model,
        ))
    }

    /// 生成摘要（用默认 Provider）
    /// 先查数据库缓存，若已有成功结果则直接返回，避免重复调用 API
    pub async fn generate_summary(
        &self,
        entry_id: i64,
        target_language: &str,
        detail_level: &str,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), AgentServiceError> {
        if let Some(text) = self
            .get_latest_summary_text(entry_id)
            .map_err(|e| AgentServiceError::Database(e))?
        {
            if !text.is_empty() {
                on_event(AiStreamEvent {
                    task_id: 0,
                    content: String::new(),
                    is_done: true,
                    agent_type: "summary".to_string(),
                    error: None,
                });
                return Ok(());
            }
        }

        let (provider_id, base_url, api_key, model) = self.get_default_provider()?;

        let config = SummaryConfig {
            target_language: target_language.to_string(),
            detail_level: detail_level.to_string(),
        };

        self.summary_agent
            .generate_summary(
                entry_id,
                config,
                provider_id,
                &base_url,
                &api_key,
                &model,
                on_event,
            )
            .await?;

        Ok(())
    }

    /// 生成摘要（指定配置）
    /// 先查数据库缓存，若已有成功结果则直接返回，避免重复调用 API
    pub async fn generate_summary_with_config(
        &self,
        entry_id: i64,
        config: SummaryConfig,
        provider_id: i64,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), AgentServiceError> {
        // 检查数据库缓存
        if let Some(text) = self
            .get_latest_summary_text(entry_id)
            .map_err(|e| AgentServiceError::Database(e))?
        {
            if !text.is_empty() {
                tracing::info!(
                    "generate_summary_with_config cache hit for entry_id={}, returning cached result",
                    entry_id
                );
                on_event(AiStreamEvent {
                    task_id: 0,
                    content: String::new(),
                    is_done: true,
                    agent_type: "summary".to_string(),
                    error: None,
                });
                return Ok(());
            }
        }

        let (base_url, api_key, model) = self.get_provider_by_id(provider_id)?;

        self.summary_agent
            .generate_summary(
                entry_id,
                config,
                provider_id,
                &base_url,
                &api_key,
                &model,
                on_event,
            )
            .await?;

        Ok(())
    }

    /// 取消摘要
    pub async fn cancel_summary(&self, entry_id: i64) -> Result<(), AgentServiceError> {
        self.summary_agent
            .cancel_summary(entry_id)
            .await?;
        Ok(())
    }

    /// 翻译文章
    /// 先查数据库缓存，若已有成功结果则直接返回，避免重复调用 API
    pub async fn translate_entry(
        &self,
        entry_id: i64,
        target_language: &str,
        concurrency: usize,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), AgentServiceError> {
        if let Some(text) = self
            .get_latest_translation_text(entry_id)
            .map_err(|e| AgentServiceError::Database(e))?
        {
            if !text.is_empty() {
                on_event(AiStreamEvent {
                    task_id: 0,
                    content: String::new(),
                    is_done: true,
                    agent_type: "translation".to_string(),
                    error: None,
                });
                return Ok(());
            }
        }

        let (provider_id, base_url, api_key, model) = self.get_default_provider()?;
        let config = TranslationConfig {
            target_language: target_language.to_string(),
            concurrency_degree: concurrency,
        };

        self.translation_agent
            .translate_entry(
                entry_id,
                config,
                provider_id,
                &base_url,
                &api_key,
                &model,
                on_event,
            )
            .await?;

        Ok(())
    }

    /// 翻译文章（指定配置）
    /// 先查数据库缓存，若已有成功结果则直接返回，避免重复调用 API
    pub async fn translate_entry_with_config(
        &self,
        entry_id: i64,
        config: TranslationConfig,
        provider_id: i64,
        on_event: impl Fn(AiStreamEvent) + Send + Sync + 'static,
    ) -> Result<(), AgentServiceError> {
        // 检查数据库缓存
        if let Some(text) = self
            .get_latest_translation_text(entry_id)
            .map_err(|e| AgentServiceError::Database(e))?
        {
            if !text.is_empty() {
                tracing::info!(
                    "translate_entry_with_config cache hit for entry_id={}, returning cached result",
                    entry_id
                );
                on_event(AiStreamEvent {
                    task_id: 0,
                    content: String::new(),
                    is_done: true,
                    agent_type: "translation".to_string(),
                    error: None,
                });
                return Ok(());
            }
        }

        let (base_url, api_key, model) = self.get_provider_by_id(provider_id)?;

        self.translation_agent
            .translate_entry(
                entry_id,
                config,
                provider_id,
                &base_url,
                &api_key,
                &model,
                on_event,
            )
            .await?;

        Ok(())
    }

    /// 取消翻译
    pub async fn cancel_translation(&self, entry_id: i64) -> Result<(), AgentServiceError> {
        self.translation_agent
            .cancel_translation(entry_id)
            .await?;
        Ok(())
    }

    /// 验证 Provider 连接
    pub async fn validate_provider(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> Result<bool, AgentServiceError> {
        self.ai_client
            .validate(base_url, api_key, model)
            .await
            .map_err(|e| {
                AgentServiceError::Summary(SummaryError::Client(e))
            })
    }

    // ================================================================
    // Provider CRUD 操作（直接在 AgentService 暴露，避免修改 commands.rs）
    // ================================================================

    /// 添加 Provider
    pub fn add_provider(
        &self,
        provider: &crate::db::model::NewProvider,
    ) -> Result<crate::db::model::Provider, String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.insert(provider).map_err(|e| e.to_string())
    }

    /// 列出所有 Provider
    pub fn list_providers(&self) -> Result<Vec<crate::db::model::Provider>, String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.list_all().map_err(|e| e.to_string())
    }

    /// 更新 Provider
    pub fn update_provider(
        &self,
        id: i64,
        update: &crate::db::model::UpdateProvider,
    ) -> Result<crate::db::model::Provider, String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.update(id, update).map_err(|e| e.to_string())
    }

    /// 删除 Provider
    pub fn delete_provider(&self, id: i64) -> Result<(), String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.delete(id).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 添加 Provider Model
    pub fn add_provider_model(
        &self,
        model: &crate::db::model::NewProviderModel,
    ) -> Result<crate::db::model::ProviderModel, String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.add_model(model).map_err(|e| e.to_string())
    }

    /// 列出 Provider 的 Models
    pub fn list_provider_models(
        &self,
        provider_id: i64,
    ) -> Result<Vec<crate::db::model::ProviderModel>, String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.list_models(provider_id).map_err(|e| e.to_string())
    }

    /// 删除 Provider Model
    pub fn delete_provider_model(&self, id: i64) -> Result<(), String> {
        let repo = ProviderRepository::new(self.pool.clone());
        repo.delete_model(id).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// 获取最新的摘要结果
    pub fn get_latest_summary_text(&self, entry_id: i64) -> Result<Option<String>, String> {
        let repo = crate::db::repository::AgentRunRepository::new(self.pool.clone());
        let run = repo
            .find_latest_by_entry_and_task(entry_id, "summary")
            .map_err(|e| e.to_string())?;

        match run {
            Some(r) if r.phase == "succeeded" => Ok(r.output_text),
            _ => Ok(None),
        }
    }

    /// 获取最新的翻译结果
    pub fn get_latest_translation_text(&self, entry_id: i64) -> Result<Option<String>, String> {
        let repo = crate::db::repository::AgentRunRepository::new(self.pool.clone());
        let run = repo
            .find_latest_by_entry_and_task(entry_id, "translation")
            .map_err(|e| e.to_string())?;

        match run {
            Some(r) if r.phase == "succeeded" => Ok(r.output_text),
            _ => Ok(None),
        }
    }

    /// 清除摘要结果
    pub fn clear_summary(&self, entry_id: i64) -> Result<(), String> {
        let repo = crate::db::repository::AgentRunRepository::new(self.pool.clone());
        let runs = repo.list_by_entry(entry_id).map_err(|e| e.to_string())?;
        for run in runs {
            if run.task_kind == "summary" {
                let _ = repo.mark_cancelled(run.id);
            }
        }
        Ok(())
    }

    /// 清除翻译结果
    pub fn clear_translation(&self, entry_id: i64) -> Result<(), String> {
        let repo = crate::db::repository::AgentRunRepository::new(self.pool.clone());
        let runs = repo.list_by_entry(entry_id).map_err(|e| e.to_string())?;
        for run in runs {
            if run.task_kind == "translation" {
                let _ = repo.mark_cancelled(run.id);
            }
        }
        Ok(())
    }
}
