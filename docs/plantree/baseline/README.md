# Baseline · recall

项目级既定事实（技术栈、架构、部署）。计划应链接到此，不复制。

> 状态：全新项目，尚无代码。下列为**已确立的设计级事实**；
> 模块图、运行时流等实现级 baseline 待 Phase 0 脚手架就绪后填充。

## 项目定位

**recall** 是一个 headless 的学习数据平台：负责**记录、存储、调度**，
自身没有 AI 能力。所有"需要智能"的活（讲解、出题、判断掌握度、生成卡片内容）
由 AI Agent 通过 API 调用来完成。

- **平台管**：存储 + 确定性逻辑（SM-2 调度、统计聚合）+ 只读展示
- **AI 管**：教学智能（诊断、苏格拉底、费曼、出题、打分）
- **边界原则**：确定性的归平台，需要理解和生成的归 AI

## 技术栈（已定）

| 维度 | 决定 |
|------|------|
| 后端语言 | Rust |
| 架构 | 前后端分离（后端纯 API，前端独立 SPA） |
| 前端 | React + Vite |
| 产品层次 | 规范的单机产品（本地优先，最后 Docker 包装） |
| 消费者 | 任意 AI Agent / 脚本 / 前端看板，通过统一 API |

详见 [plans/core/decisions/001-tech-stack.md](../plans/core/decisions/001-tech-stack.md)。

## 架构分层

见 [plans/core/topics/architecture.md](../plans/core/topics/architecture.md)。

## 存储（已定）

SQLite 为主存储；markdown 作为 import/export 格式（承接现有数据迁移）。
详见 [decisions/002-mvp-decisions.md](../plans/core/decisions/002-mvp-decisions.md)。

## 现有可继承资产

- Python 版 `sm2.py` / `quiz.py` / `learning-plan.py` 的**业务逻辑**可继承
  （SM-2 算法、测验抽取、计划骨架），但需用 Rust 重写为 API handler
- 现有数据：`~/.pi/learning-data/` 下 20 张 Rust 卡 + progress.md，可迁移

## 待补 baseline（代码就绪后建）

- `module-map.md` — 代码模块/分层
- `runtime-flows.md` — 运行时数据流
- `storage-and-state.md` — 存储实现细节
- `test-and-release-gates.md` — 测试与发布门
- `risk-hotspots.md` — 风险热点
