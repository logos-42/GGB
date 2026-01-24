import React, { useState } from 'react';
import {
  Box,
  FormControl,
  Select,
  MenuItem,
  Typography,
  Card,
  CardContent,
  Button,
  Alert,
  TextField,
  useTheme,
  alpha,
  CircularProgress,
} from '@mui/material';
import PlayArrowIcon from '@mui/icons-material/PlayArrow';
import { invoke } from '@tauri-apps/api/core';
import { ModelConfig } from '../types';
import { useModelStore } from '../store/modelStore';

export const ModelSelector: React.FC = () => {
  const theme = useTheme();
  const { selectedModel, setSelectedModel } = useModelStore();

  // 固定模型列表，只包含一个模型
  const models: ModelConfig[] = [
    {
      id: 'llama-3.2-1b',
      name: 'llama3.2 1b',
      dimensions: 2048,
      learning_rate: 0.00002,
      batch_size: 32,
      description: 'LLaMA 3.2 1B parameter model'
    }
  ];

  // 推理请求状态
  const [inferenceInput, setInferenceInput] = useState<string>('Hello, world!');
  const [inferenceLoading, setInferenceLoading] = useState<boolean>(false);
  const [inferenceResult, setInferenceResult] = useState<any>(null);
  const [inferenceError, setInferenceError] = useState<string>('');

  const handleModelChange = (event: any) => {
    const modelId = event.target.value as string;
    setSelectedModel(modelId);
    console.log(`Selected model: ${modelId}`);
  };

  const handleInferenceRequest = async () => {
    if (!selectedModel) {
      setInferenceError('请先选择一个模型');
      return;
    }

    setInferenceLoading(true);
    setInferenceError('');
    setInferenceResult(null);

    try {
      const result = await invoke<any>('request_inference_from_workers', {
        modelId: selectedModel,
        inputData: {
          text: inferenceInput,
          max_length: 100
        }
      });

      setInferenceResult(result);
      console.log('Inference request successful:', result);
    } catch (error: any) {
      console.error('Inference request failed:', error);
      setInferenceError(error || '推理请求失败');
    } finally {
      setInferenceLoading(false);
    }
  };

  return (
    <Box
      sx={{
        width: '100%',
      }}
    >
      <Card
        sx={{
          background: alpha(theme.palette.background.paper, 0.9),
          backdropFilter: 'blur(10px)',
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: 1,
        }}
      >
        <CardContent sx={{ p: 2 }}>
          <Box sx={{ display: 'flex', flexDirection: 'column', gap: 2 }}>
            {/* 模型选择 */}
            <Box>
              <Typography variant="caption" sx={{ mb: 1, color: 'text.secondary', display: 'block' }}>
                模型
              </Typography>
              <FormControl fullWidth size="small">
                <Select
                  value={selectedModel || 'llama-3.2-1b'}
                  onChange={handleModelChange}
                  disabled={inferenceLoading}
                  sx={{
                    fontSize: '0.875rem',
                    '& .MuiOutlinedInput-root': {
                      fieldset: {
                        borderColor: theme.palette.divider,
                      },
                    },
                  }}
                >
                  {models.map((model) => (
                    <MenuItem key={model.id} value={model.id} sx={{ fontSize: '0.875rem' }}>
                      {model.name}
                    </MenuItem>
                  ))}
                </Select>
              </FormControl>
            </Box>

            {/* 输入文本 */}
            <Box>
              <Typography variant="caption" sx={{ mb: 1, color: 'text.secondary', display: 'block' }}>
                输入文本
              </Typography>
              <TextField
                fullWidth
                multiline
                rows={3}
                value={inferenceInput}
                onChange={(e) => setInferenceInput(e.target.value)}
                placeholder="请输入要推理的文本..."
                disabled={inferenceLoading}
                size="small"
                sx={{
                  '& .MuiOutlinedInput-root': {
                    fieldset: {
                      borderColor: theme.palette.divider,
                    },
                  },
                }}
              />
            </Box>

            {/* 运行按钮 */}
            <Box sx={{ display: 'flex', justifyContent: 'center' }}>
              <Button
                variant="contained"
                startIcon={inferenceLoading ? <CircularProgress size={16} /> : <PlayArrowIcon />}
                onClick={handleInferenceRequest}
                disabled={inferenceLoading}
                sx={{
                  px: 3,
                  py: 1,
                  fontSize: '0.875rem',
                }}
              >
                {inferenceLoading ? '运行中...' : '运行'}
              </Button>
            </Box>

            {/* 推理结果 */}
            {inferenceError && (
              <Alert severity="error" sx={{ mt: 1 }}>
                {inferenceError}
              </Alert>
            )}

            {inferenceResult && (
              <Alert severity="success" sx={{ mt: 1 }}>
                <Typography variant="body2" gutterBottom>
                  推理请求成功！
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                  请求ID: {inferenceResult.request_id || 'N/A'}
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                  分配节点数: {inferenceResult.selected_nodes?.length || 0}
                </Typography>
                <Typography variant="caption" color="text.secondary" display="block">
                  预计总时间: {inferenceResult.estimated_total_time || 0}ms
                </Typography>
              </Alert>
            )}
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};
  </Typography>
              </Alert>
            )}
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};
