# Mercury 跨平台复刻项目 — AGENTS.md  
  
本文件是项目最高优先级规范。  
  
所有 AI Coding Agent（Cursor、Claude Code、Codex、OpenHands 等）执行任务前必须优先阅读本文件。  
  
## 文档优先级  
  
发生冲突时：  
  
1. AGENTS.md  
2. ADR（docs/adr）  
3. PLAN.md  
4. 现有代码实现  
5. AI 自身推断  
  
---  
  
# 1. 项目使命  
  
本项目不是设计新的 RSS 阅读器。  
  
本项目是 Mercury 的跨平台复刻。  
  
目标：  
  
在 Windows、macOS、Linux 上尽可能复现 Mercury 的功能与体验。  
  
允许：  
  
- 编程语言不同  
- UI 框架不同  
- 数据库实现不同  
- 内部架构不同  
  
不允许：  
  
- 用户行为改变  
- 功能语义改变  
- 工作流改变  
- Agent 行为改变  
  
---  
  
# 2. 复刻原则  
  
开发任何功能前必须先回答：  
  
“Mercury 原版是如何工作的？”  
  
然后再开始实现。  
  
优先级：  
  
Mercury 原版行为  
>  
当前实现  
>  
开发者个人偏好  
  
如果发现 Mercury 原版设计存在缺陷：  
  
1. 创建 ADR  
2. 提交人工评审  
  
禁止 AI 自行优化产品行为。  
  
---  
  
# 3. 架构原则  
  
系统架构：  
  
React UI  
↓  
Tauri Command  
↓  
Rust Core  
  
职责划分：  
  
Rust Core：  
  
- Feed  
- Reader  
- Agent Runtime  
- Database  
- Search  
- Notes  
- Export  
- Settings  
  
Tauri Command：  
  
- 参数校验  
- 权限边界  
- 错误转换  
  
React：  
  
- UI 渲染  
- 用户交互  
- 状态展示  
  
业务逻辑必须位于 Rust Core。  
  
---  
  
# 4. 功能一致性原则  
  
Mercury 的行为是唯一事实来源。  
  
允许：  
  
- 内部实现不同  
  
不允许：  
  
- 用户可见行为不同  
- 数据结构语义不同  
- 状态流转不同  
  
如果实现与 Mercury 不一致：  
  
修正实现。  
  
不要修改规范。  
  
---  
  
# 5. 数据库优先原则  
  
SQLite 是唯一事实来源。  
  
所有业务状态必须持久化。  
  
禁止：  
  
- LocalStorage 作为业务存储  
- React State 作为业务存储  
- 内存对象作为唯一状态  
  
允许：  
  
- UI 状态  
- 临时缓存  
- 查询缓存  
  
所有核心数据最终必须写入数据库。  
  
---  
  
# 6. Reader 管线规范  
  
Reader 是项目核心能力。  
  
固定流程：  
  
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
  
React 禁止：  
  
- Readability 提取  
- Markdown 转换  
- Feed 解析  
  
所有内容处理必须在 Rust Core 完成。  
  
---  
  
# 7. Agent Runtime 规范  
  
所有 Agent 必须实现统一状态机。  
  
状态：  
  
Idle  
Running  
Succeeded  
Failed  
Cancelled  
  
允许：  
  
Idle → Running  
Running → Succeeded  
Running → Failed  
Running → Cancelled  
  
禁止：  
  
Idle → Succeeded  
Failed → Running  
  
重试必须创建新的 Run。  
  
---  
  
# 8. Prompt 管理规范  
  
Prompt 属于资源文件。  
  
目录：  
  
resources/prompts/  
  
禁止：  
  
- 硬编码 Prompt  
- 运行时修改内置 Prompt  
  
允许：  
  
- 用户覆盖 Prompt  
- 用户新增 Prompt  
  
内置 Prompt 永远只读。  
  
---  
  
# 9. Command 边界  
  
所有业务能力必须通过 Tauri Command 暴露。  
  
统一调用路径：  
  
React  
↓  
invoke()  
↓  
Tauri Command  
↓  
Rust Core  
  
React 禁止：  
  
- 直接访问 SQLite  
- 直接访问文件系统  
- 直接执行 AI 请求  
- 直接执行 Feed 同步  
  
业务请求统一由 Rust Core 负责。  
  
---  
  
# 10. Repository 规范  
  
数据库访问路径：  
  
Command  
↓  
Service  
↓  
Repository  
↓  
SQLite  
  
禁止：  
  
Command 中直接执行 SQL。  
  
所有数据库操作必须经过 Repository。  
  
---  
  
# 11. 测试规范  
  
要求：  
  
- cargo test 全部通过  
- 无 panic  
- 测试结果可重复  
  
推荐：  
  
- SQLite :memory:  
- tokio::test  
- timeout 异步测试  
  
禁止：  
  
thread::sleep()  
  
---  
  
# 12. ADR 规范  
  
以下情况必须创建 ADR：  
  
- 数据库 Schema 修改  
- 新 Agent 类型  
- 新 Provider  
- 新协议  
- 新平台能力  
- 架构调整  
  
原则：  
  
无 ADR 不合并。  
  
---  
  
# 13. 代码规范  
  
沟通语言：  
  
中文  
  
代码注释：  
  
英文  
  
文档：  
  
中文优先  
  
禁止：  
  
- Emoji  
- 模糊命名  
- 魔法数字  
  
日志：  
  
使用 tracing  
  
禁止：  
  
println!  
eprintln!  
  
---  
  
# 14. 完成定义（Definition of Done）  
  
功能标记完成前必须满足：  
  
- cargo build 成功  
- cargo test 成功  
- cargo clippy 无错误  
- TypeScript 检查通过  
- 无 panic  
- 数据可持久化  
- 三平台兼容  
  
否则：  
  
不得标记完成。  
  
---  
  
# 15. 核心原则总结  
  
项目目标：  
  
复刻 Mercury。  
  
不是重新设计 Mercury。  
  
当存在疑问时：  
  
优先参考 Mercury 行为。  
  
不要根据个人喜好修改产品逻辑。  
