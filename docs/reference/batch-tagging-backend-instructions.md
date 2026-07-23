# BatchTagging 后端 AI 流水线实现指令

> 可直接复制给 AI。前端 UI 已完成，后端需新建 AI 批量分析命令。

---

## 当前状态

| 部分 | 状态 |
|---|---|
| 前端 UI (`BatchTagging` 组件) | ✅ 完成。设置 > 批量标签 |
| 候选文章计数 | ✅ `count_entries_by_date_range(days)` 真实查询 |
| AI 批量分析 | ❌ 需新建 Rust 命令 |
| 标签批量应用 | ⚠️ `tag_entries_batch(entryIds, tagId)` 命令存在，但未被调用 |

前端 `BatchTagging` 位于 `src-ui/src/components/SettingsPageView.tsx`，约 line 2000+。

---

## 需要新建的 Rust 命令

### 命令名：`analyze_entries_for_tags`

```
fn analyze_entries_for_tags(
    state: State<'_, DbPool>,
    days: i64,            // 时间范围（天），如 7/30/90
    skip_tagged: bool,    // 是否跳过已有标签的文章
    concurrency: i32,     // 并发度（1~5）
) -> Result<Vec<TagProposal>, String>
```

### 返回类型 `TagProposal`

```rust
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct TagProposal {
    tag_name: String,
    hit_count: i32,       // 该标签命中了多少篇文章
    entry_count: i32,     // 涉及多少篇文章
}
```

### 执行流程

```
1. 根据 days 参数查询候选文章列表（entry_id + title + summary）
   - SQL: SELECT id, title, summary FROM entries WHERE created_at >= datetime('now', '-' || ?1 || ' days')
   - 如果 skip_tagged: 排除已有 entry_tags 关联的文章

2. 对每篇文章，调用 LLM 分析并建议标签（复用 generate_tag_recommendations 的 AI 调用模式）
   - 输入：文章标题 + 摘要前 500 字符
   - Prompt：与 generate_tag_recommendations 相同（要求返回 JSON 数组）
   - 并发度：用 Semaphore 控制，默认 3

3. 汇总所有文章的建议标签，统计 hit_count（出现次数）和 entry_count（涉及文章数）

4. 返回 Vec<TagProposal> 给前端
```

### 前端调用

前端 `startBatch` 函数替换为：

```ts
const startBatch = async () => {
  setState(prev => ({ ...prev, phase: "running" }));
  try {
    const proposals = await invoke<TagProposal[]>("analyze_entries_for_tags", {
      days: rangeDays[state.config.range],
      skipTagged: state.config.skipTaggedEntries,
      concurrency: state.config.concurrency,
    });
    const decisions: Record<string, "keep" | "discard" | "pending"> = {};
    proposals.forEach(p => decisions[p.tagName] = "pending");
    setState(prev => ({
      ...prev,
      phase: proposals.length > 0 ? "review" : "done",
      suggestedTags: proposals.map(p => ({
        name: p.tagName,
        hitCount: p.hitCount,
        entryCount: p.entryCount,
      })),
      tagDecisions: decisions,
    }));
  } catch (e) {
    setState(prev => ({ ...prev, phase: "cancelled" }));
  }
};
```

---

## 约束

### 必须遵循的规则

1. **文件位置**：新命令放在 `src-tauri/src/commands.rs`，注册在 `src-tauri/src/lib.rs`
2. **Provider 获取**：复用 `ProviderRepository::find_default()` 获取默认 provider
3. **API Key 解密**：用 `crate::agent::crypto::decrypt(&provider.api_key_ref)`
4. **AI 调用**：用 `crate::agent::client::AiClient::new().chat(...)`（非流式，每篇一次调用）
5. **Prompt 模板**：复用 `generate_tag_recommendations` 中的 system_prompt 和 user_prompt 格式
6. **超时**：每篇文章 60 秒超时
7. **错误处理**：单篇文章失败不中止整体，跳过继续
8. **命名规范**：camelCase（前端），snake_case（Rust），`#[serde(rename_all = "camelCase")]`
9. **文件不修改**：不修改 `src-ui/src/components/SettingsPageView.tsx` 中已有的任何 UI 代码
10. **不修改前端 UI**：只负责 Rust 后端命令 + lib.rs 注册 + feed.ts API 封装

### 禁止事项

- 不要创建新文件（在现有 `commands.rs` 末尾添加即可）
- 不要修改 `tag_repo.rs`、`entry_repo.rs` 等 repository 文件
- 不要用 `println!`，用 `tracing::info!` / `warn!` / `error!`
- 不要在前端创建新组件或修改现有组件行为
- 不要硬编码 base_url / api_key / model

---

## 已有可复用的代码

### AI 调用模式（参考 `generate_tag_recommendations`）

```rust
// commands.rs 中已有的 AI 调用模式
let provider_repo = ProviderRepository::new(pool.clone());
let provider = provider_repo.find_default()?;
let api_key = crate::agent::crypto::decrypt(&provider.api_key_ref)?;
let client = crate::agent::client::AiClient::new();

let response = client.chat(
    &provider.base_url, &api_key, &model,
    system_prompt, &user_prompt
).await?;
```

### 已有命令注册模式

```rust
// lib.rs 中注册
#[cfg(feature = "tauri-runtime")]
#[tauri::command]
async fn analyze_entries_for_tags(
    state: State<'_, DbPool>,
    days: i64,
    skip_tagged: bool,
    concurrency: i32,
) -> Result<Vec<TagProposal>, String> {
    commands::analyze_entries_for_tags(&state, days, skip_tagged, concurrency).await
}
```

### 前端 API 封装模式

```ts
// feed.ts 中封装
export async function analyzeEntriesForTags(days: number, skipTagged: boolean, concurrency: number): Promise<TagProposal[]> {
  return invoke("analyze_entries_for_tags", { days, skipTagged, concurrency });
}
```

### 前端类型

```ts
interface TagProposal {
  tagName: string;
  hitCount: number;
  entryCount: number;
}
```

---

## 测试方法

- [ ] 设置 > 批量标签 → 选择时间范围 → 候选文章数显示真实数字
- [ ] 点"开始" → AI 分析中 → 显示建议标签列表
- [ ] 审查阶段：保留/丢弃标签 → 点"应用"
- [ ] 应用后在标签库中看到新创建的标签
