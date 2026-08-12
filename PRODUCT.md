# Product

<!-- impeccable:product-schema 1 -->

## Platform

web

## Users

- **trtyr**（开发者本人）— 自主学习者，学 Rust / Git / 各种技术，偏好项目驱动式学习
- **AI Agent**（Pi / Claude / Cursor）— 通过 REST API 操作平台，负责诊断、讲解、出题、评估

## Product Purpose

一个 **headless 学习管理系统**：平台自身没有 AI，只负责记录、存储、SM-2 间隔重复调度、进度聚合。所有"需要智能"的活（讲解、判断掌握度、生成卡片内容、提炼画像）由 AI Agent 通过 API 完成。

核心闭环：目标 → 路径 → 模块 → 卡片 → 复习 → 掌握度 → 画像。

## Positioning

"AI calls the API; the platform has no AI of its own."
跟 Anki 和任何 AI 教学产品都不一样——这个平台是给 AI 当工具用的，不是给人直接用的。

## Operating Context

- 开发者 macOS 环境，终端 + 浏览器
- 后端 `cargo run` 本地起，前端 `npm run dev` 本地看
- 最终 Docker 包装，单机产品
- 学习数据来自日常 AI 对话（Pi 调 API 记录）

## Capabilities and Constraints

- Rust 后端（axum）+ React/Vite 前端 + SQLite
- 15+ REST 端点：cards / topics / goals / pathways / modules / sessions / profile / stats / dashboard
- SM-2 间隔重复调度
- 温和双向记忆（LearnerProfile）：AI 积累对用户的认知，不激进第二大脑
- 前端四栏：计划（目标→路径→模块树）、复习（卡片列表）、进度（分布+会话）、画像

## Brand Commitments

- 名字：学习系统（learnsys）
- 调性：务实、高效、给 AI 用的工具，不是花哨消费品

## Evidence on Hand

- 完整工作代码：`/Users/trtyr/Documents/Code/Rust/learnsys`
- 20 张真实 Rust 学习卡片
- plan-tree 完整规划：`docs/plantree/`

## Product Principles

1. **确定性归平台，智能归 AI** — 平台只算不判
2. **给 AI 用的 LMS，不是给人用的 Anki** — AI 是操作者，人是学习者
3. **温和不激进** — 学习数据务实，不搞万能第二大脑
4. **单机优先，Docker 最后** — 先跑通再包装

## Accessibility & Inclusion

桌面端为主，中文内容。
