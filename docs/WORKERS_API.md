# Workers API 集成说明

## 概述

这个项目现在已经集成了与 Cloudflare Workers 后端的通信功能，可以将桌面测试信息发送到远程服务器。

## API 端点

**Workers 后端 URL**: `https://williw.sirazede725.workers.dev`

### 支持的 API 端点

1. **节点信息上报**
   - 端点: `POST /api/node-info`
   - 功能: 节点上报自身状态和硬件信息（包括设备信息）
   - 数据格式: JSON

2. **推理请求**
   - 端点: `POST /api/request`
   - 功能: 用户发起推理请求，Worker 调算法返回节点列表和模型切分方案
   - 数据格式: JSON

3. **发送模型名称**
   - 端点: `POST /api/model`
   - 功能: Worker 选定 Hugging Face 模型并标记为 ready
   - 数据格式: JSON

4. **训练数据上传**
   - 端点: `POST /api/training-data`
   - 功能: 上报训练数据样本
   - 数据格式: JSON

5. **节点重新分配**
   - 端点: `POST /api/reassign-node`
   - 功能: 节点无法联系部分节点时，请求重新分配新的节点
   - 数据格式: JSON

6. **节点健康检查**
   - 端点: `GET /api/node-health?node_id=...`
   - 功能: 根据已上报信息检查节点健康状态
   - 参数: `node_id` (查询参数)

7. **健康检查**
   - 端点: `GET /api/health`
   - 功能: 测试连接状态

## 后端命令

### Tauri 命令（前端可调用）

1. **`test_workers_connection`**
   - 测试与 Workers 后端的连接
   - 返回: `boolean` - 连接状态

2. **`upload_device_info_to_workers`**
   - 上传设备信息和节点状态到 Workers（/api/node-info）
   - 返回: `string` - 上传结果消息

3. **`request_inference_from_workers`**
   - 参数: `model_id: string`, `input_data: JSON`
   - 请求推理服务到 Workers（/api/request）
   - 返回: `JSON` - 推理响应，包含节点列表和模型切分方案

4. **`upload_model_selection_to_workers`**
   - 参数: `model_id: string`
   - 上传模型选择到 Workers（/api/model）
   - 返回: `string` - 上传结果消息

5. **`upload_training_data_to_workers`**
   - 上传训练数据样本到 Workers（/api/training-data）
   - 返回: `string` - 上传结果消息

6. **`reassign_node_from_workers`**
   - 参数: `failed_nodes: string[]`, `current_splits: ModelSplit[]`, `request_id: string`
   - 请求重新分配节点到 Workers（/api/reassign-node）
   - 返回: `JSON` - 重新分配结果，包含新的切分方案和分配的节点

7. **`check_node_health_from_workers`**
   - 参数: `node_id: string`
   - 检查节点健康状态到 Workers（/api/node-health）
   - 返回: `JSON` - 节点健康状态，包括健康状态、最后在线时间、当前负载等

## 数据格式

### 节点信息 (DeviceInfoPayload) - POST /api/node-info

用于上传设备信息和节点状态：

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "device_info": {
    "gpu_type": "NVIDIA RTX 3080",
    "gpu_usage": 75.5,
    "gpu_memory_total": 10.0,
    "gpu_memory_used": 6.5,
    "cpu_cores": 8,
    "total_memory_gb": 16.0,
    "battery_level": 85.0,
    "is_charging": true
  },
  "metadata": {
    "platform": "windows",
    "app_version": "0.1.0",
    "node_id": null,
    "capabilities": {
      "os": "windows",
      "arch": "x86_64",
      "family": "windows"
    }
  }
}
```

### 推理请求 (InferenceRequestPayload) - POST /api/request

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "model_id": "bert-base-uncased",
  "input_data": {
    "text": "Hello, world!",
    "max_length": 100
  }
}
```

### 推理响应 (InferenceRequestResponse)

```json
{
  "success": true,
  "message": "Inference request processed successfully",
  "request_id": "req-123456",
  "selected_nodes": [
    {
      "node_id": "node-1",
      "endpoint": "192.168.1.100:8080",
      "capabilities": {
        "max_memory_gb": 16.0,
        "gpu_type": "NVIDIA RTX 3080",
        "gpu_memory_gb": 10.0,
        "cpu_cores": 8,
        "network_bandwidth_mbps": 1000,
        "supported_models": ["bert-base"]
      },
      "current_load": 0.3,
      "latency": 50,
      "reliability": 0.95
    }
  ],
  "model_split_plan": {
    "total_layers": 12,
    "splits": [
      {
        "layer_range": [0, 6],
        "assigned_node": "node-1",
        "memory_requirement_mb": 500,
        "compute_requirement": 100.5
      }
    ],
    "communication_overhead": 10.5,
    "estimated_inference_time": 1500
  },
  "estimated_total_time": 2000,
  "fallback_nodes": []
}
```

