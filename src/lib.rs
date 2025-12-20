//! GGB 库模块
//! 
//! 将核心功能导出为库，以便测试和集成使用

pub mod comms;
pub mod consensus;
pub mod crypto;
pub mod device;
#[cfg(feature = "ffi")]
pub mod ffi;
pub mod inference;
pub mod stats;
pub mod topology;
pub mod types;
#[cfg(feature = "blockchain")]
pub mod blockchain;

// 重新导出常用类型
pub use comms::{CommsConfig, CommsHandle};
pub use consensus::{ConsensusConfig, ConsensusEngine};
pub use crypto::{CryptoConfig, CryptoSuite};
pub use device::{DeviceCapabilities, DeviceManager, NetworkType};
pub use inference::{InferenceConfig, InferenceEngine};
pub use stats::TrainingStatsManager;
pub use topology::{TopologyConfig, TopologySelector};
pub use types::{GeoPoint, GgbMessage};

