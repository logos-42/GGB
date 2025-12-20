//
//  GGB.swift
//  GGB
//
//  GGB 节点 Swift 包装类
//

import Foundation
import Network
import UIKit

/// 网络类型枚举
public enum NetworkType: String {
    case wifi = "wifi"
    case cellular5G = "5g"
    case cellular4G = "4g"
    case unknown = "unknown"
}

/// GGB 节点 Swift 包装类
public class GgbNode {
    private var nativeHandle: OpaquePointer?
    
    /// 初始化节点
    public init() {
        self.nativeHandle = ggb_node_create()
        if nativeHandle == nil {
            fatalError("Failed to create GGB node")
        }
        
        // 初始化设备能力
        updateDeviceCapabilities()
    }
    
    /// 获取设备能力信息（JSON 格式）
    public func getCapabilities() -> String? {
        guard let handle = nativeHandle else { return nil }
        
        if let jsonPtr = ggb_node_get_capabilities(handle) {
            let json = String(cString: jsonPtr)
            ggb_string_free(jsonPtr)
            return json
        }
        return nil
    }
    
    /// 更新网络类型
    public func updateNetworkType(_ type: NetworkType) {
        guard let handle = nativeHandle else { return }
        
        let result = type.rawValue.withCString { cString in
            ggb_node_update_network_type(handle, cString)
        }
        
        if result != 0 {
            print("警告: 更新网络类型失败")
        }
    }
    
    /// 自动检测并更新网络类型
    public func updateNetworkType() {
        let type = detectNetworkType()
        updateNetworkType(type)
    }
    
    /// 更新电池状态
    public func updateBattery(level: Float, isCharging: Bool) {
        guard let handle = nativeHandle else { return }
        
        let result = ggb_node_update_battery(handle, level, isCharging ? 1 : 0)
        if result != 0 {
            print("警告: 更新电池状态失败")
        }
    }
    
    /// 自动检测并更新电池状态
    public func updateBattery() {
        UIDevice.current.isBatteryMonitoringEnabled = true
        
        let batteryLevel = UIDevice.current.batteryLevel
        let isCharging = UIDevice.current.batteryState == .charging || 
                        UIDevice.current.batteryState == .full
        
        updateBattery(level: batteryLevel, isCharging: isCharging)
    }
    
    /// 更新设备能力（网络和电池）
    public func updateDeviceCapabilities() {
        updateNetworkType()
        updateBattery()
    }
    
    /// 检测网络类型
    private func detectNetworkType() -> NetworkType {
        let monitor = NWPathMonitor()
        let queue = DispatchQueue(label: "NetworkMonitor")
        var networkType: NetworkType = .unknown
        
        monitor.pathUpdateHandler = { path in
            if path.usesInterfaceType(.wifi) {
                networkType = .wifi
            } else if path.usesInterfaceType(.cellular) {
                // 简化处理，实际应该检测真实网络类型（4G/5G）
                networkType = .cellular5G
            } else {
                networkType = .unknown
            }
            monitor.cancel()
        }
        
        monitor.start(queue: queue)
        
        // 等待检测完成（简化实现）
        Thread.sleep(forTimeInterval: 0.1)
        
        return networkType
    }
    
    /// 获取推荐的模型维度
    public func getRecommendedModelDim() -> Int {
        guard let handle = nativeHandle else { return 256 }
        return Int(ggb_node_recommended_model_dim(handle))
    }
    
    /// 获取推荐的训练间隔（秒）
    public func getRecommendedTickInterval() -> UInt64 {
        guard let handle = nativeHandle else { return 10 }
        return ggb_node_recommended_tick_interval(handle)
    }
    
    /// 检查是否应该暂停训练
    public func shouldPauseTraining() -> Bool {
        guard let handle = nativeHandle else { return false }
        return ggb_node_should_pause_training(handle) != 0
    }
    
    /// 释放资源
    deinit {
        if let handle = nativeHandle {
            ggb_node_destroy(handle)
        }
    }
}

// C FFI 函数声明
@_silgen_name("ggb_node_create")
private func ggb_node_create() -> OpaquePointer?

@_silgen_name("ggb_node_destroy")
private func ggb_node_destroy(_ handle: OpaquePointer?)

@_silgen_name("ggb_node_get_capabilities")
private func ggb_node_get_capabilities(_ handle: OpaquePointer?) -> UnsafePointer<CChar>?

@_silgen_name("ggb_node_update_network_type")
private func ggb_node_update_network_type(_ handle: OpaquePointer?, _ networkType: UnsafePointer<CChar>?) -> Int32

@_silgen_name("ggb_node_update_battery")
private func ggb_node_update_battery(_ handle: OpaquePointer?, _ level: Float, _ isCharging: Int32) -> Int32

@_silgen_name("ggb_node_recommended_model_dim")
private func ggb_node_recommended_model_dim(_ handle: OpaquePointer?) -> UInt

@_silgen_name("ggb_node_recommended_tick_interval")
private func ggb_node_recommended_tick_interval(_ handle: OpaquePointer?) -> UInt64

@_silgen_name("ggb_node_should_pause_training")
private func ggb_node_should_pause_training(_ handle: OpaquePointer?) -> Int32

@_silgen_name("ggb_string_free")
private func ggb_string_free(_ ptr: UnsafePointer<CChar>?)

