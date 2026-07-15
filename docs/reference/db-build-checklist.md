# 数据库构建自检清单

> 用途：AI 完成数据库构建后逐项自检。全部通过才算完成。
> 参考文档：`AGENTS.md` §8（数据库优先规则）、`PLAN.md` §1.2（数据库 Schema）、`docs/reference/command-contract.md`（类型定义）

---

## 构建内容

需要创建/修改的文件：

```
src-tauri/src/db/
├── mod.rs              ← 数据库初始化、连接池创建、WAL 模式、migration 调用
├── migration.rs        ← 迁移系统：扫描目录、按编号排序、事务执行、版本记录
├── migrations/
│   └── 001_initial_schema.sql
├── repository/
│   ├── mod.rs
│   ├── feed_repo.rs
│   ├── entry_repo.rs
│   └── content_repo.rs
├── error.rs            ← RepositoryError（thiserror 派生）
└── model.rs            ← Feed / Entry / Content / EntryListItem 结构体
```

需要修改的文件：

```
src-tauri/Cargo.toml    ← 依赖已添加（rusqlite + r2d2 + r2d2_sqlite）
src-tauri/src/lib.rs    ← 推迟：Tauri 环境未搭建，暂用 main.rs 直接初始化
```

---

## 自检清单（逐项确认）

### A. Schema（对照 `PLAN.md` §1.2.1 的 DDL）

- [x] **A1** `feeds` 表字段与 DDL 完全一致：id, url, title, description, link, feed_type, last_synced_at, created_at
- [x] **A2** `entries` 表字段与 DDL 完全一致：id, feed_id, guid, title, author, link, summary, published_at, updated_at, is_read, is_starred, created_at
- [x] **A3** `entries` 有 `UNIQUE(feed_id, guid)` 约束
- [x] **A4** `entries.feed_id` 有 `REFERENCES feeds(id) ON DELETE CASCADE`
- [x] **A5** `contents` 表字段与 DDL 完全一致：id, entry_id, raw_html, cleaned_html, cleaned_markdown, rendered_html, readability_version, created_at, updated_at
- [x] **A6** `contents.entry_id` 有 `UNIQUE` + `REFERENCES entries(id) ON DELETE CASCADE`
- [x] **A7** `schema_version` 表存在：version(INTEGER PRIMARY KEY), applied_at(TEXT)
- [x] **A8** 除了 DDL 定义的表，没有多建或少建任何表

### B. 迁移系统（对照 `AGENTS.md` §8）

- [x] **B1** 能扫描 `migrations/` 目录下所有 `.sql` 文件
- [x] **B2** 按文件名编号排序（001 → 002 → ...）
- [x] **B3** 读取 `schema_version` 表获取当前版本（空表则视为版本 0）
- [x] **B4** 只执行版本号大于当前版本的 migration
- [x] **B5** 每个 migration 在**单个事务**内执行（失败则回滚，不部分执行）
- [x] **B6** 执行成功后更新 `schema_version` 表
- [x] **B7** migration 文件不存在或编号不连续时报错，不静默跳过

### C. 数据库初始化（对照 `AGENTS.md` §8 + `PLAN.md` §1.2.1）

- [x] **C1** 数据目录使用 `dirs::data_local_dir()` + `/mercury/`（不是硬编码路径）
- [x] **C2** 目录不存在时自动创建
- [x] **C3** 数据库文件路径：`{data_local_dir}/mercury/mercury.db`
- [x] **C4** 使用 `r2d2::Pool<SqliteConnectionManager>` 创建连接池
- [x] **C5** SQLite 使用 `rusqlite` 的 `bundled` feature（不是系统 SQLite）
- [x] **C6** 初始化时执行 `PRAGMA journal_mode=WAL;`
- [x] **C7** 初始化时执行 `PRAGMA foreign_keys=ON;`
- [x] **C8** 初始化时自动执行所有未应用的 migration
- [-] **C9** 连接池注册为 Tauri State（`app.manage(pool)`）— 推迟，需 Tauri 环境就绪后补充

### D. Repository 层（对照 `PLAN.md` §1.2.2 + `AGENTS.md` §10 "Repository 分层"）

- [x] **D1** `FeedRepository` 所有方法存在：insert, insert_full, find_by_id, find_by_url, find_all, find_all_with_unread_count, update_sync_time, update_title, delete
- [x] **D2** `EntryRepository` 所有方法存在：insert_or_ignore, find_by_id, find_by_feed_and_guid, list_by_feed(分页), list_all(分页), search, mark_read, mark_unread, mark_all_read_in_feed
- [x] **D3** `ContentRepository` 所有方法存在：insert_raw, find_by_entry_id, update_cleaned
- [x] **D4** 所有方法返回 `Result<T, RepositoryError>`
- [x] **D5** `RepositoryError` 用 `thiserror` 派生（不是手动 impl Display）
- [x] **D6** Repository 不持有 `rusqlite::Connection` 直接引用，而是通过连接池获取连接
- [x] **D7** 每个方法内部从连接池取连接 → 执行 SQL → 返回结果（不在方法之间共享连接）

