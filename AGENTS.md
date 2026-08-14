# AGENTS.md · 学习系统 (learnsys)

给未来 agent（和人类）的操作契约。先读这个，再读 [README.md](README.md) 和 [docs/plantree/](docs/plantree/README.md)（规划与决策权威）。

## 这是什么

headless 学习数据平台：**确定性的归平台**（SM-2 调度、统计聚合、存储），**需要理解和生成的归 AI**（讲解、出题、判掌握度）。平台自身没有 AI，AI Agent 是平等客户，走 REST API。

## 技术栈

| 层 | 栈 | 位置 |
|----|----|------|
| 后端 | Rust workspace (edition 2021) | `crates/*` |
| 领域层 | `learnsys-core`（无 HTTP/IO） | `crates/learnsys-core` |
| 服务层 | `learnsys-api`（axum 0.7） | `crates/learnsys-api` |
| 迁移工具 | `learnsys-migrate`（markdown → SQLite） | `crates/learnsys-migrate` |
| 前端 | React 18 + Vite 7 + TS（个人学习工作台） | `frontend/` |
| 存储 | SQLite（rusqlite bundled） | 运行时文件，`RECALL_DB` 覆盖路径 |

## 规范命令（已在本机验证）

```bash
# 后端
cargo build --workspace
cargo test --workspace          # 30 用例，core: entity/schema/sm2/repo + migrate 回导
cargo clippy --workspace --all-targets
cargo fmt --check

# 前端（cd frontend）
npm install
npm run typecheck               # tsc --noEmit
npm run lint                    # eslint (flat config)
npm test                        # vitest
npm run build                   # vite build（不查类型，别只信它）
npm run dev                     # :5173，/api 代理到 :7878

# 全链路
./scripts/e2e.sh                # 需源数据 ~/.pi/learning-data
```

CI 在 [.github/workflows/ci.yml](.github/workflows/ci.yml)，push 到 `master` 触发。

## 架构与模块边界

依赖方向严格单向：`api → core`、`migrate → core`；`frontend → 后端` 只走 REST，绝不直接碰 SQLite。

- `crates/learnsys-core/src/`：`entity`（实体类型）、`sm2`（调度算法）、`repo`（仓储 + 聚合 + 搜索/导出/备份/streak/quiz/settings/每日新卡预算/timeline）、`schema`（DDL + 版本迁移，当前 v5）、`db`（连接/路径）。
- `crates/learnsys-api/src/main.rs`：路由 + handlers + DTO + 错误映射。一个 handler 一个职责，别把业务逻辑塞进 API 层——复用 `core::repo`。
- `frontend/src/`：`api.ts`（类型化客户端）、`types.ts`（镜像 core 实体）、`App.tsx`（视图）。

**深模块纪律**：别把文件扩成 grab-bag。新领域逻辑进 `learnsys-core`，新端点进 `learnsys-api` 并同步 README 的 API 表 + `frontend/src/api.ts`。

## 错误处理

- 领域层返回 `RepoError`（thiserror）：`NotFound` / `Sqlite` / `Date`——类型化、终态（本地 SQLite 无重试）。
- API 层单一映射点 `impl From<RepoError> for ApiError`，产出 `{ code, message }` 稳定字符串码（`not_found` / `db_error` / `date_error`）。
- 规则：边界记一次、不吞错、不向客户端泄漏堆栈/SQL。新增失败路径 → 先加 `RepoError` 变体。

## 测试

- 后端：`cargo test --workspace`；新逻辑必须有真实断言（非 vacuous），覆盖核心路径 + 边界 + 失败路径。
- 前端：`npm test`（vitest，node 环境，stub `fetch` 测契约）；组件逻辑变更补测试。
- 规则：新行为带测试一起落地。

## 约定

- 单机优先，Docker 最后；SQLite 文件不入库（`.gitignore` 已覆盖 `*.db*`）。
- 中文文档与注释；commit 用中文前缀（feat/fix/refactor/…）。
- 规划权威在 `docs/plantree/`，改设计先看那里的 decision 与 open-questions。

## 初始化状态（2026-08-14）

- 基线已验证：`cargo test` 31 过、`clippy` 无警告、`fmt` 干净、`tsc`/`eslint`/`vitest`（19 用例，含 API 契约 + CardEditor/CardRow/ReminderBadges/SessionTimeline/TimelineView/QuickCapture/CaptureModal/GoalRow 组件测试）/`vite build` 全过；`./scripts/e2e.sh` 全链路通过。
- Phase A–J 全落地：内容层（编辑/搜索/标签/多模态）、调度层（新卡复习分离+每日预算/leech）、数据层（导出/备份）、体验层（提醒/streak/测验）。schema v5。
- **定位翻转**（decision 004）：个人学习系统，人是一等操作者、AI 是可选客户；前端从"只读看板"重构为"工作台"（今天 / 学习库 / 回顾 + 顶部快捷记录），后端补 `/api/timeline` 今日活动时间线。
- 前端依赖审计：`npm audit` **0 vulnerabilities**（vite 7.3.6 + `@vitejs/plugin-react` 5.2.0；CI Node 22）。
- 已知未决（open items）：
  1. `docs/plantree/plans/core/topics/api-contract.md` 是草案；权威 API 表以 `README.md` + `learnsys-api/src/main.rs` 为准。
  2. `docs/plantree/baseline/README.md` 仍写"尚无代码"，待补 module-map / runtime-flows。
- 坑：起本地服务验证前先 `lsof -nP -iTCP:7878 -sTCP:LISTEN` 清残留进程；`cargo run &` 后别只 `kill` cargo（会留孤儿 `learnsys-api`），用 `pkill -f learnsys-api`。
