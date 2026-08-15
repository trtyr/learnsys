# API 参考

所有端点前缀 `/api`，纯 JSON，无认证（单机本地）。`review` 是 SM-2 调度的唯一入口。

## Cards

| Method | Path | 说明 | Body / 参数 |
|--------|------|------|-------------|
| POST | `/api/cards` | 建卡（topic 用名，不存在自动建） | `{topic, front, back, tags?, code_block?, image_urls?, source?}` |
| GET | `/api/cards` | 列卡 | `?topic=`（主题名） |
| GET | `/api/cards/due` | 今日待复习（reps>0 且到期） | `?topic=` |
| GET | `/api/cards/search` | 搜索正面/背面/标签 | `?q=`（必填）、`?topic=` |
| GET | `/api/cards/new` | 今日新卡（每日预算，复习消耗） | — |
| GET | `/api/cards/leeches` | 顽固卡（EF<1.5 或连续失败≥4） | — |
| GET | `/api/cards/:id` | 取一张 | — |
| PUT | `/api/cards/:id` | 编辑（不改 SM-2） | `{front?, back?, topic?, tags?, code_block?, image_urls?, module_id?}` |
| DELETE | `/api/cards/:id` | 删卡 | — |
| POST | `/api/cards/:id/review` | 复习 → SM-2 调度 | `{quality: 0-5}` |

`module_id` 用空串表示"脱离模块"（降为散卡）。

## Topics

| Method | Path | 说明 | Body |
|--------|------|------|------|
| POST | `/api/topics` | 建主题 | `{name}` |
| GET | `/api/topics` | 列主题 | — |
| GET | `/api/topics/:id` | 取一个 | — |
| PUT | `/api/topics/:id` | 更新 | `{stage?, status?, next_plan?, last_studied?}` |

`status` ∈ `active / completed / paused`。

## Goals

| Method | Path | 说明 | Body |
|--------|------|------|------|
| POST | `/api/goals` | 建目标 | `{title, description?, success_criteria?, topic?}` |
| GET | `/api/goals` | 列目标 | — |
| GET | `/api/goals/:id` | 取目标 | — |
| PUT | `/api/goals/:id` | 重命名/改 | `{title?, description?, success_criteria?}` |
| DELETE | `/api/goals/:id` | 删目标（级联删路径） | — |
| PUT | `/api/goals/:id/status` | 更新状态 | `{status, achieved_at?}` |
| GET | `/api/goals/:id/progress` | 进度（模块完成度） | — |

`status` ∈ `active / achieved / abandoned`。

## Pathways

| Method | Path | 说明 | Body |
|--------|------|------|------|
| POST | `/api/pathways` | 建路径 | `{name, goal_id, methodology?, description?}` |
| GET | `/api/pathways` | 列路径 | `?goal=`（必填） |
| GET | `/api/pathways/:id` | 取路径 | — |
| PUT | `/api/pathways/:id` | 重命名/改 | `{name?, methodology?, description?}` |
| DELETE | `/api/pathways/:id` | 删路径（级联删模块序列） | — |
| POST | `/api/pathways/:id/modules` | 挂模块 | `{module_id, sort_order, depends_on?}` |
| GET | `/api/pathways/:id/modules` | 列路径模块 | — |
| GET | `/api/pathways/:id/next` | 下一个可学模块（依赖检查） | — |

## Modules

| Method | Path | 说明 | Body |
|--------|------|------|------|
| POST | `/api/modules` | 建模块 | `{title, topic?, description?}` |
| GET | `/api/modules` | 列模块 | `?topic=` |
| PUT | `/api/modules/:id` | 重命名 | `{title?, description?}` |
| DELETE | `/api/modules/:id` | 删模块（卡片降为散卡） | — |
| GET | `/api/modules/:id/mastery` | 掌握度聚合 | — |
| GET | `/api/modules/:id/cards` | 模块下的卡片 | — |
| PUT | `/api/modules/:id/status` | 更新状态 | `{status}` |

`status` ∈ `not_started / learning / mastered`。

## Sessions / Resources / Profile

| Method | Path | 说明 | Body |
|--------|------|------|------|
| POST | `/api/sessions/start` | 开会话 | `{goal_id?, pathway_id?}` |
| POST | `/api/sessions/:id/end` | 结会话 | `{summary?, new_cards?, reviewed?}` |
| GET | `/api/sessions` | 列会话 | `?limit=` |
| POST | `/api/resources` | 建资源/笔记 | `{title, url?, notes?, module_id?, card_id?}` |
| GET | `/api/resources` | 列资源 | `?module_id=` |
| GET | `/api/profile` | 读画像 | — |
| PUT | `/api/profile` | 写画像 | `{level, style, weak_points, notes, ...}` |

## 设置 / 测验 / 统计 / 导出 / 备份

| Method | Path | 说明 |
|--------|------|------|
| GET | `/api/settings` | 读设置（`new_per_day` 等） |
| PUT | `/api/settings` | 写设置 `{new_per_day?}` |
| GET | `/api/quiz` | 测验抽取 `?n=&topic=` |
| GET | `/api/stats` | 总卡片/待复习/平均 EF/主题分布 |
| GET | `/api/stats/heatmap` | 复习热力 `?days=` |
| GET | `/api/dashboard` | 看板聚合（含 streak/leech_count） |
| GET | `/api/export` | 全量 JSON 导出 |
| GET | `/api/export/markdown` | markdown 导出（migrate 兼容） |
| POST | `/api/backup` | SQLite 快照备份（`VACUUM INTO`） |
| GET | `/api/timeline` | 今日活动（建卡+复习+会话，倒序） |

## 错误模型

统一 `{code, message}`：`not_found` / `db_error` / `date_error`，HTTP 状态码相应 404 / 500。
