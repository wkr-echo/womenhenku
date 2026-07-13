# ADR 002: OpenAI 兼容协议作为 AI 接入标准

**状态**：已采纳

**日期**：2026-07-12

---

## 背景

项目要求"大模型中立，支持任意可提供标准 API 的大模型"。需决定 AI 协议策略。

## 决策

实现 **OpenAI 兼容协议**（`/v1/chat/completions` + SSE 流式），用户自行配置 `baseURL` + `API Key` + `model`。

## 理由

- OpenAI 兼容协议已是行业事实标准：DeepSeek、Ollama、vLLM、通义千问、智谱等全部兼容
- 无需自建多协议 provider trait 抽象层，v1 成本可控
- 用户可接入任意云端或本地大模型，满足"大模型中立"要求

## 后果

- 不支持非 OpenAI 兼容的原生 API（如 Anthropic Messages API、Gemini API 原生格式）
- 依赖社区/第三方代理工具补齐非兼容协议的转换
- v2 可评估引入多协议原生支持的必要性
