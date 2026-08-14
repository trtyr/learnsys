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
| 前端工作台 | `frontend/` | React + Vite，个人学习工作台（今天 / 学习库 / 回顾 + 快捷记录） |

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

### 5. Docker（单容器）

一个镜像同时跑后端 + 内置前端静态文件。推荐用 compose：

```bash
docker compose up -d --build   # 构建 + 启动
docker compose down            # 停止
# → http://localhost:7878（前端看板 + API 同端口）
```

或手动 `docker run`：

```bash
docker build -t learnsys:latest .
docker run -d -p 7878:7878 -v "$HOME/Library/Application Support/learnsys:/data" learnsys:latest
```

数据落在宿主机 `~/Library/Application Support/learnsys`（复用现有数据）；可用 `LEARNSYS_BIND`、`LEARNSYS_STATIC_DIR` 覆盖默认值。

## API 速览

### Cards / Topics / 聚合

| Method | Path | 作用 |
|--------|------|------|
| POST | `/api/cards` | 建卡 `{topic, front, back}`（topic 用名，不存在自动建主题） |
| GET | `/api/cards` | 列卡 `?topic=` |
| GET | `/api/cards/due` | 今日待复习 `?topic=` |
| GET | `/api/cards/:id` | 取一张 |
| DELETE | `/api/cards/:id` | 删卡 |
| PUT | `/api/cards/:id` | 编辑卡片 `{front?, back?, topic?, tags?, code_block?, image_urls?}`（不改 SM-2） |
| POST | `/api/cards/:id/review` | 记录复习 `{quality:0-5}` → SM-2 调度 |
| GET | `/api/cards/search` | 搜索 `?q=` 匹配正面/背面/标签 |
| GET | `/api/cards/new` | 今日新卡 `?limit=`（默认 `new_per_day`） |
| GET | `/api/cards/leeches` | 顽固卡列表（EF<1.5 或连续失败 ≥4） |
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
| GET / PUT / DELETE | `/api/goals/:id` | 取 / 改（重命名）/ 删目标（级联删路径） |
| PUT | `/api/goals/:id/status` | 更新目标状态 |
| GET | `/api/goals/:id/progress` | 目标进度（模块完成度） |
| POST | `/api/pathways` | 建路径 `{name, goal_id}` |
| GET | `/api/pathways` | 列路径 `?goal=` |
| GET / PUT / DELETE | `/api/pathways/:id` | 取 / 改（重命名）/ 删路径（级联删模块序列） |
| POST / GET | `/api/pathways/:id/modules` | 挂 / 列路径模块（含依赖） |
| GET | `/api/pathways/:id/next` | 下一个可学模块（依赖检查） |
| POST / GET | `/api/modules` | 建 / 列模块 |
| PUT / DELETE | `/api/modules/:id` | 改（重命名）/ 删模块（卡片降为散卡） |
| GET | `/api/modules/:id/mastery` | 模块掌握度聚合 |
| GET | `/api/modules/:id/cards` | 模块下的卡片列表 |
| PUT | `/api/modules/:id/status` | 更新模块状态 |
| POST | `/api/sessions/start` | 开学习会话 |
| POST | `/api/sessions/:id/end` | 结会话（summary / new_cards / reviewed） |
| GET | `/api/sessions` | 列会话 `?limit=` |
| POST / GET | `/api/resources` | 建 / 列学习资源 |
| GET / PUT | `/api/profile` | 读 / 写学习者画像（温和双向记忆） |

### 设置 / 测验 / 导出 / 备份

| Method | Path | 作用 |
|--------|------|------|
| GET / PUT | `/api/settings` | 读 / 写设置（`new_per_day` 每日新卡预算，默认 5） |
| GET | `/api/quiz` | 测验抽取 `?n=&topic=`（随机抽到期复习卡） |
| GET | `/api/export` | 全量 JSON 导出（所有实体） |
| GET | `/api/export/markdown` | markdown 导出（migrate 兼容 frontmatter） |
| POST | `/api/backup` | SQLite 一致性快照备份（`VACUUM INTO`） |
| GET | `/api/timeline` | 今日活动时间线（建卡 + 复习 + 会话，时间倒序） |

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
