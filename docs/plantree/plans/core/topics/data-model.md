# Topic: 数据模型 · recall

> capsule：核心实体 + 字段草案。schema 在 Phase 0 定稿。
> 继承自现有 Python 版（`~/.pi/learning-data`），规范化为关系模型。

## 实体

### Card（知识卡片 · SM-2 调度单位）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | 卡片唯一 id |
| topic | TEXT | 所属主题（FK→Topic） |
| front | TEXT | 正面（问题） |
| back | TEXT | 背面（答案） |
| ef | REAL | SM-2 难度系数，初值 2.5 |
| interval | INT | 当前间隔(天)，初值 0 |
| reps | INT | 成功复习次数，初值 0 |
| due | DATE | 下次到期日 |
| created | DATE | 创建日 |
| updated | TIMESTAMP | 最后更新 |

### Topic（主题/学习计划单位）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| name | TEXT | 主题名 |
| stage | TEXT | 当前阶段（自由文本，如"模块2:公式"） |
| status | TEXT | active / completed / paused |
| last_studied | DATE | |
| next_plan | TEXT | 下次从哪开始 |

### ReviewLog（复习记录 · 不可变流水）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INT PK | |
| card_id | TEXT FK | |
| quality | INT | 0-5 自评 |
| reviewed_at | TIMESTAMP | |
| prev_due / new_due | DATE | 调度变化留痕 |

### Session（学习会话 · 可选）

记录一次学习会话的开始/结束/覆盖主题，供统计。

## 设计要点

- Card 的 SM-2 字段（ef/interval/reps/due）由平台 review 接口**原子更新**
- ReviewLog 只追加，用于统计/审计，不参与调度计算（调度状态在 Card）
- Topic.stage 是自由文本（承接现有 progress.md 的"阶段"），不做强 schema
- 迁移：现有 markdown 卡片 frontmatter → Card 表字段

## 开放点

- Session 是否进 MVP，还是 Phase 2 再加
- 是否需要 Tag（跨主题分类）
