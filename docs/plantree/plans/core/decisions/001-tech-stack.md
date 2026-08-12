# Decision 001: 技术栈与架构形态

**Status**: Accepted（用户已确认方向）
**Date**: 2026-08-12

## 决定

- **后端语言**：Rust
- **架构**：前后端分离（后端纯 API 服务，前端独立 SPA）
- **产品层次**：规范的单机产品（本地优先，最后 Docker 包装）
- **核心定位**：headless 学习数据平台，AI 通过 API 操作，平台本身无 AI

## 理由

- 用户指定 Rust + 前后端分离 + Docker 包装
- "headless 平台 + AI 客户端"实现关注点分离：Pi 降级为众多 client 之一，
  平台可独立演进、可被任意 agent 消费 —— 这才是"独立产品"

## 后果

- 现有 Python 脚本（sm2.py / quiz.py）的业务**逻辑**可继承，
  但需用 Rust 重写为 API handler（不是直接跑 .py）
- 部署分两步走：先本地单机测通，再 Docker 容器化
- 工程量从"几个脚本"跨台阶到"带 API 的服务"：要定 schema、写 API、
  选持久化、做展示、想并发/备份

## 关联

- 架构细节：[topics/architecture.md](../topics/architecture.md)
- 待定子项：[open-questions.md](../open-questions.md)（API 形态 / 存储 / 前端等）
