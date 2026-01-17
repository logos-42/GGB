package com.williw.mobile;

import android.Manifest;
import android.content.pm.PackageManager;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.view.View;
import android.widget.EditText;
import android.widget.ImageButton;
import android.widget.TextView;
import android.widget.Toast;
import androidx.appcompat.app.AppCompatActivity;
import androidx.core.app.ActivityCompat;
import androidx.core.content.ContextCompat;
import androidx.recyclerview.widget.LinearLayoutManager;
import androidx.recyclerview.widget.RecyclerView;
import java.util.ArrayList;
import java.util.List;

public class MainActivity extends AppCompatActivity {
    private static final String TAG = "MainActivity";
    private static final int PERMISSION_REQUEST_CODE = 1001;
    
    // UI组件
    private EditText messageEditText;
    private ImageButton sendButton;
    private RecyclerView chatRecyclerView;
    private ChatAdapter chatAdapter;
    private List<ChatMessage> messageList;
    private TextView deviceInfoTextView;
    
    // Williw节点
    private WilliwNode williwNode;
    
    // 设备信息更新
    private Handler deviceUpdateHandler;
    private Runnable deviceUpdateRunnable;
    private static final long DEVICE_UPDATE_INTERVAL = 30000; // 30秒更新一次

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_main);
        
        Log.d(TAG, "MainActivity onCreate开始");
        
        // 检查权限
        checkAndRequestPermissions();
        
        // 初始化UI组件
        initViews();
        
        // 设置RecyclerView
        setupRecyclerView();
        
        // 初始化Williw节点
        initWilliwNode();
        
        // 设置点击监听器
        setupListeners();
        
        // 初始化设备信息更新
        initDeviceInfoUpdater();
        
        // 添加欢迎消息
        addWelcomeMessage();
        
        // 显示初始设备信息
        updateDeviceInfoDisplay();
        
        Log.d(TAG, "MainActivity onCreate完成");
    }
    
    private void initViews() {
        messageEditText = findViewById(R.id.messageEditText);
        sendButton = findViewById(R.id.sendButton);
        chatRecyclerView = findViewById(R.id.chatRecyclerView);
        deviceInfoTextView = findViewById(R.id.deviceInfoTextView);
    }
    
    /**
     * 检查和请求权限
     */
    private void checkAndRequestPermissions() {
        String[] permissions = {
            Manifest.permission.ACCESS_NETWORK_STATE,
            Manifest.permission.ACCESS_WIFI_STATE,
            Manifest.permission.READ_PHONE_STATE
        };
        
        List<String> permissionsToRequest = new ArrayList<>();
        for (String permission : permissions) {
            if (ContextCompat.checkSelfPermission(this, permission) != PackageManager.PERMISSION_GRANTED) {
                permissionsToRequest.add(permission);
            }
        }
        
        if (!permissionsToRequest.isEmpty()) {
            ActivityCompat.requestPermissions(this, permissionsToRequest.toArray(new String[0]), PERMISSION_REQUEST_CODE);
        }
    }
    
    /**
     * 初始化Williw节点
     */
    private void initWilliwNode() {
        try {
            Log.d(TAG, "开始初始化Williw节点");
            williwNode = new WilliwNode();
            
            if (williwNode.isInitialized()) {
                Log.d(TAG, "Williw节点初始化成功");
                
                // 刷新设备信息
                refreshDeviceInfo();
                
                // 显示设备信息
                updateDeviceInfoDisplay();
            } else {
                Log.e(TAG, "Williw节点初始化失败");
                Toast.makeText(this, "Williw节点初始化失败", Toast.LENGTH_LONG).show();
            }
        } catch (Exception e) {
            Log.e(TAG, "初始化Williw节点时发生异常: " + e.getMessage());
            Toast.makeText(this, "初始化失败: " + e.getMessage(), Toast.LENGTH_LONG).show();
        }
    }
    
    /**
     * 初始化设备信息更新器
     */
    private void initDeviceInfoUpdater() {
        deviceUpdateHandler = new Handler(Looper.getMainLooper());
        deviceUpdateRunnable = new Runnable() {
            @Override
            public void run() {
                if (williwNode != null && williwNode.isInitialized()) {
                    refreshDeviceInfo();
                    updateDeviceInfoDisplay();
                }
                // 重复执行
                deviceUpdateHandler.postDelayed(this, DEVICE_UPDATE_INTERVAL);
            }
        };
        
        // 开始定期更新
        deviceUpdateHandler.postDelayed(deviceUpdateRunnable, DEVICE_UPDATE_INTERVAL);
    }
    
    /**
     * 刷新设备信息
     */
    private void refreshDeviceInfo() {
        if (williwNode != null && williwNode.isInitialized()) {
            boolean success = williwNode.refreshDeviceInfo(this);
            if (success) {
                Log.d(TAG, "设备信息刷新成功");
            } else {
                Log.w(TAG, "设备信息刷新失败");
            }
        }
    }
    
    /**
     * 更新设备信息显示
     */
    private void updateDeviceInfoDisplay() {
        if (williwNode != null && williwNode.isInitialized()) {
            WilliwNode.DeviceCapabilities caps = williwNode.getDeviceCapabilities();
            if (caps != null && deviceInfoTextView != null) {
                String info = String.format(
                    "设备: %s %s\n" +
                    "CPU: %d核 %s\n" +
                    "内存: %dMB\n" +
                    "网络: %s\n" +
                    "GPU: %s\n" +
                    "推荐模型: %d维\n" +
                    "训练间隔: %ds",
                    caps.deviceBrand,
                    caps.deviceType,
                    caps.cpuCores,
                    caps.cpuArchitecture,
                    caps.maxMemoryMb,
                    caps.networkType,
                    caps.hasGpu ? "支持" : "不支持",
                    caps.recommendedModelDim,
                    caps.recommendedTickInterval
                );
                
                runOnUiThread(() -> deviceInfoTextView.setText(info));
            }
        }
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
        // 增强的AI回复逻辑，包含设备信息相关回答
        if (userMessage.contains("你好") || userMessage.contains("hi") || userMessage.contains("hello")) {
            return "你好！我是Williw AI助手，很高兴为您服务！我可以帮助您了解去中心化训练的相关内容，以及您的设备能力信息。";
        } else if (userMessage.contains("设备") || userMessage.contains("device")) {
            if (williwNode != null && williwNode.isInitialized()) {
                WilliwNode.DeviceCapabilities caps = williwNode.getDeviceCapabilities();
                if (caps != null) {
                    return String.format(
                        "您的设备信息：\n" +
                        "设备类型：%s %s\n" +
                        "CPU：%d核 %s\n" +
                        "内存：%dMB\n" +
                        "网络：%s\n" +
                        "GPU支持：%s\n" +
                        "推荐模型维度：%d\n" +
                        "推荐训练间隔：%d秒",
                        caps.deviceBrand, caps.deviceType,
                        caps.cpuCores, caps.cpuArchitecture,
                        caps.maxMemoryMb,
                        caps.networkType,
                        caps.hasGpu ? "是" : "否",
                        caps.recommendedModelDim,
                        caps.recommendedTickInterval
                    );
                }
            }
            return "设备信息检测中，请稍后再试。";
        } else if (userMessage.contains("训练") || userMessage.contains("学习")) {
            return "去中心化训练是一种创新的机器学习方式，它允许多个设备协同训练模型，同时保护数据隐私。我们的平台支持BERT、GPT等多种模型的分布式训练。\n\n根据您的设备能力，我会为您推荐最适合的训练参数。";
        } else if (userMessage.contains("模型") || userMessage.contains("model")) {
            if (williwNode != null && williwNode.isInitialized()) {
                int recommendedDim = williwNode.getRecommendedModelDim();
                return String.format("根据您的设备分析，推荐使用%d维模型。这个维度可以在性能和准确性之间取得最佳平衡。\n\n我们目前支持多种预训练模型，包括BERT-Base、GPT-2等。", recommendedDim);
            }
            return "我们目前支持多种预训练模型，包括BERT-Base、GPT-2等。您可以根据需要选择合适的模型进行训练或推理。";
        } else if (userMessage.contains("帮助") || userMessage.contains("help")) {
            return "我可以帮助您：\n1. 了解去中心化训练的原理\n2. 选择合适的模型\n3. 配置训练参数\n4. 监控训练进度\n5. 解决技术问题\n6. 查看设备能力信息\n\n请输入您的问题开始对话！";
        } else if (userMessage.contains("性能") || userMessage.contains("performance")) {
            if (williwNode != null && williwNode.isInitialized()) {
                boolean shouldPause = williwNode.shouldPauseTraining();
                if (shouldPause) {
                    return "根据当前设备状态（如低电量），建议暂停训练以保护设备。请充电后再继续训练。";
                } else {
                    return "您的设备状态良好，可以继续进行训练。我会持续监控设备状态并自动调整训练参数。";
                }
            }
            return "性能监控功能正在初始化中...";
        } else {
            return "感谢您的提问！我是Williw AI助手，专注于去中心化机器学习训练。如果您有任何相关问题，我很乐意为您解答！\n\n您可以问我关于设备信息、训练配置、模型选择等问题。";
        }
    }
    
    private void addWelcomeMessage() {
        ChatMessage welcomeMessage = new ChatMessage(
            "欢迎使用Williw AI！\n\n我是您的去中心化训练助手，可以为您提供以下服务：\n• 训练模型选择指导\n• 参数配置建议\n• 进度监控\n• 技术问题解答\n• 设备能力分析\n• 性能优化建议\n\n请输入您的问题开始对话！", 
            ChatMessage.TYPE_AI
        );
        messageList.add(welcomeMessage);
        chatAdapter.notifyItemInserted(0);
    }
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        
        // 停止设备信息更新
        if (deviceUpdateHandler != null && deviceUpdateRunnable != null) {
            deviceUpdateHandler.removeCallbacks(deviceUpdateRunnable);
        }
        
        // 销毁Williw节点
        if (williwNode != null) {
            williwNode.destroy();
            williwNode = null;
        }
        
        Log.d(TAG, "MainActivity已销毁");
    }
    
    @Override
    protected void onResume() {
        super.onResume();
        
        // 刷新设备信息
        if (williwNode != null && williwNode.isInitialized()) {
            refreshDeviceInfo();
            updateDeviceInfoDisplay();
        }
    }
    
    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        
        if (requestCode == PERMISSION_REQUEST_CODE) {
            boolean allGranted = true;
            for (int result : grantResults) {
                if (result != PackageManager.PERMISSION_GRANTED) {
                    allGranted = false;
                    break;
                }
            }
            
            if (allGranted) {
                Log.d(TAG, "所有权限已授予");
                // 重新初始化Williw节点
                if (williwNode == null) {
                    initWilliwNode();
                }
            } else {
                Log.w(TAG, "部分权限未授予，可能影响功能");
                Toast.makeText(this, "部分权限未授予，可能影响设备检测功能", Toast.LENGTH_LONG).show();
            }
        }
    }
}
