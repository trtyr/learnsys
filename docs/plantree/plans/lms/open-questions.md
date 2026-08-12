# Open Questions · lms

每个问题带**推荐**，用户确认后转为 [decisions/](decisions/)。

## OQ-1: Module:Mastery 派生 vs 存储

- 派生（实时聚合 Card 的 ef/reps）—— 数据一致，但频繁聚合
- 存储（缓存 mastery 字段）—— 快，需同步逻辑
- **推荐**：派生 + 按需缓存（写少读多场景缓存即可）

## OQ-2: 依赖严格度

- 强制（前置未 mastered，下游不解锁）—— 防跳跃，但可能卡死
- 建议（可跳，仅提示前置未完成）—— 灵活
- **推荐**：默认建议（可跳 + 提示），路径可配强制

## OQ-3: 路径预设模板

- 平台内置标准路径库（如"Rust 基础优先"）
- 完全 AI 生成 + 存平台
- **推荐**：平台不内置（平台无 AI）；AI 生成。后续可做"路径导入/导出"格式用于分享

## OQ-4: LearnerProfile 结构化程度

- 强 schema（固定字段，无自由文本）
- 半结构化（固定字段 + 自由 notes）—— 温和
- **推荐**：半结构化（温和版，给 AI notes 灵活性）

## OQ-5: 是否引入知识图谱

- 模块间复杂依赖图（全局 DAG）
- 简单线性 + pathway 内 depends_on
- **推荐**：先简单（pathway 内 order + depends_on）；复杂图谱待需求驱动

## OQ-6: Session 引入时机

- core 当初标 Session"可选"未做
- lms 要报表/趋势，Session 流水有价值
- **推荐**：lms Phase D 引入

## 收敛动作

决策确认 → 各自迁入 [decisions/](decisions/) → 进 Phase A（schema 扩展）。
