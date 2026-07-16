# ADR 006: Reader 管线固定流程

**状态**：已采纳

**日期**：2026-07-14

---

## 背景

文章从 Feed 原始 HTML 到最终展示需经过多个处理步骤。处理顺序和归属层（Rust vs React）若不固定，会导致行为不一致、调试困难、跨平台差异。

## 决策

Reader 处理管线固定为以下 7 步，**全部在 Rust Core 完成**：

```
Feed Entry → Raw HTML → Readability → HTML Sanitization
→ Markdown Conversion → Rendered HTML → Reader View（React 纯展示）
```

## 理由

- 与 Mercury 原版 Reader Pipeline 行为一致（复刻优先原则）
- 所有内容处理在 Rust 侧，React 只负责展示，不引入前端解析差异
- 管线步骤固定，调试时可逐段定位问题
- Readability 和 comrak 均为 Rust crate，天然适合在 Rust 侧执行

## 后果

- React 禁止执行 Readability、Markdown 转换、HTML 清洗
- 如需在管线中新增处理步骤，需更新本文档并创建 ADR
- 渲染后的 HTML 缓存策略（themeId + entryId + readerRenderVersion）参考原版设计
