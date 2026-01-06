//! WASM构建脚本
//! 
//! 配置WASM构建环境和条件编译

fn main() {
    // 如果目标架构是wasm32，设置特定配置
    if std::env::var("TARGET").unwrap_or_default().contains("wasm32") {
        println!("cargo:rustc-cfg=wasm_target");
        println!("cargo:rustc-env=CFG_WASM=true");
        
        // 禁用一些不兼容WASM的特性
        println!("cargo:rustc-cfg=no_std_filesystem");
        println!("cargo:rustc-cfg=no_std_threading");
    }
    
    // 设置特性标志
    if std::env::var("CARGO_FEATURE_WASM").is_ok() {
        println!("cargo:rustc-cfg=feature_wasm");
    }
    
    if std::env::var("CARGO_FEATURE_WORKERS").is_ok() {
        println!("cargo:rustc-cfg=feature_workers");
    }
    
    if std::env::var("CARGO_FEATURE_ZK_PROOF").is_ok() {
        println!("cargo:rustc-cfg=feature_zk_proof");
    }
}
