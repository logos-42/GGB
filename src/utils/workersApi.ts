// 前端调用Workers API的示例代码
// 可以添加到你的React组件中

import { invoke } from '@tauri-apps/api/core';

// 测试与Workers后端的连接
export async function testWorkersConnection(): Promise<boolean> {
  try {
    const isConnected = await invoke<boolean>('test_workers_connection');
    console.log('Workers connection:', isConnected);
    return isConnected;
  } catch (error) {
    console.error('Failed to test workers connection:', error);
    return false;
  }
}

// 上传设备信息到Workers后端
export async function uploadDeviceInfoToWorkers(): Promise<string> {
  try {
    const result = await invoke<string>('upload_device_info_to_workers');
    console.log('Device info uploaded:', result);
    return result;
  } catch (error) {
    console.error('Failed to upload device info:', error);
    throw error;
  }
}

// 上传模型选择到Workers后端
export async function uploadModelSelectionToWorkers(modelId: string): Promise<string> {
  try {
    const result = await invoke<string>('upload_model_selection_to_workers', {
      modelId: modelId
    });
    console.log('Model selection uploaded:', result);
    return result;
  } catch (error) {
    console.error('Failed to upload model selection:', error);
    throw error;
  }
}

// 上传训练数据到Workers后端
export async function uploadTrainingDataToWorkers(): Promise<string> {
  try {
    const result = await invoke<string>('upload_training_data_to_workers');
    console.log('Training data uploaded:', result);
    return result;
  } catch (error) {
    console.error('Failed to upload training data:', error);
    throw error;
  }
}

// 重新分配节点
export async function reassignNodeFromWorkers(
  failedNodes: string[],
  currentSplits: Array<{ layer_range: [number, number]; assigned_node: string; memory_requirement_mb: number; compute_requirement: number }>,
  requestId: string
): Promise<any> {
  try {
    const result = await invoke<any>('reassign_node_from_workers', {
      failedNodes: failedNodes,
      currentSplits: currentSplits,
      requestId: requestId
    });
    console.log('Node reassignment result:', result);
    return result;
  } catch (error) {
    console.error('Failed to reassign nodes:', error);
    throw error;
  }
}

// 检查节点健康状态
export async function checkNodeHealthFromWorkers(nodeId: string): Promise<any> {
  try {
    const result = await invoke<any>('check_node_health_from_workers', {
      nodeId: nodeId
    });
    console.log('Node health check result:', result);
    return result;
  } catch (error) {
    console.error('Failed to check node health:', error);
    throw error;
  }
}

// 使用示例：在组件中自动上传数据
export class WorkersDataUploader {
  private static instance: WorkersDataUploader;
  private uploadInterval: number | null = null;

  private constructor() {}

  static getInstance(): WorkersDataUploader {
    if (!WorkersDataUploader.instance) {
      WorkersDataUploader.instance = new WorkersDataUploader();
    }
    return WorkersDataUploader.instance;
  }

  // 开始定期上传数据
  async startPeriodicUpload(intervalMs: number = 60000): Promise<void> {
    // 首先测试连接
    const isConnected = await testWorkersConnection();
    if (!isConnected) {
      console.warn('Cannot connect to Workers backend');
      return;
    }

    // 立即上传一次设备信息
    try {
      await uploadDeviceInfoToWorkers();
    } catch (error) {
      console.error('Initial device info upload failed:', error);
    }

    // 设置定期上传
    this.uploadInterval = setInterval(async () => {
      try {
        await uploadTrainingDataToWorkers();
      } catch (error) {
        console.error('Periodic upload failed:', error);
      }
    }, intervalMs);

    console.log('Started periodic data upload to Workers');
  }

  // 停止定期上传
  stopPeriodicUpload(): void {
    if (this.uploadInterval) {
      clearInterval(this.uploadInterval);
      this.uploadInterval = null;
      console.log('Stopped periodic data upload');
    }
  }

  // 手动上传模型选择
  async uploadModelSelection(modelId: string): Promise<void> {
    try {
      await uploadModelSelectionToWorkers(modelId);
    } catch (error) {
      console.error('Model selection upload failed:', error);
      throw error;
    }
  }
}

// 在React组件中使用示例：
/*
import { WorkersDataUploader } from './workersApi';

function MyComponent() {
  const [uploader] = useState(() => WorkersDataUploader.getInstance());

  useEffect(() => {
    // 组件挂载时开始定期上传
    uploader.startPeriodicUpload(30000); // 每30秒上传一次

    return () => {
      // 组件卸载时停止上传
      uploader.stopPeriodicUpload();
    };
  }, []);

  const handleModelSelect = async (modelId: string) => {
    try {
      await uploader.uploadModelSelection(modelId);
      // 处理模型选择逻辑...
    } catch (error) {
      // 处理错误...
    }
  };

  return (
    <div>
      <button onClick={() => handleModelSelect('bert-base-uncased')}>
        Select BERT Model
      </button>
    </div>
  );
}
*/
