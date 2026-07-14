# ADR 005: Repository 分层架构

**状态**：已采纳

**日期**：2026-07-14

---

## 背景

Tauri Command 直接拼接 SQL 会导致业务逻辑散落、难以测试、数据库迁移时变更范围不可控。需确定数据库访问的内部架构分层。

## 决策

采用 **Repository 三层分层**：

```
Tauri Command（参数校验、错误转换）
    ↓
Service（业务逻辑编排）
    ↓
Repository（数据访问封装）
    ↓
SQLite
```

## 理由

- Command 层只做参数校验和错误转换，不含业务逻辑，与架构原则「Tauri Command 禁止业务逻辑」一致
- Service 层编排业务逻辑，可独立单元测试（mock Repository）
- Repository 层封装所有 SQL，schema 变更时只需修改 Repository 实现
- 分层边界清晰，AI 辅助开发时不易越界

## 后果

- 每个数据实体需对应 Repository 实现，初期代码量略增
- Command → Service → Repository 调用链增加一层间接性
- 禁止在 Command 中直接拼接 SQL，违反者 Code Review 不通过
