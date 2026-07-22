# v2 功能实现指令（Token 用量 + 多语言）

> 可直接复制给 AI。附带测试方法。已对齐现有代码。

---

## 一、Token 用量统计

### 1.1 当前状态

**后端已完成**：数据库表 `llm_usage_events`（migration `007_llm_usage.sql`）、Repository（`llm_usage_repo.rs`）、Tauri 命令全部实现。

**前端待完成**：设置页缺少 Usage 报表面板。

### 1.2 后端命令（已存在，直接调用）

| 命令 | 参数 | 返回 |
|---|---|---|
| `get_llm_usage_stats` | `{ days, agentType }` (agentType 可选: "summary"/"translation") | `LlmUsageStats { totalTokens, promptTokens, completionTokens, requestCount, successRate, avgTokensPerRequest }` |
| `get_llm_daily_usage` | `{ days, agentType }` | `DailyUsage[]` 按日期 |
| `get_llm_provider_usage` | `{ days }` | `ProviderUsage[]` 按 Provider |
| `get_llm_model_usage` | `{ days }` | `ModelUsage[]` 按 Model |
| `get_llm_agent_usage` | `{ days }` | `AgentUsage[]` 按 Agent 类型 |
| `cleanup_old_llm_events` | `{ retentionDays }` | `number` 清理条数 |

前端 API 已封装在 `src-ui/src/api/feed.ts`：
```ts
getLlmUsageStats(days, agentType) → LlmUsageStats
getLlmDailyUsage(days, agentType) → DailyUsage[]
getLlmProviderUsage(days) → ProviderUsage[]
getLlmModelUsage(days) → ModelUsage[]
```

### 1.3 类型定义（`src-ui/src/lib/types.ts` 已有）

```ts
export interface LlmUsageStats {
  totalTokens: number;
  promptTokens: number;
  completionTokens: number;
  requestCount: number;
  successRate: number;
  avgTokensPerRequest: number;
}
export interface DailyUsage { date: string; totalTokens: number; promptTokens: number; completionTokens: number; requestCount: number; }
export interface ProviderUsage { providerId: number; providerName: string; totalTokens: number; requestCount: number; }
export interface ModelUsage { modelId: number; modelName: string; totalTokens: number; requestCount: number; }
```

### 1.4 报表 UI

在 `SettingsPageView.tsx` 新增 `usage` 板块。需要：

1. 在 settings sections 列表加 `{ key: "usage", label: t("用量") }`
2. 新建 `UsageReport.tsx` 组件，包含以下三个子视图：

**A. 单对象统计面板**

```
时段选择下拉：[30天 | 7天 | 14天 | 90天]
任务过滤下拉：[全部 | 摘要 | 翻译]

┌─────────────────────────────────────────┐
│  每日 Token 折线图（用 CSS/SVG 简单柱状图）│
│  鼠标悬停某一天显示当天明细（可选）         │
├─────────────────────────────────────────┤
│  总计 token      │  totalTokens         │
│  输入 token      │  promptTokens        │
│  输出 token      │  completionTokens    │
│  请求数          │  requestCount        │
│  成功率          │  successRate         │
│  平均每次请求    │  avgTokensPerRequest  │
└─────────────────────────────────────────┘
```

**B. 对比报表**

```
Provider 用量对比（水平柱状图，按 totalTokens 降序）
Model 用量对比（同上）
Agent 用量对比（summary vs translation）
```

**C. 数据管理**

```
数据保留：[180天 ▾]  ← 30/90/180/365/永久
[清除过期] [清除全部]
```

保留天数存入 settings 表：`get_setting("llm_usage_retention_days")` / `set_setting(...)`

### 1.5 文件

```
新建：src-ui/src/components/UsageReport.tsx
修改：src-ui/src/components/SettingsPageView.tsx  ← 增加 "usage" 板块
```

前端 API 和 Rust 后端均已完成，**不需要修改任何后端代码**。

### 1.6 测试

- [ ] 执行摘要/翻译 → 设置 > 用量 有数据显示
- [ ] 切换时间窗口 → 数据变化
- [ ] 切换任务过滤 → 仅显示选中类型
- [ ] 清除过期数据 → 超期数据被删除
- [ ] Provider 删除后 → 报表仍显示（因为存了 provider_name 快照）

---

## 二、多语言 UI

### 2.1 当前状态

**已有基础**：`src-ui/src/lib/utils.ts` 中的 `t()` 函数 + `zh.json` / `en.json` 语言文件。前端组件已全部用 `t()` 包裹。

**待完成**：设置页语言切换控件。

### 2.2 现有 i18n 实现

```ts
// src-ui/src/lib/utils.ts
let currentLang = "zh";

export function setLanguage(lang: string) { currentLang = lang; }
export function getLanguage(): string { return currentLang; }
export function t(key: string): string { return translations[currentLang]?.[key] || key; }
```

语言资源文件（已存在）：
- `src-ui/src/locales/zh.json` — 中文翻译
- `src-ui/src/locales/en.json` — 英文翻译

语言设置持久化到 SQLite settings 表：key = `app_language`，通过 `get_setting` / `set_setting` 读写。

启动时从 settings 表读取语言，无值时跟随系统（`navigator.language.startsWith('zh') ? 'zh' : 'en'`）。

### 2.3 待实现：设置页语言切换

在 `SettingsPageView.tsx` 的 `appearance` 板块增加语言选择：

```
外观设置
  主题：[亮色 / 暗色 / 跟随系统]
  语言：[中文 / English / 跟随系统]
```

交互：
- 下拉选择后调用 `setLanguage(lang)` 即时生效
- 调用 `setSetting("app_language", lang)` 持久化
- 启动时调用 `getSetting("app_language")` 读取并 `setLanguage()`

### 2.4 约束

- 所有面向用户的字符串必须走 `t()`，禁止硬编码
- 调试日志、错误堆栈不翻译（保持英文）
- UI 语言只有中文（`zh`）和英文（`en`）两种
- Agent 输出语言（摘要/翻译的目标语言）和 UI 语言是两个独立设置

### 2.5 文件

```
修改：src-ui/src/components/SettingsPageView.tsx  ← 增加语言下拉框
修改：src-ui/src/main.tsx 或 App.tsx ← 启动时读取语言设置
新增：src-ui/src/locales/zh.json 和 en.json 中补全缺失的翻译键
```

### 2.6 测试

- [ ] 设置 > 外观 > 语言下拉 → 切换为 English → 界面即时变英文
- [ ] 切换回中文 → 恢复中文
- [ ] 重启应用 → 语言保持最后选择
- [ ] UI 语言切换不影响 Agent 目标语言设置（设置 > Agent > 目标语言独立）
- [ ] 错误提示保持英文不翻译
