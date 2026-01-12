import React, { useState } from 'react';
import {
  Box,
  Switch,
  Typography,
  Card,
  CardContent,
  useTheme,
  alpha,
} from '@mui/material';
import { useTrainingStore } from '../store/trainingStore';
import { invoke } from '@tauri-apps/api/core';

export const TrainingSwitch: React.FC = () => {
  const theme = useTheme();
  const { isRunning, setRunning } = useTrainingStore();
  const [loading, setLoading] = useState(false);

  const handleToggle = async () => {
    if (loading) return;

    setLoading(true);
    try {
      if (isRunning) {
        const result = await invoke<string>('stop_training');
        console.log(result);
        setRunning(false);
      } else {
        const result = await invoke<string>('start_training');
        console.log(result);
        setRunning(true);
      }
    } catch (error) {
      console.error('Error toggling training:', error);
    } finally {
      setLoading(false);
    }
  };

  return (
    <Box
      sx={{
        position: 'absolute',
        top: 16,
        left: 16,
        zIndex: 10,
      }}
    >
      <Card
        sx={{
          background: alpha(theme.palette.background.paper, 0.9),
          backdropFilter: 'blur(10px)',
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: 1,
          minWidth: 150,
        }}
      >
        <CardContent sx={{ display: 'flex', alignItems: 'center', gap: 2, p: 1.5 }}>
          <Typography variant="body2" sx={{ fontWeight: 600 }}>
            {isRunning ? '已启动' : '已关闭'}
          </Typography>
          <Switch
            checked={isRunning}
            onChange={handleToggle}
            disabled={loading}
            size="small"
            sx={{
              '& .MuiSwitch-switchBase.Mui-checked': {
                color: '#4caf50',
              },
              '& .MuiSwitch-switchBase.Mui-checked + .MuiSwitch-track': {
                backgroundColor: '#4caf50',
              },
            }}
          />
        </CardContent>
      </Card>
    </Box>
  );
};
