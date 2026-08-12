# Roadmap · lms

## Done

- 🟢 方向重定义：卡片工具 → AI 用的 LMS（[decision 001](decisions/001-lms-direction.md)）
- 🟢 数据模型设计：Goal / Pathway / Module / PathwayModule / Session / LearnerProfile（[topics/data-model.md](topics/data-model.md)）

## In Progress

- 🔵 收敛 [open-questions.md](open-questions.md) → schema 设计定稿

## Next（演进阶段）

| Phase | 目标 | 关键产出 | 验证 |
|-------|------|---------|------|
| A | schema 扩展 | Goal/Pathway/Module/PathwayModule/Session/Profile 建表 + 迁移 | 现有数据兼容（Card 加 module_id nullable） |
| B | 路径与计划 API | Goal/Pathway/Module CRUD + `next_module` 推进 + 依赖检查 | curl 建目标→路径→模块链路 |
| C | 进度与掌握度 | Mastery 派生聚合 + 进度报表 | 掌握度从卡片正确聚合 |
| D | 学习会话 | Session 记录 API + 会话报表 | 一次学习会话被完整记录 |
| E | 双向记忆 | LearnerProfile API（温和）+ 盲点辅助 | Profile 跨 session 读写 |
| F | 前端演进 | LMS 仪表盘：计划/路径树/进度/画像 | 浏览器看到完整学习管理视图 |

## Deferred

- ⚪ 知识图谱可视化
- ⚪ 路径模板库 / 分享格式
- ⚪ 多用户
- ⚪ 自动生成教学内容（永远 AI 干）

## 里程碑

- **LMS 核心（Phase A–C）**：能定义目标、规划路径、跟踪进度
- **闭环（Phase A–E）**：计划→学习→复习→掌握→画像 完整闭环
- **可视（Phase F）**：浏览器看到完整学习管理视图
