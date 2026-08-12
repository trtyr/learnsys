# Topic: API 契约 · 学习系统

> capsule：REST 端点草案（OQ-2 待确认 REST 为核心）。
> 细节在 Phase 1 实现时定稿。

## Cards

| Method | Path | 作用 |
|--------|------|------|
| POST | /api/cards | 建卡片（AI 讲完一个点后调用） |
| GET | /api/cards/due | 今日待复习（核心，看板/API 都用） |
| GET | /api/cards/:id | 取一张 |
| GET | /api/cards?topic=X | 按主题列 |
| POST | /api/cards/:id/review | 记录复习（body: `{quality:0-5}`）→ 平台算下次到期 |
| DELETE | /api/cards/:id | 删卡 |

## Topics / Progress

| Method | Path | 作用 |
|--------|------|------|
| POST | /api/topics | 建主题 |
| GET | /api/topics | 列主题 + 状态 |
| PUT | /api/topics/:id | 更新阶段/状态/下次计划 |
| GET | /api/dashboard | 聚合：待复习数 + 进行中主题 + 到期预警（看板专用） |

## Stats

| Method | Path | 作用 |
|--------|------|------|
| GET | /api/stats | 总卡片/待复习/平均EF/按主题分布 |
| GET | /api/stats/heatmap | 复习/学习热力（待定） |

## 设计约定

- 纯 JSON，无认证（单机本地）
- `review` 是**唯一**触发 SM-2 调度的入口，原子更新 Card + 追加 ReviewLog
- `/dashboard` 是为前端看板优化的聚合端点，减少往返

## 开放点

- 是否版本化前缀 `/api/v1`
- 错误模型（统一 `{code,message}` 还是 HTTP 状态码为主）
- 分页策略
