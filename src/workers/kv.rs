//! KV 存储实现模块
//!
//! 实现 Workers KV 存储的内存版本（用于测试）和适配接口。

use super::db::{Database, DatabaseError, KVStorage, KVListResult};
use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use parking_lot::RwLock;

/// 内存 KV 存储（用于测试）
///
/// 在实际的 Workers 环境中，这应该替换为真实的 KV 存储调用。
#[derive(Clone)]
pub struct InMemoryKV {
    /// 内部存储
    storage: Arc<RwLock<HashMap<String, KVEntry>>>,
    /// 缓存大小限制
    max_cache_size: usize,
}

/// KV 条目
struct KVEntry {
    /// 数据
    data: Vec<u8>,
    /// 过期时间（可选，毫秒时间戳）
    expires_at: Option<i64>,
}

impl InMemoryKV {
    /// 创建新的内存 KV 存储
    pub fn new(max_cache_size: usize) -> Self {
        Self {
            storage: Arc::new(RwLock::new(HashMap::new())),
            max_cache_size,
        }
    }

    /// 清理过期条目
    pub fn cleanup_expired(&self) {
        let mut storage = self.storage.write();
        let now = super::time::TimestampUtils::now_millis();

        storage.retain(|_, entry| {
            if let Some(expires_at) = entry.expires_at {
                expires_at > now
            } else {
                true
            }
        });
    }

    /// 获取存储大小
    pub fn size(&self) -> usize {
        self.storage.read().len()
    }
}

impl Database for InMemoryKV {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        let storage = self.storage.read();

        if let Some(entry) = storage.get(key) {
            // 检查是否过期
            if let Some(expires_at) = entry.expires_at {
                let now = super::time::TimestampUtils::now_millis();
                if expires_at <= now {
                    return Ok(None);
                }
            }

            Ok(Some(entry.data.clone()))
        } else {
            Ok(None)
        }
    }

    async fn set(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<()> {
        let mut storage = self.storage.write();

        // 检查缓存大小
        if storage.len() >= self.max_cache_size {
            return Err(anyhow!("KV 存储已满"));
        }

        let expires_at = ttl_secs.map(|ttl| {
            super::time::TimestampUtils::now_millis() + (ttl * 1000) as i64
        });

        storage.insert(key.to_string(), KVEntry {
            data: value.to_vec(),
            expires_at,
        });

        Ok(())
    }

    async fn delete(&self, key: &str) -> Result<()> {
        let mut storage = self.storage.write();
        storage.remove(key);
        Ok(())
    }

    async fn list(&self, prefix: &str) -> Result<Vec<String>> {
        let storage = self.storage.read();
        let keys: Vec<String> = storage
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        Ok(keys)
    }

    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>> {
        let storage = self.storage.read();
        let mut result = HashMap::new();
        let now = super::time::TimestampUtils::now_millis();

        for key in keys {
            if let Some(entry) = storage.get(key) {
                if let Some(expires_at) = entry.expires_at {
                    if expires_at <= now {
                        continue;
                    }
                }
                result.insert(key.clone(), entry.data.clone());
            }
        }

        Ok(result)
    }

    async fn set_batch(&self, items: &[(String, Vec<u8>)], ttl_secs: Option<u64>) -> Result<()> {
        let mut storage = self.storage.write();

        // 检查缓存大小
        if storage.len() + items.len() > self.max_cache_size {
            return Err(anyhow!("KV 存储空间不足"));
        }

        let expires_at = ttl_secs.map(|ttl| {
            super::time::TimestampUtils::now_millis() + (ttl * 1000) as i64
        });

        for (key, value) in items {
            storage.insert(key.clone(), KVEntry {
                data: value.clone(),
                expires_at,
            });
        }

        Ok(())
    }

    async fn delete_batch(&self, keys: &[String]) -> Result<()> {
        let mut storage = self.storage.write();

        for key in keys {
            storage.remove(key);
        }

        Ok(())
    }
}