### 模型选择 (ModelSelectionPayload) - POST /api/model

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "model_selection": {
    "model_id": "bert-base-uncased",
    "model_name": "BERT Base",
    "selected_at": "2024-01-01T00:00:00Z",
    "selection_reason": "User selected from desktop interface"
  },
  "training_config": {
    "learning_rate": 0.00002,
    "batch_size": 32,
    "epochs": 100,
    "enable_distributed": true
  }
}
```

### 训练数据 (TrainingStatusPayload) - POST /api/training-data

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "training_status": {
    "is_running": true,
    "current_epoch": 45,
    "total_epochs": 100,
    "accuracy": 0.85,
    "loss": 0.25,
    "samples_processed": 1000000
  },
  "node_id": "node-identifier-if-available"
}
```

### 节点信息 (NodeInfo) - POST /api/node-info

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "node_info": {
    "node_id": "node-unique-id",
    "endpoint": "ip:port",
    "capabilities": {
      "max_memory_gb": 16.0,
      "gpu_type": "NVIDIA RTX 3080",
      "gpu_memory_gb": 10.0,
      "cpu_cores": 8,
      "network_bandwidth_mbps": 1000,
      "supported_models": ["bert-base", "gpt-2"]
    },
    "current_load": 0.75,
    "latency": 50,
    "reliability": 0.95
  }
}
```

### 节点重新分配 (NodeReassignmentPayload) - POST /api/reassign-node

```json
{
  "device_id": "unique-device-identifier",
  "timestamp": "2024-01-01T00:00:00Z",
  "failed_nodes": ["node-id-1", "node-id-2"],
  "current_splits": [
    {
      "layer_range": [0, 10],
      "assigned_node": "node-id-1",
      "memory_requirement_mb": 500,
      "compute_requirement": 100.5
    }
  ],
  "request_id": "request-unique-id"
}
```

## 使用示例

### 前端调用示例

```typescript
import { invoke } from '@tauri-apps/api/core';

// 测试连接
const isConnected = await invoke<boolean>('test_workers_connection');
console.log('连接状态:', isConnected);

// 上传设备信息
try {
  const result = await invoke<string>('upload_device_info_to_workers');
  console.log('设备信息上传结果:', result);
} catch (error) {
  console.error('上传失败:', error);
}

// 请求推理服务
try {
  const result = await invoke<Record<string, any>>('request_inference_from_workers', {
    modelId: 'bert-base-uncased',
    inputData: {
      text: 'Hello, world!',
      max_length: 100
    }
  });
  console.log('推理请求结果:', result);
  console.log('选择的节点:', result.selected_nodes);
  console.log('模型切分方案:', result.model_split_plan);
} catch (error) {
  console.error('推理请求失败:', error);
}

// 上传模型选择
try {
  const result = await invoke<string>('upload_model_selection_to_workers', {
    modelId: 'bert-base-uncased'
  });
  console.log('模型选择上传结果:', result);
} catch (error) {
  console.error('上传失败:', error);
}

// 上传训练数据
try {
  const result = await invoke<string>('upload_training_data_to_workers');
  console.log('训练数据上传结果:', result);
} catch (error) {
  console.error('上传失败:', error);
}

// 重新分配节点
try {
  const result = await invoke<any>('reassign_node_from_workers', {
    failedNodes: ['node-1', 'node-2'],
    currentSplits: [
      {
        layer_range: [0, 6],
        assigned_node: 'node-1',
        memory_requirement_mb: 500,
        compute_requirement: 100.5
      }
    ],
    requestId: 'req-123456'
  });
  console.log('节点重新分配结果:', result);
  console.log('新的切分方案:', result.new_splits);
  console.log('重新分配的节点:', result.reassigned_nodes);
} catch (error) {
  console.error('节点重新分配失败:', error);
}

