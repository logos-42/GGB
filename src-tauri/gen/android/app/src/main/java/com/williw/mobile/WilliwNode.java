package com.williw.mobile;

import android.content.Context;
import android.util.Log;
import org.json.JSONObject;
import org.json.JSONException;

/**
 * Williw节点包装类 - 简化版本
 * 直接调用JNI方法，避免复杂的库加载
 */
public class WilliwNode {
    private static final String TAG = "WilliwNode";
    
    // 原生句柄
    private long nativeHandle;
    private boolean isInitialized = false;
    
    // 静态加载库
    static {
        try {
            System.loadLibrary("williw");
            Log.d(TAG, "Successfully loaded libwilliw.so");
        } catch (UnsatisfiedLinkError e) {
            Log.e(TAG, "Failed to load libwilliw.so: " + e.getMessage());
        }
    }
    
    /**
     * 构造函数
     */
    public WilliwNode() {
        try {
            nativeHandle = nativeCreate();
            if (nativeHandle != 0) {
                isInitialized = true;
                Log.d(TAG, "Williw节点初始化成功，句柄: " + nativeHandle);
            } else {
                Log.e(TAG, "Williw节点初始化失败，句柄为0");
            }
        } catch (Exception e) {
            Log.e(TAG, "Williw节点初始化异常: " + e.getMessage());
        }
    }
    
    /**
     * 检查是否已初始化
     */
    public boolean isInitialized() {
        return isInitialized && nativeHandle != 0;
    }
    
    /**
     * 销毁节点
     */
    public void destroy() {
        if (isInitialized && nativeHandle != 0) {
            try {
                nativeDestroy(nativeHandle);
                nativeHandle = 0;
                isInitialized = false;
                Log.d(TAG, "Williw节点已销毁");
            } catch (Exception e) {
                Log.e(TAG, "销毁Williw节点异常: " + e.getMessage());
            }
        }
    }
    
    /**
     * 获取设备能力
     */
    public DeviceCapabilities getDeviceCapabilities() {
        if (!isInitialized()) {
            Log.w(TAG, "节点未初始化，返回默认设备能力");
            return DeviceCapabilities.getDefault();
        }
        
        try {
            String json = nativeGetCapabilities(nativeHandle);
            if (json != null && !json.isEmpty()) {
                return DeviceCapabilities.fromJson(json);
            }
        } catch (Exception e) {
            Log.e(TAG, "获取设备能力异常: " + e.getMessage());
        }
        
        return DeviceCapabilities.getDefault();
    }
    
    /**
     * 更新网络类型
     */
    public boolean updateNetworkType(String networkType) {
        if (!isInitialized()) {
            return false;
        }
        
        try {
            int result = nativeUpdateNetworkType(nativeHandle, networkType);
            boolean success = result == 0; // FfiError.Success
            if (success) {
                Log.d(TAG, "网络类型更新成功: " + networkType);
            } else {
                Log.w(TAG, "网络类型更新失败，错误码: " + result);
            }
            return success;
        } catch (Exception e) {
            Log.e(TAG, "更新网络类型异常: " + e.getMessage());
            return false;
        }
    }
    
    /**
     * 更新电池信息
     */
    public boolean updateBattery(float batteryLevel, boolean isCharging) {
        if (!isInitialized()) {
            return false;
        }
        
        try {
            int result = nativeUpdateBattery(nativeHandle, batteryLevel, isCharging ? 1 : 0);
            boolean success = result == 0; // FfiError.Success
            if (success) {
                Log.d(TAG, String.format("电池信息更新成功: 电量=%.2f, 充电=%s", 
                    batteryLevel, isCharging));
            } else {
                Log.w(TAG, "电池信息更新失败，错误码: " + result);
            }
            return success;
        } catch (Exception e) {
            Log.e(TAG, "更新电池信息异常: " + e.getMessage());
            return false;
        }
    }
    
