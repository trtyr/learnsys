# 学习系统 (learnsys)

> 一个本地优先的**个人学习系统**：你定义目标、拆路径、记卡片、复习、看进度——**你是一等操作者**。
> AI 是**可选的平等客户**，想用也能走同一套 REST API（讲解、出题、判掌握度），但系统不依赖它。

**设计哲学**：确定性的归平台（SM-2 调度、统计聚合、存储），需要理解和生成的归 AI（可选）。

## 它是什么

一个"人跟人学习一样"的自学工作台，核心闭环：

```text
目标 → 路径 → 模块 → 卡片 → 复习 → 掌握度 → 画像
```

- **记**：顶部「＋记卡 / 开目标 / 记笔记」，一键落库，实时反映到今日时间线
- **学**：新卡队列 + 每日预算，节奏可控，不会一上来几十张糊脸
- **复习**：SM-2 间隔重复，翻卡自评 0-5 分，自动排下次到期
- **看**：今日时间线、连续天数 streak、复习热力图、掌握度分布

## 核心特性

| 层 | 能力 |
|----|------|
| 内容 | 卡片增删改查、全文搜索、标签、代码块 / 图片多模态 |
| 规划 | 目标 → 路径 → 模块 的树，含依赖检查、内联编辑、级联删除 |
| 调度 | SM-2 + 新卡 / 复习分离 + 每日新卡预算 + leech 顽固卡识别 |
| 体验 | 今日时间线、提醒红点、streak、测验抽取 |
| 数据 | JSON / markdown 全量导出、SQLite 快照备份 |
| 部署 | 单容器 Docker，本地优先，Docker 最后 |

## 架构

```text
你（人，一等操作者） ── 可选：AI Agent（平等客户）
        │                      │
        └─────── REST API ─────┘
                   │
           学习系统平台 ── SQLite
                   │
          React 工作台（今天 / 学习库 / 回顾）
```

| 组件 | 路径 | 职责 |
|------|------|------|
| 服务层 | `crates/learnsys-api` | axum REST（约 40 端点）+ 静态托管 |
| 领域层 | `crates/learnsys-core` | 实体 · SM-2 · 仓储 · 聚合 · 导出/备份 |
| 迁移工具 | `crates/learnsys-migrate` | markdown → SQLite 导入 |
| 前端工作台 | `frontend/` | React + Vite（今天 / 学习库 / 回顾 + 快捷记录） |

依赖方向单向：`api → core`、`migrate → core`；前端只走 REST，绝不直接碰 SQLite。

## 技术栈

| 层 | 栈 |
|----|----|
| 后端 | Rust (edition 2021) + axum 0.7 + tokio |
| 存储 | SQLite（rusqlite bundled，schema v7） |
| 前端 | React 18 + Vite 7 + TypeScript |
| 部署 | Docker 多阶段，单容器 |

## 快速开始

需要 Rust + Node，或直接 Docker。

### 开发（前后端分离）

```bash
# 后端
cargo run -p learnsys-api              # → http://127.0.0.1:7878

# 前端（另开终端）
cd frontend && npm install && npm run dev   # → http://localhost:5173（/api 代理到后端）
```

数据默认 `~/Library/Application Support/learnsys/learnsys.db`，可用 `RECALL_DB=/path/to.db` 覆盖。

### 导入现有数据（可选）

```bash
cargo run -p learnsys-migrate -- ~/.pi/learning-data
# 导入 cards/<topic>/*.md + progress.md（幂等，可重跑）
```

### Docker（推荐，单容器）

```bash
docker compose up -d --build   # 构建 + 启动
# → http://localhost:7878（看板 + API 同端口）
```

数据落在宿主机 `~/Library/Application Support/learnsys`，复用现有数据；`LEARNSYS_BIND` / `LEARNSYS_STATIC_DIR` 可覆盖默认值。

## API 概览

`review` 是 SM-2 调度的唯一入口，原子更新卡 + 追加复习记录。全部端点：

### Cards / Topics / 聚合