// 检查节点健康状态
try {
  const result = await invoke<any>('check_node_health_from_workers', {
    nodeId: 'node-123'
  });
  console.log('节点健康状态:', result);
  console.log('是否健康:', result.is_healthy);
  console.log('当前负载:', result.current_load);
  console.log('问题列表:', result.issues);
} catch (error) {
  console.error('节点健康检查失败:', error);
}
```

### 自动上传机制

系统会自动在以下时机上传数据：

1. **应用启动时**: 上传设备信息和节点状态到 /api/node-info
2. **用户发起推理时**: 请求推理服务到 /api/request
3. **模型选择时**: 上传模型配置到 /api/model
4. **训练过程中**: 定期上传训练数据样本到 /api/training-data
5. **定期刷新**: 每60秒刷新设备信息

## 设备ID生成

系统会自动生成唯一的设备ID，优先级如下：

1. **主板UUID** (通过 `wmic csproduct get UUID`)
2. **MAC地址** (通过 `getmac` 命令)
3. **随机UUID** (备选方案)

## 错误处理

所有API调用都包含完整的错误处理：

- **网络错误**: 连接超时、DNS解析失败等
- **服务器错误**: Workers 后端返回错误状态
- **数据错误**: 数据格式不正确或缺失

## 配置

Workers 后端 URL 在 `src-tauri/src/state.rs` 中配置：

```rust
api_client: crate::api_client::WorkersApiClient::new(
    "https://williw.sirazede725.workers.dev".to_string()
)
```

可以根据需要修改为其他URL。

API 客户端实现位于 `src-tauri/src/api_client.rs`，包含所有端点的实现方法。

## API 端点映射表

| 功能 | 方法 | 端点 | Tauri命令 | 客户端方法 | 前端方法 |
|------|------|------|------|-----------|---------|
| 节点信息上报 | POST | `/api/node-info` | `upload_device_info_to_workers` | `upload_node_info_from_device` / `upload_node_info` | `uploadDeviceInfoToWorkers` |
| 推理请求 | POST | `/api/request` | `request_inference_from_workers` | `request_inference` | - |
| 发送模型名称 | POST | `/api/model` | `upload_model_selection_to_workers` | `upload_selected_model` | `uploadModelSelectionToWorkers` |
| 上传训练数据 | POST | `/api/training-data` | `upload_training_data_to_workers` | `upload_training_data` | `uploadTrainingDataToWorkers` |
| 节点重新分配 | POST | `/api/reassign-node` | `reassign_node_from_workers` | `reassign_node` | `reassignNodeFromWorkers` ✨ |
| 节点健康检查 | GET | `/api/node-health` | `check_node_health_from_workers` | `check_node_health` | `checkNodeHealthFromWorkers` ✨ |

## 安全性

- 所有数据通过 HTTPS 加密传输
- 设备ID自动生成，不包含敏感信息
- 可以在 Workers 后端添加认证机制

## 测试

运行以下命令测试API功能：

```bash
cd src-tauri
cargo run
```

应用启动后会自动：
1. 检测与 Workers 后端的连接
2. 上传设备信息
3. 准备接收模型选择和训练状态上传

## 故障排除

### 常见问题

1. **连接失败**: 检查网络连接和 Workers 后端状态
2. **上传失败**: 检查数据格式和服务器日志
3. **设备ID问题**: 检查系统权限和命令行工具可用性

### 调试

在 `src-tauri/src/api_client.rs` 中可以添加更多日志来调试网络请求。

## 新增 API 端点说明

除了前端直接调用的端点外，还新增了以下 API 端点，主要用于内部逻辑和高级功能：

### 1. 节点信息上报 (POST /api/node-info)
- **用途**: 节点上报自身状态和硬件信息，包括设备信息和P2P节点状态
- **客户端方法**:
  - `WorkersApiClient::upload_node_info_from_device()` - 从客户端设备上报信息
  - `WorkersApiClient::upload_node_info()` - 从P2P节点上报信息
- **使用场景**:
  - 应用启动时上报设备信息
  - P2P节点状态变化时上报节点信息
  - 定期刷新节点状态
- **状态**: 已实现，前端已集成

### 2. 节点重新分配 (POST /api/reassign-node)
- **用途**: 当推理过程中部分节点不可用时，请求后端重新分配节点
- **客户端方法**: `WorkersApiClient::reassign_node()`
- **Tauri命令**: `reassign_node_from_workers`
- **使用场景**: 推理任务执行过程中遇到节点故障
- **状态**: ✅ 已实现并集成

### 3. 节点健康检查 (GET /api/node-health)
- **用途**: 查询特定节点的健康状态
- **客户端方法**: `WorkersApiClient::check_node_health()`
- **Tauri命令**: `check_node_health_from_workers`
- **使用场景**: 定期监控节点可用性
- **状态**: ✅ 已实现并集成

## 客户端 API 方法参考

完整的 API 客户端方法列表（位于 `src-tauri/src/api_client.rs`）：

```rust
impl WorkersApiClient {
    // 基础连接
    pub fn new(base_url: String) -> Self
    pub async fn test_connection(&self) -> Result<bool>

    // 设备和推理相关
    pub async fn upload_node_info_from_device(&self, device_info: DeviceInfo) -> Result<ApiResponse>
    pub async fn request_inference(&self, model_id: String, input_data: serde_json::Value) -> Result<InferenceRequestResponse>
    pub async fn upload_selected_model(&self, model_config: ModelConfig, training_config: TrainingConfigData) -> Result<ApiResponse>

    // 训练相关
    pub async fn upload_training_data(&self, training_status: TrainingStatus, node_id: Option<String>) -> Result<ApiResponse>

    // 节点管理
    pub async fn upload_node_info(&self, node_info: NodeInfo) -> Result<ApiResponse>
    pub async fn reassign_node(&self, failed_nodes: Vec<String>, current_splits: Vec<ModelSplit>, request_id: String) -> Result<NodeReassignmentResponse>
    pub async fn check_node_health(&self, node_id: String) -> Result<NodeHealthResponse>

    // 辅助方法
    fn get_device_id(&self) -> String
    fn get_device_metadata(&self) -> DeviceMetadata
}
```
