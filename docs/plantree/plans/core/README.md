# Plan: core — 核心平台构建

学习系统 的唯一计划：把"学习数据平台"从 0 做到单机可用、Docker 可部署。

## Scope

- ✅ **In**：后端 API（Rust）、数据模型、SM-2 调度、前端只读看板、Docker 打包
- ❌ **Out（Deferred）**：网页交互式复习、MCP 适配层、多用户/认证、云部署、移动端

完整边界见 [roadmap.md](roadmap.md) 的 Deferred。

## 文件地图

| 文件 | 作用 |
|------|------|
| [roadmap.md](roadmap.md) | 分阶段路线图 + 当前状态 |
| [open-questions.md](open-questions.md) | 待决策项（每个带推荐） |
| [decisions/](decisions/) | 已定决策 |
| [topics/](topics/) | 架构 / 数据模型 / API 设计 capsule |

## 阅读路径

1. 本 README（scope）
2. [roadmap.md](roadmap.md)（看当前在哪、下一步）
3. [open-questions.md](open-questions.md)（看还有什么没定）
4. 需要时读对应 [topics/](topics/)

## 当前状态

- **Phase**：🟢 MVP（Phase 0–3）已交付
- **Next**：Phase 4（Pi 接入验证端到端学习闭环）/ Phase 5（Docker 包装）
- **实现选型**：axum 0.7 + rusqlite + React18+Vite5（OQ-7/8 已落实，见 [open-questions.md](open-questions.md)）

## Resume

接手时按序读：本 README → roadmap → open-questions → 当前 phase 的 topic 文件。
