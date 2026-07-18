// Agent 统一状态机（ADR 007）
//
// 所有 Agent（Summary、Translation）必须实现此状态机。
//
// 状态转换规则：
//   Idle → Running       用户触发
//   Running → Succeeded   流式完成
//   Running → Failed      API 错误
//   Running → Cancelled   用户取消
//   Succeeded → Idle      用户清除结果
//   Failed → Cancelled    用户取消
//
// 禁止的转换：Idle → Succeeded、Failed → Running（重试必须创建新 Run）

use serde::{Deserialize, Serialize};
use std::fmt;

/// Agent 阶段状态
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum AgentPhase {
    #[serde(rename = "idle")]
    Idle,
    #[serde(rename = "running")]
    Running,
    #[serde(rename = "succeeded")]
    Succeeded,
    #[serde(rename = "failed")]
    Failed,
    #[serde(rename = "cancelled")]
    Cancelled,
}

impl fmt::Display for AgentPhase {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AgentPhase::Idle => write!(f, "idle"),
            AgentPhase::Running => write!(f, "running"),
            AgentPhase::Succeeded => write!(f, "succeeded"),
            AgentPhase::Failed => write!(f, "failed"),
            AgentPhase::Cancelled => write!(f, "cancelled"),
        }
    }
}

impl From<&str> for AgentPhase {
    fn from(s: &str) -> Self {
        match s {
            "idle" => AgentPhase::Idle,
            "running" => AgentPhase::Running,
            "succeeded" => AgentPhase::Succeeded,
            "failed" => AgentPhase::Failed,
            "cancelled" => AgentPhase::Cancelled,
            _ => AgentPhase::Idle,
        }
    }
}

/// 状态转换错误
#[derive(Debug, thiserror::Error)]
pub enum StateTransitionError {
    #[error("不允许的转换: {from} → {to}")]
    InvalidTransition { from: AgentPhase, to: AgentPhase },
}

/// 检查状态转换是否合法
pub fn check_transition(from: &AgentPhase, to: &AgentPhase) -> Result<(), StateTransitionError> {
    let allowed = matches!(
        (from, to),
        (AgentPhase::Idle, AgentPhase::Running)
            | (AgentPhase::Running, AgentPhase::Succeeded)
            | (AgentPhase::Running, AgentPhase::Failed)
            | (AgentPhase::Running, AgentPhase::Cancelled)
            | (AgentPhase::Succeeded, AgentPhase::Idle)
            | (AgentPhase::Failed, AgentPhase::Cancelled)
    );

    if allowed {
        Ok(())
    } else {
        Err(StateTransitionError::InvalidTransition {
            from: from.clone(),
            to: to.clone(),
        })
    }
}

/// Agent 任务类型
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum TaskKind {
    #[serde(rename = "summary")]
    Summary,
    #[serde(rename = "translation")]
    Translation,
}

impl fmt::Display for TaskKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TaskKind::Summary => write!(f, "summary"),
            TaskKind::Translation => write!(f, "translation"),
        }
    }
}

impl From<&str> for TaskKind {
    fn from(s: &str) -> Self {
        match s {
            "summary" => TaskKind::Summary,
            "translation" => TaskKind::Translation,
            _ => TaskKind::Summary,
        }
    }
}

/// Agent 插槽（用于 latest-only 队列管理）
#[derive(Debug)]
pub struct AgentSlot {
    /// 当前正在运行的任务
    pub active: Option<i64>, // run_id
    /// 等待中的任务（被最新请求覆盖）
    pub waiting: Option<i64>, // run_id
}

impl AgentSlot {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            active: None,
            waiting: None,
        }
    }

    /// 是否有正在运行的任务
    pub fn is_busy(&self) -> bool {
        self.active.is_some()
    }

    /// 尝试获取执行许可
    /// 返回 true 表示可以直接执行，false 表示进入等待队列
    pub fn try_acquire(&mut self, run_id: i64) -> bool {
        if self.active.is_none() {
            self.active = Some(run_id);
            self.waiting = None;
            true
        } else {
            // latest-only 替换策略
            self.waiting = Some(run_id);
            false
        }
    }

    /// 完成任务，如果有等待任务则提升为 active
    pub fn complete(&mut self) -> Option<i64> {
        self.active = None;
        let promoted = self.waiting.take();
        if let Some(id) = promoted {
            self.active = Some(id);
        }
        promoted
    }

    /// 取消当前任务
    pub fn cancel(&mut self) {
        self.active = None;
        self.waiting = None;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_transitions() {
        // Idle → Running
        assert!(check_transition(&AgentPhase::Idle, &AgentPhase::Running).is_ok());
        // Running → Succeeded
        assert!(check_transition(&AgentPhase::Running, &AgentPhase::Succeeded).is_ok());
        // Running → Failed
        assert!(check_transition(&AgentPhase::Running, &AgentPhase::Failed).is_ok());
        // Running → Cancelled
        assert!(check_transition(&AgentPhase::Running, &AgentPhase::Cancelled).is_ok());
        // Succeeded → Idle
        assert!(check_transition(&AgentPhase::Succeeded, &AgentPhase::Idle).is_ok());
        // Failed → Cancelled
        assert!(check_transition(&AgentPhase::Failed, &AgentPhase::Cancelled).is_ok());
    }

    #[test]
    fn test_invalid_transitions() {
        // Idle → Succeeded（跳过执行）
        assert!(check_transition(&AgentPhase::Idle, &AgentPhase::Succeeded).is_err());
        // Failed → Running（不能恢复失败任务）
        assert!(check_transition(&AgentPhase::Failed, &AgentPhase::Running).is_err());
        // Cancelled → Running
        assert!(check_transition(&AgentPhase::Cancelled, &AgentPhase::Running).is_err());
        // Succeeded → Running
        assert!(check_transition(&AgentPhase::Succeeded, &AgentPhase::Running).is_err());
    }

    #[test]
    fn test_agent_slot_acquire_and_complete() {
        let mut slot = AgentSlot::new();
        assert!(!slot.is_busy());

        // 第一次获取 → true（可以执行）
        assert!(slot.try_acquire(1));
        assert!(slot.is_busy());

        // 第二次获取 → false（进入等待）
        assert!(!slot.try_acquire(2));
        assert_eq!(slot.waiting, Some(2));

        // 完成当前任务，等待的提升为 active
        let promoted = slot.complete();
        assert_eq!(promoted, Some(2));
        assert_eq!(slot.active, Some(2));
        assert!(slot.waiting.is_none());
    }

    #[test]
    fn test_agent_slot_cancel() {
        let mut slot = AgentSlot::new();
        slot.try_acquire(1);
        slot.try_acquire(2);

        slot.cancel();
        assert!(slot.active.is_none());
        assert!(slot.waiting.is_none());
    }
}
