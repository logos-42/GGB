//! Cloudflare Workers 集成模块
//!
//! 本模块提供与 Cloudflare Workers 环境的适配，包括：
//! 1. 数据库适配器（D1, KV, Durable Objects）
//! 2. 时间适配（Workers 时间钟方法）
//! 3. 长期进程支持（Durable Objects）

// 导出子模块
pub mod time;
pub mod db;
pub mod durable_objects;
pub mod kv;

// 重新导出常用类型
pub use time::*;
pub use db::*;
pub use durable_objects::*;
pub use kv::*;
