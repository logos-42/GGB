import React, { useState, useEffect } from 'react';
import {
  Box,
  Dialog,
  DialogTitle,
  DialogContent,
  DialogActions,
  Button,
  TextField,
  Typography,
  Grid,
  useTheme,
  alpha,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { AppSettings } from '../types';

interface SettingsPanelProps {
  onClose: () => void;
}

export const SettingsPanel: React.FC<SettingsPanelProps> = ({ onClose }) => {
  const theme = useTheme();
  const [settings, setSettings] = useState<AppSettings>({
    privacy_level: 'medium',
    bandwidth_budget: 10,
    network_config: {
      max_peers: 10,
      bootstrap_nodes: [],
      port: 9000,
    },
    checkpoint_settings: {
      enabled: true,
      interval_minutes: 5,
      max_checkpoints: 10,
    },
  });

  useEffect(() => {
    loadSettings();
  }, []);

  const loadSettings = async () => {
    try {
      const loadedSettings = await invoke<AppSettings>('get_settings');
      setSettings(loadedSettings);
    } catch (error) {
      console.error('Error loading settings:', error);
    }
  };

  const handleSave = async () => {
    try {
      await invoke('update_settings', { newSettings: settings });
      onClose();
    } catch (error) {
      console.error('Error saving settings:', error);
    }
  };

  return (
    <Dialog
      open={true}
      onClose={onClose}
      maxWidth="md"
      fullWidth
      PaperProps={{
        sx: {
          background: alpha(theme.palette.background.paper, 0.95),
          backdropFilter: 'blur(20px)',
          border: `1px solid ${theme.palette.divider}`,
        },
      }}
    >
      <DialogTitle>
        <Typography variant="h6" component="span">设置</Typography>
      </DialogTitle>

      <DialogContent>
        <Grid container spacing={3}>
          {/* 隐私设置 */}
          <Grid item xs={12}>
            <Typography variant="h6" gutterBottom>
              隐私保护
            </Typography>
            <TextField
              select
              fullWidth
              label="隐私级别"
              value={settings.privacy_level}
              onChange={(e) => setSettings({ ...settings, privacy_level: e.target.value })}
              SelectProps={{
                native: true,
              }}
              helperText="选择隐私保护级别：高（本地训练）、中（差分隐私）、低（开放共享）"
            >
              <option value="high">高 - 仅本地训练</option>
              <option value="medium">中 - 差分隐私</option>
              <option value="low">低 - 开放共享</option>
            </TextField>
          </Grid>

          {/* 带宽设置 */}
          <Grid item xs={12} sm={6}>
            <Typography variant="h6" gutterBottom>
              带宽预算
            </Typography>
            <TextField
              fullWidth
              type="number"
              label="最大带宽 (MB/s)"
              value={settings.bandwidth_budget}
              onChange={(e) => setSettings({ 
                ...settings, 
                bandwidth_budget: parseInt(e.target.value) || 0 
              })}
              helperText="限制训练节点的网络带宽使用"
            />
          </Grid>

          {/* 网络配置 */}
          <Grid item xs={12}>
            <Typography variant="h6" gutterBottom>
              网络配置
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  type="number"
                  label="最大连接节点数"
                  value={settings.network_config.max_peers}
                  onChange={(e) => setSettings({
                    ...settings,
                    network_config: {
                      ...settings.network_config,
                      max_peers: parseInt(e.target.value) || 0
                    }
                  })}
                />
              </Grid>
              <Grid item xs={12} sm={6}>
                <TextField
                  fullWidth
                  type="number"
                  label="监听端口"
                  value={settings.network_config.port}
                  onChange={(e) => setSettings({
                    ...settings,
                    network_config: {
                      ...settings.network_config,
                      port: parseInt(e.target.value) || 0
                    }
                  })}
                />
              </Grid>
            </Grid>
          </Grid>

          {/* Checkpoint 设置 */}
          <Grid item xs={12}>
            <Typography variant="h6" gutterBottom>
              Checkpoint 设置
            </Typography>
            <Grid container spacing={2}>
              <Grid item xs={12} sm={4}>
                <TextField
                  fullWidth
                  select
                  label="启用 Checkpoint"
                  value={settings.checkpoint_settings.enabled}
                  onChange={(e) => setSettings({
                    ...settings,
                    checkpoint_settings: {
                      ...settings.checkpoint_settings,
                      enabled: e.target.value === 'true'
                    }
                  })}
                  SelectProps={{
                    native: true,
                  }}
                >
                  <option value="true">启用</option>
                  <option value="false">禁用</option>
                </TextField>
              </Grid>
              <Grid item xs={12} sm={4}>
                <TextField
                  fullWidth
                  type="number"
                  label="保存间隔 (分钟)"
                  value={settings.checkpoint_settings.interval_minutes}
                  onChange={(e) => setSettings({
                    ...settings,
                    checkpoint_settings: {
                      ...settings.checkpoint_settings,
                      interval_minutes: parseInt(e.target.value) || 0
                    }
                  })}
                  disabled={!settings.checkpoint_settings.enabled}
                />
              </Grid>
              <Grid item xs={12} sm={4}>
                <TextField
                  fullWidth
                  type="number"
                  label="最大 Checkpoint 数量"
                  value={settings.checkpoint_settings.max_checkpoints}
                  onChange={(e) => setSettings({
                    ...settings,
                    checkpoint_settings: {
                      ...settings.checkpoint_settings,
                      max_checkpoints: parseInt(e.target.value) || 0
                    }
                  })}
                  disabled={!settings.checkpoint_settings.enabled}
                />
              </Grid>
            </Grid>
          </Grid>
        </Grid>
      </DialogContent>

      <DialogActions>
        <Button onClick={onClose}>取消</Button>
        <Button onClick={handleSave} variant="contained">
          保存
        </Button>
      </DialogActions>
    </Dialog>
  );
};
