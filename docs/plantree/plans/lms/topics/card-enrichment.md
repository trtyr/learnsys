# Topic: 卡片内容层增肥（card-first）

> capsule：聚焦卡片内容层，把天天用的"肉"长起来。P0 建卡入口补齐 + 出处 + 标签浏览；P1 卡片关联；P2 复习洞察。

## P0：建卡入口补齐 + 标签 + 出处（先做）

现状：tags / code_block / image_urls 字段数据模型里早有了，但**只在"编辑"能填，"建卡"填不进去**。

- 后端 `CreateCard` DTO 补 `tags` / `code_block` / `image_urls` / `source`
- CLI `card create` 加 `--tags` `--code-block` `--image-urls` `--source`
- 前端 CaptureModal 加对应输入（标签、代码块、图片、出处）
- Card 加 `source` 字段（schema v6），记"来自哪个视频/文章/文档"
- 卡片库加**标签筛选** + 每张卡显示标签（按标签聚合浏览）

验收：一条命令能建一张带标签+代码块+出处的卡；卡片库能按标签筛。

## P1：知识连接

- Card 加 `related` 字段（卡片 id 列表，双向链接）
- 卡片库 / 详情里"相关卡片"区
- 让底层同源的卡（如"连接复用"与"所有权"同源 RAII）互相指

验收：能建一条卡↔卡的关联，并在界面上跳转。

## P2：复习洞察

- `GET /api/stats/upcoming?days=7`：未来 7 天每天要复习几张（排期预测）
- leech / 低 EF 卡按主题聚类：回答"你 Rust 哪块最弱"
- 语义搜索（最重，可能延后；先用标签 + 关联顶一阵）

## 边界

- Goal→Pathway→Module 骨架**不扩展、不删**（维持现状）。
- 不引入向量库/嵌入（语义搜索到 P2 再评估，届时可能只做"按标签+关联"的近似）。
