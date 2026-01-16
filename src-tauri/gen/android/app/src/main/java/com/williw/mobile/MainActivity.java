package com.williw.mobile;

import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.widget.Toast;

import androidx.appcompat.app.AppCompatActivity;

import com.williw.mobile.databinding.ActivityMainBinding;

/**
 * 主活动 - Williw去中心化训练移动端
 * 实现桌面版的所有功能
 */
public class MainActivity extends AppCompatActivity {
    private static final String TAG = "WilliwMobile";
    private ActivityMainBinding binding;
    private Handler mainHandler;
    private boolean isTrainingRunning = false;
    
    // Rust库加载
    static {
        try {
            System.loadLibrary("williw");
            Log.i(TAG, "Williw Rust库加载成功");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Williw Rust库加载失败: " + e.getMessage());
        }
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        binding = ActivityMainBinding.inflate(getLayoutInflater());
        setContentView(binding.getRoot());
        
        // 初始化主线程Handler
        mainHandler = new Handler(Looper.getMainLooper());
        
        // 初始化界面
        initializeUI();
        
        // 初始化训练服务
        initializeTrainingService();
        
        Log.i(TAG, "MainActivity创建完成");
    }
    
    /**
     * 初始化用户界面
     */
    private void initializeUI() {
        // 设置标题
        if (getSupportActionBar() != null) {
            getSupportActionBar().setTitle("Williw - 去中心化训练");
        }
        
        // 显示启动信息
        showToast("Williw移动端已启动");
    }
    
    /**
     * 初始化训练服务
     */
    private void initializeTrainingService() {
        // 检查权限
        if (!hasRequiredPermissions()) {
            requestPermissions();
            return;
        }
        
        // 启动后台服务
        Intent serviceIntent = new Intent(this, TrainingService.class);
        startForegroundService(serviceIntent);
        
        Log.i(TAG, "训练服务已启动");
    }
    
    /**
     * 检查必需权限
     */
    private boolean hasRequiredPermissions() {
        String[] permissions = {
            android.Manifest.permission.INTERNET,
            android.Manifest.permission.ACCESS_NETWORK_STATE,
            android.Manifest.permission.ACCESS_WIFI_STATE,
            android.Manifest.permission.FOREGROUND_SERVICE,
            android.Manifest.permission.WAKE_LOCK
        };
        
        for (String permission : permissions) {
            if (checkSelfPermission(permission) != PackageManager.PERMISSION_GRANTED) {
                return false;
            }
        }
        return true;
    }
    
    /**
     * 请求权限
     */
    private void requestPermissions() {
        String[] permissions = {
            android.Manifest.permission.INTERNET,
            android.Manifest.permission.ACCESS_NETWORK_STATE,
            android.Manifest.permission.ACCESS_WIFI_STATE,
            android.Manifest.permission.FOREGROUND_SERVICE,
            android.Manifest.permission.WAKE_LOCK
        };
        
        requestPermissions(permissions, 1);
    }
    
    @Override
    public void onRequestPermissionsResult(int requestCode, String[] permissions, int[] grantResults) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults);
        
        if (requestCode == 1) {
            boolean allGranted = true;
            for (int result : grantResults) {
                if (result != PackageManager.PERMISSION_GRANTED) {
                    allGranted = false;
                    break;
                }
            }
            
            if (allGranted) {
                showToast("所有权限已授予");
                // 重新启动服务
                initializeTrainingService();
            } else {
                showToast("需要所有权限才能正常运行");
            }
        }
    }
    
    /**
     * 显示Toast消息
     */
    private void showToast(String message) {
        Toast.makeText(this, message, Toast.LENGTH_SHORT).show();
    }
    
    /**
     * 启动训练 - 对应桌面版的start_training
     */
    public void startTraining() {
        if (!isTrainingRunning) {
            isTrainingRunning = true;
            
            // 调用Rust库启动训练
            boolean success = nativeStartTraining();
            
            if (success) {
                showToast("训练已启动");
                Log.i(TAG, "训练启动成功");
            } else {
                showToast("训练启动失败");
                Log.e(TAG, "训练启动失败");
            }
        } else {
            showToast("训练已在运行中");
        }
    }
    
    /**
     * 停止训练 - 对应桌面版的stop_training
     */
    public void stopTraining() {
        if (isTrainingRunning) {
            isTrainingRunning = false;
            
            // 调用Rust库停止训练
            boolean success = nativeStopTraining();
            
            if (success) {
                showToast("训练已停止");
                Log.i(TAG, "训练停止成功");
            } else {
                showToast("训练停止失败");
                Log.e(TAG, "训练停止失败");
            }
        } else {
            showToast("训练未在运行");
        }
    }
    
    /**
     * 获取训练状态 - 对应桌面版的get_training_status
     */
    public String getTrainingStatus() {
        // 调用Rust库获取状态
        return nativeGetTrainingStatus();
    }
    
    /**
     * 获取设备信息 - 对应桌面版的get_device_info
     */
    public String getDeviceInfo() {
        // 调用Rust库获取设备信息
        return nativeGetDeviceInfo();
    }
    
    /**
     * 选择模型 - 对应桌面版的select_model
     */
    public boolean selectModel(String modelId) {
        // 调用Rust库选择模型
        boolean success = nativeSelectModel(modelId);
        
        if (success) {
            showToast("模型已选择: " + modelId);
            Log.i(TAG, "模型选择成功: " + modelId);
        } else {
            showToast("模型选择失败");
            Log.e(TAG, "模型选择失败: " + modelId);
        }
        
        return success;
    }
    
    // ========== Rust原生方法声明 ==========
    
    /**
     * 启动训练 - Rust实现
     */
    private native boolean nativeStartTraining();
    
    /**
     * 停止训练 - Rust实现
     */
    private native boolean nativeStopTraining();
    
    /**
     * 获取训练状态 - Rust实现
     */
    private native String nativeGetTrainingStatus();
    
    /**
     * 获取设备信息 - Rust实现
     */
    private native String nativeGetDeviceInfo();
    
    /**
     * 选择模型 - Rust实现
     */
    private native boolean nativeSelectModel(String modelId);
    
    @Override
    protected void onDestroy() {
        super.onDestroy();
        
        // 停止训练
        if (isTrainingRunning) {
            stopTraining();
        }
        
        Log.i(TAG, "MainActivity已销毁");
    }
}