    /**
     * 更新硬件信息
     */
    public boolean updateHardware(int memoryMb, int cpuCores) {
        if (!isInitialized()) {
            return false;
        }
        
        try {
            int result = nativeUpdateHardware(nativeHandle, memoryMb, cpuCores);
            boolean success = result == 0; // FfiError.Success
            if (success) {
                Log.d(TAG, String.format("硬件信息更新成功: 内存=%dMB, CPU=%d核", 
                    memoryMb, cpuCores));
            } else {
                Log.w(TAG, "硬件信息更新失败，错误码: " + result);
            }
            return success;
        } catch (Exception e) {
            Log.e(TAG, "更新硬件信息异常: " + e.getMessage());
            return false;
        }
    }
    
    /**
     * 获取推荐模型维度
     */
    public int getRecommendedModelDim() {
        if (!isInitialized()) {
            return 256; // 默认值
        }
        
        try {
            return nativeGetRecommendedModelDim(nativeHandle);
        } catch (Exception e) {
            Log.e(TAG, "获取推荐模型维度异常: " + e.getMessage());
            return 256;
        }
    }
    
    /**
     * 获取推荐tick间隔
     */
    public long getRecommendedTickInterval() {
        if (!isInitialized()) {
            return 10; // 默认10秒
        }
        
        try {
            return nativeGetRecommendedTickInterval(nativeHandle);
        } catch (Exception e) {
            Log.e(TAG, "获取推荐tick间隔异常: " + e.getMessage());
            return 10;
        }
    }
    
    /**
     * 是否应该暂停训练
     */
    public boolean shouldPauseTraining() {
        if (!isInitialized()) {
            return false;
        }
        
        try {
            return nativeShouldPauseTraining(nativeHandle) != 0;
        } catch (Exception e) {
            Log.e(TAG, "检查是否应该暂停训练异常: " + e.getMessage());
            return false;
        }
    }
    
    /**
     * 刷新设备信息
     */
    public boolean refreshDeviceInfo(Context context) {
        if (!isInitialized()) {
            return false;
        }
        
        try {
            // 获取设备信息
            DeviceInfoProvider.DeviceInfo info = DeviceInfoProvider.getDeviceInfo(context);
            
            // 更新硬件信息
            boolean hardwareSuccess = updateHardware((int) info.totalMemoryMb, info.cpuCores);
            
            // 更新网络类型
            boolean networkSuccess = updateNetworkType(info.networkType);
            
            // 更新电池信息
            boolean batterySuccess = updateBattery(info.batteryInfo.level, info.batteryInfo.isCharging);
            
            boolean overallSuccess = hardwareSuccess && networkSuccess && batterySuccess;
            
            if (overallSuccess) {
                Log.d(TAG, "设备信息刷新成功");
            } else {
                Log.w(TAG, "设备信息刷新部分失败: " +
                    "硬件=" + hardwareSuccess + 
                    ", 网络=" + networkSuccess + 
                    ", 电池=" + batterySuccess);
            }
            
            return overallSuccess;
        } catch (Exception e) {
            Log.e(TAG, "刷新设备信息异常: " + e.getMessage());
            return false;
        }
    }
    
    // === 原生方法声明 ===
    
    /**
     * 创建原生节点
     */
    private native long nativeCreate();
    
    /**
     * 销毁原生节点
     */
    private native void nativeDestroy(long ptr);
    
    /**
     * 获取设备能力JSON
     */
    private native String nativeGetCapabilities(long ptr);
    
    /**
     * 更新网络类型
     */
    private native int nativeUpdateNetworkType(long ptr, String networkType);
    
    /**
     * 更新电池信息
     */
    private native int nativeUpdateBattery(long ptr, float batteryLevel, int isCharging);
    
    /**
     * 更新硬件信息
     */
    private native int nativeUpdateHardware(long ptr, int memoryMb, int cpuCores);
    
    /**
     * 获取推荐模型维度
     */
    private native int nativeGetRecommendedModelDim(long ptr);
    
    /**
     * 获取推荐tick间隔
     */
    private native long nativeGetRecommendedTickInterval(long ptr);
    
    /**
     * 是否应该暂停训练
     */
    private native int nativeShouldPauseTraining(long ptr);
    
