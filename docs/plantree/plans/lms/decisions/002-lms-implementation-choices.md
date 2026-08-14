# Decision 002: LMS Phase A–F 落地选择

**Status**: Accepted（由实现落地，以代码为准）
**Date**: 2026-08-12

Phase A–F 实现时对 open-questions 的实际收敛结果。原 OQ-1~6 已据此归档，
权威以 `learnsys-core` 代码为准，本记录只做"决策对账"。

| OQ | 问题 | 落地选择 | 代码依据 |
|----|------|---------|---------|
| OQ-1 | Module:Mastery 派生 vs 存储 | **派生**（实时聚合 cards 的 ef/reps） | `repo::module_mastery` |
| OQ-2 | 依赖严格度 | **建议**（可跳 + 提示前置未完成） | `repo::next_module` |
| OQ-3 | 路径预设模板 | **平台不内置**（平台无 AI），AI 生成 | 无路径模板表 |
| OQ-4 | LearnerProfile 结构化程度 | **半结构化**（固定字段 + 自由 notes） | `entity::LearnerProfile` |
| OQ-5 | 是否引入知识图谱 | **不引入**（pathway 内 depends_on 线性依赖） | `pathway_modules.depends_on` |
| OQ-6 | Session 引入时机 | **Phase D 引入** | `sessions` 表 + API |

## 关联

- [roadmap.md](../roadmap.md)
- [topics/data-model.md](../topics/data-model.md)
- [decision 001](001-lms-direction.md)
