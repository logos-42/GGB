//! GGB 库模块
//! 
//! 将核心功能导出为库，以便测试和集成使用

#![allow(non_snake_case)] // 允许使用 GGB 作为 crate 名称

pub mod args;
pub mod comms;
pub mod config;
pub mod consensus;
pub mod crypto;
pub mod device;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod inference;
pub mod node;
pub mod security;
pub mod stats;
pub mod topology;
pub mod training;
pub mod types;
#[cfg(feature = "blockchain")]
pub mod blockchain;

// 重新导出常用类型
pub use comms::{CommsConfig, CommsHandle};
pub use consensus::{ConsensusConfig, ConsensusEngine};
pub use crypto::{CryptoConfig, CryptoSuite};
pub use device::{DeviceCapabilities, DeviceDetector, DeviceManager, DeviceType, GpuComputeApi, NetworkType};
pub use inference::{InferenceConfig, InferenceEngine};
pub use stats::TrainingStatsManager;
pub use topology::{TopologyConfig, TopologySelector};
pub use types::{GeoPoint, GgbMessage};

