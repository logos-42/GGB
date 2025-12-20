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
        
        // 设置设备信息回调
        setDeviceInfoCallback()
        
        // 初始化设备能力
        updateDeviceCapabilities()
    }
    
    /// 设置设备信息回调，让 Rust 层可以通过回调获取真实设备信息
    private func setDeviceInfoCallback() {
        guard let handle = nativeHandle else { return }
        ggb_node_set_device_callback(handle, deviceInfoCallback)
    }
    
    /// 设备信息回调函数（从 iOS 获取设备信息）
    private let deviceInfoCallback: @convention(c) (
        UnsafeMutablePointer<UInt32>?,
        UnsafeMutablePointer<UInt32>?,
        UnsafeMutablePointer<CChar>?,
        Int,
        UnsafeMutablePointer<Float>?,
        UnsafeMutablePointer<Int32>?
    ) -> Int32 = { memoryMb, cpuCores, networkType, networkTypeLen, batteryLevel, isCharging in
        // 获取内存
        if let memoryMb = memoryMb {
            let totalMemory = ProcessInfo.processInfo.physicalMemory
            memoryMb.pointee = UInt32(totalMemory / (1024 * 1024)) // 转换为 MB
        }
        
        // 获取 CPU 核心数
        if let cpuCores = cpuCores {
            cpuCores.pointee = UInt32(ProcessInfo.processInfo.processorCount)
        }
        
        // 获取网络类型
        if let networkType = networkType, networkTypeLen > 0 {
            let networkTypeStr = detectNetworkTypeSync()
            let cString = networkTypeStr.cString(using: .utf8) ?? []
            let copyLen = min(cString.count, networkTypeLen - 1)
            if copyLen > 0 {
                cString.withUnsafeBufferPointer { buffer in
                    networkType.initialize(from: buffer.baseAddress!, count: copyLen)
                }
                networkType[copyLen] = 0 // null terminator
            } else {
                networkType[0] = 0
            }
        }
        
        // 获取电池状态
        UIDevice.current.isBatteryMonitoringEnabled = true
        if let batteryLevel = batteryLevel {
            let level = UIDevice.current.batteryLevel
            batteryLevel.pointee = level >= 0 ? level : -1.0
        }
        
        if let isCharging = isCharging {
            let state = UIDevice.current.batteryState
            isCharging.pointee = (state == .charging || state == .full) ? 1 : 0
        }
        
        return 0 // 成功
    }
    
    /// 同步检测网络类型（用于回调函数）
    private static func detectNetworkTypeSync() -> String {
        // 使用信号量同步等待网络检测
        let semaphore = DispatchSemaphore(value: 0)
        var networkType: NetworkType = .unknown
        
        let monitor = NWPathMonitor()
        let queue = DispatchQueue(label: "NetworkMonitorSync")
        
        monitor.pathUpdateHandler = { path in
            if path.usesInterfaceType(.wifi) {
                networkType = .wifi
            } else if path.usesInterfaceType(.cellular) {
                // 检测真实网络类型（4G/5G）
                if #available(iOS 14.0, *) {
                    if path.status == .satisfied {
                        // iOS 14+ 可以使用更精确的检测
                        // 简化处理，实际应该使用 CoreTelephony
                        networkType = .cellular5G
                    } else {
                        networkType = .cellular4G
                    }
                } else {
                    networkType = .cellular4G
                }
            } else {
                networkType = .unknown
            }
            semaphore.signal()
            monitor.cancel()
        }
        
        monitor.start(queue: queue)
        
        // 等待检测完成（最多 1 秒）
        _ = semaphore.wait(timeout: .now() + 1.0)
        
        return networkType.rawValue
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
    
    /// 检测网络类型（使用真实 iOS API）
    private func detectNetworkType() -> NetworkType {
        return NetworkType(rawValue: Self.detectNetworkTypeSync()) ?? .unknown
    }
    
    /// 获取设备内存（MB）
    public func getDeviceMemoryMB() -> UInt32 {
        let totalMemory = ProcessInfo.processInfo.physicalMemory
        return UInt32(totalMemory / (1024 * 1024))
    }
    
    /// 获取 CPU 核心数
    public func getCpuCores() -> UInt32 {
        return UInt32(ProcessInfo.processInfo.processorCount)
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

@_silgen_name("ggb_node_set_device_callback")
private func ggb_node_set_device_callback(_ handle: OpaquePointer?, _ callback: @escaping @convention(c) (
    UnsafeMutablePointer<UInt32>?,
    UnsafeMutablePointer<UInt32>?,
    UnsafeMutablePointer<CChar>?,
    Int,
    UnsafeMutablePointer<Float>?,
    UnsafeMutablePointer<Int32>?
) -> Int32) -> Int32

@_silgen_name("ggb_node_refresh_device_info")
private func ggb_node_refresh_device_info(_ handle: OpaquePointer?) -> Int32

