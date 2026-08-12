//! recall-core —— 共享领域层：实体、SM-2 调度、持久化仓储。
//!
//! 平台原则：**确定性的归平台**（调度、聚合），**需要理解和生成的归 AI**。
//! 本 crate 不含任何 HTTP / IO 编排，只提供可被 `recall-api` / `recall-migrate`
//! 复用的领域逻辑。

pub mod db;
pub mod entity;
pub mod repo;
pub mod schema;
pub mod sm2;