| Method | Path | 作用 |
|--------|------|------|
| POST | `/api/cards` | 建卡 `{topic, front, back, tags?, code_block?, image_urls?, source?, related?}`（topic 用名，不存在自动建主题） |
| GET | `/api/cards` | 列卡 `?topic=` |
| GET | `/api/cards/due` | 今日待复习 `?topic=` |
| GET | `/api/cards/search` | 搜索 `?q=` 匹配正面 / 背面 / 标签 / 出处 |
| GET | `/api/cards/new` | 今日新卡（默认 `new_per_day`，复习消耗预算） |
| GET | `/api/cards/leeches` | 顽固卡列表（EF<1.5 或连续失败 ≥4） |
| GET / PUT / DELETE | `/api/cards/:id` | 取 / 编辑 / 删卡（编辑不改 SM-2，可改标签/代码块/图片/出处） |
| POST | `/api/cards/:id/review` | 记录复习 `{quality:0-5}` → SM-2 |
| POST / GET | `/api/topics` | 建 / 列主题 |
| GET / PUT | `/api/topics/:id` | 取 / 更新主题 |
| GET | `/api/stats` | 总卡片 / 待复习 / 平均 EF / 主题分布 |
| GET | `/api/stats/heatmap` | 复习热力 `?days=` |
| GET | `/api/stats/upcoming` | 未来 `?days=` 天每天到期数（排期预测） |
| GET | `/api/stats/weak-topics` | 薄弱点聚类（leech / 低 EF 按主题） |
| GET | `/api/dashboard` | 看板聚合（待复习 + 进行中主题 + 预警） |

### LMS（目标 → 路径 → 模块 → 会话 → 画像）

| Method | Path | 作用 |
|--------|------|------|
| POST / GET | `/api/goals` | 建 / 列目标 |
| GET / PUT / DELETE | `/api/goals/:id` | 取 / 重命名 / 删目标（级联删路径） |
| PUT | `/api/goals/:id/status` | 更新目标状态 |
| GET | `/api/goals/:id/progress` | 目标进度（模块完成度） |
| POST / GET | `/api/pathways` | 建路径 / 列路径 `?goal=` |
| GET / PUT / DELETE | `/api/pathways/:id` | 取 / 重命名 / 删路径 |
| POST / GET | `/api/pathways/:id/modules` | 挂 / 列路径模块（含依赖） |
| GET | `/api/pathways/:id/next` | 下一个可学模块（依赖检查） |
| POST / GET | `/api/modules` | 建 / 列模块 |
| PUT / DELETE | `/api/modules/:id` | 重命名 / 删模块（卡片降为散卡） |
| GET | `/api/modules/:id/mastery` | 模块掌握度聚合 |
| GET | `/api/modules/:id/cards` | 模块下的卡片列表 |
| PUT | `/api/modules/:id/status` | 更新模块状态 |
| POST | `/api/sessions/start` | 开学习会话 |
| POST | `/api/sessions/:id/end` | 结会话（summary / new_cards / reviewed） |
| GET | `/api/sessions` | 列会话 `?limit=` |
| POST / GET | `/api/resources` | 建 / 列学习资源（笔记） |
| GET / PUT | `/api/profile` | 读 / 写学习者画像 |

### 设置 / 测验 / 导出 / 备份

| Method | Path | 作用 |
|--------|------|------|
| GET / PUT | `/api/settings` | 读 / 写设置（`new_per_day` 每日新卡预算，默认 5） |
| GET | `/api/quiz` | 测验抽取 `?n=&topic=`（随机抽到期复习卡） |
| GET | `/api/export` | 全量 JSON 导出 |
| GET | `/api/export/markdown` | markdown 导出（migrate 兼容） |
| POST | `/api/backup` | SQLite 一致性快照备份（`VACUUM INTO`） |
| GET | `/api/timeline` | 今日活动时间线（建卡 + 复习 + 会话） |

## 项目结构

```text
crates/
  learnsys-core/      领域层：entity / sm2 / repo / schema / db
  learnsys-api/       服务层：main.rs（路由 + handlers + 错误映射）
  learnsys-migrate/   markdown 导入 CLI
frontend/
  src/                api.ts（类型化客户端）/ types.ts / App.tsx / 组件
docs/plantree/        规划与决策（权威）
scripts/e2e.sh        全链路验证脚本
```

## 开发与测试

```bash
# 后端：单测 + lint + 格式
cargo test --workspace
cargo clippy --workspace --all-targets
cargo fmt --check

# 前端：类型检查 + lint + 单测 + 构建
cd frontend
npm run typecheck
npm run lint
npm test
npm run build

# 全链路（单测 → 迁移 → 后端 → API → review → 前端构建）
./scripts/e2e.sh
```

CI 在 [.github/workflows/ci.yml](.github/workflows/ci.yml)，push 到 `master` 自动跑上述检查。

## 文档

- [DESIGN.md](DESIGN.md) — 视觉设计（split-flap 出发板）
- [PRODUCT.md](PRODUCT.md) — 产品定位与原则
- [AGENTS.md](AGENTS.md) — 给未来 agent 的操作契约
- [docs/plantree/](docs/plantree/README.md) — 规划、决策、路线图（权威）
