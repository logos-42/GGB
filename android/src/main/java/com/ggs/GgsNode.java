package com.ggb;

import android.content.Context;
import android.net.ConnectivityManager;
import android.net.NetworkInfo;
import android.os.BatteryManager;
import android.os.Build;
import android.util.Log;

/**
 * GGB 节点 Java 包装类
 * 
 * 提供 Android 平台特定的设备能力检测和节点管理功能
 */
public class GgbNode {
    private static final String TAG = "GgbNode";
    
    private long nativeHandle = 0;
    private Context context;
    
    static {
        System.loadLibrary("ggb");
    }
    
    /**
     * 创建新的节点实例
     */
    public GgbNode(Context context) {
        this.context = context;
        this.nativeHandle = nativeCreate();
        if (nativeHandle == 0) {
            throw new RuntimeException("Failed to create GGB node");
        }
        
        // 初始化设备能力
        updateDeviceCapabilities();
    }
    
    /**
     * 获取设备能力信息（JSON 格式）
     */
    public String getCapabilities() {
        if (nativeHandle == 0) {
            return "{}";
        }
        String jsonPtr = nativeGetCapabilities(nativeHandle);
        if (jsonPtr == null) {
            return "{}";
        }
        // 先创建 Java String 副本，然后再释放 native 内存
        String result = jsonPtr;
        nativeStringFree(jsonPtr);
        return result;
    }
    
    /**
     * 更新网络类型
     */
    public void updateNetworkType() {
        if (nativeHandle == 0) {
            return;
        }
        
        String networkType = detectNetworkType();
        nativeUpdateNetworkType(nativeHandle, networkType);
        Log.d(TAG, "网络类型更新: " + networkType);
    }
    
    /**
     * 更新电池状态
     */
    public void updateBattery() {
        if (nativeHandle == 0) {
            return;
        }
        
        BatteryManager batteryManager = (BatteryManager) context.getSystemService(Context.BATTERY_SERVICE);
        if (batteryManager != null) {
            int level = batteryManager.getIntProperty(BatteryManager.BATTERY_PROPERTY_CAPACITY);
            boolean isCharging = batteryManager.isCharging();
            
            float batteryLevel = level / 100.0f;
            nativeUpdateBattery(nativeHandle, batteryLevel, isCharging);
            Log.d(TAG, String.format("电池状态更新: %.0f%%, 充电: %s", batteryLevel * 100, isCharging));
        }
    }
    
    /**
     * 更新设备能力（网络和电池）
     */
    public void updateDeviceCapabilities() {
        updateNetworkType();
        updateBattery();
    }
    
    /**
     * 检测网络类型
     */
    private String detectNetworkType() {
        ConnectivityManager cm = (ConnectivityManager) context.getSystemService(Context.CONNECTIVITY_SERVICE);
        if (cm == null) {
            return "unknown";
        }
        
        NetworkInfo activeNetwork = cm.getActiveNetworkInfo();
        if (activeNetwork == null || !activeNetwork.isConnected()) {
            return "unknown";
        }
        
        if (activeNetwork.getType() == ConnectivityManager.TYPE_WIFI) {
            return "wifi";
        } else if (activeNetwork.getType() == ConnectivityManager.TYPE_MOBILE) {
            // 检测移动网络类型
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                // Android 11+ 可以使用更精确的检测
                return "5g"; // 简化处理，实际应该检测真实网络类型
            }
            return "4g";
        }
        
        return "unknown";
    }
    
    /**
     * 获取推荐的模型维度
     */
    public int getRecommendedModelDim() {
        if (nativeHandle == 0) {
            return 256;
        }
        return (int) nativeRecommendedModelDim(nativeHandle);
    }
    
    /**
     * 获取推荐的训练间隔（秒）
     */
    public long getRecommendedTickInterval() {
        if (nativeHandle == 0) {
            return 10;
        }
        return nativeRecommendedTickInterval(nativeHandle);
    }
    
    /**
     * 检查是否应该暂停训练
     */
    public boolean shouldPauseTraining() {
        if (nativeHandle == 0) {
            return false;
        }
        return nativeShouldPauseTraining(nativeHandle) != 0;
    }
    
    /**
     * 释放资源
     */
    public void destroy() {
        if (nativeHandle != 0) {
            nativeDestroy(nativeHandle);
            nativeHandle = 0;
        }
    }
    
    @Override
    protected void finalize() throws Throwable {
        destroy();
        super.finalize();
    }
    
    // Native 方法声明
    private native long nativeCreate();
    private native void nativeDestroy(long handle);
    private native String nativeGetCapabilities(long handle);
    private native int nativeUpdateNetworkType(long handle, String networkType);
    private native int nativeUpdateBattery(long handle, float level, boolean isCharging);
    private native long nativeRecommendedModelDim(long handle);
    private native long nativeRecommendedTickInterval(long handle);
    private native int nativeShouldPauseTraining(long handle);
    private native void nativeStringFree(String ptr);
}

