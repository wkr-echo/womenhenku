# ADR 009: 并行开发与 mock 先行策略

**状态**：已采纳

**日期**：2026-07-14

---

## 背景

项目分四阶段递增交付，但三人按功能模块分工（Feed 管线 / AI 智能体 / 笔记与基础设施）。若严格按阶段串行推进，Stage 1 期间刘欣慧和杜偲妍无事可做，Stage 3 期间杜偲妍无事可做。

## 决策

**阶段是里程碑标签，不意味串行等待。** 数据库 Schema 落定后，三人各自用 mock 数据独立推进：

- 王康睿：Feed 解析 → CRUD → Readability → Markdown（实际数据驱动）
- 刘欣慧：用 mock HTML 开发 OpenAI 协议 + SSE 流式 + 状态机 → 等 Readability 完成后再对接真实文章
- 杜偲妍：用 mock Entry 开发笔记 CRUD + 设置页 + 通用组件 → 等 Entry 数据就绪后对接

唯一真正的串行依赖：Readability 输出 → AI Agent 的真实文章输入。此依赖用 mock 绕过。

## 理由

- 三人能力不同但应全程有活干，避免空等
- AI Agent 模块是独立封闭的——用 mock HTML 即可完整开发测试，不对其他模块产生耦合
- 笔记模块同样独立——CRUD 不需要真实 Entry 数据即可完成开发
- mock 数据同时作为测试 fixture 复用，减少重复造数据

## 后果

- 每个模块需自行准备 mock 数据（HTML fixture、Entry fixture、Feed fixture）
- 联调期（mock → 真实数据对接）是额外阶段，需预留时间
- 模块间契约（Tauri Command 签名 + 数据库 Schema）必须在 mock 开发前确定，否则 mock 白做
