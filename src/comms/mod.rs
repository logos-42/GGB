//! 通信模块 - 负责节点间的网络通信
//! 
//! 该模块包含以下子模块：
//! - config: 通信配置和带宽预算管理
//! - quic: QUIC网关实现
//! - routing: 智能路由系统
//! - libp2p: libp2p网络行为定义

pub mod config;
pub mod quic;
pub mod routing;
pub mod libp2p;

// 重新导出主要类型
pub use config::{CommsConfig, BandwidthBudgetConfig};
pub use libp2p::{Behaviour, OutEvent};
pub use routing::{RouteType, RouteQuality, RouteScore, RouteInfo, RouteSelection, RoutingError};

// 主通信句柄
mod handle;
pub use handle::CommsHandle;
