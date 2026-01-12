import React, { useEffect, useState } from 'react';
import {
  Box,
  Card,
  CardContent,
  Grid,
  Typography,
  LinearProgress,
  useTheme,
  alpha,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { TrainingStatus, DeviceInfo } from '../types';
import { useTrainingStore } from '../store/trainingStore';

export const TrainingDashboard: React.FC = () => {
  const theme = useTheme();
  const { isRunning } = useTrainingStore();
  const [trainingStatus, setTrainingStatus] = useState<TrainingStatus | null>(null);
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);

  useEffect(() => {
    loadStatus();
    loadDeviceInfo();
    
    // Poll for updates every second
    const interval = setInterval(() => {
      loadStatus();
    }, 1000);

    return () => clearInterval(interval);
  }, []);

  const loadStatus = async () => {
    try {
      const status = await invoke<TrainingStatus>('get_training_stats');
      setTrainingStatus(status);
    } catch (error) {
      console.error('Error loading training status:', error);
    }
  };

  const loadDeviceInfo = async () => {
    try {
      const info = await invoke<DeviceInfo>('get_device_info');
      setDeviceInfo(info);
    } catch (error) {
      console.error('Error loading device info:', error);
    }
  };

  return (
    <Grid container spacing={2}>
      {/* 训练状态卡片 */}
      <Grid item xs={12} md={6}>
        <Card
          sx={{
            background: alpha(theme.palette.background.paper, 0.6),
            backdropFilter: 'blur(10px)',
            border: `1px solid ${theme.palette.divider}`,
          }}
        >
          <CardContent sx={{ p: 2 }}>
            <Typography variant="subtitle2" gutterBottom sx={{ fontSize: '0.875rem' }}>
              训练状态
            </Typography>
            
            {trainingStatus ? (
              <Box>
                <Box sx={{ mb: 2 }}>
                  <Box sx={{ display: 'flex', justifyContent: 'space-between', mb: 1 }}>
                    <Typography variant="body2" color="text.secondary">
                      训练进度
                    </Typography>
                    <Typography variant="body2" color="text.secondary">
                      {trainingStatus.current_epoch} / {trainingStatus.total_epochs}
                    </Typography>
                  </Box>
                  <LinearProgress
                    variant="determinate"
                    value={(trainingStatus.current_epoch / trainingStatus.total_epochs) * 100}
                    sx={{
                      height: 8,
                      borderRadius: 4,
                      background: alpha(theme.palette.primary.main, 0.1),
                    }}
                  />
                </Box>

                <Grid container spacing={2}>
                  <Grid item xs={6}>
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      准确率
                    </Typography>
                    <Typography variant="h4" color="primary">
                      {(trainingStatus.accuracy * 100).toFixed(1)}%
                    </Typography>
                  </Grid>
                  <Grid item xs={6}>
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      损失
                    </Typography>
                    <Typography variant="h4" color="secondary">
                      {trainingStatus.loss.toFixed(4)}
                    </Typography>
                  </Grid>

                  <Grid item xs={6}>
                    <Typography variant="body2" color="text.secondary" gutterBottom>
                      处理样本数
                    </Typography>
                    <Typography variant="h5">
                      {trainingStatus.samples_processed.toLocaleString()}
                    </Typography>
                  </Grid>
                </Grid>
              </Box>
            ) : (
              <Typography color="text.secondary">加载中...</Typography>
            )}
          </CardContent>
        </Card>
      </Grid>

      {/* 设备信息卡片 */}
      <Grid item xs={12} md={6}>
        <Card
          sx={{
            background: alpha(theme.palette.background.paper, 0.6),
            backdropFilter: 'blur(10px)',
            border: `1px solid ${theme.palette.divider}`,
          }}
        >
          <CardContent sx={{ p: 2 }}>
            <Typography variant="subtitle2" gutterBottom sx={{ fontSize: '0.875rem' }}>
              设备信息
            </Typography>
            
            {deviceInfo ? (
              <Grid container spacing={1.5}>
                <Grid item xs={6}>
                  <Typography variant="caption" color="text.secondary" display="block">
                    GPU类型
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {deviceInfo.gpu_type || '未检测到'}
                  </Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="caption" color="text.secondary" display="block">
                    GPU使用率
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {deviceInfo.gpu_usage != null ? `${deviceInfo.gpu_usage.toFixed(1)}%` : 'N/A'}
                  </Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="caption" color="text.secondary" display="block">
                    GPU内存
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {deviceInfo.gpu_memory_used != null && deviceInfo.gpu_memory_total != null ? 
                      `${deviceInfo.gpu_memory_used.toFixed(1)}/${deviceInfo.gpu_memory_total.toFixed(1)} GB` : 'N/A'}
                  </Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="caption" color="text.secondary" display="block">
                    CPU核心
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {deviceInfo.cpu_cores}核
                  </Typography>
                </Grid>
                <Grid item xs={6}>
                  <Typography variant="caption" color="text.secondary" display="block">
                    总内存
                  </Typography>
                  <Typography variant="body2" sx={{ fontWeight: 500 }}>
                    {deviceInfo.total_memory_gb.toFixed(1)}GB
                  </Typography>
                </Grid>
                {deviceInfo.battery_level != null && (
                  <Grid item xs={6}>
                    <Typography variant="caption" color="text.secondary" display="block">
                      电池
                    </Typography>
                    <Typography variant="body2" sx={{ fontWeight: 500 }}>
                      {deviceInfo.battery_level.toFixed(0)}%
                    </Typography>
                  </Grid>
                )}
              </Grid>
            ) : (
              <Typography color="text.secondary" variant="body2">加载中...</Typography>
            )}
          </CardContent>
        </Card>
      </Grid>

      {/* 网络状态卡片 */}
      <Grid item xs={12}>
        <Card
          sx={{
            background: alpha(theme.palette.background.paper, 0.6),
            backdropFilter: 'blur(10px)',
            border: `1px solid ${theme.palette.divider}`,
          }}
        >
          <CardContent sx={{ p: 2 }}>
            <Typography variant="subtitle2" gutterBottom sx={{ fontSize: '0.875rem' }}>
              网络状态
            </Typography>
            
            <Grid container spacing={2}>
              <Grid item xs={12} md={6}>
                <Box
                  sx={{
                    p: 1.5,
                    borderRadius: 1,
                    background: alpha(isRunning ? theme.palette.success.main : theme.palette.grey[700], 0.1),
                    textAlign: 'center',
                  }}
                >
                  <Typography variant="h4" color={isRunning ? 'success.main' : 'text.secondary'} sx={{ fontSize: '1.5rem' }}>
                    {isRunning ? '在线' : '离线'}
                  </Typography>
                  <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                    节点状态
                  </Typography>
                </Box>
              </Grid>
              

              
              <Grid item xs={12} md={6}>
                <Box
                  sx={{
                    p: 1.5,
                    borderRadius: 1,
                    background: alpha(theme.palette.warning.main, 0.1),
                    textAlign: 'center',
                  }}
                >
                  <Typography variant="h4" color="warning.main" sx={{ fontSize: '1.5rem' }}>
                    P2P
                  </Typography>
                  <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                    网络类型
                  </Typography>
                </Box>
              </Grid>
            </Grid>
          </CardContent>
        </Card>
      </Grid>
    </Grid>
  );
};
