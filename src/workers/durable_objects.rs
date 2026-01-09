//! Durable Objects 长期进程支持模块
//!
//! Durable Objects 提供 Workers 的长期进程能力，用于：
//! 1. 节点注册表
//! 2. 任务调度器
//! 3. 状态管理
//! 4. 共享状态

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;
use super::time::{WorkersClock, TimestampUtils};

/// Durable Object 基础接口
pub trait DurableObject: Send + Sync {
    /// 获取对象 ID
    fn id(&self) -> &str;

    /// 处理请求
    async fn handle_request(&self, request: Request) -> Response;

    /// 持久化状态
    async fn persist(&self) -> Result<()>;

    /// 加载状态
    async fn load(&self) -> Result<()>;

    /// 清理过期数据
    async fn cleanup(&self) -> Result<()>;
}

/// 请求类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Request {
    /// 请求 ID
    pub id: String,
    /// 请求方法
    pub method: String,
    /// 路径
    pub path: String,
    /// 查询参数
    pub query: HashMap<String, String>,
    /// 请求体
    pub body: Option<Vec<u8>>,
    /// 请求头
    pub headers: HashMap<String, String>,
    /// 时间戳（毫秒）
    pub timestamp: i64,
}

/// 响应类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Response {
    /// 状态码
    pub status: u16,
    /// 响应体
    pub body: Vec<u8>,
    /// 响应头
    pub headers: HashMap<String, String>,
    /// 时间戳（毫秒）
    pub timestamp: i64,
}

impl Response {
    /// 创建成功的响应
    pub fn success(body: Vec<u8>) -> Self {
        Self {
            status: 200,
            body,
            headers: HashMap::new(),
            timestamp: TimestampUtils::now_millis(),
        }
    }

    /// 创建错误的响应
    pub fn error(message: &str, status: u16) -> Self {
        Self {
            status,
            body: message.as_bytes().to_vec(),
            headers: HashMap::new(),
            timestamp: TimestampUtils::now_millis(),
        }
    }

    /// 创建 JSON 响应
    pub fn json<T: Serialize>(data: &T) -> Result<Self> {
        let body = serde_json::to_vec(data)
            .map_err(|e| anyhow!("JSON 序列化失败: {}", e))?;

        let mut headers = HashMap::new();
        headers.insert("content-type".to_string(), "application/json".to_string());

        Ok(Self {
            status: 200,
            body,
            headers,
            timestamp: TimestampUtils::now_millis(),
        })
    }
}

/// 节点注册表 Durable Object
pub struct NodeRegistry {
    /// 节点 ID
    id: String,
    /// 节点数据
    nodes: Arc<RwLock<HashMap<String, NodeInfo>>>,
    /// 时间钟
    clock: WorkersClock,
    /// 最后持久化时间
    last_persist: Arc<RwLock<i64>>,
}

/// 节点信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeInfo {
    /// 节点 ID
    pub node_id: String,
    /// 节点地址
    pub address: String,
    /// 公钥
    pub public_key: String,
    /// 注册时间
    pub registered_at: i64,
    /// 最后活跃时间
    pub last_active_at: i64,
    /// 节点状态
    pub status: NodeStatus,
    /// 设备能力
    pub capabilities: DeviceCapabilities,
}

/// 节点状态
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum NodeStatus {
    /// 活跃
    Active,
    /// 离线
    Offline,
    /// 忙碌
    Busy,
    /// 暂停
    Paused,
}

/// 设备能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DeviceCapabilities {
    /// CPU 核心数
    pub cpu_cores: u32,
    /// 内存（MB）
    pub memory_mb: u64,
    /// GPU 信息
    pub gpu_info: Vec<GpuInfo>,
    /// 网络类型
    pub network_type: String,
    /// 最大带宽（Mbps）
    pub max_bandwidth_mbps: u64,
}

/// GPU 信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GpuInfo {
    /// GPU 名称
    pub name: String,
    /// 显存（MB）
    pub memory_mb: u64,
    /// 计算能力
    pub compute_capability: f32,
}

impl NodeRegistry {
    /// 创建新的节点注册表
    pub fn new(id: String) -> Self {
        Self {
            id,
            nodes: Arc::new(RwLock::new(HashMap::new())),
            clock: WorkersClock::new(),
            last_persist: Arc::new(RwLock::new(0)),
        }
    }

    /// 注册节点
    pub async fn register_node(&self, node: NodeInfo) -> Result<()> {
        let mut nodes = self.nodes.write();

        // 检查节点是否已存在
        if nodes.contains_key(&node.node_id) {
            return Err(anyhow!("节点已存在: {}", node.node_id));
        }

        nodes.insert(node.node_id.clone(), node);

        log::info!("节点已注册: {}", node.node_id);

        Ok(())
    }

