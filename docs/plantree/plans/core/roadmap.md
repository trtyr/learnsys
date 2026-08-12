# Roadmap · core

## Done

- 🟢 架构方向确立：headless 平台 + AI 客户端，关注点分离
- 🟢 技术栈确立：Rust 后端 / 前后端分离 / 单机 / Docker 最后
- 🟢 现有资产盘点：Python 版逻辑可继承；20 张 Rust 卡可迁移

## In Progress

- 🟢 架构与数据模型定稿：[topics/architecture.md](topics/architecture.md)、[topics/data-model.md](topics/data-model.md)
- 🟢 MVP 决策收敛：[decisions/002-mvp-decisions.md](decisions/002-mvp-decisions.md)
- 🔵 进 Phase 0 前：实现选型（框架/驱动）见 [open-questions.md](open-questions.md)

## Next（按顺序）

| Phase | 目标 | 关键产出 | 验证 |
|-------|------|---------|------|
| 0 | 脚手架 + 数据模型 | Rust workspace、框架/驱动选型、SQLite schema、迁移脚本 | 能读入 20 张 Rust 卡 |
| 1 | 后端 API 核心 | cards CRUD + due + review + SM-2 调度 | curl 跑通 add/due/review 闭环 |
| 2 | 进度 + 统计 API | topics progress、stats、sessions、dashboard 聚合 | 统计接口返回正确聚合 |
| 3 | 前端看板（只读） | 今日待复习、进行中主题、统计图 | 浏览器打开看到"学习舱面板" |
| 4 | 接入验证 | Pi 通过 API 调用，跑通真实学习闭环 | 端到端：学→做卡→复习→看板 |
| 5 | Docker 包装 | 本地测好后，容器化前后端 | `docker compose up` 一键起 |

## Deferred

- ⚪ 网页交互式复习（翻卡 + 打分）
- ⚪ MCP 适配层（让 Pi 等 agent 零摩擦接入）
- ⚪ 多用户 / 认证
- ⚪ 云部署 / SaaS
- ⚪ 移动端
- ⚪ 导出 Anki apkg 互通格式

## 里程碑定义

- **MVP（Phase 0–3）**：单机本地，后端 API + 只读看板跑通，现有数据迁入
- **闭环可用（Phase 4）**：Pi 真能用它跑完整学习闭环
- **可分发（Phase 5）**：Docker 一键部署，换机器能跑
