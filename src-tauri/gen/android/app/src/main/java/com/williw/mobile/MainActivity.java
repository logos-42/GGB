package com.williw.mobile;

import android.os.Bundle;
import android.view.View;
import android.widget.EditText;
import android.widget.ImageButton;
import androidx.appcompat.app.AppCompatActivity;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import java.util.ArrayList;
import java.util.List;

public class MainActivity extends AppCompatActivity {
    private EditText messageEditText;
    private ImageButton sendButton;
    private RecyclerView chatRecyclerView;
    private ChatAdapter chatAdapter;
    private List<ChatMessage> messageList;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        // 初始化UI组件
        initViews();
        
        // 设置RecyclerView
        setupRecyclerView();
        
        // 设置点击监听器
        setupListeners();
        
        // 添加欢迎消息
        addWelcomeMessage();
    }
    
    private void initViews() {
        messageEditText = findViewById(R.id.messageEditText);
        sendButton = findViewById(R.id.sendButton);
        chatRecyclerView = findViewById(R.id.chatRecyclerView);
    }
    
    private void setupRecyclerView() {
        messageList = new ArrayList<>();
        chatAdapter = new ChatAdapter(messageList);
        chatRecyclerView.setLayoutManager(new LinearLayoutManager(this));
        chatRecyclerView.setAdapter(chatAdapter);
    }
    
    private void setupListeners() {
        sendButton.setOnClickListener(v -> sendMessage());
        
        // 监听EditText文本变化，动态启用/禁用发送按钮
        messageEditText.addTextChangedListener(new android.text.TextWatcher() {
            @Override
            public void beforeTextChanged(CharSequence s, int start, int count, int after) {}
            
            @Override
            public void onTextChanged(CharSequence s, int start, int before, int count) {
                sendButton.setEnabled(s.toString().trim().length() > 0);
            }
            
            @Override
            public void afterTextChanged(android.text.Editable s) {}
        });
        
        // 初始状态下禁用发送按钮
        sendButton.setEnabled(false);
    }
    
    private void sendMessage() {
        String message = messageEditText.getText().toString().trim();
        if (message.isEmpty()) {
            return;
        }
        
        // 添加用户消息
        ChatMessage userMessage = new ChatMessage(message, ChatMessage.TYPE_USER);
        messageList.add(userMessage);
        chatAdapter.notifyItemInserted(messageList.size() - 1);
        
        // 清空输入框
        messageEditText.setText("");
        
        // 滚动到底部
        chatRecyclerView.smoothScrollToPosition(messageList.size() - 1);
        
        // 模拟AI回复
        simulateAIResponse(message);
    }
    
    private void simulateAIResponse(String userMessage) {
        // 延迟1秒后添加AI回复，模拟思考过程
        messageEditText.postDelayed(() -> {
            String aiResponse = generateAIResponse(userMessage);
            ChatMessage aiMessage = new ChatMessage(aiResponse, ChatMessage.TYPE_AI);
            messageList.add(aiMessage);
            chatAdapter.notifyItemInserted(messageList.size() - 1);
            
            // 滚动到底部
            chatRecyclerView.smoothScrollToPosition(messageList.size() - 1);
        }, 1000);
    }
    
    private String generateAIResponse(String userMessage) {
        // 简单的AI回复逻辑
        if (userMessage.contains("你好") || userMessage.contains("hi") || userMessage.contains("hello")) {
            return "你好！我是Williw AI助手，很高兴为您服务！我可以帮助您了解去中心化训练的相关内容。";
        } else if (userMessage.contains("训练") || userMessage.contains("学习")) {
            return "去中心化训练是一种创新的机器学习方式，它允许多个设备协同训练模型，同时保护数据隐私。我们的平台支持BERT、GPT等多种模型的分布式训练。";
        } else if (userMessage.contains("模型") || userMessage.contains("model")) {
            return "我们目前支持多种预训练模型，包括BERT-Base、GPT-2等。您可以根据需要选择合适的模型进行训练或推理。";
        } else if (userMessage.contains("帮助") || userMessage.contains("help")) {
            return "我可以帮助您：\n1. 了解去中心化训练的原理\n2. 选择合适的模型\n3. 配置训练参数\n4. 监控训练进度\n5. 解决技术问题";
        } else {
            return "感谢您的提问！我是Williw AI助手，专注于去中心化机器学习训练。如果您有任何相关问题，我很乐意为您解答！";
        }
    }
    
    private void addWelcomeMessage() {
        ChatMessage welcomeMessage = new ChatMessage(
            "欢迎使用Williw AI助手！\n\n我是您的去中心化训练助手，可以为您提供以下服务：\n• 训练模型选择指导\n• 参数配置建议\n• 进度监控\n• 技术问题解答\n\n请输入您的问题开始对话！", 
            ChatMessage.TYPE_AI
        );
        messageList.add(welcomeMessage);
        chatAdapter.notifyItemInserted(0);
    }
}
