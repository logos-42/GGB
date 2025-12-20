cmake_minimum_required(VERSION 3.22.1)

project("ggb")

# 设置 C++ 标准
set(CMAKE_CXX_STANDARD 17)
set(CMAKE_CXX_STANDARD_REQUIRED ON)

# 添加 Rust 库路径
# 这些路径需要在构建时通过环境变量或参数传入
set(RUST_LIB_DIR "${CMAKE_SOURCE_DIR}/../../../../target/${ANDROID_ABI}/release")

# 包含目录
include_directories(
    ${CMAKE_CURRENT_SOURCE_DIR}
)

# JNI 源文件
add_library(
    ggb
    SHARED
    ggb_jni.cpp
)

# 链接 Rust 库
# 注意：实际的库名可能不同，需要根据 Rust 构建输出调整
find_library(
    log-lib
    log
)

target_link_libraries(
    ggb
    ${log-lib}
    # 链接 Rust 编译的库
    # ${RUST_LIB_DIR}/libggb.a 或 libggb.so
)

