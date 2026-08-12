# Decision 003: MVP 交付（Phase 0–3）

**Status**: Shipped
**Date**: 2026-08-12

## 交付内容

- **Phase 0**：cargo workspace（recall-core / recall-api / recall-migrate）+ SQLite schema（topics/cards/review_logs）+ markdown 迁移工具（20 张卡导入验证）
- **Phase 1**：SM-2 算法移植（8 单测对拍 Python）+ 仓储层 + cards REST API（POST/GET/due/:id/review/DELETE）
- **Phase 2**：topics CRUD + stats 聚合 + dashboard 聚合端点
- **Phase 3**：React + Vite 只读看板（统计卡 + 进行中主题 + 待复习列表 🔴🟢 + 主题分布）

## 验证

- `cargo test --workspace`：recall-core 16 用例全过（entity/schema/sm2/repo）
- e2e：迁移 20 卡 → 后端全端点 → review q=5 得 reps=1/ef=2.6/due+1 → 前端 dist 产出
- dev proxy 打通（Vite :5173 → 后端 :7878）

## 实现选型（落实 OQ-7/8）

- 后端框架：axum 0.7
- SQLite 驱动：rusqlite（bundled）
- 前端：React 18 + Vite 5

## 未做（Deferred，属 Phase 4/5）

- Pi 接入端到端学习闭环（Phase 4）
- Docker 容器化（Phase 5）
- 网页交互式复习、MCP 适配层、复习热力（待 review_logs 积累）

## 关联

- [Decision 001: 技术栈](001-tech-stack.md)
- [Decision 002: MVP 范围](002-mvp-decisions.md)
