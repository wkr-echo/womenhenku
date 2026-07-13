# ADR 001: Rust 作为核心语言

**状态**：已采纳

**日期**：2026-07-12

---

## 背景

跨平台复刻需选择核心语言。选项：Swift 6（复用原项目代码）vs Rust（重写）。

## 决策

选择 **Rust**。

## 理由

- Swift on Windows 工具链不成熟，社区资料稀缺，需 Sidecar IPC 架构引入额外复杂度
- Rust 编译器报错极其详细，编译通过即高概率正确，适合 AI 辅助开发
- Rust + Tauri 同语言单进程，零 IPC，复杂度低一个数量级
- AI 训练数据中 Rust 代码丰富，生成质量可控

## 后果

- 无法复用原 Mercury 的 Swift 代码（Readability、Agent 状态机、数据库层），需用 Rust 生态重写
- 获得三平台（Windows/macOS/Linux）统一工具链与构建流程
- 单进程架构消除 IPC 调试与性能开销
