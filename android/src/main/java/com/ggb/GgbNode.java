package com.williw;

import android.app.ActivityManager;
import android.content.Context;
import android.net.ConnectivityManager;
import android.net.Network;
import android.net.NetworkCapabilities;
import android.net.NetworkInfo;
import android.os.BatteryManager;
import android.os.Build;
import android.telephony.TelephonyManager;
import android.util.Log;

/**
 * williw 节点 Java 包装类
 *
 * 提供 Android 平台特定的设备能力检测和节点管理功能
 */
public class WilliwNode {
    private static final String TAG = "GgbNode";
    
    private long nativeHandle = 0;
    private Context context;
    
    static {
        System.loadLibrary("ggb");
    }
    
    /**
     * 创建新的节点实例
     */
    public WilliwNode(Context context) {
        this.context = context;
        this.nativeHandle = nativeCreate();
        if (nativeHandle == 0) {
            throw new RuntimeException("Failed to create williw node");
        }
        
        // 设置设备信息回调
        setDeviceInfoCallback();
        
        // 初始化设备能力
        updateDeviceCapabilities();
    }
    
    /**
     * 设置设备信息回调，让 Rust 层可以通过回调获取真实设备信息
     */
    private void setDeviceInfoCallback() {
        if (nativeHandle == 0) {
            return;
        }
        nativeSetDeviceCallback(nativeHandle, context);
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
        // 刷新设备信息（会触发回调）
        refreshDeviceInfo();
    }
    
    /**
     * 刷新设备信息（从回调获取）
     */
    public void refreshDeviceInfo() {
        if (nativeHandle == 0) {
            return;
        }
        nativeRefreshDeviceInfo(nativeHandle);
    }
    
    /**
     * 检测网络类型（使用真实 Android API）
     * 供 JNI 回调使用
     */
    public String detectNetworkType() {
        ConnectivityManager cm = (ConnectivityManager) context.getSystemService(Context.CONNECTIVITY_SERVICE);
        if (cm == null) {
            return "unknown";
        }
        
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.M) {
            // Android 6.0+ 使用 NetworkCapabilities
            Network network = cm.getActiveNetwork();
            if (network == null) {
                return "unknown";
            }
            
            NetworkCapabilities capabilities = cm.getNetworkCapabilities(network);
            if (capabilities == null) {
                return "unknown";
            }
            
            if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_WIFI)) {
                return "wifi";
            } else if (capabilities.hasTransport(NetworkCapabilities.TRANSPORT_CELLULAR)) {
                // 检测移动网络类型（4G/5G）
                return detectCellularNetworkType(cm, network);
            }
        } else {
            // Android 6.0 以下使用 NetworkInfo（已废弃但兼容）
            NetworkInfo activeNetwork = cm.getActiveNetworkInfo();
            if (activeNetwork == null || !activeNetwork.isConnected()) {
                return "unknown";
            }
            
            if (activeNetwork.getType() == ConnectivityManager.TYPE_WIFI) {
                return "wifi";
            } else if (activeNetwork.getType() == ConnectivityManager.TYPE_MOBILE) {
                return detectCellularNetworkTypeLegacy(activeNetwork);
            }
        }
        
        return "unknown";
    }
    
    /**
     * 检测移动网络类型（4G/5G）- Android 6.0+
     */
    private String detectCellularNetworkType(ConnectivityManager cm, Network network) {
        // 使用 TelephonyManager 检测真实网络类型
        TelephonyManager telephonyManager = (TelephonyManager) context.getSystemService(Context.TELEPHONY_SERVICE);
        if (telephonyManager != null) {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.R) {
                int networkType = telephonyManager.getDataNetworkType();
                if (networkType == TelephonyManager.NETWORK_TYPE_NR) {
                    return "5g"; // 5G
                } else if (networkType == TelephonyManager.NETWORK_TYPE_LTE ||
                           networkType == TelephonyManager.NETWORK_TYPE_LTE_CA) {
                    return "4g"; // 4G
                } else if (networkType == TelephonyManager.NETWORK_TYPE_UMTS ||
                           networkType == TelephonyManager.NETWORK_TYPE_HSPA ||
                           networkType == TelephonyManager.NETWORK_TYPE_HSPAP) {
                    return "4g"; // 3G/3.5G 归类为 4G
                }
            } else {
                // Android 10 以下使用 getNetworkType
                int networkType = telephonyManager.getNetworkType();
                if (networkType == TelephonyManager.NETWORK_TYPE_LTE ||
                    networkType == TelephonyManager.NETWORK_TYPE_LTE_CA) {
                    return "4g";
                } else if (networkType == TelephonyManager.NETWORK_TYPE_UMTS ||
                           networkType == TelephonyManager.NETWORK_TYPE_HSPA ||
                           networkType == TelephonyManager.NETWORK_TYPE_HSPAP) {
                    return "4g";
                }
            }
        }
        
        return "4g"; // 默认返回 4G
    }
    
    /**
     * 检测移动网络类型（4G/5G）- Android 6.0 以下
     */
    @SuppressWarnings("deprecation")
    private String detectCellularNetworkTypeLegacy(NetworkInfo networkInfo) {
        int networkType = networkInfo.getSubtype();
        if (networkType == TelephonyManager.NETWORK_TYPE_LTE ||
            networkType == TelephonyManager.NETWORK_TYPE_LTE_CA) {
            return "4g";
        } else if (networkType == TelephonyManager.NETWORK_TYPE_UMTS ||
                   networkType == TelephonyManager.NETWORK_TYPE_HSPA ||
                   networkType == TelephonyManager.NETWORK_TYPE_HSPAP) {
            return "4g"; // 3G 归类为 4G
        }
        return "4g"; // 默认
    }
    
    /**
     * 获取设备内存（MB）
     */
    public int getDeviceMemoryMB() {
        ActivityManager activityManager = (ActivityManager) context.getSystemService(Context.ACTIVITY_SERVICE);
        if (activityManager != null) {
            ActivityManager.MemoryInfo memInfo = new ActivityManager.MemoryInfo();
            activityManager.getMemoryInfo(memInfo);
            // 返回总内存（MB）
            return (int) (memInfo.totalMem / (1024 * 1024));
        }
        // 如果无法获取，返回默认值
        return 2048;
    }
    
    /**
     * 获取 CPU 核心数
     */
    public int getCpuCores() {
        return Runtime.getRuntime().availableProcessors();
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
    private native void nativeSetDeviceCallback(long handle, Context context);
    private native int nativeRefreshDeviceInfo(long handle);
}

