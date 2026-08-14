# Topic: 内容层补全（Phase G）

> capsule：补上内容层四个硬缺口——卡片编辑、搜索、标签、多模态。当前只能建/删不能改、搜不到、打不了标、front/back 纯文本。

## 现状缺口

| 缺口 | 现状 | 为什么疼 |
|------|------|---------|
| 卡片编辑 | 只有 POST 建 / DELETE 删，无 PUT | 写错一个字得整张删了重建 |
| 搜索 | 无任何搜索端点 | 卡片上百后查"记没记过这个点"查不到 |
| 标签 | 无 tags 字段 | 只能靠 topic 粗分类，跨主题关联做不到 |
| 多模态 | front/back 纯 TEXT | 代码块、图、音频没地方放 |

## 方案（选项 + 推荐）

### G1 卡片编辑

- 新增 `PUT /api/cards/:id`，body `{front?, back?, topic?, tags?}` 全可选。
- **推荐**：改 front/back/tags **不重置 SM-2 调度**（是内容修正，不是重学）；改 `topic` 只是重新挂靠父主题，统计归属随之变化；只刷 `updated` 时间戳。

### G2 搜索

- 新增 `GET /api/cards/search?q=`，匹配 front/back/tags。
- 选项 A：`LIKE '%q%'`——简单，中文子串匹配天然可用，数据量小（<1 万卡）够用。
- 选项 B：SQLite FTS5——快，但 `rusqlite bundled` 是否编入 FTS5 需验证，且 FTS5 默认 tokenizer 对中文切词差（需 trigram tokenizer）。
- **推荐**：先 A（LIKE + 可选 topic 过滤），scale 不够再上 B。中文为主，LIKE 子串匹配反而是最符合直觉的。

### G3 标签

- 选项 A：`Card.tags` 存 JSON 数组（TEXT 列）——零新表，单用户够用，查询用 LIKE。
- 选项 B：独立 `tags` 表 + `card_tags` 关联表——正规化，支持去重/统计。
- **推荐**：先 A（JSON 数组），正规化待需求驱动。schema 升 v3。

### G4 多模态

- 选项 A：加几个可选字段 `code_block` / `image_urls` / `audio_url`——轻量。
- 选项 B：统一 `content_blocks`（JSON 有序块）——灵活但重。
- **推荐**：先 A，加 `code_block: Option<String>` + `image_urls: Option<Vec<String>>`，够覆盖"学 Rust 记代码 + 配图"。

## 验收标准

- `PUT /api/cards/:id` 改 front 后，due/ef/interval/reps 不变，`updated` 变。
- `GET /api/cards/search?q=borrow` 返回 front/back 含 "borrow" 的卡。
- 打标后 `GET /api/cards/:id` 返回 tags；按 tag 搜索命中。
- schema 升 v3 且旧库迁移无丢数据（`ensure_column` 幂等）。

## 影响面

- `schema.rs`（v3 迁移：cards 加 tags/code_block/image_urls）
- `entity.rs`（Card 加字段）、`repo.rs`（update/search）、`main.rs`（2 新端点）
- `frontend/api.ts` + `App.tsx`（编辑 UI + 搜索框 + 标签展示）

## 风险 / 待定

- 编辑是否允许改 topic、是否重置 SM-2 → 见 [OQ-7](../open-questions.md#oq-7-卡片编辑范围)
- 搜索 FTS5 vs LIKE → 见 [OQ-8](../open-questions.md#oq-8-搜索引擎)
- 标签建模 → 见 [OQ-9](../open-questions.md#oq-9-标签建模)
- 多模态字段 → 见 [OQ-10](../open-questions.md#oq-10-多模态建模)