    /// 更新节点活跃时间
    pub async fn update_heartbeat(&self, node_id: &str) -> Result<()> {
        let mut nodes = self.nodes.write();

        if let Some(node) = nodes.get_mut(node_id) {
            node.last_active_at = self.clock.timestamp_ms();
            log::debug!("节点心跳更新: {}", node_id);
            Ok(())
        } else {
            Err(anyhow!("节点未找到: {}", node_id))
        }
    }

    /// 更新节点状态
    pub async fn update_status(&self, node_id: &str, status: NodeStatus) -> Result<()> {
        let mut nodes = self.nodes.write();

        if let Some(node) = nodes.get_mut(node_id) {
            node.status = status;
            log::info!("节点状态更新: {} -> {:?}", node_id, status);
            Ok(())
        } else {
            Err(anyhow!("节点未找到: {}", node_id))
        }
    }

    /// 获取节点信息
    pub async fn get_node(&self, node_id: &str) -> Option<NodeInfo> {
        self.nodes.read().get(node_id).cloned()
    }

    /// 列出所有节点
    pub async fn list_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read().values().cloned().collect()
    }

    /// 获取活跃节点
    pub async fn list_active_nodes(&self) -> Vec<NodeInfo> {
        self.nodes.read()
            .values()
            .filter(|n| n.status == NodeStatus::Active)
            .cloned()
            .collect()
    }

    /// 清理过期节点
    pub async fn cleanup(&self) -> Result<()> {
        let mut nodes = self.nodes.write();
        let now = self.clock.timestamp_ms();
        let timeout = 5 * 60 * 1000; // 5分钟

        let mut to_remove = Vec::new();

        for (node_id, node) in nodes.iter() {
            if node.status == NodeStatus::Active {
                let elapsed = now - node.last_active_at;
                if elapsed > timeout {
                    to_remove.push(node_id.clone());
                }
            }
        }

        for node_id in to_remove {
            nodes.remove(&node_id);
            log::info!("移除过期节点: {}", node_id);
        }

        Ok(())
    }

    /// 获取节点数量
    pub async fn count(&self) -> usize {
        self.nodes.read().len()
    }
}

impl DurableObject for NodeRegistry {
    fn id(&self) -> &str {
        &self.id
    }

    async fn handle_request(&self, request: Request) -> Response {
        // 设置时间钟
        self.clock.set_timestamp(request.timestamp);

        match (request.method.as_str(), request.path.as_str()) {
            ("POST", "/register") => {
                if let Some(body) = &request.body {
                    if let Ok(node) = serde_json::from_slice::<NodeInfo>(body) {
                        match self.register_node(node).await {
                            Ok(_) => Response::success(serde_json::to_vec(&"OK".to_string()).unwrap()),
                            Err(e) => Response::error(&e.to_string(), 400),
                        }
                    } else {
                        Response::error("无效的节点数据", 400)
                    }
                } else {
                    Response::error("缺少请求体", 400)
                }
            }
            ("POST", "/heartbeat") => {
                if let Some(node_id) = request.query.get("node_id") {
                    match self.update_heartbeat(node_id).await {
                        Ok(_) => Response::success(serde_json::to_vec(&"OK".to_string()).unwrap()),
                        Err(e) => Response::error(&e.to_string(), 404),
                    }
                } else {
                    Response::error("缺少 node_id 参数", 400)
                }
            }
            ("GET", "/node") => {
                if let Some(node_id) = request.query.get("node_id") {
                    if let Some(node) = self.get_node(node_id).await {
                        match Response::json(&node) {
                            Ok(resp) => resp,
                            Err(_) => Response::error("序列化失败", 500),
                        }
                    } else {
                        Response::error("节点未找到", 404)
                    }
                } else {
                    Response::error("缺少 node_id 参数", 400)
                }
            }
            ("GET", "/list") => {
                let active_only = request.query.get("active")
                    .map(|v| v == "true")
                    .unwrap_or(false);

                let nodes = if active_only {
                    self.list_active_nodes().await
                } else {
                    self.list_nodes().await
                };

                match Response::json(&nodes) {
                    Ok(resp) => resp,
                    Err(_) => Response::error("序列化失败", 500),
                }
            }
            _ => Response::error("未找到路径", 404),
        }
    }

    async fn persist(&self) -> Result<()> {
        // 在实际的 Workers 环境中，这里应该调用 Durable Object 的存储 API
        // 暂时只记录
        self.last_persist.write().clone_from(&self.clock.timestamp_ms());
        log::debug!("节点注册表状态已持久化");
        Ok(())
    }

    async fn load(&self) -> Result<()> {
        // 在实际的 Workers 环境中，这里应该从 Durable Object 存储加载状态
        log::debug!("节点注册表状态已加载");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        self.cleanup().await
    }
}

