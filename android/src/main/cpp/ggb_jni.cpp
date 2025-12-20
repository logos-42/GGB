// JNI 绑定文件，连接 Java 和 Rust FFI
#include <jni.h>
#include <string>
#include <android/log.h>

#define LOG_TAG "GgbJNI"
#define LOGI(...) __android_log_print(ANDROID_LOG_INFO, LOG_TAG, __VA_ARGS__)
#define LOGE(...) __android_log_print(ANDROID_LOG_ERROR, LOG_TAG, __VA_ARGS__)

// 全局 Context 引用（用于回调函数）
static jobject g_context = nullptr;
static JavaVM* g_jvm = nullptr;

// Rust FFI 函数声明
extern "C" {
    void* ggb_node_create();
    void ggb_node_destroy(void* handle);
    char* ggb_node_get_capabilities(void* handle);
    int ggb_node_update_network_type(void* handle, const char* network_type);
    int ggb_node_update_battery(void* handle, float level, int is_charging);
    unsigned long ggb_node_recommended_model_dim(void* handle);
    unsigned long ggb_node_recommended_tick_interval(void* handle);
    int ggb_node_should_pause_training(void* handle);
    void ggb_string_free(char* ptr);
    int ggb_node_set_device_callback(void* handle, void* callback);
    int ggb_node_refresh_device_info(void* handle);
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
    if (!g_jvm || !g_context) {
        LOGE("Context not initialized");
        return 1;
    }
    
    JNIEnv* env;
    if (g_jvm->GetEnv((void**)&env, JNI_VERSION_1_6) != JNI_OK) {
        LOGE("Failed to get JNI environment");
        return 1;
    }
    
    // 获取 Context 类
    jclass context_class = env->GetObjectClass(g_context);
    if (!context_class) {
        LOGE("Failed to get Context class");
        return 1;
    }
    
    // 调用 Java 方法获取设备信息
    // 注意：这里需要实际的实现，通过 JNI 调用 Java 方法
    // 为了简化，这里返回默认值
    *memory_mb = 2048;
    *cpu_cores = 4;
    strncpy(network_type, "unknown", network_type_len - 1);
    network_type[network_type_len - 1] = '\0';
    *battery_level = -1.0f;
    *is_charging = 0;
    
    return 0;
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_ggb_GgbNode_nativeCreate(JNIEnv* env, jobject thiz) {
    void* handle = ggb_node_create();
    return reinterpret_cast<jlong>(handle);
}

extern "C" JNIEXPORT void JNICALL
Java_com_ggb_GgbNode_nativeDestroy(JNIEnv* env, jobject thiz, jlong handle) {
    ggb_node_destroy(reinterpret_cast<void*>(handle));
}

extern "C" JNIEXPORT jstring JNICALL
Java_com_ggb_GgbNode_nativeGetCapabilities(JNIEnv* env, jobject thiz, jlong handle) {
    char* json = ggb_node_get_capabilities(reinterpret_cast<void*>(handle));
    if (!json) {
        return env->NewStringUTF("{}");
    }
    jstring result = env->NewStringUTF(json);
    ggb_string_free(json);
    return result;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_ggb_GgbNode_nativeUpdateNetworkType(JNIEnv* env, jobject thiz, jlong handle, jstring network_type) {
    const char* net_type = env->GetStringUTFChars(network_type, nullptr);
    if (!net_type) {
        return 1;
    }
    int result = ggb_node_update_network_type(reinterpret_cast<void*>(handle), net_type);
    env->ReleaseStringUTFChars(network_type, net_type);
    return result;
}

extern "C" JNIEXPORT jint JNICALL
Java_com_ggb_GgbNode_nativeUpdateBattery(JNIEnv* env, jobject thiz, jlong handle, jfloat level, jboolean is_charging) {
    return ggb_node_update_battery(reinterpret_cast<void*>(handle), level, is_charging ? 1 : 0);
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_ggb_GgbNode_nativeRecommendedModelDim(JNIEnv* env, jobject thiz, jlong handle) {
    return static_cast<jlong>(ggb_node_recommended_model_dim(reinterpret_cast<void*>(handle)));
}

extern "C" JNIEXPORT jlong JNICALL
Java_com_ggb_GgbNode_nativeRecommendedTickInterval(JNIEnv* env, jobject thiz, jlong handle) {
    return static_cast<jlong>(ggb_node_recommended_tick_interval(reinterpret_cast<void*>(handle)));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_ggb_GgbNode_nativeShouldPauseTraining(JNIEnv* env, jobject thiz, jlong handle) {
    return ggb_node_should_pause_training(reinterpret_cast<void*>(handle));
}

extern "C" JNIEXPORT void JNICALL
Java_com_ggb_GgbNode_nativeStringFree(JNIEnv* env, jobject thiz, jstring ptr) {
    // Rust 层已经处理了字符串释放
    // 这里不需要额外操作
}

extern "C" JNIEXPORT void JNICALL
Java_com_ggb_GgbNode_nativeSetDeviceCallback(JNIEnv* env, jobject thiz, jlong handle, jobject context) {
    // 保存 Context 的全局引用
    if (g_context) {
        env->DeleteGlobalRef(g_context);
    }
    g_context = env->NewGlobalRef(context);
    
    // 获取 JVM 引用
    env->GetJavaVM(&g_jvm);
    
    // 设置回调函数
    ggb_node_set_device_callback(reinterpret_cast<void*>(handle), reinterpret_cast<void*>(android_get_device_info));
}

extern "C" JNIEXPORT jint JNICALL
Java_com_ggb_GgbNode_nativeRefreshDeviceInfo(JNIEnv* env, jobject thiz, jlong handle) {
    return ggb_node_refresh_device_info(reinterpret_cast<void*>(handle));
}

