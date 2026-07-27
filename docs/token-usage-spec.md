# Token 用量记录 — 完整功能规范

> 可直接复制给 AI。附带测试方法。

---

## 一、记录什么

每次 LLM 调用（无论成功/失败/取消/超时）自动记录一条 `usage_events`。

### 数据库表

```sql
CREATE TABLE IF NOT EXISTS usage_events (
    id                          INTEGER PRIMARY KEY AUTOINCREMENT,
    task_run_id                 INTEGER,          -- 关联 AgentTaskRun（可为空）
    entry_id                    INTEGER,          -- 文章 ID（可为空）
    task_type                   TEXT NOT NULL,    -- "summary" / "translation" / "tagging"
    provider_profile_id         INTEGER,
    model_profile_id            INTEGER,
    provider_base_url_snapshot  TEXT NOT NULL,    -- Provider 快照：baseURL
    provider_host_snapshot      TEXT,             -- Provider 快照：主机名
    provider_name_snapshot      TEXT,             -- Provider 快照：名称
    model_name_snapshot         TEXT NOT NULL,    -- Model 快照：名称
    request_phase               TEXT NOT NULL,    -- "normal" / "retry"
    request_status              TEXT NOT NULL,    -- "succeeded" / "failed" / "cancelled" / "timed_out"
    prompt_tokens               INTEGER,          -- 输入 token（失败时可为空）
    completion_tokens           INTEGER,          -- 输出 token（失败时可为空）
    total_tokens                INTEGER,          -- prompt + completion
    usage_availability          TEXT NOT NULL,    -- "actual" / "missing"
    started_at                  TEXT,             -- ISO 8601
    finished_at                 TEXT,             -- ISO 8601
    created_at                  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_usage_task_type_created ON usage_events(task_type, created_at);
CREATE INDEX IF NOT EXISTS idx_usage_provider_created ON usage_events(provider_profile_id, created_at);
```

### 字段说明

| 字段 | 类型 | 说明 |
|---|---|---|
| `task_type` | TEXT | 哪个 Agent 发起的调用。**三个都记录**：summary / translation / tagging |
| `request_phase` | TEXT | `normal`（正常请求）/ `retry`（重试） |
| `request_status` | TEXT | `succeeded`（成功）、`failed`（失败）、`cancelled`（用户取消）、`timed_out`（超时） |
| `prompt_tokens` | INTEGER? | 输入 token 数。失败/取消/超时时为 NULL |
| `completion_tokens` | INTEGER? | 输出 token 数。同上 |
| `total_tokens` | INTEGER? | prompt + completion。同上 |
| `usage_availability` | TEXT | `actual`（提供商返回了 token 数）/ `missing`（提供商没返回） |
| `*_snapshot` | TEXT | Provider/Model 的快照字段。即使 Provider 被删除，报表仍能正常显示 |

### 为什么需要快照

用户在设置中删除或修改了一个 Provider。如果只存 `provider_id`，报表里这些历史记录就变成 "Unknown Provider" 了。快照保存了调用时刻的 URL、名称、主机名，历史报表永远完整。

---

## 二、什么时候记录

三个 Agent 的 **每次 LLM 请求完成后**自动记录：

```
Summary Agent:
  ├── 请求成功 → recordUsageEvent(status="succeeded", tokens=API返回值)
  ├── 请求失败 → recordUsageEvent(status="failed", tokens=NULL)
  └── 用户取消 → recordUsageEvent(status="cancelled", tokens=NULL)

Translation Agent:
  ├── 每个段落翻译成功 → recordUsageEvent(status="succeeded", ...)
  ├── 每个段落翻译失败 → recordUsageEvent(status="failed", ...)
  └── 用户取消 → recordUsageEvent(status="cancelled", ...)

Tagging Agent:
  ├── 标签推荐成功 → recordUsageEvent(status="succeeded", ...)
  ├── 标签推荐失败 → recordUsageEvent(status="failed", ...)
  └── 用户关闭面板 → recordUsageEvent(status="cancelled", ...)
```

注意：Translation 是**每个段落单独记录一次**，不是整篇文章一次。10 个段落翻译成功 = 10 条记录。

---

## 三、不记录什么

