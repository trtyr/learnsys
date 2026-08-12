# 学习系统

> headless 学习数据平台：负责**记录、存储、调度**，自身没有 AI。
> 所有"需要智能"的活由 AI Agent 通过 REST API 完成。

原则：**确定性的归平台**（SM-2 调度、统计聚合），**需要理解和生成的归 AI**。

## 架构

```
AI Agent (Pi / Claude / 脚本 … 都是平等客户)
   │  REST API（契约边界）
   ▼
学习系统 平台 ── SQLite ── 只读看板 (React)
```

| 组件 | 路径 | 职责 |
|------|------|------|
| 后端服务 | `crates/学习系统-api` | axum REST：cards / topics / stats / dashboard |
| 领域层 | `crates/学习系统-core` | 实体 · SM-2 算法 · SQLite 仓储 · 聚合 |
| 迁移工具 | `crates/学习系统-migrate` | markdown → SQLite 导入 |
| 前端看板 | `frontend/` | React + Vite，只读学习舱 |

## 快速开始

需要 Rust + Node。

### 1. 起后端
```bash
cargo run -p 学习系统-api
# → http://127.0.0.1:7878
```
数据默认在 `~/Library/Application Support/学习系统/学习系统.db`，可用 `RECALL_DB=/path/to.db` 覆盖（Docker 挂卷也用它）。

### 2. 导入现有数据（可选）
```bash
cargo run -p 学习系统-migrate -- ~/.pi/learning-data
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
cargo build --release -p 学习系统-api
```

## API 速览

| Method | Path | 作用 |
|--------|------|------|
| POST | `/api/cards` | 建卡 `{topic, front, back}`（topic 用名，不存在自动建主题） |
| GET | `/api/cards/due` | 今日待复习 `?topic=` |
| GET | `/api/cards/:id` | 取一张 |
| DELETE | `/api/cards/:id` | 删卡 |
| POST | `/api/cards/:id/review` | 记录复习 `{quality:0-5}` → SM-2 调度 |
| GET | `/api/topics` | 列主题 |
| PUT | `/api/topics/:id` | 更新阶段/状态/下次计划 |
| GET | `/api/stats` | 总卡片 / 待复习 / 平均 EF / 主题分布 |
| GET | `/api/dashboard` | 看板聚合（待复习 + 进行中主题 + 预警） |

`review` 是 SM-2 调度的唯一入口，原子更新卡 + 追加复习记录。

## 测试
```bash
cargo test --workspace   # 学习系统-core: entity/schema/sm2/repo，16 用例
```

## 规划

设计决策、路线图、数据模型在 [docs/plantree/](docs/plantree/README.md)。
