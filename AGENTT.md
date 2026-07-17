# Mercury 跨平台复刻 — AGENTT.md

> 刘欣慧个人工作笔记 | v1.0 | 基于 AGENTS.md 三版整合版

---

## 我的分工

| 功能模块 | 我的职责 |
|---|---|
| **AI 智能体（③④）** | Rust 侧：Provider 管理、OpenAI 协议封装、Summary/Translation Agent |
| | React 侧：摘要面板、翻译面板、Agent 配置页 |

---

## 开发计划

### Stage 3 任务分解（我主导）

| 任务 | 预计开始 | 前置依赖 |
|---|---|---|
| Provider 配置与管理 | Stage 2 完成后 | 数据库 Schema 就绪 |
| OpenAI 协议封装 + SSE 流式 | 与 Provider 并行 | 无（可用 mock） |
| Summary Agent 实现 | 协议封装完成后 | Readability 管线可用 |
| Translation Agent 实现 | Summary 之后 | Readability 管线可用 |

**注意**：我可以用 mock HTML 数据提前开发 AI 协议和 SSE 流式，不阻塞在 Stage 1/2。

---

## 快速参考

### 我的 Tauri Command 命名规范

| 功能 | Command 命名 |
|---|---|
| 添加 Provider | `add_provider` |
| 列表 Provider | `list_providers` |
| 验证连接 | `validate_provider` |
| 生成摘要 | `generate_summary` |
| 取消摘要 | `cancel_summary` |
| 翻译文章 | `translate_entry` |
| 重试段落 | `retry_translation` |
| 清除翻译 | `clear_translations` |

### SSE 事件格式

```typescript
// Rust → Frontend
interface AiStreamEvent {
  task_id: string;
  content: string;
  is_done: boolean;
  agent_type: "summary" | "translation";
  error?: string;
}
```

### Provider 模型

```rust
struct Provider {
    id: i32,
    name: String,
    base_url: String,
    api_key: String,       // 加密存储
    default_model: String,
    thinking_model: String,
    created_at: String,
}
```

---

## 提醒清单

- [ ] 先读 AGENTS.md 再开工
- [ ] Prompt 模板放 `resources/prompts/`，禁止硬编码
- [ ] 新增 crate 先问团队
- [ ] Command 签名变更通知前后端同步
- [ ] 测试用 mock HTTP server，不要调真实 API
- [ ] SSE 流式通过 Tauri Event 推送，前端不直连 HTTP
- [ ] API Key 用 secure-store 加密，不存数据库明文

---

*个人备忘，不替代 AGENTS.md*