| 不记录 | 原因 |
|---|---|
| Provider 验证请求（`validate_provider` 发的 "hi"） | 不是业务调用，不消耗有意义 token |
| 前端到 Tauri 的 invoke 调用 | 只记录 LLM API 调用，不记录内部 IPC |
| 用户手动输入的标签 | 不是 AI 调用 |
| Readability 提取 | 不涉及 LLM |
| Feed 抓取 | 不涉及 LLM |

---

## 四、报表和记录的分离

| 层面 | Summary | Translation | Tagging |
|---|---|---|---|
| 数据库记录 | ✅ 有 | ✅ 有 | ✅ 有 |
| 报表折线图 | ✅ 显示 | ✅ 显示 | ❌ 不显示 |
| 任务过滤下拉框 | ✅ "摘要" | ✅ "翻译" | ❌ 无标签选项 |
| 对比报表 | ✅ 有 | ✅ 有 | ❌ 无 |

Tagging 记录但不显示的原因：标签推荐的 token 量和摘要/翻译不在一个量级（几十 vs 几百上千），混在一起干扰趋势判断。但数据保留了，以后如果需要可以打开。

---

## 五、Rust 侧实现

### Recorder 模块

```rust
// src-tauri/src/usage/recorder.rs

pub struct UsageEventContext {
    pub task_type: String,          // "summary" | "translation" | "tagging"
    pub entry_id: Option<i32>,
    pub provider_id: Option<i32>,
    pub model_id: Option<i32>,
    pub provider_base_url: String,
    pub provider_host: Option<String>,
    pub provider_name: Option<String>,
    pub model_name: String,
    pub request_phase: String,      // "normal" | "retry"
    pub request_status: String,     // "succeeded" | "failed" | "cancelled" | "timed_out"
    pub prompt_tokens: Option<i32>,
    pub completion_tokens: Option<i32>,
    pub started_at: Option<String>,
    pub finished_at: Option<String>,
}

pub fn record_usage_event(pool: &DbPool, ctx: UsageEventContext) -> Result<(), String> {
    let total = match (ctx.prompt_tokens, ctx.completion_tokens) {
        (Some(p), Some(c)) => Some(p + c),
        _ => None,
    };
    let availability = if total.is_some() { "actual" } else { "missing" };

    let conn = pool.get().map_err(|e| e.to_string())?;
    conn.execute(
        "INSERT INTO usage_events (...) VALUES (...)",
        params![...],
    ).map_err(|e| e.to_string())?;
    Ok(())
}
```

### 调用位置

在三个 Agent 的 LLM 请求完成后调用 `record_usage_event`：

```rust
// Summary Agent 中
match llm_response {
    Ok(result) => {
        record_usage_event(&pool, UsageEventContext {
            task_type: "summary".into(),
            request_status: "succeeded".into(),
            prompt_tokens: result.usage.prompt_tokens,
            completion_tokens: result.usage.completion_tokens,
            ...
        })?;
    }
    Err(e) => {
        record_usage_event(&pool, UsageEventContext {
            task_type: "summary".into(),
            request_status: "failed".into(),
            prompt_tokens: None,
            completion_tokens: None,
            ...
        })?;
    }
}
```

---

## 六、数据保留策略

| 配置 | 默认值 | 选项 |
|---|---|---|
| 保留期限 | 6 个月 | 1月 / 3月 / 6月 / 1年 / 永久 |

- 应用启动时自动删除超过保留期限的记录（`DELETE FROM usage_events WHERE created_at < datetime('now', '-6 months')`）
- 设置页提供「立即清除过期」按钮
- 设置页提供「清除全部」按钮（需确认弹窗）

---

## 七、测试方法

- [ ] 执行一次摘要 → `usage_events` 表新增一条记录，task_type="summary"
- [ ] 执行一次翻译（3 段落）→ 新增 3 条记录，task_type="translation"
- [ ] 打开标签面板 → AI 推荐标签完成后 → 新增记录，task_type="tagging"
- [ ] 摘要成功：request_status="succeeded"，prompt_tokens > 0，total_tokens = prompt + completion
- [ ] 摘要失败：request_status="failed"，prompt_tokens 和 completion_tokens 为 NULL
- [ ] Provider 被删除后 → 历史记录仍能正常读取（快照字段不变）
- [ ] 启动时自动清理 → 超过 6 个月的记录被删除
- [ ] 手动清除全部 → 确认弹窗 → 所有记录被删除
- [ ] Tagging 的记录存在但报表不显示
