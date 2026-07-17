# Stage 4 前端开发约束 — 杜偲妍

> 给杜偲妍和她的 AI 助手 — 只能动前端，不能碰后端。

---

## 允许修改的文件

| 允许 | 说明 |
|---|---|
| `src-ui/src/components/ContentAreaView.tsx` | 工具栏加"多选导出"按钮 |
| `src-ui/src/components/EntryListView.tsx` | 加多选 checkbox，尽量不改已有结构 |
| `src-ui/src/components/ReaderView.tsx` | 导出功能已有，可优化 |
| `src-ui/src/components/NoteEditorView.tsx` | 可优化 |
| `src-ui/src/components/SettingsPageView.tsx` | 可优化 |
| `src-ui/src/hooks/useKeyboard.ts` | 加 `s` `t` `n` `r` 快捷键 |
| `src-ui/src/api/agent.ts` | **新建**（刘欣慧也会建，协调好） |
| `src-ui/src/contexts/AppContext.tsx` | **仅追加**新 Action，不改已有 |
| `src-ui/src/lib/types.ts` | 可加类型 |
| `src-ui/package.json` | 加 `@tauri-apps/plugin-notification` 依赖 |
| `src-ui/src/styles/globals.css` | 可加样式 |

## 绝对禁止

| 禁止 | 原因 |
|---|---|
| `src-tauri/` 整个目录 | 后端已封版，一个字节都不改 |
| `src-ui/src/api/feed.ts` | 已有 API 够用，要新功能找王康睿 |
| `src-ui/src/components/SidebarView.tsx` | 侧边栏逻辑已稳定 |
| 删除已有 Action 或 case | 只能追加 |

## 任务清单

| # | 任务 | 后端接口（已就绪） |
|---|---|---|
| 1 | 文章列表多选 + 批量导出 | `export_multi_digest(entryIds, format)` |
| 2 | 键盘快捷键 `s` `t` `n` `r` | 在 `useKeyboard.ts` 加，对应切换摘要/翻译/笔记面板、刷新 |
| 3 | 桌面通知 | `npm install @tauri-apps/plugin-notification` |
| 4 | 设置页数据持久化 | 目前用 localStorage，后续可接 DB |

## 验证

```bash
cd src-ui && npx tsc --noEmit
# 零 error 才能提交
```
