# Mercury 跨平台移植项目 - 初始化指南

> 生成日期：2026-07-12

---

## 1. 项目概述

### 1.1 项目介绍

**Mercury** 是一款 macOS 原生、本地优先的 RSS 阅读器，专注于舒适的信息聚合与阅读体验，并配备高度可定制的 AI 功能。

本项目目标是将 Mercury 从 macOS 平台移植到 **Windows** 和 **Linux**，采用 Tauri + Web UI 的架构方案。

### 1.2 核心技术栈

| 层级 | 技术 |
|---|---|
| 语言 | Swift（Swift 6 并发） |
| UI | SwiftUI + AppKit（混合） |
| 存储 | SQLite（通过 GRDB，WAL 模式） |
| 网络 | URLSession |
| Feed 解析 | FeedKit |
| HTML 清理 | SwiftSoup |
| 文章提取 | 自研 `Readability`（无 WebKit 依赖） |
| Markdown → HTML | swift-markdown + 内置 `MarkupHTMLVisitor` |
| LLM 客户端 | SwiftOpenAI |
| 测试 | Swift Testing（`@Suite`、`@Test`、`#expect`） |

### 1.3 核心功能模块

| 模块 | 说明 |
|---|---|
| Feed | RSS/Atom/JSON Feed 解析、OPML 导入导出、订阅源同步、侧边栏计数 |
| Reader | 自研 Readability 提取、Markdown 转换、主题/字体定制、阅读管线 |
| Agent - Summary | 可配置语言与详细程度的文章摘要、流式输出、自定义提示词 |
| Agent - Translation | 段落级双语对照布局、逐段并发、重试/恢复/清除、HY-MT2 优化 |
| Agent - Tagging | AI 标签建议、批量打标签、标签库维护（合并/别名/清理） |
| Digest | 单篇 Markdown 笔记、单篇/多篇文摘导出、通过 macOS 服务分享 |
| Usage | Token 用量统计与对比报表（Provider/Model/Agent 维度） |
| Settings | Provider/Model/Agent 配置、提示词定制、文摘模板定制 |

---

## 2. 环境要求

### 2.1 macOS 开发环境

| 组件 | 版本要求 |
|---|---|
| macOS | 14+ (Sonoma) |
| Xcode | 15+ |
| Swift | 6 |
| Swift Package Manager | Latest |

### 2.2 Windows 开发环境

| 组件 | 版本要求 |
|---|---|
| Windows | 10+ (10.0.22621+) |
| Visual Studio | 2022 |
| Windows SDK | 10.0.22621+ |
| Swift for Windows | 6 (社区版) |

### 2.3 Linux 开发环境

| 组件 | 版本要求 |
|---|---|
| Ubuntu | 22.04+ |
| Swift for Linux | 6 |
| SQLite | 3.35+ (系统自带) |

### 2.4 Tauri 开发环境

| 组件 | 版本要求 |
|---|---|
| Node.js | 18+ |
| Rust | 1.75+ |
| Tauri CLI | 2.x |

---

## 3. 快速开始

### 3.1 克隆项目

```bash
git clone <repository-url>
cd mercury
```

### 3.2 macOS 开发环境设置

```bash
# 安装 Xcode
xcode-select --install

# 验证 Swift 版本
swift --version

# 构建项目
swift build

# 运行测试
swift test
```

### 3.3 Windows 开发环境设置

```bash
# 安装 Visual Studio 2022（包含 C++ 工具集）
# 安装 Windows SDK 10.0.22621
# 安装 Swift for Windows 6

# 验证 Swift 版本
swift --version

# 构建项目
swift build
```

### 3.4 Linux 开发环境设置

```bash
# 安装依赖
sudo apt-get update
sudo apt-get install clang libicu-dev libcurl4-openssl-dev libsqlite3-dev

# 安装 Swift for Linux
# 验证 Swift 版本
swift --version

# 构建项目
swift build
```

### 3.5 Tauri 项目初始化

```bash
# 安装 Tauri CLI
npm install -g @tauri-apps/cli

# 在项目根目录初始化 Tauri
pnpm create tauri-app@2.x

# 安装依赖
pnpm install

# 开发模式运行
pnpm tauri dev

# 构建生产版本
pnpm tauri build
```

---

## 4. 需要解决的问题

### 4.1 技术问题

| # | 问题 | 严重程度 | 关联模块 |
|---|---|---|---|
| 1 | GRDB 在 Windows 上的可用性验证 | 严重 | MercuryStore |
| 2 | Swift 6 并发运行时在 Windows 上的稳定性 | 高 | MercuryCore |
| 3 | Darwin API 在 Readability 中的依赖审计 | 中 | Reader |
| 4 | IPC 协议设计与可靠性 | 高 | Tauri + Swift Sidecar |
| 5 | Web UI 与 SwiftUI 体验差距 | 中 | 前端 |

