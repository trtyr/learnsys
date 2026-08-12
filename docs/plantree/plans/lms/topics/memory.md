# Topic: 双向记忆（温和版）

> capsule：用户记忆 + AI 记忆。**用户要求温和**，不激进。

## 双向性

- **用户记忆**：他学了啥 → Card + ReviewLog + Module.status（core/lms 已有）
- **AI 记忆**：AI 对他的认知 → LearnerProfile（lms 新增）

> "我在学的同时，你（AI）也在学。" —— 双向的精髓。

## AI 记忆存什么（温和）

只存"学习领域专用"的结构化认知，不存泛生活：

| 字段 | 存什么 | 谁判断 |
|------|--------|--------|
| level | 整体水平定位（ZPD） | AI |
| style | 学习风格（项目驱动/教材/...） | AI |
| weak_points | 反复卡的知识点（盲点） | AI（平台可给候选） |
| preferences | 学习偏好 | AI |
| notes | 自由记忆（关键认知、决策） | AI |

## 互相喂养

```
卡片复习数据 → 提炼盲点 → 更新 Profile → AI 下次教学更准
   ↑                                          │
   └─────────── AI 据 Profile 调整教学 ←───────┘
```

这让 AI **跨 session 越来越懂这个用户**——下一次对话不用从零开始诊断。

## 边界（克制 · 用户明确）

- ❌ 不做激进的个人第二大脑（不存生活琐事、不存所有对话）
- ❌ 不替代 pi 的 hermes-memory：通用身份/偏好归 hermes；学习认知归 recall
- ✅ 只存学习领域、和学习数据强耦合的认知

## 更新方式

- AI 通过 API 更新 Profile（PUT /api/profile）
- 平台**不自动生成** Profile 内容（判断归 AI）
- 平台可提供**盲点候选**辅助：从 low-ef / 反复重置的卡片聚合，供 AI 参考
  （但最终 weak_points 的认定权在 AI）

## 与 hermes-memory 的分工

| 维度 | hermes-memory | recall LearnerProfile |
|------|---------------|----------------------|
| 范围 | 通用（身份/偏好/项目约定） | 学习领域专用 |
| 耦合 | 独立 | 强耦合学习数据（掌握度→盲点→画像） |
| 位置 | pi 全局 | recall 内部 |
