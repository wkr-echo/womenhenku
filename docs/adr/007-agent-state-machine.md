# ADR 007: Agent 统一状态机

**状态**：已采纳

**日期**：2026-07-14

---

## 背景

项目包含多种 AI Agent（Summary、Translation、Tagging），各自有运行生命周期。若无统一状态模型，会导致 UI 状态同步不一致、取消/重试行为各异、调试困难。

## 决策

所有 Agent 必须实现**统一 5 状态机**：

```
Idle → Running → Succeeded
              → Failed
              → Cancelled
```

**允许的转换**：
- `Idle → Running → Succeeded`
- `Idle → Running → Failed`
- `Idle → Running → Cancelled`

**禁止的转换**：
- `Idle → Succeeded`（跳过执行）
- `Failed → Running`（直接恢复失败任务，必须创建新 Run）

## 理由

- 与 Mercury 原版 `AgentRunStateMachine` 行为一致（复刻优先原则）
- 统一状态机简化 UI 层的状态展示逻辑（所有 Agent 共用同一套状态驱动）
- 明确禁止的转换防止 AI 辅助开发时写出不合理的状态跳转
- 重试必须创建新 Run，保留历史执行记录便于调试

## 后果

- 所有 Agent 实现必须包含状态机字段，增加少量模板代码
- Translation 和 Summary 的激活/恢复/取消流程需遵循状态机转换规则
- 全局执行策略：不自动取消进行中的任务；取消只能来自用户显式操作
- 等待队列 latest-only 替换策略：每类 Agent active slot 1 + waiting slot 1
