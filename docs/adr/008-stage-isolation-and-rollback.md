# ADR 008: 阶段隔离与回滚机制

**状态**：已采纳

**日期**：2026-07-14

---

## 背景

项目分四阶段递增交付。团队为大一新生，全依赖 AI 辅助编码。若某阶段崩盘后无法干净回滚，需回到项目起点重建，代价不可接受。

## 决策

每个阶段崩盘时仅需回到上一阶段，不回退到项目起点。通过三个硬约束保证：

**1. 数据库迁移只增不改**

```
001_initial → 002_fts → 003_agent → 004_notes → 005_tags → 006_usage_stats
```

每阶段新增 migration 只创建新表/列（带 DEFAULT），绝不修改已有表结构。回滚时后阶段表留在数据库中无害（空表）。

**2. 功能模块文件隔离**

```
Stage 1: src-tauri/src/feed/    src-tauri/src/db/
Stage 2: src-tauri/src/reader/                           ← 不改 feed/, db/
Stage 3: src-tauri/src/agent/                            ← 不改 reader/, feed/
Stage 4: src-tauri/src/digest/  src-tauri/src/notes/     ← 不改 agent/, reader/
```

如需增强上一阶段功能，新建文件而非原地修改（如 `CleanedReaderView.tsx` 而非改 `ReaderView.tsx`）。

**3. Git 里程碑标签**

```
v0.1-stage1 → v0.2-stage2 → v0.3-stage3 → v0.4-stage4
```

崩盘时 `git reset --hard <上一阶段tag>` 即可回到已知良好状态。

## 理由

- 大一新生无法处理复杂回滚操作，需要一键恢复
- AI 辅助开发不确定性高，某阶段失败概率不可忽视
- 三个约束均为低成本规则（命名约定 + 目录纪律），不增加架构复杂度

## 后果

- 每个阶段必须打 tag 后才能进入下一阶段
- 禁止修改已有文件和已有 migration，违反者 Code Review 不通过
- 新建文件优于修改文件的策略可能导致文件名增多，Stage 4 后可统一清理
