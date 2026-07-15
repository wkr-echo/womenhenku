# INIT 决策记录

> 此文档记录 INIT.md 编写过程中的所有关键讨论、冲突与决议，可喂给 AI 生成 ADR 或后续文档。
> 日期：2026-07-12 ~ 2026-07-13

---

## 一、三版 INIT.md 差异

三人分别产出 INIT.md，核心分歧如下：

| 决策点   | 刘欣慧（英文详细版）      | 杜偲妍（Windows 优先版）          | 王康睿（我们的版本）                          |
| -------- | ------------------------- | --------------------------------- | --------------------------------------------- |
| 核心语言 | 未明确（提到 Rust crate） | **Swift 6**（复用原有代码） | **Rust**（跨平台一等公民）              |
| 架构     | Tauri IPC                 | Swift Sidecar + JSON-RPC IPC      | **单进程**（Rust + Tauri）              |
| 前端     | React                     | **Svelte > React > Vue**    | **React + TypeScript**                  |
| 数据库   | 未明确                    | GRDB（复用原有）                  | SQLite（`rusqlite`，新实现）                |
| 目标平台 | Win + Mac + Linux         | **Windows 优先**            | Win + Mac + Linux 同时交付                    |
| v1 范围  | 5 大类（含笔记/统计）     | 最宽（含笔记/统计/本地化）        | 聚焦（Feed + 阅读 + AI 摘要/翻译 + 笔记导出） |

---

## 二、冲突解决过程与最终决策

### 决策 1：核心语言 → Rust

讨论过程：

- 队员 B 倾向 Swift：可复用 Readability、Agent 状态机、数据库层
- 分析 Swift on Windows 的现状：工具链不成熟、社区资料少、需 Sidecar IPC 架构
- 从 AI 辅助开发角度对比：
  - Swift 跨平台代码 AI 训练数据少，生成质量不可控
  - Rust 编译器报错极其详细，编译通过即高概率正确
  - Rust + Tauri 同语言单进程，零 IPC，复杂度低一个数量级

**最终：Rust**

### 决策 2：架构 → 单进程

讨论过程：

- 队员 B 方案：Swift Sidecar + Tauri IPC（JSON-RPC stdin/stdout）
- 如果已选 Rust，Tauri 天然支持同语言，IPC 完全不需要

**最终：Rust 核心直接嵌入 Tauri，单进程，通过 Tauri Command 暴露接口**

### 决策 3：AI 协议 → OpenAI 兼容

讨论过程：

- 用户确认老师要求"大模型中立，支持任意可提供标准 API 的大模型"
- 解释 OpenAI 兼容协议已是行业事实标准：DeepSeek/Ollama/vLLM/通义千问/智谱全部兼容
- 如做多协议原生支持需自建 provider trait 抽象层，v1 成本太高

**最终：实现 `/v1/chat/completions` + SSE 流式，用户配置 baseURL + API Key + model**

### 决策 4：前端 → React

讨论过程：

- 队员 B 优先级：Svelte > React > Vue
- 从 AI 辅助开发视角：AI 见过的 React 代码量远超 Svelte/Vue
- React 生态最丰富，UI 组件库选择最多

**最终：React + TypeScript**

### 决策 5：功能优先级调整

讨论过程：

- 初始 v1 含标签系统基础功能
- 用户要求将标签整体后移，将笔记与文摘导出提前

**最终：v1 = ①Feed ②文章清洗 ③AI 摘要 ④AI 翻译 ⑤笔记与文摘导出；v2 = ⑥标签 ⑦用量统计 ⑧多语言**

---

## 三、技术选型总表

| 层级      | 选型                                           | 理由                                                                   |
| --------- | ---------------------------------------------- | ---------------------------------------------------------------------- |
| 语言      | Rust                                           | 跨平台一等公民、编译器即 reviewer、AI 训练数据丰富、Tauri 同语言单进程 |
| UI 壳     | Tauri 2                                        | 同语言零 IPC、WebView2/WKWebView/WebViewGTK 原生支持三平台             |
| 前端      | React + TypeScript                             | AI 见过的 React 代码量最大，生成质量最高                               |
| 数据库    | SQLite（`rusqlite`，WAL 模式）               | schema 参考原始 Mercury 设计                                           |
| HTTP      | `reqwest`                                    | Rust 生态标准                                                          |
| Feed 解析 | `feed-rs`                                    | 支持 RSS/Atom/JSON Feed                                                |
| 文章提取  | `readability` crate                          | Mozilla Readability 的 Rust 移植                                       |
| HTML 清洗 | `scraper`                                    | CSS 选择器友好，AI 辅助效率高                                          |
| Markdown  | `comrak`                                     | GFM 完整支持                                                           |
| AI 协议   | OpenAI 兼容                                    | 覆盖 DeepSeek/Ollama/vLLM 等所有兼容服务                               |
| 打包      | `.msi` / `.dmg` / `.AppImage` + `.deb` | 各平台原生安装格式                                                     |

