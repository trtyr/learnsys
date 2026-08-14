# Topic: 数据层（Phase I）

> capsule：数据只进不出——只有 markdown 单向导入，没有导出、没有备份。补 JSON/markdown 导出 + SQLite 快照备份，Anki 互通视需求。

## 现状缺口

| 缺口 | 现状 | 为什么疼 |
|------|------|---------|
| 无导出 | 只有 `learnsys-migrate` 单向导入 | 数据被锁死在 SQLite 里，换系统/看原始内容都难 |
| 无备份 | 单文件 SQLite 裸奔 | 误删/损坏 = 全丢，无快照无恢复 |

## 方案（选项 + 推荐）

### I1 导出

- 选项 A：`GET /api/export` 返回全量 JSON（topics/cards/goals/pathways/modules/sessions/profile）。
- 选项 B：CLI `learnsys-export` 导出 markdown（与 migrate 的格式对齐，可回导）。
- **推荐**：A + B 都做——JSON 是 API 契约（AI 也能拉），markdown 是人工可读 + 可与 migrate 闭环。

### I2 备份快照

- 用 SQLite 自带 `VACUUM INTO` / backup API 做在线一致性快照，不靠 `cp`（避免写一半的库）。
- **推荐**：新增 `POST /api/backup`（生成时间戳快照到 `backups/` 目录）或 CLI 子命令，二选一即可。

### I3 Anki 互通（可选，Deferred）

- apkg 是 zip+sqlite 复合格式，需评估是否引外部 crate。
- **推荐**：先 JSON 导出当桥；apkg 等真有"导入 Anki"需求再评估。→ 关联 ideas inbox「导出 Anki apkg 格式」。

## 验收标准

- `GET /api/export` 导出 JSON，删库后用 migrate 或导入脚本能重建核心数据。
- 快照能恢复到某时间点，且恢复后 cards/review_logs 完整。

## 影响面

- `main.rs`（export/backup 端点）、`repo.rs`（全量 dump）
- 可能新增 `crates/learnsys-export` 或并入 migrate（待 OQ-13）

## 风险 / 待定

- 导出格式优先级（JSON vs apkg）→ 见 [OQ-13](../open-questions.md#oq-13-导出格式优先级)
