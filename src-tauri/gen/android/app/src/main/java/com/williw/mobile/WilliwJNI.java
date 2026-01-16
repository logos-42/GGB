package com.williw.mobile;

/**
 * Williw JNI桥接类
 * 连接Java层和Rust层，实现桌面版的核心功能
 */
public class WilliwJNI {
    private static final String TAG = "WilliwJNI";
    
    // 加载Rust库
    static {
        try {
            System.loadLibrary("williw");
            android.util.Log.i(TAG, "Williw Rust库在JNI中加载成功");
        } catch (UnsatisfiedLinkError e) {
            android.util.Log.e(TAG, "Williw Rust库在JNI中加载失败: " + e.getMessage());
        }
    }
    
    // ========== 训练控制方法 ==========
    
    /**
     * 启动训练
     * 对应桌面版的start_training命令
     */
    public static native boolean startTraining();
    
    /**
     * 停止训练
     * 对应桌面版的stop_training命令
     */
    public static native boolean stopTraining();
    
    /**
     * 获取训练状态
     * 对应桌面版的get_training_status命令
     */
    public static native String getTrainingStatus();
    
    /**
     * 选择模型
     * 对应桌面版的select_model命令
     */
    public static native boolean selectModel(String modelId);
    
    /**
     * 获取可用模型列表
     * 对应桌面版的get_available_models命令
     */
    public static native String getAvailableModels();
    
    // ========== 设备信息方法 ==========
    
    /**
     * 获取设备信息
     * 对应桌面版的get_device_info命令
     */
    public static native String getDeviceInfo();
    
    /**
     * 获取电池状态
     * 扩展桌面版的设备检测功能
     */
    public static native String getBatteryStatus();
    
    /**
     * 获取网络类型
     * 扩展桌面版的设备检测功能
     */
    public static native String getNetworkType();
    
    /**
     * 刷新设备信息
     * 对应桌面版的refresh_device_info功能
     */
    public static native boolean refreshDeviceInfo();
    
    // ========== 配置管理方法 ==========
    
    /**
     * 更新设置
     * 对应桌面版的update_settings命令
     */
    public static native boolean updateSettings(String settingsJson);
    
    /**
     * 获取设置
     * 对应桌面版的get_settings命令
     */
    public static native String getSettings();
    
    // ========== API密钥管理 ==========
    
    /**
     * 创建API密钥
     * 对应桌面版的create_api_key命令
     */
    public static native String createApiKey(String name);
    
    /**
     * 删除API密钥
     * 对应桌面版的delete_api_key命令
     */
    public static native boolean deleteApiKey(String keyId);
    
    /**
     * 获取所有API密钥
     * 对应桌面版的get_api_keys命令
     */
    public static native String getApiKeys();
    
    /**
     * 更新API密钥名称
     * 对应桌面版的update_api_key_name命令
     */
    public static native boolean updateApiKeyName(String keyId, String newName);
}