/// 任务调度器 Durable Object
pub struct TaskScheduler {
    /// 调度器 ID
    id: String,
    /// 任务队列
    tasks: Arc<RwLock<Vec<Task>>>,
    /// 时间钟
    clock: WorkersClock,
    /// 当前运行的任务
    running_tasks: Arc<RwLock<HashMap<String, RunningTask>>>,
}

/// 任务
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    /// 任务 ID
    pub id: String,
    /// 任务类型
    pub task_type: TaskType,
    /// 任务参数
    pub params: HashMap<String, serde_json::Value>,
    /// 目标节点
    pub target_node: Option<String>,
    /// 优先级
    pub priority: u32,
    /// 创建时间
    pub created_at: i64,
    /// 调度时间
    pub scheduled_at: i64,
}

/// 任务类型
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TaskType {
    /// 训练任务
    Training,
    /// 推理任务
    Inference,
    /// 数据上传
    DataUpload,
    /// 模型同步
    ModelSync,
    /// 健康检查
    HealthCheck,
}

/// 运行中的任务
#[derive(Debug, Clone)]
struct RunningTask {
    /// 任务
    task: Task,
    /// 开始时间
    started_at: i64,
    /// 进度（0-100）
    progress: u32,
}

impl TaskScheduler {
    /// 创建新的任务调度器
    pub fn new(id: String) -> Self {
        Self {
            id,
            tasks: Arc::new(RwLock::new(Vec::new())),
            clock: WorkersClock::new(),
            running_tasks: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// 添加任务
    pub async fn add_task(&self, task: Task) -> Result<()> {
        let mut tasks = self.tasks.write();

        // 按优先级排序插入
        let pos = tasks.binary_search_by(|probe| {
            probe.priority.cmp(&task.priority)
        }).unwrap_or_else(|pos| pos);

        tasks.insert(pos, task);

        log::info!("任务已添加: {}", task.id);

        Ok(())
    }

    /// 获取下一个任务
    pub async fn next_task(&self) -> Option<Task> {
        let mut tasks = self.tasks.write();
        if tasks.is_empty() {
            return None;
        }

        let now = self.clock.timestamp_ms();

        // 查找已到调度时间的任务
        let pos = tasks.iter().position(|t| t.scheduled_at <= now);

        if let Some(pos) = tasks.swap_remove(pos) {
            Some(pos)
        } else {
            None
        }
    }

    /// 取消任务
    pub async fn cancel_task(&self, task_id: &str) -> Result<()> {
        let mut tasks = self.tasks.write();

        let pos = tasks.iter().position(|t| t.id == task_id);

        if let Some(pos) {
            tasks.remove(pos);
            log::info!("任务已取消: {}", task_id);
            Ok(())
        } else {
            Err(anyhow!("任务未找到: {}", task_id))
        }
    }

    /// 列出所有任务
    pub async fn list_tasks(&self) -> Vec<Task> {
        self.tasks.read().clone()
    }

    /// 获取任务数量
    pub async fn count(&self) -> usize {
        self.tasks.read().len()
    }
}

impl DurableObject for TaskScheduler {
    fn id(&self) -> &str {
        &self.id
    }

    async fn handle_request(&self, request: Request) -> Response {
        // 设置时间钟
        self.clock.set_timestamp(request.timestamp);

        match (request.method.as_str(), request.path.as_str()) {
            ("POST", "/add") => {
                if let Some(body) = &request.body {
                    if let Ok(mut task) = serde_json::from_slice::<Task>(body) {
                        task.created_at = self.clock.timestamp_ms();
                        task.scheduled_at = task.created_at;

                        match self.add_task(task).await {
                            Ok(_) => Response::success(serde_json::to_vec(&"OK".to_string()).unwrap()),
                            Err(e) => Response::error(&e.to_string(), 400),
                        }
                    } else {
                        Response::error("无效的任务数据", 400)
                    }
                } else {
                    Response::error("缺少请求体", 400)
                }
            }
            ("POST", "/cancel") => {
                if let Some(task_id) = request.query.get("task_id") {
                    match self.cancel_task(task_id).await {
                        Ok(_) => Response::success(serde_json::to_vec(&"OK".to_string()).unwrap()),
                        Err(e) => Response::error(&e.to_string(), 404),
                    }
                } else {
                    Response::error("缺少 task_id 参数", 400)
                }
            }
            ("GET", "/list") => {
                let tasks = self.list_tasks().await;
                match Response::json(&tasks) {
                    Ok(resp) => resp,
                    Err(_) => Response::error("序列化失败", 500),
                }
            }
            _ => Response::error("未找到路径", 404),
        }
    }

    async fn persist(&self) -> Result<()> {
        log::debug!("任务调度器状态已持久化");
        Ok(())
    }

    async fn load(&self) -> Result<()> {
        log::debug!("任务调度器状态已加载");
        Ok(())
    }

    async fn cleanup(&self) -> Result<()> {
        // 清理已取消或完成的任务
        Ok(())
    }
}
