# Topic: 体验层 + 测验（Phase J）

> capsule：看板目前"打开才知道状态"，缺提醒、缺激励、缺会话回看；测验机械抽取在架构里声明归平台但没实现。补提醒 + streak + 会话时间轴 + 测验抽取。

## 现状缺口

| 缺口 | 现状 | 为什么疼 |
|------|------|---------|
| 无提醒 | 只能主动打开看板看 | 到期了不知道，错过复习窗口 |
| 无激励 | 有 heatmap 但无 streak 计数 | 缺少"连续 X 天"的正反馈 |
| 会话回看弱 | sessions 有表但前端展示薄 | "今天学过啥"看不到 |
| 测验缺失 | 架构写"机械抽取归平台"但无端点 | AI 要抽题还得自己拼 |

## 方案（选项 + 推荐）

### J1 提醒

- 选项 A：看板内红点/badge（逾期计数高亮）——零依赖。
- 选项 B：macOS 系统通知（`osascript` 或 notify crate）——单机可做但侵入。
- **推荐**：先 A（dashboard 已算 overdue，加个显眼的红点 + "今日待复习 N"），B 视痛点再上。

### J2 streak 连续天数

- review_logs 已存 `reviewed_at` 日期，**纯派生**，无需新表。
- **推荐**：从 review_logs 算"连续复习天数 + 今日是否已学"，并入 dashboard。

### J3 会话时间轴

- sessions 表已存在，前端 ProgressView 已有部分展示。
- **推荐**：前端补一条"今日/最近会话"时间轴（起止时间 + summary + 新卡/复习数），后端可能只需 `GET /api/sessions` 已有数据。

### J4 测验机械抽取

- 架构声明"测验题目的机械抽取"归平台（[architecture.md](../../core/topics/architecture.md)）。
- **推荐**：`GET /api/quiz?n=5&topic=` 从 due 卡里机械抽 N 张返回 front，AI 负责问答与判分（判分 = 调 `/review` 喂 quality）。本质是 `/cards/due` 的薄封装。

## 验收标准

- dashboard 显示 streak + 今日是否已学。
- 前端出现会话时间轴。
- `GET /api/quiz?n=5` 返回 ≤5 张待复习卡，AI 可据此出题。

## 影响面

- `repo.rs`（streak 派生、quiz 抽取）、`main.rs`（quiz 端点）
- `frontend/App.tsx`（提醒红点、streak、时间轴）

## 风险 / 待定

- 提醒形态（本地通知 vs 看板红点）→ 见 [OQ-14](../open-questions.md#oq-14-提醒形态)
- 测验题型（问答 vs 选择）→ 见 [OQ-15](../open-questions.md#oq-15-测验题型)
