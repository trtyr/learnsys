# Decision 003: Phase G–J 补齐落地选择

**Status**: Accepted（由实现落地，以代码为准）
**Date**: 2026-08-14

Phase G–J（内容层/调度层/数据层/体验层）实现时对 OQ-7~15 的实际收敛结果。
权威以 `learnsys-core` / `learnsys-api` 代码为准，本记录做"决策对账"。

| OQ | 问题 | 落地选择 | 代码依据 |
|----|------|---------|---------|
| OQ-7 | 卡片编辑范围 | front/back/tags/topic/code_block/image_urls 全可改，SM-2 不重置 | `repo::update_card` |
| OQ-8 | 搜索引擎 | `LIKE '%q%'` 子串匹配（front/back/tags） | `repo::search_cards` |
| OQ-9 | 标签建模 | `Card.tags` JSON 数组（TEXT 列） | schema v3 `cards.tags` |
| OQ-10 | 多模态建模 | 加 `code_block` / `image_urls` 可选字段，前端可编辑可渲染 | schema v3 + `CardEditor`/`CardRow` |
| OQ-11 | 新卡节奏配置 | 全局 `new_per_day`（默认 5）；预算由**新卡首次复习**消耗（`review_logs.is_new`），`/api/cards/new` 返回剩余预算张 | `repo::new_per_day` / `new_introduced_today` |
| OQ-12 | leech 阈值 | 连续失败 ≥4 或 EF<1.5，只标记不处置 | `repo::leech_cards` |
| OQ-13 | 导出格式 | JSON + markdown 都做（含 tags/code_block/image_urls，migrate 可回导）；apkg Deferred | `repo::export_all` / `export_markdown` |
| OQ-14 | 提醒形态 | 看板内红点/badge（header 待出发/延误/顽固/streak） | 前端 header-stats |
| OQ-15 | 测验题型 | 问答：平台**机械抽题**（`/api/quiz` 抽 due 卡），问答与判分由 AI 走 `/api/cards/:id/review`（前端复用自评按钮） | `repo::quiz_cards` |

## 新增端点（Phase G–J）

`PUT /api/cards/:id` · `GET /api/cards/search` · `GET /api/cards/new` · `GET /api/cards/leeches` · `GET /api/quiz` · `GET/PUT /api/settings` · `GET /api/export` · `GET /api/export/markdown` · `POST /api/backup`

## 关联

- [roadmap.md](../roadmap.md)
- [decision 002](002-lms-implementation-choices.md)