### E. 编码约束（对照 `AGENTS.md` §6）

- [x] **E1** 模块/文件名用 `snake_case`：`feed_repo.rs` 而非 `FeedRepo.rs`
- [x] **E2** 类型/trait 名用 `CamelCase`：`FeedRepository` 而非 `feed_repository`
- [x] **E3** 默认数值类型 `i32`（不是 `usize`、`u64`）。数据库 ID 使用 `i64` 适配 `rusqlite::last_insert_rowid()` 返回类型
- [x] **E4** 所有迁移 SQL 在 `.sql` 文件中（不是 Rust 代码里拼接字符串）
- [x] **E5** 数据库代码在 `src-tauri/src/db/` 下，不放在 `src-tauri/src/` 根目录
- [x] **E6** 不在 Tauri Command 中直接使用 `rusqlite::Connection`——必须走 Repository

### F. 测试（对照 `AGENTS.md` §10 "测试约束"）

- [x] **F1** 所有 Repository 有对应的 `#[cfg(test)] mod tests`
- [x] **F2** 测试使用**内存 SQLite**（`:memory:`）
- [x] **F3** 每个测试方法自己执行 migration（不依赖外部数据库状态）
- [x] **F4** 测试覆盖：insert → find_by_id → 字段值匹配
- [x] **F5** 测试覆盖：insert 重复 url → 报错
- [x] **F6** 测试覆盖：find_by_feed_id 分页 → page/page_size 正确
- [x] **F7** 测试覆盖：delete feed → 级联删除 entries 和 contents
- [x] **F8** 测试覆盖：mark_read → is_read 变为 1
- [x] **F9** 测试命名按行为：`test_insert_feed_success` 而非 `test_insert`
- [x] **F10** `cargo test` 全部通过（零失败）— **19 passed**

### G. 全局约束（对照 `AGENTS.md` §15 "Definition of Done"）

- [x] **G1** `cargo build` 编译通过，零 warning
- [-] **G2** `cargo clippy` 零 error（如已安装）— clippy 未安装，待后续 CI 集成
- [x] **G3** 无 `unwrap()`、无 `expect()`、无 `panic!`——所有错误走 Result 传播（测试代码中 `expect` 用于测试 panic，属允许范围）
- [x] **G4** 无 `println!` 或 `eprintln!`——日志走 `tracing`（`info!`/`debug!`/`error!`）
- [x] **G5** 日志：数据库初始化打 `info!`，迁移执行打 `info!`，SQL 错误打 `error!`

---

## 验证命令

```bash
# 编译检查
cargo build

# 运行全部测试
cargo test

# 代码风格检查（如已安装 clippy）
cargo clippy

# 查看数据库文件（手动验证）
sqlite3 ~/.local/share/mercury/mercury.db ".tables"
# 应输出：contents  entries  feeds  schema_version
```

---

## 常见遗漏提醒

1. `r2d2_sqlite` 和 `rusqlite` 是两个不同的 crate，都需要在 `Cargo.toml` 里声明
2. `rusqlite` 的 `bundled` feature 必须开启——否则 Windows 上编译 SQLite 会失败
3. `RepositoryError` 如果只 `impl Display` 而不 `impl std::error::Error`，`anyhow` 的 `?` 操作符会报错。**直接 `#[derive(Error, Debug)]` 用 thiserror 最安全。**
4. 时间字段用 ISO 8601 字符串：`datetime('now')` 而非 Rust 的 `SystemTime`
5. 连接池放 Tauri State 的类型是 `r2d2::Pool<SqliteConnectionManager>`，注意 `SqliteConnectionManager` 来自 `r2d2_sqlite` crate

---

## 自检结果（2026-07-15）

| 类别 | 通过 | 推迟 | 状态 |
|---|---|---|---|
| A. Schema | 8/8 | 0 | ✅ |
| B. 迁移系统 | 7/7 | 0 | ✅ |
| C. 数据库初始化 | 8/9 | 1 (C9: Tauri State) | ✅ |
| D. Repository 层 | 7/7 | 0 | ✅ |
| E. 编码约束 | 6/6 | 0 | ✅ |
| F. 测试 | 10/10 | 0 | ✅ (19 passed) |
| G. 全局约束 | 4/5 | 1 (G2: clippy 未安装) | ✅ |
| **总计** | **50/52** | **2** | ✅ |

推迟项（不阻塞当前阶段）：
- **C9**：需 Tauri 环境就绪后补充 `app.manage(pool)`
- **G2**：clippy 未安装，后续 CI 流程中集成
