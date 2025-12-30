# GGB 多机节点发现配置示例

## 场景：三台电脑在同一局域网内

假设三台电脑的IP地址分别为：
- 电脑A: 192.168.1.100
- 电脑B: 192.168.1.101
- 电脑C: 192.168.1.102

### 电脑A上启动节点：
```bash
cargo run -- --node-id 0 --quic-port 9234 \
  --bootstrap 192.168.1.101:9235 \
  --bootstrap 192.168.1.102:9236
```

### 电脑B上启动节点：
```bash
cargo run -- --node-id 1 --quic-port 9235 \
  --bootstrap 192.168.1.100:9234 \
  --bootstrap 192.168.1.102:9236
```

### 电脑C上启动节点：
```bash
cargo run -- --node-id 2 --quic-port 9236 \
  --bootstrap 192.168.1.100:9234 \
  --bootstrap 192.168.1.101:9235
```

## 发现机制说明

### 1. 手动Bootstrap连接
- 使用 `--bootstrap IP:PORT` 指定其他节点的地址
- 节点启动后会立即尝试连接指定的bootstrap节点
- QUIC协议建立低延迟的P2P连接

### 2. mDNS自动发现（局域网内）
- 即使没有手动配置，节点也会通过mDNS自动发现局域网内的其他节点
- 这提供了额外的发现途径

### 3. Kademlia DHT网络
- 一旦连接到任何一个节点，DHT网络会帮助发现更多节点
- 支持大规模网络的节点发现

### 4. 智能拓扑选择
- 节点会评估其他节点的地理位置和模型相似度
- 选择最合适的邻居节点进行训练协作

## 公网环境配置

如果电脑分布在不同网络，需要：

1. **配置公网IP和端口转发**
2. **防火墙开放相应端口**
3. **使用动态DNS服务**（如果IP动态变化）

### 示例（公网环境）：
```bash
# 节点A (公网IP: 203.0.113.1)
cargo run -- --node-id 0 --quic-port 9234 \
  --bootstrap node-b.example.com:9235 \
  --bootstrap node-c.example.com:9236

# 节点B (公网IP: 203.0.113.2)
cargo run -- --node-id 1 --quic-port 9235 \
  --bootstrap node-a.example.com:9234 \
  --bootstrap node-c.example.com:9236
```