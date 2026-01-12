import React, { useState, useRef, useEffect } from 'react';
import {
  Box,
  Card,
  CardContent,
  TextField,
  IconButton,
  Typography,
  List,
  ListItem,
  ListItemText,
  useTheme,
  alpha,
} from '@mui/material';
import SendIcon from '@mui/icons-material/Send';
import SmartToyIcon from '@mui/icons-material/SmartToy';
import PersonIcon from '@mui/icons-material/Person';

interface ChatMessage {
  id: string;
  content: string;
  sender: 'user' | 'assistant';
  timestamp: Date;
}

interface ChatBoxProps {
  expanded?: boolean;
  onExpand?: () => void;
}

// 常量配置
const AI_REPLY_DELAY = 1000; // AI回复延迟（毫秒）

export const ChatBox: React.FC<ChatBoxProps> = ({ expanded = false, onExpand }) => {
  const theme = useTheme();
  const [messages, setMessages] = useState<ChatMessage[]>([]);
  const [inputText, setInputText] = useState('');
  const messagesEndRef = useRef<HTMLDivElement>(null);

  const scrollToBottom = () => {
    messagesEndRef.current?.scrollIntoView({ behavior: 'smooth' });
  };

  useEffect(() => {
    scrollToBottom();
  }, [messages]);

  const handleSendMessage = () => {
    if (!inputText.trim()) return;

    const userMessage: ChatMessage = {
      id: Date.now().toString(),
      content: inputText.trim(),
      sender: 'user',
      timestamp: new Date(),
    };

    setMessages(prev => [...prev, userMessage]);
    setInputText('');

    // 只在首次发送消息时展开聊天框
    if (messages.length === 0) {
      onExpand?.();
    }

    // 模拟AI回复
    setTimeout(() => {
      const assistantMessage: ChatMessage = {
        id: (Date.now() + 1).toString(),
        content: '这是一个模拟回复。在实际应用中，我将连接AI模型来提供智能回答。',
        sender: 'assistant',
        timestamp: new Date(),
      };
      setMessages(prev => [...prev, assistantMessage]);
    }, AI_REPLY_DELAY);
  };

  const handleKeyPress = (e: React.KeyboardEvent) => {
    if (e.key === 'Enter' && !e.shiftKey) {
      e.preventDefault();
      handleSendMessage();
    }
  };

  return (
    <Box
      sx={{
        width: '100%',
        height: '100%',
        display: 'flex',
        flexDirection: 'column',
      }}
    >
      <Card
        sx={{
          background: alpha(theme.palette.background.paper, 0.9),
          backdropFilter: 'blur(10px)',
          border: `1px solid ${theme.palette.divider}`,
          borderRadius: 1,
          flex: 1,
          display: 'flex',
          flexDirection: 'column',
          transition: 'all 0.3s ease',
          minHeight: expanded ? 500 : 300,
        }}
      >
        <CardContent sx={{ p: 1.5, flex: 1, display: 'flex', flexDirection: 'column', minHeight: 0 }}>
          <>
            <Box sx={{ flex: 1, overflow: 'auto', mb: 1.5, minHeight: 0 }}>
            {messages.length === 0 ? (
              <Box sx={{ display: 'flex', alignItems: 'center', justifyContent: 'center', height: '100%', color: 'text.secondary' }}>
                <Typography variant="body2">开始对话...</Typography>
              </Box>
            ) : (
              <List sx={{ p: 0 }}>
                {messages.map((message) => (
                  <ListItem
                    key={message.id}
                    sx={{
                      py: 0.5,
                      px: 0,
                      alignItems: 'flex-start',
                      flexDirection: message.sender === 'user' ? 'row-reverse' : 'row',
                    }}
                  >
                    <Box
                      sx={{
                        display: 'flex',
                        alignItems: 'center',
                        justifyContent: 'center',
                        width: 24,
                        height: 24,
                        borderRadius: '50%',
                        mr: message.sender === 'user' ? 0 : 1,
                        ml: message.sender === 'user' ? 1 : 0,
                        background: message.sender === 'user'
                          ? alpha(theme.palette.primary.main, 0.2)
                          : alpha(theme.palette.secondary.main, 0.2),
                        color: message.sender === 'user'
                          ? theme.palette.primary.main
                          : theme.palette.secondary.main,
                      }}
                    >
                      {message.sender === 'user' ? (
                        <PersonIcon sx={{ fontSize: 16 }} />
                      ) : (
                        <SmartToyIcon sx={{ fontSize: 16 }} />
                      )}
                    </Box>

                    <ListItemText
                      primary={
                        <Box
                          sx={{
                            p: 1,
                            borderRadius: 1,
                            background: message.sender === 'user'
                              ? alpha(theme.palette.primary.main, 0.1)
                              : alpha(theme.palette.secondary.main, 0.1),
                            border: `1px solid ${alpha(theme.palette.divider, 0.3)}`,
                            maxWidth: '70%',
                          }}
                        >
                          <Typography variant="body2" sx={{ fontSize: '0.875rem' }}>
                            {message.content}
                          </Typography>
                        </Box>
                      }
                      secondary={
                        <Typography
                          variant="caption"
                          sx={{
                            display: 'block',
                            mt: 0.5,
                            textAlign: message.sender === 'user' ? 'right' : 'left',
                            color: 'text.secondary',
                          }}
                        >
                          {message.timestamp.toLocaleTimeString([], { hour: '2-digit', minute: '2-digit' })}
                        </Typography>
                      }
                      sx={{
                        mx: 0,
                        '& .MuiListItemText-primary': { mb: 0.5 },
                      }}
                    />
                  </ListItem>
                ))}
                <div ref={messagesEndRef} />
                </List>
              )}
            </Box>

            <Box sx={{ display: 'flex', gap: 1 }}>
            <TextField
              fullWidth
              size="small"
              placeholder="输入消息..."
              value={inputText}
              onChange={(e) => setInputText(e.target.value)}
              onKeyPress={handleKeyPress}
              multiline
              maxRows={2}
              sx={{
                '& .MuiOutlinedInput-root': {
                  fontSize: '0.875rem',
                  fieldset: {
                    borderColor: alpha(theme.palette.divider, 0.5),
                  },
                },
              }}
            />
            <IconButton
              size="small"
              onClick={handleSendMessage}
              disabled={!inputText.trim()}
              sx={{
                background: inputText.trim()
                  ? alpha(theme.palette.primary.main, 0.2)
                  : 'transparent',
                color: inputText.trim()
                  ? theme.palette.primary.main
                  : theme.palette.text.disabled,
              }}
            >
              <SendIcon sx={{ fontSize: 20 }} />
            </IconButton>
            </Box>
          </>
        </CardContent>
      </Card>
    </Box>
  );
};