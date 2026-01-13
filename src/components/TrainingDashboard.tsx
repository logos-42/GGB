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
  IconButton,
} from '@mui/material';
import ExpandMoreIcon from '@mui/icons-material/ExpandMore';
import ExpandLessIcon from '@mui/icons-material/ExpandLess';
import { invoke } from '@tauri-apps/api/core';
import { listen } from '@tauri-apps/api/event';
import { TrainingStatus, DeviceInfo } from '../types';
import { useTrainingStore } from '../store/trainingStore';

// 可折叠卡片组件
interface CollapsibleCardProps {
  title: string;
  children: React.ReactNode;
  defaultCollapsed?: boolean;
}

const CollapsibleCard: React.FC<CollapsibleCardProps> = ({ title, children, defaultCollapsed = false }) => {
  const [collapsed, setCollapsed] = useState(defaultCollapsed);

  return (
    <Card
      onClick={() => collapsed && setCollapsed(false)}
      sx={{
        background: alpha('#000000', 0.6),
        backdropFilter: 'blur(10px)',
        border: '1px solid rgba(255, 255, 255, 0.1)',
        cursor: collapsed ? 'pointer' : 'default',
      }}
    >
      <CardContent sx={{ p: 2 }}>
        <Box
          sx={{
            display: 'flex',
            justifyContent: 'space-between',
            alignItems: 'center',
            mb: collapsed ? 0 : 1,
          }}
        >
          <Typography variant="subtitle2" sx={{ fontSize: '0.875rem' }}>
            {title}
          </Typography>
          <IconButton
            size="small"
            onClick={(e) => {
              e.stopPropagation();
              setCollapsed(!collapsed);
            }}
            sx={{
              color: 'text.secondary',
              '&:hover': {
                color: 'text.primary',
              },
            }}
          >
            {collapsed ? <ExpandMoreIcon /> : <ExpandLessIcon />}
          </IconButton>
        </Box>
        {!collapsed && children}
      </CardContent>
    </Card>
  );
};

export const TrainingDashboard: React.FC = () => {
  const theme = useTheme();
  const { isRunning } = useTrainingStore();
  const [trainingStatus, setTrainingStatus] = useState<TrainingStatus | null>(null);
  const [deviceInfo, setDeviceInfo] = useState<DeviceInfo | null>(null);
  const [connectedPeers, setConnectedPeers] = useState(0);
  const [maxPeers] = useState(10);

  useEffect(() => {
    loadStatus();
    loadDeviceInfo();
    loadConnectedPeers();

    // Poll for training status every second
    const statusInterval = setInterval(() => {
      loadStatus();
    }, 1000);

    // Poll for connected peers every minute
    const peersInterval = setInterval(() => {
      loadConnectedPeers();
    }, 60000);

    // Poll for device info every minute
    const deviceInterval = setInterval(() => {
      loadDeviceInfo();
    }, 60000);

    // Listen for backend device info refresh events
    const unlisten = listen('device_info_refresh', () => {
      loadDeviceInfo();
    });

    return () => {
      clearInterval(statusInterval);
      clearInterval(peersInterval);
      clearInterval(deviceInterval);
      unlisten.then(fn => fn());
    };
  }, []);

  const loadConnectedPeers = async () => {
    try {
      // TODO: 调用后端API获取实际连接数
      // const peers = await invoke<number>('get_connected_peers');
      // 暂时使用模拟数据（固定值，避免跳动）
      const peers = isRunning ? 3 : 0;
      setConnectedPeers(peers);
    } catch (error) {
      console.error('Error loading connected peers:', error);
      setConnectedPeers(0);
    }
  };

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
        <CollapsibleCard title="训练状态">
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
        </CollapsibleCard>
      </Grid>

      {/* 设备信息卡片 */}
      <Grid item xs={12} md={6}>
        <CollapsibleCard title="设备信息">
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
        </CollapsibleCard>
      </Grid>

      {/* 网络状态卡片 */}
      <Grid item xs={12}>
        <CollapsibleCard title="网络状态">
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
                  {connectedPeers}/{maxPeers}
                </Typography>
                <Typography variant="caption" color="text.secondary" sx={{ mt: 0.5, display: 'block' }}>
                  连接节点数
                </Typography>
              </Box>
            </Grid>
          </Grid>
        </CollapsibleCard>
      </Grid>
    </Grid>
  );
};