    /**
     * 设备能力数据类
     */
    public static class DeviceCapabilities {
        public long maxMemoryMb;
        public int cpuCores;
        public boolean hasGpu;
        public String cpuArchitecture;
        public String[] gpuComputeApis;
        public Boolean hasTpu;
        public String networkType;
        public Float batteryLevel;
        public Boolean isCharging;
        public String deviceType;
        public String deviceBrand;
        public int recommendedModelDim;
        public long recommendedTickInterval;
        
        public static DeviceCapabilities getDefault() {
            DeviceCapabilities caps = new DeviceCapabilities();
            caps.maxMemoryMb = 2048;
            caps.cpuCores = 4;
            caps.hasGpu = true;
            caps.cpuArchitecture = "unknown";
            caps.gpuComputeApis = new String[]{"opengl_es"};
            caps.hasTpu = false;
            caps.networkType = "unknown";
            caps.batteryLevel = null;
            caps.isCharging = null;
            caps.deviceType = "android";
            caps.deviceBrand = "unknown";
            caps.recommendedModelDim = 256;
            caps.recommendedTickInterval = 10;
            return caps;
        }
        
        public static DeviceCapabilities fromJson(String json) {
            try {
                JSONObject obj = new JSONObject(json);
                DeviceCapabilities caps = new DeviceCapabilities();
                caps.maxMemoryMb = obj.optLong("max_memory_mb", 2048);
                caps.cpuCores = obj.optInt("cpu_cores", 4);
                caps.hasGpu = obj.optBoolean("has_gpu", true);
                caps.cpuArchitecture = obj.optString("cpu_architecture", "unknown");
                caps.networkType = obj.optString("network_type", "unknown");
                caps.batteryLevel = obj.has("battery_level") ? 
                    (float) obj.getDouble("battery_level") : null;
                caps.isCharging = obj.has("is_charging") ? 
                    obj.getBoolean("is_charging") : null;
                caps.deviceType = obj.optString("device_type", "android");
                caps.deviceBrand = obj.optString("device_brand", "unknown");
                caps.recommendedModelDim = obj.optInt("recommended_model_dim", 256);
                caps.recommendedTickInterval = obj.optLong("recommended_tick_interval", 10);
                return caps;
            } catch (JSONException e) {
                Log.e("DeviceCapabilities", "解析JSON失败: " + e.getMessage());
                return getDefault();
            }
        }
        
        private static int calculateRecommendedModelDim(long memoryMb) {
            // 基于内存和CPU核心数计算推荐模型维度
            if (memoryMb >= 8192) return 1024;      // 8GB+
            else if (memoryMb >= 4096) return 512;  // 4GB+
            else if (memoryMb >= 2048) return 256;  // 2GB+
            else return 128;                        // <2GB
        }
        
        private static long calculateRecommendedTickInterval(String deviceType, Float batteryLevel) {
            // 基于设备类型和电池电量计算推荐训练间隔
            if (deviceType.equals("Phone")) {
                if (batteryLevel != null && batteryLevel < 0.2) return 30; // 低电量时延长间隔
                return 10; // 手机默认10秒
            } else if (deviceType.equals("Tablet")) {
                return 8; // 平板性能更好，8秒
            } else {
                return 5; // 桌面5秒
            }
        }
        
        private static boolean calculateShouldPauseTraining(Float batteryLevel, Boolean isCharging) {
            if (batteryLevel == null) return false;
            // 电量低于10%且未充电时暂停训练
            return batteryLevel < 0.1 && (isCharging == null || !isCharging);
        }
        
        @Override
        public String toString() {
            return "DeviceCapabilities{" +
                    "maxMemoryMb=" + maxMemoryMb +
                    ", cpuCores=" + cpuCores +
                    ", hasGpu=" + hasGpu +
                    ", cpuArchitecture='" + cpuArchitecture + '\'' +
                    ", hasTpu=" + hasTpu +
                    ", networkType='" + networkType + '\'' +
                    ", batteryLevel=" + batteryLevel +
                    ", isCharging=" + isCharging +
                    ", deviceType='" + deviceType + '\'' +
                    ", recommendedModelDim=" + recommendedModelDim +
                    ", recommendedTickInterval=" + recommendedTickInterval +
                    '}';
        }
    }
}