impl KVStorage for InMemoryKV {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        <Self as Database>::get(self, key).await
    }

    async fn put(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<()> {
        <Self as Database>::set(self, key, value, ttl_secs).await
    }

    async fn delete(&self, key: &str) -> Result<()> {
        <Self as Database>::delete(self, key).await
    }

    async fn list(&self, prefix: &str, limit: Option<u64>, cursor: Option<String>) -> Result<KVListResult> {
        let storage = self.storage.read();
        let mut keys: Vec<String> = storage
            .keys()
            .filter(|k| k.starts_with(prefix))
            .cloned()
            .collect();

        // 应用光标（简单实现，基于索引）
        if let Some(cursor_str) = cursor {
            if let Ok(cursor_index) = cursor_str.parse::<usize>() {
                if cursor_index < keys.len() {
                    keys = keys[cursor_index..].to_vec();
                } else {
                    keys.clear();
                }
            }
        }

        // 应用限制
        let list_complete = if let Some(limit) = limit {
            if keys.len() > limit as usize {
                keys.truncate(limit as usize);
                false
            } else {
                true
            }
        } else {
            true
        };

        // 生成下一个光标
        let next_cursor = if !list_complete {
            Some(format!("{}", keys.len()))
        } else {
            None
        };

        Ok(KVListResult {
            keys,
            list_complete,
            cursor: next_cursor,
        })
    }
}

/// Workers KV 存储适配器（占位符）
///
/// 在实际的 Workers 环境中，这应该通过 `env.KV` 绑定访问。
#[derive(Clone)]
pub struct WorkersKVAdapter {
    /// 绑定名称
    binding_name: String,
}

impl WorkersKVAdapter {
    /// 创建新的 Workers KV 适配器
    pub fn new(binding_name: String) -> Self {
        Self { binding_name }
    }

    /// 获取绑定名称
    pub fn binding_name(&self) -> &str {
        &self.binding_name
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_kv_basic() {
        let kv = InMemoryKV::new(100);

        // 设置值
        kv.set("key1", b"value1", None).await.unwrap();

        // 获取值
        let value = kv.get("key1").await.unwrap();
        assert_eq!(value, Some(b"value1".to_vec()));

        // 删除值
        kv.delete("key1").await.unwrap();

        // 验证已删除
        let value = kv.get("key1").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_kv_expiration() {
        let kv = InMemoryKV::new(100);

        // 设置带 TTL 的值（1秒）
        kv.set("expiring_key", b"value", Some(1)).await.unwrap();

        // 立即获取应该成功
        let value = kv.get("expiring_key").await.unwrap();
        assert!(value.is_some());

        // 等待过期
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;

        // 清理过期条目
        kv.cleanup_expired();

        // 再次获取应该失败
        let value = kv.get("expiring_key").await.unwrap();
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_kv_list() {
        let kv = InMemoryKV::new(100);

        // 设置多个值
        kv.set("prefix:1", b"value1", None).await.unwrap();
        kv.set("prefix:2", b"value2", None).await.unwrap();
        kv.set("other:1", b"value3", None).await.unwrap();

        // 列出带前缀的键
        let keys = kv.list("prefix:").await.unwrap();
        assert_eq!(keys.len(), 2);
        assert!(keys.contains(&"prefix:1".to_string()));
        assert!(keys.contains(&"prefix:2".to_string()));
        assert!(!keys.contains(&"other:1".to_string()));
    }

    #[tokio::test]
    async fn test_memory_kv_batch() {
        let kv = InMemoryKV::new(100);

        // 批量设置
        let items = vec![
            ("key1".to_string(), b"value1".to_vec()),
            ("key2".to_string(), b"value2".to_vec()),
            ("key3".to_string(), b"value3".to_vec()),
        ];
        kv.set_batch(&items, None).await.unwrap();

        // 批量获取
        let keys = vec!["key1".to_string(), "key2".to_string()];
        let values = kv.get_batch(&keys).await.unwrap();
        assert_eq!(values.get("key1"), Some(&b"value1".to_vec()));
        assert_eq!(values.get("key2"), Some(&b"value2".to_vec()));

        // 批量删除
        let keys = vec!["key1".to_string(), "key2".to_string()];
        kv.delete_batch(&keys).await.unwrap();

        // 验证已删除
        assert!(kv.get("key1").await.unwrap().is_none());
        assert!(kv.get("key2").await.unwrap().is_none());
        assert!(kv.get("key3").await.unwrap().is_some());
    }
}
