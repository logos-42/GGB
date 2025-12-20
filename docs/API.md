# GGB API 文档

## 概述

本文档描述 GGB (Geo-Gossip Base) 去中心化训练节点的核心 API。

## 核心模块

### 设备管理 (`device`)

#### `DeviceManager`

设备能力管理器，负责检测和管理设备能力。

```rust
pub struct DeviceManager {
    // ...
}

impl DeviceManager {
    pub fn new() -> Self;
    pub fn get(&self) -> DeviceCapabilities;
    pub fn update_network_type(&self, network_type: NetworkType);
    pub fn update_battery(&self, level: Option<f32>, is_charging: bool);
    pub fn refresh(&self);
}
```

#### `DeviceCapabilities`

设备能力信息结构。

```rust
pub struct DeviceCapabilities {
    pub max_memory_mb: usize,
    pub cpu_cores: usize,
    pub has_gpu: bool,
    pub network_type: NetworkType,
    pub battery_level: Option<f32>,
    pub is_charging: bool,
    pub device_type: DeviceType,
}
```

### 推理引擎 (`inference`)

#### `InferenceEngine`

模型推理和训练引擎。

```rust
pub struct InferenceEngine {
    // ...
}

impl InferenceEngine {
    pub fn new(config: InferenceConfig) -> Result<Self>;
    pub fn model_dim(&self) -> usize;
    pub fn embedding(&self) -> Vec<f32>;
    pub fn tensor_snapshot(&self) -> TensorSnapshot;
    pub fn tensor_hash(&self) -> String;
    pub fn make_sparse_update(&self, k: usize) -> SparseUpdate;
    pub fn apply_sparse_update(&self, update: &SparseUpdate);
    pub fn apply_dense_snapshot(&self, snapshot: &TensorSnapshot);
    pub fn local_train_step(&self);
    pub fn convergence_score(&self) -> f32;
    pub fn validate_model_file(path: &Path, expected_dim: Option<usize>) -> Result<()>;
}
```

### 通信层 (`comms`)

#### `CommsHandle`

P2P 通信处理器。

```rust
pub struct CommsHandle {
    // ...
}

impl CommsHandle {
    pub async fn new(config: CommsConfig) -> Result<Self>;
    pub fn publish(&mut self, signed: &SignedGossip) -> Result<()>;
    pub fn add_peer(&mut self, peer: &PeerId);
    pub fn remove_peer(&mut self, peer: &PeerId);
    pub fn save_bootstrap_peers(&self, path: &Path) -> Result<()>;
}
```

### 共识引擎 (`consensus`)

#### `ConsensusEngine`

共识和质押管理。

```rust
pub struct ConsensusEngine {
    // ...
}

impl ConsensusEngine {
    pub fn new(crypto: Arc<CryptoSuite>, config: ConsensusConfig) -> Self;
    #[cfg(feature = "blockchain")]
    pub fn with_blockchain_client(self, client: Arc<dyn BlockchainClient>) -> Self;
    pub fn sign(&self, payload: GgbMessage) -> Result<SignedGossip>;
    pub fn verify(&self, msg: &SignedGossip) -> bool;
    pub fn stake_weight(&self, peer: &str) -> f32;
    #[cfg(feature = "blockchain")]
    pub async fn sync_stake_from_chain(&self, peer: &str);
}
```

### 区块链客户端 (`blockchain`)

#### `BaseNetworkClient`

Base 网络 RPC 客户端。

```rust
pub struct BaseNetworkClient {
    // ...
}

impl BaseNetworkClient {
    pub fn new(rpc_url: String) -> Self;
    pub fn new_sepolia(rpc_url: String) -> Self;
    pub async fn get_balance(&self, address: &str) -> Result<u64>;
    pub async fn query_stake(&self, contract_address: &str, user_address: &str) -> Result<f64>;
}
```

## FFI 接口

### C API

#### 节点管理

- `ggb_node_create() -> *mut NodeHandle`
- `ggb_node_destroy(ptr: *mut NodeHandle)`
- `ggb_node_get_capabilities(ptr: *const NodeHandle) -> *mut c_char`
- `ggb_node_set_device_callback(ptr: *mut NodeHandle, callback: Option<DeviceInfoCallback>) -> c_int`
- `ggb_node_refresh_device_info(ptr: *mut NodeHandle) -> c_int`

#### 设备信息

- `ggb_node_update_network_type(ptr: *mut NodeHandle, network_type_str: *const c_char) -> c_int`
- `ggb_node_update_battery(ptr: *mut NodeHandle, level: f32, is_charging: c_int) -> c_int`
- `ggb_node_recommended_model_dim(ptr: *const NodeHandle) -> usize`
- `ggb_node_recommended_tick_interval(ptr: *const NodeHandle) -> u64`
- `ggb_node_should_pause_training(ptr: *const NodeHandle) -> c_int`

## 移动端集成

### Android

Java 包装类：`com.ggb.GgbNode`

```java
public class GgbNode {
    public GgbNode(Context context);
    public String getCapabilities();
    public void updateNetworkType();
    public void updateBattery();
    public void refreshDeviceInfo();
    public int getRecommendedModelDim();
    public long getRecommendedTickInterval();
    public boolean shouldPauseTraining();
    public void destroy();
}
```

### iOS

Swift 包装类：`GgbNode`

```swift
public class GgbNode {
    public init()
    public func getCapabilities() -> String?
    public func updateNetworkType(_ type: NetworkType)
    public func updateBattery(level: Float, isCharging: Bool)
    public func refreshDeviceInfo()
    public func getRecommendedModelDim() -> Int
    public func getRecommendedTickInterval() -> UInt64
    public func shouldPauseTraining() -> Bool
}
```

## 配置

### `CommsConfig`

通信配置。

```rust
pub struct CommsConfig {
    pub topic: String,
    pub listen_addr: Option<Multiaddr>,
    pub quic_bind: Option<SocketAddr>,
    pub quic_bootstrap: Vec<SocketAddr>,
    pub bandwidth: BandwidthBudgetConfig,
    pub enable_dht: bool,
    pub bootstrap_peers_file: Option<PathBuf>,
}
```

### `InferenceConfig`

推理配置。

```rust
pub struct InferenceConfig {
    pub model_dim: usize,
    pub model_path: Option<PathBuf>,
}
```

### `TopologyConfig`

拓扑配置。

```rust
pub struct TopologyConfig {
    pub max_neighbors: usize,
    pub failover_pool: usize,
    pub min_score: f32,
    pub geo_scale_km: f32,
    pub peer_stale_secs: u64,
}
```

## 错误处理

所有 API 函数返回 `Result<T>` 类型，错误使用 `anyhow::Error`。

常见错误：
- `InvalidArgument`: 无效参数
- `OutOfMemory`: 内存不足
- `NetworkError`: 网络错误

## 示例

### 创建节点

```rust
use GGB::*;

let device_manager = DeviceManager::new();
let capabilities = device_manager.get();

let inference_config = InferenceConfig {
    model_dim: capabilities.recommended_model_dim(),
    model_path: Some("examples/sample_model.npy".into()),
};

let inference = InferenceEngine::new(inference_config)?;
```

### 验证模型文件

```rust
use GGB::inference::validate_model_file;

validate_model_file("examples/sample_model.npy", Some(256))?;
```

