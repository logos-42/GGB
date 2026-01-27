/**
 * 前端集成模块
 * 包含前端管理、Web集成等功能
 */

pub mod manager;
pub mod starter;
pub mod web;

// 重新导出常用类型
pub use manager::P2PFrontendManager;
pub use starter::P2PFrontendStarter;
