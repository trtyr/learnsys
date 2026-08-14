# Roadmap · lms

## Done

- 🟢 方向重定义：卡片工具 → AI 用的 LMS（[decision 001](decisions/001-lms-direction.md)）
- 🟢 数据模型设计：Goal / Pathway / Module / PathwayModule / Session / LearnerProfile（[topics/data-model.md](topics/data-model.md)）
- 🟢 **Phase A**：schema 扩展 + 6 新实体 + repo CRUD（commit `41bf519`）
- 🟢 **Phase B**：goals / pathways / modules / next REST API（commit `f885feb`）
- 🟢 **Phase C/D/E**：mastery / sessions / profile API（commit `9048256`）
- 🟢 **Phase F**：前端 LMS 仪表盘（commit `4e44573`）
- 🟢 视觉重设计 + 六大功能补齐（commit `9b863df` / `cf33dad`）
- 🟢 **Phase G**：内容层补全（卡片编辑 / 搜索 / 标签 / 多模态）
- 🟢 **Phase H**：调度层补强（新卡复习分离 / leech 管理）
- 🟢 **Phase I**：数据层（JSON/markdown 导出 + SQLite 快照备份）
- 🟢 **Phase J**：体验层（提醒红点 / streak / 会话时间轴 / 测验抽取）
- 🟢 **工作台重构**：定位翻转（decision 004）+ 前端工作台（今天 / 学习库 / 回顾 + 快捷记录）+ `/api/timeline`

> 落地选择对账：Phase A–F 见 [decision 002](decisions/002-lms-implementation-choices.md)，Phase G–J 见 [decision 003](decisions/003-gap-fill-implementation-choices.md)，工作台重构见 [decision 004](decisions/004-human-first-positioning.md)。

## Deferred

- ⚪ 多用户 / 认证
- ⚪ 知识图谱可视化
- ⚪ 路径模板库 / 分享格式
- ⚪ 移动端
- ⚪ 云同步
- ⚪ Anki apkg 互通（OQ-13，暂缓）
- ⚪ 自动生成教学内容（永远 AI 干）
- ⚪ 卡片双向自动配对（概念↔定义）

## 里程碑

- **LMS 核心（Phase A–F）✅**：定义目标、规划路径、跟踪进度、掌握度、画像、可视化
- **补齐（Phase G–J）✅**：内容/调度/数据/体验四层补强，12 项全部落地
