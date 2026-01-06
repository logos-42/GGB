//! Cloudflare Workers适配模块
//! 
//! 提供与Cloudflare Workers兼容的接口和功能

pub mod edge_server;
pub mod algorithms;
pub mod network;
pub mod storage;

// 重新导出常用类型
pub use edge_server::{EdgeServerWorker, EdgeServerConfig};
pub use algorithms::{WorkerAlgorithmManager, AlgorithmRequest, AlgorithmResponse};
pub use network::{WorkerNetworkAdapter, WorkerNetworkConfig};
pub use storage::{WorkerStorage, WorkerKVStorage, WorkerDurableObject};

/// Workers应用配置
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct WorkersConfig {
    /// 边缘服务器配置
    pub edge_server: EdgeServerConfig,
    /// 算法配置
    pub algorithms: WorkerAlgorithmManagerConfig,
    /// 网络配置
    pub network: WorkerNetworkConfig,
    /// 存储配置
    pub storage: WorkerStorageConfig,
    /// 是否启用ZK证明
    pub enable_zk_proof: bool,
    /// 最大并发请求数
    pub max_concurrent_requests: usize,
    /// 请求超时时间（毫秒）
    pub request_timeout_ms: u64,
}

/// Workers应用
pub struct WorkersApp {
    config: WorkersConfig,
    edge_server: EdgeServerWorker,
    algorithm_manager: WorkerAlgorithmManager,
    network_adapter: WorkerNetworkAdapter,
    storage: WorkerStorage,
}

impl WorkersApp {
    /// 创建新的Workers应用
    pub fn new(config: WorkersConfig) -> anyhow::Result<Self> {
        let edge_server = EdgeServerWorker::new(config.edge_server.clone())?;
        let algorithm_manager = WorkerAlgorithmManager::new(config.algorithms.clone());
        let network_adapter = WorkerNetworkAdapter::new(config.network.clone())?;
        let storage = WorkerStorage::new(config.storage.clone())?;
        
        Ok(Self {
            config,
            edge_server,
            algorithm_manager,
            network_adapter,
            storage,
        })
    }
    
    /// 处理HTTP请求
    pub async fn handle_request(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let path = request.path();
        
        match path.as_str() {
            "/api/nodes/register" => self.handle_node_registration(request).await,
            "/api/nodes/heartbeat" => self.handle_heartbeat(request).await,
            "/api/tasks/submit" => self.handle_task_submission(request).await,
            "/api/tasks/match" => self.handle_task_matching(request).await,
            "/api/algorithms/allocate" => self.handle_algorithm_allocation(request).await,
            "/api/zk/verify" => self.handle_zk_verification(request).await,
            "/api/stats" => self.handle_stats(request).await,
            _ => Ok(worker::Response::error("Not Found", 404)?),
        }
    }
    
    /// 处理节点注册
    async fn handle_node_registration(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let node_info: NodeInfo = request.json().await?;
        let registration = self.edge_server.register_node(node_info).await?;
        
        worker::Response::from_json(&registration)
    }
    
    /// 处理心跳
    async fn handle_heartbeat(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let heartbeat: Heartbeat = request.json().await?;
        self.edge_server.update_heartbeat(heartbeat).await?;
        
        worker::Response::ok("OK")
    }
    
    /// 处理任务提交
    async fn handle_task_submission(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let task_request: TaskRequest = request.json().await?;
        let task_response = self.edge_server.submit_task(task_request).await?;
        
        worker::Response::from_json(&task_response)
    }
    
    /// 处理任务匹配
    async fn handle_task_matching(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let match_request: MatchRequest = request.json().await?;
        let matched_nodes = self.edge_server.match_nodes(match_request).await?;
        
        worker::Response::from_json(&matched_nodes)
    }
    
    /// 处理算法分配
    async fn handle_algorithm_allocation(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        let algorithm_request: AlgorithmRequest = request.json().await?;
        let allocation = self.algorithm_manager.allocate(algorithm_request).await?;
        
        worker::Response::from_json(&allocation)
    }
    
    /// 处理ZK验证
    async fn handle_zk_verification(&self, request: worker::Request) -> anyhow::Result<worker::Response> {
        if !self.config.enable_zk_proof {
            return Ok(worker::Response::error("ZK证明功能未启用", 400)?);
        }
        
        let proof_request: ProofVerificationRequest = request.json().await?;
        let verification_result = self.verify_zk_proof(proof_request).await?;
        
        worker::Response::from_json(&verification_result)
    }
    
    /// 处理统计信息
    async fn handle_stats(&self, _request: worker::Request) -> anyhow::Result<worker::Response> {
        let stats = self.get_system_stats().await;
        worker::Response::from_json(&stats)
    }
    
    /// 验证ZK证明
    async fn verify_zk_proof(&self, request: ProofVerificationRequest) -> anyhow::Result<ProofVerificationResult> {
        // 这里实现ZK证明验证逻辑
        // 暂时返回模拟结果
        Ok(ProofVerificationResult {
            proof_id: request.proof_id,
            valid: true,
            verification_time_ms: 10,
            timestamp: chrono::Utc::now().timestamp(),
        })
    }
    
    /// 获取系统统计信息
    async fn get_system_stats(&self) -> SystemStats {
        SystemStats {
            total_nodes: 0,
            active_nodes: 0,
            total_tasks: 0,
            completed_tasks: 0,
            algorithm_allocations: 0,
            zk_verifications: 0,
            uptime_seconds: 0,
            timestamp: chrono::Utc::now().timestamp(),
        }
    }
}

// 类型定义
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct NodeInfo {
    pub node_id: String,
    pub capabilities: DeviceCapabilities,
    pub network_info: NetworkInfo,
    pub location: Option<GeoLocation>,
    pub available: bool,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Heartbeat {
    pub node_id: String,
    pub timestamp: i64,
    pub status: NodeStatus,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct TaskRequest {
    pub task_id: String,
    pub task_type: TaskType,
    pub input_data: Vec<u8>,
    pub requirements: TaskRequirements,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct MatchRequest {
    pub task: TaskRequest,
    pub strategy: MatchingStrategy,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProofVerificationRequest {
    pub proof_id: String,
    pub proof_data: Vec<u8>,
    pub public_inputs: Vec<u8>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ProofVerificationResult {
    pub proof_id: String,
    pub valid: bool,
    pub verification_time_ms: u64,
    pub timestamp: i64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SystemStats {
    pub total_nodes: usize,
    pub active_nodes: usize,
    pub total_tasks: usize,
    pub completed_tasks: usize,
    pub algorithm_allocations: usize,
    pub zk_verifications: usize,
    pub uptime_seconds: u64,
    pub timestamp: i64,
}

// 枚举类型
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum NodeStatus {
    Online,
    Offline,
    Busy,
    Maintenance,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum TaskType {
    Training,
    Inference,
    Validation,
    Optimization,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub enum MatchingStrategy {
    Performance,
    Geography,
    LoadBalance,
    Hybrid,
}

// 占位符类型（需要从其他模块导入）
pub type DeviceCapabilities = serde_json::Value;
pub type NetworkInfo = serde_json::Value;
pub type GeoLocation = serde_json::Value;
pub type TaskRequirements = serde_json::Value;
pub type EdgeServerConfig = serde_json::Value;
pub type WorkerAlgorithmManagerConfig = serde_json::Value;
pub type WorkerNetworkConfig = serde_json::Value;
pub type WorkerStorageConfig = serde_json::Value;
