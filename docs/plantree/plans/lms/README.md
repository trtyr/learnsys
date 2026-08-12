# Plan: lms — 从卡片工具到学习管理系统

把 学习系统 从"卡片复习笔记本"演进为 **AI 用的学习管理系统（LMS）**。
建立在 [core](../core/README.md)（存储底座）之上，补齐"系统管理"维度。

## 定位跃迁

| 维度 | core（v0.1） | lms（v1.0） |
|------|------|------|
| 核心单元 | 卡片（原子知识点） | 计划/路径/模块（体系） |
| 管什么 | 复习调度 | 学习全过程：目标→路径→进度→掌握→画像 |
| 类比 | 记账流水 | 预算 + 账户体系 + 余额 + 报表 |
| 给谁用 | AI 记卡片/复习 | AI 规划/追踪/个性化教学 |

**记账类比**：core 是"交易流水"，lms 补"预算(计划) + 账户体系(路径) + 余额(掌握度) + 报表 + 对账"。

## Scope

- ✅ **In**：学习目标(Goal)、学习路径(Pathway)、知识模块(Module)、进度/掌握度、学习会话(Session)、温和双向记忆(LearnerProfile)、LMS 报表
- ❌ **Out（克制）**：激进的个人第二大脑、知识图谱可视化、多用户、自动生成教学内容（永远归 AI）

## 文件地图

| 文件 | 作用 |
|------|------|
| [roadmap.md](roadmap.md) | 演进阶段 |
| [topics/data-model.md](topics/data-model.md) | LMS 新实体设计（核心） |
| [topics/pathway-design.md](topics/pathway-design.md) | 学习路径模型（"路子"） |
| [topics/memory.md](topics/memory.md) | 温和双向记忆 |
| [decisions/](decisions/) | 方向决策 |
| [open-questions.md](open-questions.md) | 待决策 |

## 与 core 的关系

- 建立在 core 的 Card/Topic/ReviewLog/SM-2 之上，**不重写底座**
- Module 是 Card 的父级（1:N），Topic 升级为"领域"
- 共享同一 SQLite + 同一 API 服务

## 当前状态

- **Phase**：Planning（数据模型设计完成，待收敛 open-questions）
- **Next**：收敛 [open-questions.md](open-questions.md) → 进 Phase A（schema 扩展）
