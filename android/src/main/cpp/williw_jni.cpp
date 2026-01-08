// JNI 绑定文件，连接 Java 和 Rust FFI
#include <jni.h>
#include <string>
#include <android/log.h>

#define LOG_TAG "WilliwJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// 全局引用（用于回调函数）
static jobject g_ggb_node = nullptr;  // GgbNode 实例
static jobject g_context = nullptr;   // Context 实例
static JavaVM* g_jvm = nullptr;

// Rust FFI 函数声明
extern "C" {
    void* williw_node_create();
    void williw_node_destroy(void* handle);
    char* williw_node_get_capabilities(void* handle);
    int williw_node_update_network_type(void* handle, const char* network_type);
    int williw_node_update_battery(void* handle, float level, int is_charging);
    unsigned long williw_node_recommended_model_dim(void* handle);
    unsigned long williw_node_recommended_tick_interval(void* handle);
    int williw_node_should_pause_training(void* handle);
    void williw_string_free(char* ptr);
    int williw_node_set_device_callback(void* handle, void* callback);
    int williw_node_refresh_device_info(void* handle);
}

// 设备信息回调函数（从 Android 获取设备信息）
extern "C" int android_get_device_info(
    unsigned int* memory_mb,
    unsigned int* cpu_cores,
    char* network_type,
    size_t network_type_len,
    float* battery_level,
    int* is_charging
) {
    if (!g_jvm || !g_ggb_node) {
        LOGE("GgbNode not initialized");
        return 1;
    }
    
    JNIEnv* env;
    // 获取 JNI 环境，可能需要附加线程
    jint result = g_jvm->GetEnv((void**)&env, JNI_VERSION_1_6);
    if (result == JNI_EDETACHED) {
        // 当前线程未附加到 JVM，需要附加
        if (g_jvm->AttachCurrentThread(&env, nullptr) != JNI_OK) {
            LOGE("Failed to attach thread to JVM");
            return 1;
        }
    } else if (result != JNI_OK) {
        LOGE("Failed to get JNI environment");
        return 1;
    }
    
    // 获取 GgbNode 类
    jclass ggb_node_class = env->GetObjectClass(g_ggb_node);
    if (!ggb_node_class) {
        LOGE("Failed to get GgbNode class");
        return 1;
    }
    
    // 获取方法 ID
    jmethodID get_memory_mid = env->GetMethodID(ggb_node_class, "getDeviceMemoryMB", "()I");
    jmethodID get_cpu_mid = env->GetMethodID(ggb_node_class, "getCpuCores", "()I");
    jmethodID detect_network_mid = env->GetMethodID(ggb_node_class, "detectNetworkType", "()Ljava/lang/String;");
    
    if (!get_memory_mid || !get_cpu_mid || !detect_network_mid) {
        LOGE("Failed to get method IDs");
        env->DeleteLocalRef(ggb_node_class);
        return 1;
    }
    
    // 调用 Java 方法获取内存
    if (memory_mb) {
        jint memory = env->CallIntMethod(g_ggb_node, get_memory_mid);
        if (env->ExceptionCheck()) {
            LOGE("Exception calling getDeviceMemoryMB");
            env->ExceptionClear();
            *memory_mb = 2048; // 默认值
        } else {
            *memory_mb = static_cast<unsigned int>(memory);
        }
    }
    
    // 调用 Java 方法获取 CPU 核心数
    if (cpu_cores) {
        jint cores = env->CallIntMethod(g_ggb_node, get_cpu_mid);
        if (env->ExceptionCheck()) {
            LOGE("Exception calling getCpuCores");
            env->ExceptionClear();
            *cpu_cores = 4; // 默认值
        } else {
            *cpu_cores = static_cast<unsigned int>(cores);
        }
    }
    
    // 调用 Java 方法获取网络类型
    if (network_type && network_type_len > 0) {
        jstring network_str = (jstring)env->CallObjectMethod(g_ggb_node, detect_network_mid);
        if (env->ExceptionCheck()) {
            LOGE("Exception calling detectNetworkType");
            env->ExceptionClear();
            strncpy(network_type, "unknown", network_type_len - 1);
            network_type[network_type_len - 1] = '\0';
        } else if (network_str) {
            const char* net_cstr = env->GetStringUTFChars(network_str, nullptr);
            if (net_cstr) {
                size_t copy_len = strlen(net_cstr);
                if (copy_len >= network_type_len) {
                    copy_len = network_type_len - 1;
                }
                strncpy(network_type, net_cstr, copy_len);
                network_type[copy_len] = '\0';
                env->ReleaseStringUTFChars(network_str, net_cstr);
            } else {
                strncpy(network_type, "unknown", network_type_len - 1);
                network_type[network_type_len - 1] = '\0';
            }
            env->DeleteLocalRef(network_str);
        } else {
            strncpy(network_type, "unknown", network_type_len - 1);
            network_type[network_type_len - 1] = '\0';
        }
    }
    
    // 获取电池状态（通过 Context 获取 BatteryManager）
    if (battery_level || is_charging) {
        if (g_context) {
            jclass context_class = env->GetObjectClass(g_context);
            if (context_class) {
                jmethodID get_system_service_mid = env->GetMethodID(
                    context_class, "getSystemService", "(Ljava/lang/String;)Ljava/lang/Object;");
                
                if (get_system_service_mid) {
                    jstring battery_service_str = env->NewStringUTF("battery");
                    jobject battery_manager_obj = env->CallObjectMethod(
                        g_context, get_system_service_mid, battery_service_str);
                    env->DeleteLocalRef(battery_service_str);
                    
                    if (battery_manager_obj && !env->ExceptionCheck()) {
                        jclass battery_manager_class = env->GetObjectClass(battery_manager_obj);
                        if (battery_manager_class) {
                            // 获取电池电量
                            if (battery_level) {
                                jmethodID get_int_property_mid = env->GetMethodID(
                                    battery_manager_class, "getIntProperty", "(I)I");
                                if (get_int_property_mid) {
                                    // BATTERY_PROPERTY_CAPACITY = 4
                                    jint level = env->CallIntMethod(
                                        battery_manager_obj, get_int_property_mid, 4);
                                    if (!env->ExceptionCheck() && level >= 0 && level <= 100) {
                                        *battery_level = level / 100.0f;
                                    } else {
                                        *battery_level = -1.0f;
                                    }
                                } else {
                                    *battery_level = -1.0f;
                                }
                            }
                            
                            // 获取充电状态
                            if (is_charging) {
                                jmethodID is_charging_mid = env->GetMethodID(
                                    battery_manager_class, "isCharging", "()Z");
                                if (is_charging_mid) {
                                    jboolean charging = env->CallBooleanMethod(
                                        battery_manager_obj, is_charging_mid);
                                    if (!env->ExceptionCheck()) {
                                        *is_charging = charging ? 1 : 0;
                                    } else {
                                        *is_charging = 0;
                                    }
                                } else {
                                    *is_charging = 0;
                                }
                            }
                            
                            env->DeleteLocalRef(battery_manager_class);
                        }
                        env->DeleteLocalRef(battery_manager_obj);
                    }
                    env->DeleteLocalRef(context_class);
                }
            }
        } else {
            if (battery_level) *battery_level = -1.0f;
            if (is_charging) *is_charging = 0;
        }
    }
    
    // 清理本地引用
    env->DeleteLocalRef(ggb_node_class);
    
    // 如果之前附加了线程，现在分离它
    if (result == JNI_EDETACHED) {
        g_jvm->DetachCurrentThread();
    }
    
    return 0;
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_williw_WilliwNode_nativeCreate(JNIEnv* env, jobject thiz) {
    void* handle = williw_node_create();
    return reinterpret_cast<jlong>(handle);
}

extern "C" JNIEXPORT void JNICALL
Java_com_williw_WilliwNode_nativeDestroy(JNIEnv* env, jobject thiz, jlong handle) {
    williw_node_destroy(reinterpret_cast<void*>(handle));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_williw_WilliwNode_nativeGetCapabilities(JNIEnv* env, jobject thiz, jlong handle) {
    char* json = williw_node_get_capabilities(reinterpret_cast<void*>(handle));
    if (!json) {
        return env->NewStringUTF("{}");
    }
    jstring result = env->NewStringUTF(json);
    williw_string_free(json);
    return result;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_williw_WilliwNode_nativeUpdateNetworkType(JNIEnv* env, jobject thiz, jlong handle, jstring network_type) {
    const char* net_type = env->GetStringUTFChars(network_type, nullptr);
    if (!net_type) {
        return 1;
    }
    int result = williw_node_update_network_type(reinterpret_cast<void*>(handle), net_type);
    env->ReleaseStringUTFChars(network_type, net_type);
    return result;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_williw_WilliwNode_nativeUpdateBattery(JNIEnv* env, jobject thiz, jlong handle, jfloat level, jboolean is_charging) {
    return williw_node_update_battery(reinterpret_cast<void*>(handle), level, is_charging ? 1 : 0);
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_williw_WilliwNode_nativeRecommendedModelDim(JNIEnv* env, jobject thiz, jlong handle) {
    return static_cast<jlong>(williw_node_recommended_model_dim(reinterpret_cast<void*>(handle)));
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_williw_WilliwNode_nativeRecommendedTickInterval(JNIEnv* env, jobject thiz, jlong handle) {
    return static_cast<jlong>(williw_node_recommended_tick_interval(reinterpret_cast<void*>(handle)));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_williw_WilliwNode_nativeShouldPauseTraining(JNIEnv* env, jobject thiz, jlong handle) {
    return williw_node_should_pause_training(reinterpret_cast<void*>(handle));
}

extern "C" JNIEXPORT void JNICALL
Java_com_ggb_GgbNode_nativeStringFree(JNIEnv* env, jobject thiz, jstring ptr) {
    // Rust 层已经处理了字符串释放
    // 这里不需要额外操作
}

extern "C" JNIEXPORT void JNICALL
Java_com_williw_WilliwNode_nativeSetDeviceCallback(JNIEnv* env, jobject thiz, jlong handle, jobject context) {
    // 保存 WilliwNode 实例的全局引用（用于回调中调用方法）
    if (g_ggb_node) {
        env->DeleteGlobalRef(g_ggb_node);
    }
    g_ggb_node = env->NewGlobalRef(thiz);
    
    // 保存 Context 的全局引用（用于获取系统服务）
    if (g_context) {
        env->DeleteGlobalRef(g_context);
    }
    g_context = env->NewGlobalRef(context);
    
    // 获取 JVM 引用
    env->GetJavaVM(&g_jvm);
    
    // 设置回调函数
    williw_node_set_device_callback(reinterpret_cast<void*>(handle), reinterpret_cast<void*>(android_get_device_info));
}

// JNI_OnLoad 和 JNI_OnUnload 用于管理全局引用
extern "C" JNIEXPORT jint JNICALL
JNI_OnLoad(JavaVM* vm, void* reserved) {
    g_jvm = vm;
    return JNI_VERSION_1_6;
}

extern "C" JNIEXPORT void JNICALL
JNI_OnUnload(JavaVM* vm, void* reserved) {
    // 清理全局引用
    if (g_ggb_node || g_context) {
        JNIEnv* env;
        if (vm->GetEnv((void**)&env, JNI_VERSION_1_6) == JNI_OK) {
            if (g_ggb_node) {
                env->DeleteGlobalRef(g_ggb_node);
                g_ggb_node = nullptr;
            }
            if (g_context) {
                env->DeleteGlobalRef(g_context);
                g_context = nullptr;
            }
        }
    }
    g_jvm = nullptr;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_williw_WilliwNode_nativeRefreshDeviceInfo(JNIEnv* env, jobject thiz, jlong handle) {
    return williw_node_refresh_device_info(reinterpret_cast<void*>(handle));
}

