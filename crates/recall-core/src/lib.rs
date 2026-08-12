//! recall-core —— 共享领域层：实体、SM-2 调度、持久化仓储。
//!
//! 平台原则：**确定性的归平台**（调度、聚合），**需要理解和生成的归 AI**。
//! 本 crate 不含任何 HTTP / IO 编排，只提供可被 api/migrate 复用的领域逻辑。
//!
//! 模块按后续 Task 填充：
//! - [`entity`]  —— Card / Topic / ReviewLog 实体 (Task 2)
//! - [`sm2`]     —— SM-2 间隔重复算法 (Task 4)
//! - [`repo`]    —— SQLite 仓储层 (Task 4)
