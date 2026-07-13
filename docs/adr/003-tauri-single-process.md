# ADR 003: Tauri 单进程架构

**状态**：已采纳

**日期**：2026-07-12

---

## 背景

跨平台桌面应用需选择 UI 壳与进程架构。选项：Electron vs Tauri，以及是否采用 Sidecar IPC 多进程。

## 决策

选择 **Tauri 2 + Rust 单进程**，Rust 核心直接嵌入 Tauri，通过 Tauri Command 暴露接口。

## 理由

- 核心语言已选定 Rust，Tauri 天然同语言，无需 IPC 桥接
- 单进程消除 JSON-RPC stdin/stdout 通信的序列化开销与调试复杂度
- Tauri 2 原生支持 WebView2（Windows）、WKWebView（macOS）、WebViewGTK（Linux），三平台统一
- 相比 Electron，安装包体积小一个数量级，内存占用更低

## 后果

- 前端必须通过 Tauri Command（Rust 函数）调用核心逻辑，不能直接访问文件系统/数据库
- 需协定 Command 接口契约后再并行开发
- 放弃 Electron 生态的部分成熟方案（如自动更新），需寻找 Tauri 替代
