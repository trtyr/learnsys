# Plan Tree · recall

本项目的工作计划入口。所有规划状态在此注册和导航。

## 权威顺序（遇到冲突，上面优先）

1. 本 README（注册表 + 活跃计划）
2. [baseline/](baseline/README.md) — 项目级既定事实（技术栈、架构、部署）
3. [plans/core/](plans/core/README.md) — 当前唯一计划：核心平台构建
4. [ideas/inbox.md](ideas/inbox.md) — 低承诺想法池

## 活跃计划

| Plan | Status | Current Phase | Last Landed | Next Target |
|------|--------|---------------|-------------|-------------|
| [core](plans/core/README.md) | 🟡 Ready | Phase 0：脚手架 + 数据模型 | — | 进 Phase 0 |

## 怎么读这棵树

- **第一次来**：本 README → [baseline/README](baseline/README.md) → [plans/core/README](plans/core/README.md) → [roadmap](plans/core/roadmap.md)
- **要决策**：看 [plans/core/open-questions.md](plans/core/open-questions.md)（每个带推荐）
- **要执行**：确认 roadmap 当前 Phase + open-questions 已收敛
- **接手/续做**：按 [plans/core/README 的 Resume 段](plans/core/README.md#resume) 读状态

## 状态约定

🟢 Done · 🟡 Planning · 🔵 In Progress · 🔴 Blocked · ⚪ Deferred

## 当前模式

`ready-check → execute-ready`：MVP 大决策已收敛（项目名 recall / REST / SQLite / React+Vite / 只读看板），
见 [decisions/002-mvp-decisions.md](plans/core/decisions/002-mvp-decisions.md)。
下一步进 Phase 0（搭 Rust workspace + schema），实现选型细节见 open-questions。