### 4.2 架构问题

| # | 问题 | 说明 |
|---|---|---|
| 6 | macOS 专属 API 抽象 | 需要设计 MercuryPlatform 协议 |
| 7 | 数据库跨平台兼容性 | 验证 SQLite WAL 模式在各平台的行为 |
| 8 | 本地化资源加载 | 脱离 macOS Bundle 体系 |
| 9 | 凭证存储跨平台方案 | Keychain → Credential Manager / libsecret |
| 10 | 自动更新机制 | Sparkle 替代方案 |

### 4.3 流程问题

| # | 问题 | 说明 |
|---|---|---|
| 11 | 工具链版本固定 | 需要记录确切版本确保可复现 |
| 12 | 跨平台 CI 搭建 | GitHub Actions 配置 |
| 13 | 测试覆盖 | 跨平台验证稳定性 |
| 14 | 打包分发 | MSI/AppX/AppImage/Flatpak |

---

## 5. 移植阶段规划

### 阶段 0：可行性验证（决策关口）

> **目标**：回答"Mercury 的 Readability 和数据库层能否在 Windows 上可靠运行？"
> **产出**：Go / No-Go 决策。

| # | 任务 | 详情 |
|---|---|---|
| 0.1 | 选择 Windows Swift 工具链 | 评估 swift.org 官方工具链与社区发行版 |
| 0.2 | 验证 Swift 6 并发运行时 | 在 Windows 上编译并运行 async/await、actor、Task、Sendable 小程序 |
| 0.3 | 验证 Swift Testing 可用性 | 检查 Swift Testing 是否在所选 Windows 工具链上可用 |
| 0.4 | 解决 SQLite 链接问题 | Windows 无系统 SQLite，需选择 SPM 静态编译、预编译 amalgamation 或 vcpkg |
| 0.5 | 审计 Readability 中的 Darwin API | 扫描 Darwin.C、平台条件编译、OSLog 使用 |
| 0.6 | 构建 MercuryPortingProbe | 独立 SwiftPM 包，包含 Readability + SwiftSoup + GRDB + 最小 Mercury schema |
| 0.7 | 探针验证测试 | 至少 3-5 个 HTML fixture，验证完整流程 |

### 阶段 1：核心模块提取与平台抽象

> **目标**：在不破坏 macOS 应用的前提下，将可复用代码与 macOS 专属代码分离。

| # | 任务 | 详情 |
|---|---|---|
| 1.1 | 创建 MercuryCore 模块 | 提取纯业务逻辑，无 AppKit/UIKit/SwiftUI 导入 |
| 1.2 | 创建 MercuryStore 模块 | 提取数据库层，零 UI 导入 |
| 1.3 | 定义 MercuryPlatform 协议 | 为所有平台依赖能力设计抽象接口 |
| 1.4 | 实现 macOS 适配器 | 将现有 AppKit/SwiftUI 代码接入 MercuryPlatform 协议 |
| 1.5 | 移除 Core/Store 中的平台导入 | 确保 MercuryCore 和 MercuryStore 编译时不带平台专属导入 |
| 1.6 | 解耦本地化基础设施 | 使 LanguageManager 脱离 macOS Bundle 体系运行 |
| 1.7 | 验证 macOS 应用仍可构建运行 | 提取后运行完整测试，零回归 |

### 阶段 2：Windows UI 框架

> **目标**：用 Tauri + Web UI 构建 Windows UI 壳。

| # | 任务 | 详情 |
|---|---|---|
| 2.1 | 初始化 Tauri v2 项目 | 配置 Tauri 与 WebView2 |
| 2.2 | 设计 IPC 协议 | 定义 JSON-RPC 命令规范，仅限粗粒度命令 |
| 2.3 | 实现 Swift 边车进程 | 将 MercuryCore + MercuryStore 编译为 Windows 可执行文件 |
| 2.4 | 实现 Windows 平台适配器 | 文件对话框、剪贴板、通知、凭证存储、路径管理 |
| 2.5 | 构建 Web UI 壳 | 实现侧边栏布局、订阅源列表、文章列表、阅读区、工具栏 |
| 2.6 | 阅读器内容渲染 | 在 WebView 中渲染清理后的 HTML/Markdown |
| 2.7 | Agent 面板 UI | 摘要面板、双语翻译布局、标签面板 |

### 阶段 3：功能迁移

> **目标**：达到与 macOS 版本的功能对等。

