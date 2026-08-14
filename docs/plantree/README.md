# Plan Tree · 学习系统

本项目的工作计划入口。所有规划状态在此注册和导航。

## 权威顺序（遇到冲突，上面优先）

1. 本 README（注册表 + 活跃计划）
2. [baseline/](baseline/README.md) — 项目级既定事实（技术栈、架构、部署）
3. [plans/core/](plans/core/README.md) — 存储底座（卡片/复习，MVP 已交付）
4. [plans/lms/](plans/lms/README.md) — 学习管理系统演进（当前主线）
5. [ideas/inbox.md](ideas/inbox.md) — 低承诺想法池

## 活跃计划

| Plan | Status | Current Phase | Last Landed | Next Target |
|------|--------|---------------|-------------|-------------|
| [core](plans/core/README.md) | 🟢 Done (MVP) | Phase 0–3 已交付 | commit 784f13d + 前端 | Phase 4 接入验证 |
| [lms](plans/lms/README.md) | 🟢 Done | Phase A–J 全落地 | decision 003 (Phase J) | Deferred 项按需 |

## 怎么读这棵树

- **第一次来**：本 README → [baseline/README](baseline/README.md) → [plans/lms/README](plans/lms/README.md)（当前主线）→ [lms roadmap](plans/lms/roadmap.md)
- **要决策**：看 [plans/lms/open-questions.md](plans/lms/open-questions.md)（每个带推荐）
- **要执行**：确认 roadmap 当前 Phase + open-questions 已收敛
- **接手/续做**：按对应 plan README 的 Resume 段读状态

## 状态约定

🟢 Done · 🟡 Planning · 🔵 In Progress · 🔴 Blocked · ⚪ Deferred

## 当前模式

- `core`：`execute-ready`（MVP Phase 0–3 已交付，见 [core decision 003](plans/core/decisions/003-mvp-shipped.md)）
- `lms`：`execute-ready`（Phase A–J 全落地，OQ 全收敛，见 [decision 003](plans/lms/decisions/003-gap-fill-implementation-choices.md)）