---

## 四、差异化策略

三人讨论了如何在多数组走相似技术路线的情况下拉开差距：

1. **跨平台 CI Day 1**：push 一次三平台自动编译+测试，README 放绿勾截图
2. **ADR 架构决策记录**：4 份极简 Markdown（Rust over Swift / OpenAI 协议 / Tauri over Electron / React over Vue/Svelte），每份 15-20 行
3. **多模型对比实验**：同一 Prompt 给 DeepSeek/Claude/Copilot，对比生成质量，记录到文档
4. **Tauri 原生桌面通知**：新文章到达系统通知
5. **文章离线缓存**：已加载文章缓存 SQLite，断网可看
6. **键盘快捷键体系**：`j/k` 切换、`s` 摘要、`t` 翻译、`n` 笔记

---

## 五、模块拆分方案（按功能分工）

按功能模块端到端划分，每人负责一个功能领域的 Rust 核心 + React UI，三人三模块：

| 功能模块                   | 负责人 | Rust 侧                                                                                   | React 侧                                         |
| -------------------------- | ------ | ----------------------------------------------------------------------------------------- | ------------------------------------------------ |
| Feed 与内容管线（①②）    | 王康睿 | Feed 解析/同步、OPML、SQLite schema 与迁移、Readability 提取管线、Markdown 转换、搜索索引 | 订阅源管理页、文章列表页、阅读器页、侧边栏       |
| AI 智能体（③④）          | 刘欣慧 | Provider 管理、OpenAI 兼容协议封装、Summary Agent、Translation Agent                      | 摘要面板、翻译面板、Agent 配置页                 |
| 笔记、文摘与基础设施（⑤） | 杜偲妍 | 笔记 CRUD、文摘导出策略与模板渲染、设置持久化、Tauri Command 契约定义与维护               | 笔记编辑页、文摘导出面板、设置页、通用 UI 组件库 |

契约先行：三人先协定 Tauri Command 列表与数据库 Schema，然后各自 mock 先行独立开发，互不阻塞。

---

## 六、团队角色建议（按功能模块分配）

| 人员   | 功能模块                   | 原因                                                                                                                                                                                  |
| ------ | -------------------------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| 王康睿 | Feed 与内容管线（①②）    | 数据库 Schema 是系统性风险最高的模块——字段/索引/迁移设计一错全崩。需自行设计 SQLite schema（功能语义参考原版，内部结构重新设计），设计决策影响所有模块。Readability 中文页面提取质量是经验性调参工作。 |
| 刘欣慧 | AI 智能体（③④）          | AI 模块独立封闭——用 mock HTML 即可完整开发测试（OpenAI 协议 + SSE 流式 + 状态机），不阻塞其他人。单点技术深度高（异步流解析、并发控制）但不连累全局。                               |
| 杜偲妍 | 笔记、文摘与基础设施（⑤） | 功能相对独立（CRUD + 模板渲染），React 工作量较大（编辑页/设置页/通用组件），成果可视化强，成就感高。                                                                                 |

---

## 七、架构原则

1. 业务逻辑不得依赖 UI
2. 数据库层不得依赖平台 API
3. 平台能力必须经过抽象层
4. UI 仅负责展示，所有业务逻辑走 Rust 核心
5. 数据库单进程写入
6. 三模块通过 Tauri Command 解耦，契约先行

---

## 八、约束条件

- 平台中立：Windows 10+ / macOS 12+ / Linux（Wayland + X11）
- 大模型中立：OpenAI 兼容协议，不绑定特定提供商
- 本地优先：无需账号/登录，数据全存本地
- AI 编码可追溯：AGENTS.md + PLAN.md + ADR 决策记录
