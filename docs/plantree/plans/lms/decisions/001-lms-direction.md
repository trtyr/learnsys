# Decision 001: LMS 方向重定义

**Status**: Accepted（用户确认）
**Date**: 2026-08-12

## 决定

recall 从"卡片复习工具"演进为 **AI 用的学习管理系统（LMS）**。

## 背景

core MVP（卡片 + 复习）已交付，但用户指出它"像笔记本不像系统"——
缺学习计划、路径、进度、掌握度、报表等"系统管理"维度。
类比：记账系统不能只有流水，还要有预算、账户体系、余额、报表。

## 理由

- **记账类比**：core 是流水，lms 补预算(计划) + 账户体系(路径) + 余额(掌握度) + 报表
- **路径是一等公民**：同一门学问有多种"路子"（基础优先/项目驱动/源码驱动），系统要支持
- **双向记忆**：用户学的同时 AI 也积累认知，让教学跨 session 个性化

## 克制（用户明确）

- 记忆系统**温和**：不激进第二大脑，只存学习领域认知
- 不替代 pi 的 hermes-memory（通用身份/偏好归 hermes）

## 影响

- 建立在 core 上，不重写底座
- 新增实体：Goal / Pathway / Module / PathwayModule / Session / LearnerProfile
- Card 加 module_id（nullable，兼容现有）；Topic 升级为"领域"
- 前端从只读看板演进为 LMS 仪表盘
- 延续 headless：平台管确定性（依赖检查/聚合），AI 管智能（设计/评估/画像）

## 关联

- [core](../../core/README.md)（存储底座，MVP 已交付）
- [topics/data-model.md](../topics/data-model.md)
- [topics/pathway-design.md](../topics/pathway-design.md)
- [topics/memory.md](../topics/memory.md)
