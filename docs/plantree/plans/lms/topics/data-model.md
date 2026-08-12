# Topic: LMS 数据模型 · recall

> capsule：LMS 新实体设计。建立在 core（Card/Topic/ReviewLog）之上。
> 字段草案，schema 在 Phase A 定稿。

## 实体全景

```
Goal（学什么、到什么程度）
 └─→ Pathway（按哪条路子学，可多条）
       └─→ Module（体系节点，有序+依赖）
             └─→ Card（原子知识点，core 已有）
                   └─→ ReviewLog（复习流水，core 已有）

Session（学习会话流水）      LearnerProfile（AI 温和记忆）
```

## Goal（学习目标）— "我要学到啥程度"

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| title | TEXT | 如"能写 Rust 后端服务" |
| description | TEXT | 为什么学、背景 |
| success_criteria | TEXT | 验收标准（可验证） |
| topic | TEXT FK | 关联领域 |
| status | TEXT | active / achieved / abandoned |
| created | DATE | |
| achieved_at | DATE | 达成日（nullable） |

记账类比：存款目标。

## Pathway（学习路径 / "路子"）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| name | TEXT | 如"Rust 基础优先路径" |
| methodology | TEXT | 基础优先 / 项目驱动 / 源码驱动 / 问题驱动 |
| description | TEXT | |
| goal_id | TEXT FK | 服务于哪个目标 |
| is_active | BOOL | 当前走这条？（一个 goal 同时只走一条） |
| created | DATE | |

记账类比：资产配置方案（同一笔钱不同打法）。详见 [pathway-design.md](pathway-design.md)。

## Module（知识模块 / 体系节点）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | TEXT PK | |
| title | TEXT | 如"所有权与借用" |
| topic | TEXT FK | 所属领域 |
| description | TEXT | |
| status | TEXT | not_started / learning / mastered |

比 Card 大一级，是体系的节点。一个 Module 下挂多张 Card。

## PathwayModule（路径中的模块序列 + 依赖）

| 字段 | 类型 | 说明 |
|------|------|------|
| pathway_id | TEXT FK | |
| module_id | TEXT FK | |
| order | INT | 路径中的顺序 |
| depends_on | TEXT | 前置模块 id（逗号分隔，简单依赖） |

M:N：同一模块可出现在多路径。依赖在路径上下文里（同一模块在不同路径依赖不同）。

## Card（core 已有）— 加 module_id

core 的 Card 增加 `module_id`（nullable，兼容现有无模块的卡）。
`module_id` 为 null 的卡仍属于 Topic 级别的散卡。

## Session（学习会话流水）

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INT PK | |
| started_at | TIMESTAMP | |
| ended_at | TIMESTAMP | nullable（进行中） |
| goal_id | TEXT FK | 本次上下文（nullable） |
| pathway_id | TEXT FK | nullable |
| summary | TEXT | AI 写的会话小结 |
| new_cards | INT | 本次新建卡片数 |
| reviewed | INT | 本次复习卡片数 |

记账类比：一笔交易。

## LearnerProfile（温和双向记忆 · AI 侧）

**单例**（一个用户一份，id 固定=1）。温和：半结构化，AI 更新，平台存。
详见 [memory.md](memory.md)。

| 字段 | 类型 | 说明 |
|------|------|------|
| id | INT PK（=1） | 单例 |
| level | TEXT | 整体水平定位（AI 判断） |
| style | TEXT | 学习风格（项目驱动/教材/...） |
| weak_points | TEXT | 盲点（JSON 数组：反复卡的知识点） |
| preferences | TEXT | 偏好（JSON） |
| notes | TEXT | AI 自由记忆（自由文本） |
| updated | TIMESTAMP | |

## 派生 vs 存储

- **Mastery（掌握度）**：**派生**。从 Module 下 Card 的 ef/reps/due 聚合（不单独存，避免不一致）。API 实时算 + 可选缓存。
- **Progress**：Module.status + 聚合掌握度，反映进度。

## 设计要点

- Module 是 Card 的父级（1:N）；Topic 升级为"领域"（Rust/数据库/...）
- 路径定义模块的顺序 + 依赖；模块本身领域内复用
- Session 只追加（流水），供报表
- LearnerProfile 温和：AI 写、平台存、跨 session 读
- **延续 headless**：平台存 + 算确定性（依赖检查/聚合），AI 管智能（设计路径/评估/画像）

## 开放点

- Mastery 派生 vs 缓存粒度（见 [open-questions.md](../open-questions.md) OQ-1）
- 依赖严格度（OQ-2）