| # | 任务 | 详情 |
|---|---|---|
| 3.1 | 订阅源管理 | 添加/删除/刷新订阅源、OPML 导入导出、订阅源分组 |
| 3.2 | 文章列表与筛选 | 文章列表含已读/未读状态、搜索、按标签筛选 |
| 3.3 | 阅读器完整功能 | 阅读模式、主题切换、字体切换、响应式布局 |
| 3.4 | 摘要智能体 | 生成摘要、流式输出、语言与详细程度选择 |
| 3.5 | 翻译智能体 | 段落级双语布局、重试失败段落、清除翻译 |
| 3.6 | 标签系统 | 手动标签、AI 建议标签、批量打标签、标签库维护 |
| 3.7 | 文摘与笔记 | 单篇 Markdown 笔记编辑、单篇/多篇文摘导出 |
| 3.8 | 设置面板 | Provider/Model/Agent 配置 UI、提示词定制 |
| 3.9 | 用量统计 | Token 用量追踪、基于 Web 图表的对比报表 |
| 3.10 | 界面本地化 | Web UI 中英文切换 |

### 阶段 4：Linux 支持（次要目标）

> **目标**：扩展到 Linux，但不主导架构决策。

| # | 任务 | 详情 |
|---|---|---|
| 4.1 | 验证 Linux Swift 工具链 | 在 Linux 上编译 MercuryCore + MercuryStore |
| 4.2 | SQLite 兼容性 | 验证系统 SQLite 版本与 WAL 支持 |
| 4.3 | Linux 平台适配器 | 文件对话框、剪贴板、通知、凭证存储、路径管理 |
| 4.4 | WebView 方案 | 评估 WebViewGTK |
| 4.5 | Linux 打包 | AppImage、Flatpak 或发行版特定包 |

### 阶段 5：质量与发布

> **目标**：生产就绪的跨平台发布。

| # | 任务 | 详情 |
|---|---|---|
| 5.1 | 跨平台 CI | GitHub Actions 在 macOS、Windows、Linux 上构建并测试 |
| 5.2 | IPC 合约测试 | 验证稳定性、错误恢复、超时处理、协议版本控制 |
| 5.3 | 数据库迁移测试 | 验证迁移在三个平台上正确执行 |
| 5.4 | Windows 打包 | MSI 或 AppX 安装器，代码签名 |
| 5.5 | Linux 打包 | AppImage / Flatpak |
| 5.6 | 文档 | 更新 README，添加安装说明和故障排除指南 |
| 5.7 | 本地化同步 | 确保所有 Web UI 字符串可本地化 |

---

## 6. 目标架构

### 6.1 三层分离

```
┌──────────────────────────────────────────────────────────┐
│                    共享 Swift 模块                        │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────┐      │
│  │ MercuryCore │  │ MercuryStore│  │MercuryPlatform│     │
│  │ 纯业务逻辑   │  │ 数据库持久化 │  │ 抽象接口     │      │
│  └──────┬──────┘  └──────┬──────┘  └──────┬──────┘      │
└─────────┼────────────────┼────────────────┼──────────────┘
          │                │                │
          │                │                │
┌─────────▼────────────────┼────────────────▼──────────────┐
│       macOS              │              Windows          │
│  ┌─────────────┐         │       ┌─────────────┐         │
│  │ SwiftUI+AppKit│       │       │ Tauri+WebUI │         │
│  └──────┬──────┘         │       └──────┬──────┘         │
│         │                │               │                │
│  ┌──────▼──────┐         │  ┌────────────▼────────────┐   │
│  │ macOS适配器 │         │  │   Swift边车进程        │   │
│  └──────┬──────┘         │  │ MercuryCore+MercuryStore│   │
│         └────────────────┼──┴────────────┬────────────┘   │
│                          │               │                │
│                          │  ┌────────────▼────────────┐   │
│                          │  │   Windows适配器        │   │
│                          │  └────────────────────────┘   │
└──────────────────────────┴───────────────────────────────┘
```

### 6.2 各层职责

#### MercuryCore（纯业务逻辑）
- Feed 用例与同步策略
- Reader 管线合约
- Markdown 转换策略
- Agent 运行时与状态机
- 提示词/模板解析
- Summary、Translation、Tagging 合约
- Digest 组合/导出策略
- 任务生命周期逻辑
- Usage 报表合约
- 标签规范化与本地标签回退

#### MercuryStore（持久化层）
- DatabaseManager
- Schema 迁移
- GRDB 模型
- Feed、Entry、Content、Tag、Note、Agent、Translation、Summary、Digest、Usage 存储
- 查询构建器
- 数据库相关测试

#### MercuryPlatform（抽象接口）
- 凭证存储
- 设置存储
- 文件选择器与导出目录访问
- 剪贴板
- 分享服务
- 字体目录
- 拼写检查辅助
- WebView 托管
- 通知
- 自动更新
- 操作系统特定路径与权限
- 本地化资源加载

---

## 7. IPC 命令域（边车协议）

Swift 边车进程应暴露以下粗粒度命令域：

