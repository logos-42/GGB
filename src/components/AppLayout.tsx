import React from 'react';
import { Box } from '@mui/material';
import { TrainingSwitch } from './TrainingSwitch';
import { ModelSelector } from './ModelSelector';
import { SettingsButton } from './SettingsButton';
import { TrainingDashboard } from './TrainingDashboard';
import { SettingsPanel } from './SettingsPanel';
import { useTrainingStore } from '../store/trainingStore';

export const AppLayout: React.FC = () => {
  const { isSettingsOpen, openSettings, closeSettings } = useTrainingStore();

  return (
    <Box
      sx={{
        width: '100vw',
        height: '100vh',
        position: 'relative',
        background: 'black',
      }}
    >
      {/* 左侧训练开关 */}
      <TrainingSwitch />

      {/* 右上角设置按钮 */}
      <SettingsButton onClick={openSettings} />

      {/* 主内容区域 */}
      <Box
        sx={{
          position: 'absolute',
          top: '80px',
          left: '80px',
          right: 0,
          bottom: '70px',
          overflow: 'auto',
          p: 3,
        }}
      >
        <TrainingDashboard />
      </Box>

      {/* 底部模型选择 */}
      <ModelSelector />

      {/* 设置面板 */}
      {isSettingsOpen && <SettingsPanel onClose={closeSettings} />}
    </Box>
  );
};
