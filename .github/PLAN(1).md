# Mercury 跨平台复刻项目 — PLAN.md  
  
版本：v1.0  
  
状态：Planning  
  
---  
  
# 1. 项目背景  
  
Mercury 是一款优秀的本地优先 RSS 阅读器。  
  
原版采用：  
  
- Swift  
- SwiftUI  
- GRDB  
  
主要运行于 macOS。  
  
本项目目标是在保留 Mercury 产品体验的前提下，实现：  
  
- Windows  
- macOS  
- Linux  
  
三平台运行。  
  
---  
  
# 2. 项目目标  
  
## v1 必须实现  
  
### Feed 系统  
  
- RSS  
- Atom  
- JSON Feed  
- OPML 导入  
- OPML 导出  
- 自动同步  
  
### Reader  
  
- Readability 提取  
- HTML 清洗  
- Markdown 转换  
- 阅读模式  
  
### AI 功能  
  
- AI 摘要  
- AI 翻译  
- Prompt 管理  
  
### 笔记  
  
- Markdown 笔记  
- 文摘导出  
  
---  
  
## v2 计划实现  
  
- 标签系统  
- Token 用量统计  
- 多语言界面  
- AI 标签推荐  
  
---  
  
# 3. 技术栈  
  
## 后端  
  
语言：  
  
Rust  
  
框架：  
  
Tauri 2  
  
数据库：  
  
SQLite  
  
数据库库：  
  
rusqlite  
  
网络：  
  
reqwest  
  
日志：  
  
tracing  
  
---  
  
## 前端  
  
框架：  
  
React  
  
语言：  
  
TypeScript  
  
样式：  
  
TailwindCSS  
  
组件库：  
  
shadcn/ui  
  
---  
  
## Reader  
  
文章提取：  
  
readability  
  
HTML 清洗：  
  
scraper  
  
Markdown：  
  
comrak  
  
---  
  
## Feed  
  
feed-rs  
  
支持：  
  
- RSS  
- Atom  
- JSON Feed  
  
---  
  
## AI  
  
统一协议：  
  
OpenAI Compatible API  
  
兼容：  
  
- OpenAI  
- DeepSeek  
- Ollama  
- vLLM  
- OpenRouter  
  
---  
  
# 4. 系统架构  
  
整体架构：  
  
React UI  
↓  
Tauri Command  
↓  
Rust Service  
↓  
Repository  
↓  
SQLite  
  
所有业务逻辑位于 Rust。  
  
---  
  
# 5. 核心模块  
  
## Feed Module  
  
职责：  
  
- Feed 管理  
- Feed 同步  
- OPML 导入导出  
  
---  
  
## Reader Module  
  
职责：  
  
- 内容提取  
- 内容清洗  
- Markdown 转换  
  
---  
  
## Agent Runtime  
  
职责：  
  
- Summary  
- Translation  
- Future Tagging  
  
---  
  
## Notes Module  
  
职责：  
  
- Markdown 笔记  
- 文摘管理  
  
---  
  
## Export Module  
  
职责：  
  
- Markdown 导出  
- 文摘导出  
  
---  
  
## Settings Module  
  
职责：  
  
- Provider 配置  
- Prompt 配置  
- 同步配置  
  
---  
  
# 6. 数据库设计  
  
核心表：  
  
feeds  
  
entries  
  
contents  
  
summaries  
  
translations  
  
notes  
  
settings  
  
---  
  
未来扩展：  
  
tags  
  
usage_stats  
  
providers  
  
---  
  
# 7. Reader 设计  
  
固定管线：  
  
Feed Entry  
↓  
Raw HTML  
↓  
Readability  
↓  
HTML Sanitization  
↓  
Markdown Conversion  
↓  
Rendered HTML  
↓  
Reader View  
  
---  
  
# 8. Agent 设计  
  
## Summary Agent  
  
能力：  
  
- 流式输出  
- Prompt 配置  
- 结果持久化  
  
---  
  
## Translation Agent  
  
能力：  
  
- 段落翻译  
- 并发执行  
- 双语展示  
- 结果持久化  
  
默认并发：  
  
3  
  
允许范围：  
  
1~5  
  
---  
  
# 9. Prompt 系统  
  
内置 Prompt：  
  
resources/prompts/  
  
用户 Prompt：  
  
用户数据目录  
  
支持：  
  
- 覆盖  
- 自定义  
- 导入导出  
  
---  
  
# 10. 第一阶段  
  
目标：  
  
完成基础阅读器。  
  
交付：  
  
- Feed 管理  
- Feed 同步  
- SQLite  
- 文章列表  
- 阅读页面  
  
---  
  
# 11. 第二阶段  
  
目标：  
  
完成阅读体验。  
  
交付：  
  
- Readability  
- Markdown  
- 搜索  
- 主题切换  
  
---  
  
# 12. 第三阶段  
  
目标：  
  
完成 AI 功能。  
  
交付：  
  
- Provider 管理  
- Summary  
- Translation  
  
---  
  
# 13. 第四阶段  
  
目标：  
  
完成知识管理。  
  
交付：  
  
- Notes  
- Digest Export  
- Settings  
  
---  
  
# 14. 风险分析  
  
技术风险：  
  
- 非标准 RSS 兼容性  
- Readability 中文质量  
- Linux WebViewGTK  
  
产品风险：  
  
- 偏离 Mercury 行为  
- AI 成本控制  
  
---  
  
# 15. 当前状态  
  
项目文档：  
已完成  
  
架构设计：  
已完成  
  
代码实现：  
未开始  
  
下一步：  
  
1. 初始化 Tauri 项目  
2. 创建数据库 Schema  
3. 实现 Feed 模块  
4. 完成第一阶段 MVP  
  
---  
  
# 16. 里程碑  
  
Milestone 1：  
  
基础阅读器  
  
Milestone 2：  
  
阅读体验增强  
  
Milestone 3：  
  
AI 功能  
  
Milestone 4：  
  
知识管理  
  
Milestone 5：  
  
正式发布 v1  
  
---  
  
# 17. 成功标准  
  
满足以下条件视为成功：  
  
- 支持 RSS/Atom/JSON Feed  
- 支持 Readability 阅读模式  
- 支持 AI 摘要  
- 支持 AI 翻译  
- 支持笔记系统  
- 支持文摘导出  
- Windows/macOS/Linux 可运行  
  
最终目标：  
  
成为 Mercury 在跨平台生态中的高质量复刻版本。  
