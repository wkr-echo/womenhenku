# ADR 004: React + TypeScript 作为前端框架

**状态**：已采纳

**日期**：2026-07-12

---

## 背景

Tauri 前端需选择 UI 框架。选项：React、Svelte、Vue。

## 决策

选择 **React + TypeScript**。

## 理由

- AI 训练数据中 React 代码量远超 Svelte/Vue，AI 辅助开发生成质量最高
- React 生态最丰富，UI 组件库（Ant Design、MUI、shadcn/ui 等）选择最多
- TypeScript 提供类型安全，与 Rust 侧的类型契约对接更可靠
- 团队已有 TypeScript 基础，学习曲线平缓

## 后果

- React 打包体积略大于 Svelte，但在 Tauri 桌面场景下差异可忽略
- 需注意 React 状态管理与 Rust 侧数据同步的边界设计
- 放弃 Svelte 的编译时响应式优势，采用 React 运行时 VDOM 模式
