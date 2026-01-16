//
//  williw.h
//  williw
//
//  Objective-C 桥接头文件
//

#ifndef williw_h
#define williw_h

#import <Foundation/Foundation.h>

// C FFI 函数声明
void* ggb_node_create(void);
void ggb_node_destroy(void* handle);
char* ggb_node_get_capabilities(void* handle);
int ggb_node_update_network_type(void* handle, const char* network_type);
int ggb_node_update_battery(void* handle, float level, int is_charging);
unsigned long ggb_node_recommended_model_dim(void* handle);
unsigned long ggb_node_recommended_tick_interval(void* handle);
int ggb_node_should_pause_training(void* handle);
void ggb_string_free(char* ptr);
int ggb_node_set_device_callback(void* handle, int (*callback)(
    unsigned int* memory_mb,
    unsigned int* cpu_cores,
    char* network_type,
    size_t network_type_len,
    float* battery_level,
    int* is_charging
));
int ggb_node_refresh_device_info(void* handle);

#endif /* GGB_h */

