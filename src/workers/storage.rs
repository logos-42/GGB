//! Workers存储模块
//! 
//! 提供Cloudflare Workers兼容的存储功能

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 存储配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStorageConfig {
    /// 是否启用KV存储
    pub enable_kv: bool,
    /// 是否启用Durable Objects
    pub enable_durable_objects: bool,
    /// 最大缓存大小（条目）
    pub max_cache_size: usize,
    /// 缓存过期时间（秒）
    pub cache_expiration_secs: u64,
}

/// 存储管理器
pub struct WorkerStorage {
    config: WorkerStorageConfig,
    #[cfg(feature = "workers")]
    kv: Option<worker::kv::KvStore>,
    cache: HashMap<String, (Vec<u8>, i64)>,
}

impl WorkerStorage {
    /// 创建新的存储管理器
    pub fn new(config: WorkerStorageConfig) -> Result<Self> {
        #[cfg(feature = "workers")]
        let kv = if config.enable_kv {
            // 在Workers环境中初始化KV存储
            // 这里简化处理，实际需要从环境变量获取绑定
            None
        } else {
            None
        };
        
        #[cfg(not(feature = "workers"))]
        let kv = None;
        
        Ok(Self {
            config,
            kv,
            cache: HashMap::new(),
        })
    }
    
    /// 存储数据
    pub async fn put(&mut self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<()> {
        let expiration = ttl_secs.map(|ttl| {
            chrono::Utc::now().timestamp() + ttl as i64
        });
        
        // 更新缓存
        self.cache.insert(
            key.to_string(),
            (value.to_vec(), expiration.unwrap_or(i64::MAX)),
        );
        
        // 清理过期缓存
        self.cleanup_cache().await;
        
        // 如果启用KV存储，也存储到KV
        #[cfg(feature = "workers")]
        if let Some(kv_store) = &self.kv {
            let mut put_options = worker::kv::KvPutOptions::new();
            if let Some(ttl) = ttl_secs {
                put_options = put_options.with_expiration_ttl(ttl);
            }
            
            kv_store.put(key, value.to_vec())
                .put_options(put_options)
                .execute()
                .await?;
        }
        
        Ok(())
    }
    
    /// 获取数据
    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // 首先检查缓存
        if let Some((value, expiration)) = self.cache.get(key) {
            if *expiration > chrono::Utc::now().timestamp() {
                return Ok(Some(value.clone()));
            }
        }
        
        // 从KV存储获取
        #[cfg(feature = "workers")]
        if let Some(kv_store) = &self.kv {
            if let Some(value) = kv_store.get(key).bytes().await? {
                return Ok(Some(value));
            }
        }
        
        Ok(None)
    }
    
    /// 删除数据
    pub async fn delete(&mut self, key: &str) -> Result<()> {
        // 从缓存删除
        self.cache.remove(key);
        
        // 从KV存储删除
        #[cfg(feature = "workers")]
        if let Some(kv_store) = &self.kv {
            kv_store.delete(key).await?;
        }
        
        Ok(())
    }
    
    /// 列出所有键
    pub async fn list_keys(&self, prefix: Option<&str>) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        
        // 从缓存获取
        for key in self.cache.keys() {
            if let Some(prefix) = prefix {
                if key.starts_with(prefix) {
                    keys.push(key.clone());
                }
            } else {
                keys.push(key.clone());
            }
        }
        
        // 从KV存储获取
        #[cfg(feature = "workers")]
        if let Some(kv_store) = &self.kv {
            let list_options = if let Some(prefix) = prefix {
                worker::kv::KvListOptions::new().prefix(prefix)
            } else {
                worker::kv::KvListOptions::new()
            };
            
            let listed_keys = kv_store.list().options(list_options).execute().await?;
            
            for key in listed_keys.keys {
                keys.push(key.name);
            }
        }
        
        // 去重
        keys.sort();
        keys.dedup();
        
        Ok(keys)
    }
    
    /// 批量操作
    pub async fn batch_put(&mut self, items: Vec<(String, Vec<u8>)>) -> Result<()> {
        for (key, value) in items {
            self.put(&key, &value, None).await?;
        }
        Ok(())
    }
    
    /// 批量获取
    pub async fn batch_get(&self, keys: &[String]) -> Result<HashMap<String, Option<Vec<u8>>>> {
        let mut results = HashMap::new();
        
        for key in keys {
            let value = self.get(key).await?;
            results.insert(key.clone(), value);
        }
        
        Ok(results)
    }
    
    /// 清理过期缓存
    async fn cleanup_cache(&mut self) {
        let now = chrono::Utc::now().timestamp();
        let max_size = self.config.max_cache_size;
        
        // 清理过期条目
        self.cache.retain(|_, (_, expiration)| *expiration > now);
        
        // 如果超过最大大小，清理最旧的条目
        if self.cache.len() > max_size {
            let mut entries: Vec<_> = self.cache.drain().collect();
            
            // 按过期时间排序
            entries.sort_by_key(|(_, (_, expiration))| *expiration);
            
            // 保留最新的条目
            let to_keep = entries.len().min(max_size);
            for (key, (value, expiration)) in entries.into_iter().take(to_keep) {
                self.cache.insert(key, (value, expiration));
            }
        }
    }
    
    /// 获取存储统计信息
    pub async fn get_stats(&self) -> StorageStats {
        let now = chrono::Utc::now().timestamp();
        let mut expired_count = 0;
        
        for (_, expiration) in self.cache.values().map(|(_, exp)| exp) {
            if *expiration <= now {
                expired_count += 1;
            }
        }
        
        StorageStats {
            total_items: self.cache.len(),
            expired_items: expired_count,
            cache_hit_rate: 0.0, // 简化处理
            timestamp: now,
        }
    }
}

/// 存储统计信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_items: usize,
    pub expired_items: usize,
    pub cache_hit_rate: f64,
    pub timestamp: i64,
}

/// Durable Object包装器
pub struct WorkerDurableObject {
    #[cfg(feature = "workers")]
    object: worker::durable_object::DurableObject,
}

impl WorkerDurableObject {
    /// 创建新的Durable Object
    #[cfg(feature = "workers")]
    pub fn new(name: &str) -> Result<Self> {
        let namespace = worker::durable_object::DurableObjectNamespace::from_name(name)?;
        let id = namespace.id_from_name(name)?;
        let object = namespace.get(&id)?;
        
        Ok(Self { object })
    }
    
    /// 调用Durable Object方法
    #[cfg(feature = "workers")]
    pub async fn call_method(&self, method: &str, args: serde_json::Value) -> Result<serde_json::Value> {
        let response = self.object.fetch_with_str(method).await?;
        let json = response.json().await?;
        Ok(json)
    }
}

/// KV存储包装器
pub struct WorkerKVStorage {
    #[cfg(feature = "workers")]
    store: worker::kv::KvStore,
}

impl WorkerKVStorage {
    /// 创建新的KV存储
    #[cfg(feature = "workers")]
    pub fn new(binding: &str) -> Result<Self> {
        let store = worker::kv::KvStore::from_binding(binding)?;
        Ok(Self { store })
    }
    
    /// 获取KV存储引用
    #[cfg(feature = "workers")]
    pub fn get_store(&self) -> &worker::kv::KvStore {
        &self.store
    }
}
