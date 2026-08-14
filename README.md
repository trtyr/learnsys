# 学习系统

> headless 学习数据平台：负责**记录、存储、调度**，自身没有 AI。
> 所有"需要智能"的活由 AI Agent 通过 REST API 完成。

原则：**确定性的归平台**（SM-2 调度、统计聚合），**需要理解和生成的归 AI**。

## 架构

```text
AI Agent (Pi / Claude / 脚本 … 都是平等客户)
   │  REST API（契约边界）
   ▼
学习系统 平台 ── SQLite ── 只读看板 (React)
```

| 组件 | 路径 | 职责 |
|------|------|------|
| 后端服务 | `crates/learnsys-api` | axum REST：cards / topics / goals / pathways / modules / sessions / stats |
| 领域层 | `crates/learnsys-core` | 实体 · SM-2 算法 · SQLite 仓储 · 聚合 |
| 迁移工具 | `crates/learnsys-migrate` | markdown → SQLite 导入 |
| 前端看板 | `frontend/` | React + Vite，只读学习舱 |

## 快速开始

需要 Rust + Node。

### 1. 起后端

```bash
cargo run -p learnsys-api
# → http://127.0.0.1:7878
```

数据默认在 `~/Library/Application Support/learnsys/learnsys.db`，
可用 `RECALL_DB=/path/to.db` 覆盖（Docker 挂卷也用它）。

### 2. 导入现有数据（可选）

```bash
cargo run -p learnsys-migrate -- ~/.pi/learning-data
# 导入 cards/<topic>/*.md + progress.md（幂等，可重跑）
```

### 3. 起前端看板

```bash
cd frontend && npm install && npm run dev
# → http://localhost:5173（/api 自动代理到后端）
```

### 4. 生产构建

```bash
cd frontend && npm run build   # → frontend/dist
cargo build --release -p learnsys-api
```

## API 速览

### Cards / Topics / 聚合

| Method | Path | 作用 |
|--------|------|------|
| POST | `/api/cards` | 建卡 `{topic, front, back}`（topic 用名，不存在自动建主题） |
| GET | `/api/cards` | 列卡 `?topic=` |
| GET | `/api/cards/due` | 今日待复习 `?topic=` |
| GET | `/api/cards/:id` | 取一张 |
| DELETE | `/api/cards/:id` | 删卡 |
| POST | `/api/cards/:id/review` | 记录复习 `{quality:0-5}` → SM-2 调度 |
| POST | `/api/topics` | 建主题 |
| GET | `/api/topics` | 列主题 |
| GET | `/api/topics/:id` | 取一个 |
| PUT | `/api/topics/:id` | 更新阶段/状态/下次计划 |
| GET | `/api/stats` | 总卡片 / 待复习 / 平均 EF / 主题分布 |
| GET | `/api/stats/heatmap` | 复习热力 `?days=` |
| GET | `/api/dashboard` | 看板聚合（待复习 + 进行中主题 + 预警） |

### LMS（目标 → 路径 → 模块 → 会话 → 画像）

| Method | Path | 作用 |
|--------|------|------|
| POST / GET | `/api/goals` | 建 / 列目标 |
| GET | `/api/goals/:id` | 取目标 |
| PUT | `/api/goals/:id/status` | 更新目标状态 |
| GET | `/api/goals/:id/progress` | 目标进度（模块完成度） |
| POST | `/api/pathways` | 建路径 `{name, goal_id}` |
| GET | `/api/pathways` | 列路径 `?goal=` |
| GET | `/api/pathways/:id` | 取路径 |
| POST / GET | `/api/pathways/:id/modules` | 挂 / 列路径模块（含依赖） |
| GET | `/api/pathways/:id/next` | 下一个可学模块（依赖检查） |
| POST / GET | `/api/modules` | 建 / 列模块 |
| GET | `/api/modules/:id/mastery` | 模块掌握度聚合 |
| PUT | `/api/modules/:id/status` | 更新模块状态 |
| POST | `/api/sessions/start` | 开学习会话 |
| POST | `/api/sessions/:id/end` | 结会话（summary / new_cards / reviewed） |
| GET | `/api/sessions` | 列会话 `?limit=` |
| POST / GET | `/api/resources` | 建 / 列学习资源 |
| GET / PUT | `/api/profile` | 读 / 写学习者画像（温和双向记忆） |

`review` 是 SM-2 调度的唯一入口，原子更新卡 + 追加复习记录。

## 验证与测试

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

## 规划

设计决策、路线图、数据模型在 [docs/plantree/](docs/plantree/README.md)。
