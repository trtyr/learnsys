# Topic: 调度层补强（Phase H）

> capsule：SM-2 本身没问题，但"学新东西"和"复习旧东西"混在一个 due 队列里，且顽固卡（leech）无管理。补新卡节奏 + leech 识别。

## 现状缺口

| 缺口 | 现状 | 为什么疼 |
|------|------|---------|
| 新卡/复习不分 | 新卡 interval=0，全部当天到期涌进 `/cards/due` | 一上来 21 张糊脸上，没有"每天学 N 张"节奏 |
| 无 leech 管理 | 反复答错的卡无标记，永远占到期队列 | 顽固卡拖累复习效率 |

## 方案（选项 + 推荐）

### H1 新卡 vs 复习分离

- 概念：卡分两个队列——**新卡队列**（从未复习过，interval=0）与**复习队列**（reps>0 且到期）。
- 选项 A：加全局/按主题的 `new_per_day` 预算，`/cards/due` 只返复习，新增 `GET /api/cards/new?limit=` 返新卡。
- 选项 B：给 Card 加 `state` 字段（new/learning/review），显式状态机。
- **推荐**：先 A（预算 + 拆端点），改动最小、语义清晰；state 状态机留到真要"学习中/重新学习"分级时再上。

### H2 leech（顽固卡）识别

- 定义：连续失败（quality<3）达阈值、或 EF 跌破地板附近，标记为 leech。
- 选项 A：派生标记——查询时算"最近 N 次 review 全失败"或"EF < 1.5"，不落库，dashboard 里加"顽固卡"面板。
- 选项 B：落库 `suspended` 标志 + 手动/自动暂停，移出到期队列。
- **推荐**：先 A（派生标记 + 展示），识别是平台的事、处置（重写/拆分/删除）归 AI/人——符合"平台只算不判"原则。

## 验收标准

- 设 `new_per_day=5` 后，`/cards/new` 只给 5 张，`/cards/due` 不再混入 interval=0 的新卡。
- dashboard 出现"顽固卡"计数：连续失败 ≥4 次或 EF<1.5 的卡被列出。

## 影响面

- `repo.rs`（due/new/leech 查询拆分）、`main.rs`（新端点 + dashboard 扩展）
- `schema.rs` 或纯派生（取决于 OQ-11/12）
- `frontend`（复习视图加"新卡"入口 + 顽固卡面板）

## 风险 / 待定

- 新卡节奏配置粒度 → 见 [OQ-11](../open-questions.md#oq-11-新卡节奏配置)
- leech 阈值 → 见 [OQ-12](../open-questions.md#oq-12-leech-阈值)
