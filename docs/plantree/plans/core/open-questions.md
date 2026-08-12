# Open Questions · core

> MVP 大决策已收敛 → [decisions/002-mvp-decisions.md](decisions/002-mvp-decisions.md)。
> 下列为实现级窄问题，Phase 0–1 自然解决。

## OQ-7: 后端框架

- **候选**：axum（主流，tower 生态好）/ actix-web（成熟，性能强）
- **推荐**：axum
- Phase 0 搭脚手架时定

## OQ-8: SQLite 驱动

- **候选**：rusqlite（同步，简单）/ sqlx（异步，编译期 SQL 检查）
- **推荐**：rusqlite（单机、简单、依赖少）；若后端要全异步再考虑 sqlx

## OQ-9: API 细节

- 版本化前缀 `/api/v1`？—— 推荐加（低成本前瞻）
- 错误模型：统一 `{code,message}` + HTTP 状态码
- 分页：offset/limit 够用（数据量小）

## OQ-10: 实体取舍

- Session（学习会话）进 MVP 吗？—— 倾向 Phase 2 再加
- Tag（跨主题分类）？—— 倾向不要（topic 已够）

## 收敛动作

这些在 Phase 0–1 实现时直接定，定了就补进对应 decision 或 topic，不长期挂着。
