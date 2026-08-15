# Decision 005: 重内容轻结构 —— 聚焦卡片内容层

**Status**: Accepted（用户确认）
**Date**: 2026-08-14

## 决定

LMS 骨架（Goal→Pathway→Module）**过度建设**，用户实际只用 topic+card 一层。
方向回调：**聚焦卡片内容层**（增肥卡片），骨架维持现状、不再扩展。

## 背景

用户实际使用复盘：goal list 空、pathway 空、timeline 空——"目标→路径→模块"
骨架从未使用；而天天碰的卡片却是最单薄的一层（front+back 两段干文字）。

## 理由

- **结构过剩、内容贫血**：花大力气搭八层骨架，但天天打交道的"肉"没长。
- 卡片内容层具体缺失（按优先级）：
  - P0 标签形同虚设（建卡打不上标签、无标签浏览入口）；卡片无出处；code_block/图片录不进
  - P1 卡片是孤岛（无"相关卡片"双向链接）
  - P2 复习洞察太粗（无未来到期预测、薄弱点不聚类、搜索仅文本匹配）

## 影响

- 新阶段 P0/P1/P2 聚焦卡片内容层，见 [topics/card-enrichment.md](../topics/card-enrichment.md)。
- Goal→Pathway→Module 骨架**维持现状**：不扩展、不删（未来可能用上）。
- schema 升 v6（Card 加 `source` 字段）。

## 关联

- [roadmap.md](../roadmap.md)
- [topics/card-enrichment.md](../topics/card-enrichment.md)
- 部分推翻 [decision 001](001-lms-direction.md) 的"骨架优先"倾向（骨架保留，但重心转回卡片）
