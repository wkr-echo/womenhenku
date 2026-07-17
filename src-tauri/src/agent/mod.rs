// Agent 模块 — Stage 3
//
// 提供 Summary 和 Translation 两个 AI Agent，使用 OpenAI 兼容协议。
// 实现统一的 5 状态状态机（ADR 007）。

pub mod client;
pub mod prompt;
pub mod service;
pub mod state;
pub mod summary;
pub mod translation;
