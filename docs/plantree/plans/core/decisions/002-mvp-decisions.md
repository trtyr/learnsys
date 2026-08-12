# Decision 002: MVP 范围与实现选型

**Status**: Accepted（用户确认，2026-08-12）
**Date**: 2026-08-12

收敛 [open-questions.md](../open-questions.md) 的 OQ-1 ~ OQ-6。

## 决定

| 项 | 决定 | 理由 |
|----|------|------|
| 项目名 | **学习系统** | active 学习系统 主动检索，crate/目录名干净 |
| API 形态 | **REST 核心** | 最通用，前端看板/curl/任意 agent 都能调；MCP 留后续适配 |
| 存储 | **SQLite 主**，markdown 作 import/export | 单机零配置、单文件、强查询；markdown 承接现有数据迁移 |
| 前端 | **React + Vite** | 生态最大最稳；用户选定 |
| 展示深度 | **MVP 只读看板** | 先把"看"做扎实；交互复习 Deferred |
| 数据迁移 | **一次性导入脚本**（Phase 0） | 把现有 20 张 Rust 卡 + progress 导入，兼作格式验证 |

## Deferred（后续考虑）

- MCP 适配层：REST 稳定后再包薄层，让 Pi 等原生 agent 零摩擦接入
- 网页交互式复习：翻卡 + 打分（涉及与 AI 分工）
- markdown 双向同步（导入单向，导出可选）

## 影响

- Phase 0 明确：Rust workspace + SQLite schema + 迁移脚本
- Phase 3 前端用 React + Vite 搭只读看板
- 后端框架 / SQL 驱动等实现选型留 [open-questions.md](../open-questions.md)（Phase 0 定）

## 关联

- [Decision 001: 技术栈](001-tech-stack.md)
- [topics/architecture.md](../topics/architecture.md)
