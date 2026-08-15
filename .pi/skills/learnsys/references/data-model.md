# 数据模型

核心实体与关系。中文字段按 API 返回的 snake_case JSON 展示。

## 关系图

```text
Goal 1 ── N Pathway 1 ── N PathwayModule N ── 1 Module 1 ── N Card
                                                    │
Topic 1 ────────────────────────────── N Card ─────┘（card.topic 指向主题）
Card 1 ── N ReviewLog（复习流水）
Session（学习会话，可挂 goal/pathway）
Resource（学习资源/笔记，可挂 module/card）
LearnerProfile（单例画像，id=1）
```

## 实体字段

### Card（卡片，SM-2 最小单位）

| 字段 | 类型 | 说明 |
|------|------|------|
| `id` | str | `YYYY-MM-DD-<6位hex>` |
| `topic` | str | 主题 **id**（API 层会替换为名） |
| `front` / `back` | str | 正面（问题）/ 背面（答案） |
| `ef` | float | 难度系数 1.3–2.8，默认 2.5 |
| `interval` | int | 复习间隔（天） |
| `reps` | int | 连续成功次数（0 = 新卡） |
| `due` | str | 下次到期日 `YYYY-MM-DD` |
| `module_id` | str\|null | 所属模块（null = 散卡） |
| `tags` | list[str] | 标签 |
| `code_block` | str\|null | 代码块 |
| `image_urls` | list[str] | 配图 |
| `source` | str\|null | 出处（视频/文章/文档） |
| `related` | list[str] | 关联卡片 id（双向） |

### Topic（主题/领域）

`id / name / stage / status(active|completed|paused) / last_studied / next_plan / created`

### Goal（目标）

`id / title / description / success_criteria / topic / status(active|achieved|abandoned) / created / achieved_at`

### Pathway（路径，一条学习"路子"）

`id / name / methodology / description / goal_id / is_active / created`

### Module（模块，知识单元）

`id / title / topic / description / status(not_started|learning|mastered)`

### PathwayModule（路径↔模块关联，含顺序与依赖）

`pathway_id / module_id / sort_order / depends_on: list[str]`

### Session（学习会话）

`id / started_at / ended_at / goal_id / pathway_id / summary / new_cards / reviewed`

### Resource（资源/笔记）

`id / title / url / notes / module_id / card_id / created`

### ReviewLog（复习流水，不可变）

`id / card_id / quality / reviewed_at / prev_due / new_due / is_new`

### LearnerProfile（画像，单例 id=1）

`id / level / style / weak_points: list[str] / preferences: object / notes / updated`

## 关键规则

- **SM-2 语义**：quality 0-2 重置 reps、3-5 递增间隔；`review` 原子更新卡 + 追加 ReviewLog。
- **新卡 vs 复习**：`reps=0` 是新卡（走 `/cards/new`，受 `new_per_day` 预算，首次复习消耗预算）；`reps>0` 且到期走 `/cards/due`。
- **leech**：EF<1.5 或最近 4 次复习全失败（quality<3）标记，只展示不自动处置。
- **级联删除**：删 goal → 删其 pathway → 删 pathway_modules；删 module → 卡片降为散卡（module_id 置 null）；删 card → 删其 review_logs。
- **关联双向**：`related` 是双向链接——A 关联 B 时 B 自动反链 A；建卡/编辑/删卡都在事务内维护两端；自关联与悬空 id 会被自动过滤。
- **依赖**：pathway 内 `depends_on` 是"建议"而非强制——`next` 会跳过未掌握但可跳的前置。
