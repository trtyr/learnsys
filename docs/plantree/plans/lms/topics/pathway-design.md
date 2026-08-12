# Topic: 学习路径设计（"路子"）

> capsule：Pathway 的模型与推进逻辑。用户强调"有各种各样的路子"。

## 为什么路径是一等公民

学同一门（如 Rust），不同人 / 不同目标走不同路子。路径不是线性列表，
是**带依赖的模块序列 + 关联一套方法论**。系统必须能定义、对比、切换路径。

## 路径类型（methodology）

| 路子 | 特点 | 适合 |
|------|------|------|
| 基础优先 | 教材式：先底层后应用 | 系统建基 |
| 项目驱动 | 实战式：做项目边补 | 快速上手 |
| 源码驱动 | 逆向式：读源码反查 | 深入理解 |
| 问题驱动 | 从具体痛点切入 | 解决当下问题 |

方法论只是标签 + AI 参考，平台不做内容生成。

## 路径推进（平台确定性逻辑）

- `next_module(pathway)`：按 order + depends_on 算下一个可学模块（前置已 mastered）
- 依赖检查：模块未 mastered，下游模块是否解锁（可配置严格/建议）
- 路径切换：一个 goal 同时 active 一条；切换时保留历史路径（不删，只改 is_active）

## 推进算法草案

```
给定 pathway:
  候选 = 按 order 排序的 modules
  for m in 候选:
    if m.status == mastered: continue
    if 所有 depends_on 模块都 mastered: return m  // 下一个可学
  return None  // 路径完成 或 被依赖卡住
```

## 路径与卡片的关系

```
Pathway ──order/depends──> Module ──1:N──> Card
                                      └─→ ReviewLog (复习驱动掌握)
```

掌握度沿这条链聚合：Card 的 SM-2 状态 → Module 的 mastery → Pathway 的进度。

## 开放点

- 依赖是强制（卡住）还是建议（可跳）—— [open-questions.md](../open-questions.md) OQ-2
- 路径预设模板（库里给标准路径）vs 完全 AI 生成 —— OQ-3
