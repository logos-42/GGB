import React, { useEffect, useState } from 'react';
import {
  Box,
  FormControl,
  Select,
  MenuItem,
  Typography,
  Card,
  CardContent,
  Chip,
  useTheme,
  alpha,
} from '@mui/material';
import { invoke } from '@tauri-apps/api/core';
import { ModelConfig } from '../types';

export const ModelSelector: React.FC = () => {
  const theme = useTheme();
  const [models, setModels] = useState<ModelConfig[]>([]);
  const [selectedModel, setSelectedModel] = useState<string>('');

  useEffect(() => {
    loadModels();
  }, []);

  const loadModels = async () => {
    try {
      const availableModels = await invoke<ModelConfig[]>('get_available_models');
      setModels(availableModels);
      if (availableModels.length > 0) {
        setSelectedModel(availableModels[0].id);
      }
    } catch (error) {
      console.error('Error loading models:', error);
    }
  };

  const handleModelChange = async (event: any) => {
    const modelId = event.target.value as string;
    setSelectedModel(modelId);
    
    try {
      await invoke('select_model', { modelId });
      console.log(`Selected model: ${modelId}`);
    } catch (error) {
      console.error('Error selecting model:', error);
    }
  };

  const currentModel = models.find(m => m.id === selectedModel);

  return (
    <Box
      sx={{
        position: 'absolute',
        bottom: 8,
        left: 8,
        zIndex: 10,
        width: 'auto',
        minWidth: 300,
        maxWidth: 400,
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
        <CardContent sx={{ p: 1, '&:last-child': { pb: 1 } }}>
          <Box sx={{ display: 'flex', alignItems: 'center', gap: 2, flexWrap: 'wrap' }}>
            <Box sx={{ flex: 1, minWidth: 150 }}>
              <Typography variant="caption" sx={{ mb: 0.5, color: 'text.secondary', display: 'block' }}>
                模型
              </Typography>
              <FormControl fullWidth size="small">
                <Select
                  value={selectedModel}
                  onChange={handleModelChange}
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

            {currentModel && (
              <Box sx={{ display: 'flex', gap: 0.5, flexWrap: 'wrap' }}>
                <Chip
                  label={`${currentModel.dimensions}d`}
                  size="small"
                  sx={{ 
                    background: alpha(theme.palette.primary.main, 0.1),
                    fontSize: '0.75rem',
                    height: 20,
                  }}
                />
                <Chip
                  label={`lr: ${currentModel.learning_rate}`}
                  size="small"
                  sx={{ 
                    background: alpha(theme.palette.primary.main, 0.1),
                    fontSize: '0.75rem',
                    height: 20,
                  }}
                />
                <Chip
                  label={`bs: ${currentModel.batch_size}`}
                  size="small"
                  sx={{ 
                    background: alpha(theme.palette.primary.main, 0.1),
                    fontSize: '0.75rem',
                    height: 20,
                  }}
                />
              </Box>
            )}
          </Box>
        </CardContent>
      </Card>
    </Box>
  );
};
