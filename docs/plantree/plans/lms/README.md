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
| [roadmap.md](roadmap.md) | 演进阶段（Phase A–F 已落地，G–J 补齐中） |
| [topics/data-model.md](topics/data-model.md) | LMS 新实体设计（核心） |
| [topics/pathway-design.md](topics/pathway-design.md) | 学习路径模型（"路子"） |
| [topics/memory.md](topics/memory.md) | 温和双向记忆 |
| [topics/content-layer.md](topics/content-layer.md) | Phase G 内容层（编辑/搜索/标签/多模态） |
| [topics/scheduling-hardening.md](topics/scheduling-hardening.md) | Phase H 调度层（新卡节奏/leech） |
| [topics/data-export-backup.md](topics/data-export-backup.md) | Phase I 数据层（导出/备份） |
| [topics/experience-and-quiz.md](topics/experience-and-quiz.md) | Phase J 体验层（提醒/streak/测验） |
| [topics/workbench-redesign.md](topics/workbench-redesign.md) | 工作台重构（今天/学习库/回顾 + 快捷记录） |
| [topics/card-enrichment.md](topics/card-enrichment.md) | 卡片内容层增肥（P0/P1/P2） |
| [decisions/](decisions/) | 方向决策（001 方向 / 002 A–F / 003 G–J / 004 定位翻转 / 005 card-first） |
| [open-questions.md](open-questions.md) | 待决策（已全部收敛） |

## 与 core 的关系

- 建立在 core 的 Card/Topic/ReviewLog/SM-2 之上，**不重写底座**
- Module 是 Card 的父级（1:N），Topic 升级为"领域"
- 共享同一 SQLite + 同一 API 服务

## 当前状态

- **Phase**：🟢 P0/P1/P2 全部落地（decision 005 card-first：内容层增肥 + 知识连接 + 复习洞察）
- **Next**：语义搜索（Deferred，需向量库）；其余按需