| 命令域 | 操作 |
|---|---|
| Feed 增删改查 | 添加、删除、刷新、列表、OPML 导入导出 |
| Entry 查询 | 列表（分页、筛选）、按 ID 获取、标记已读/未读、搜索 |
| Reader 内容 | 构建管线、获取清洗后 HTML、获取 Markdown、获取渲染后 HTML |
| Agent - Summary | 启动、获取状态、获取结果、取消 |
| Agent - Translation | 启动、获取状态、获取结果、重试段落、取消、清除 |
| Agent - Tagging | 建议标签、批量打标签、管理标签库 |
| Settings | 获取/设置 provider、model、agent 配置；获取/设置提示词 |
| Digest | 获取/设置笔记、准备单篇/多篇文摘、导出 |
| Usage | 查询报表、获取统计 |
| App 状态 | 获取侧边栏计数、获取同步状态 |

---

## 8. 风险评估

| 风险 | 严重程度 | 缓解措施 |
|---|---|---|
| GRDB 在 Windows 上不可用 | 严重 | 阶段 0 验证，受阻则评估 SQLite.swift 或原始 SQLite |
| Windows 上 Swift 6 并发 bug | 高 | 阶段 0 运行时测试，必要时回退到串行调度 |
| IPC 复杂性与可靠性 | 高 | 从简单的 stdin/stdout JSON 行开始，设计超时与心跳机制 |
| Web UI 体验与 SwiftUI 差距 | 中 | 聚焦功能对等，而非像素级复刻 |
| 三个平台的维护成本 | 中 | 将 MercuryCore 和 MercuryStore 作为唯一事实来源 |
| Readability 包含 Darwin API | 中 | 阶段 0 审计，用日志协议替换 OSLog |
| 工具链不稳定 | 中 | 固定确切工具链版本，记录设置步骤 |

---

## 9. 项目目录结构

```
Mercury/
├── Mercury/                          # 主应用
│   ├── App/                          # 入口点、根组合、应用生命周期
│   ├── Agent/                        # AI 智能体基础设施
│   │   ├── Provider/                 # LLM 提供商管理与验证
│   │   ├── Runtime/                  # 状态机、引擎、存储、激活
│   │   ├── Summary/                  # 文章摘要
│   │   ├── Translation/              # 段落级双语翻译
│   │   ├── Tagging/                  # AI 标签建议
│   │   ├── Settings/                 # 智能体配置
│   │   └── Shared/                   # 共享智能体工具
│   ├── Core/                         # 核心逻辑（待提取）
│   │   ├── Database/                 # GRDB 模型、迁移、存储
│   │   ├── Tasking/                  # TaskQueue、TaskCenter
│   │   └── Shared/                   # 共享工具
│   ├── Reader/                       # 文章提取、Markdown 转换、主题
│   ├── Feed/                         # 订阅源管理、同步、OPML
│   ├── Digest/                       # 笔记、文摘分享与导出
│   ├── Tags/                         # 标签规范化、本地标签
│   ├── Usage/                        # LLM token 用量追踪
│   └── Resources/                    # 智能体提示词、文摘模板
├── MercuryCore/                      # 纯业务逻辑模块（新建）
├── MercuryStore/                     # 数据库持久化模块（新建）
├── MercuryPlatform/                  # 平台抽象模块（新建）
├── MercuryMac/                       # macOS 适配器（新建）
├── MercuryWin/                       # Windows 适配器（新建）
├── MercuryLinux/                     # Linux 适配器（新建）
├── tauri/                            # Tauri 项目（新建）
│   ├── src/                          # 前端源码
│   └── src-tauri/                    # Tauri 后端
├── scripts/                          # 构建与测试脚本
├── tests/                            # 测试用例
├── cross-platform-analysis.md        # 跨平台分析与移植计划
├── INIT.md                           # 项目初始化指南（本文件）
├── PORTING.md                        # 原始移植分析
├── AGENTS.md                         # Agent 工程笔记
└── docs/                             # 技术文档
    ├── file-structure.md             # 文件结构重构计划
    ├── l10n.md                       # 本地化设计
    ├── db-test.md                    # 数据库测试设计笔记
    ├── markdown-engine.md            # Markdown 渲染引擎
    ├── reader-mode.md                # Reader 模式架构
    └── swift-concurrency.md          # Swift 并发指南
```

---

## 10. 关键参考文献

- [cross-platform-analysis.md](./cross-platform-analysis.md) — 跨平台分析与移植计划
- PORTING.md — 原始移植分析与探针计划
- AGENTS.md — Agent 工程笔记与仓库规则
- docs/file-structure.md — 文件结构重构计划
- docs/l10n.md — 本地化设计
- docs/db-test.md — 数据库测试设计笔记
- docs/markdown-engine.md — Markdown 渲染引擎
- docs/reader-mode.md — Reader 模式架构
- docs/swift-concurrency.md — Swift 并发指南