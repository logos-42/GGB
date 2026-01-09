//! Workers 数据库适配模块
//!
//! 提供与 Cloudflare Workers 数据库服务的抽象接口，包括：
//! 1. D1 数据库（SQLite 兼容）
//! 2. KV 存储键值对
//! 3. R2 对象存储
//! 4. 统一的数据库接口

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// 数据库类型
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DatabaseType {
    /// D1 数据库（SQLite 兼容）
    D1,
    /// KV 存储（键值对）
    KV,
    /// R2 对象存储
    R2,
    /// Durable Objects（长期进程）
    DO,
}

/// 数据库配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseConfig {
    /// 数据库类型
    pub db_type: DatabaseType,
    /// 数据库绑定名称
    pub binding_name: String,
    /// 数据库 ID（可选）
    pub db_id: Option<String>,
    /// 最大缓存大小
    pub max_cache_size: usize,
    /// 缓存过期时间（秒）
    pub cache_ttl_secs: u64,
}

impl Default for DatabaseConfig {
    fn default() -> Self {
        Self {
            db_type: DatabaseType::KV,
            binding_name: "DEFAULT_KV".to_string(),
            db_id: None,
            max_cache_size: 1000,
            cache_ttl_secs: 300,
        }
    }
}

/// 数据库错误
#[derive(Debug, Clone, thiserror::Error)]
pub enum DatabaseError {
    #[error("数据库连接失败: {0}")]
    ConnectionError(String),

    #[error("查询执行失败: {0}")]
    QueryError(String),

    #[error("数据未找到: {0}")]
    NotFound(String),

    #[error("序列化失败: {0}")]
    SerializationError(String),

    #[error("反序列化失败: {0}")]
    DeserializationError(String),

    #[error("超时")]
    Timeout,
}

/// 统一数据库接口
pub trait Database: Send + Sync {
    /// 获取键值对
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 设置键值对
    async fn set(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<()>;

    /// 删除键
    async fn delete(&self, key: &str) -> Result<()>;

    /// 列出所有键
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// 批量获取
    async fn get_batch(&self, keys: &[String]) -> Result<HashMap<String, Vec<u8>>>;

    /// 批量设置
    async fn set_batch(&self, items: &[(String, Vec<u8>)], ttl_secs: Option<u64>) -> Result<()>;

    /// 批量删除
    async fn delete_batch(&self, keys: &[String]) -> Result<()>;
}

/// D1 数据库接口
pub trait D1Database: Send + Sync {
    /// 执行 SQL 查询
    async fn query(&self, sql: &str, params: &[serde_json::Value]) -> Result<Vec<serde_json::Value>>;

    /// 执行 SQL 语句（INSERT, UPDATE, DELETE）
    async fn execute(&self, sql: &str, params: &[serde_json::Value]) -> Result<u64>;

    /// 开始事务
    async fn begin_transaction(&self) -> Result<Box<dyn Transaction>>;

    /// 检查表是否存在
    async fn table_exists(&self, table_name: &str) -> Result<bool>;

    /// 创建表
    async fn create_table(&self, sql: &str) -> Result<()>;
}

/// 事务接口
pub trait Transaction: Send + Sync {
    /// 执行查询
    async fn query(&mut self, sql: &str, params: &[serde_json::Value]) -> Result<Vec<serde_json::Value>>;

    /// 执行语句
    async fn execute(&mut self, sql: &str, params: &[serde_json::Value]) -> Result<u64>;

    /// 提交事务
    async fn commit(self: Box<Self>) -> Result<()>;

    /// 回滚事务
    async fn rollback(self: Box<Self>) -> Result<()>;
}

/// KV 存储接口
pub trait KVStorage: Send + Sync {
    /// 获取值
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 设置值
    async fn put(&self, key: &str, value: &[u8], ttl_secs: Option<u64>) -> Result<()>;

    /// 删除值
    async fn delete(&self, key: &str) -> Result<()>;

    /// 列出键
    async fn list(&self, prefix: &str, limit: Option<u64>, cursor: Option<String>) -> Result<KVListResult>;
}

/// KV 列表结果
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KVListResult {
    /// 键列表
    pub keys: Vec<String>,
    /// 列表是否完整（可能还有更多键）
    pub list_complete: bool,
    /// 下一个光标（用于分页）
    pub cursor: Option<String>,
}

/// R2 对象存储接口
pub trait R2Storage: Send + Sync {
    /// 上传对象
    async fn put(&self, key: &str, data: &[u8], metadata: Option<HashMap<String, String>>) -> Result<()>;

    /// 下载对象
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// 删除对象
    async fn delete(&self, key: &str) -> Result<()>;

    /// 列出对象
    async fn list(&self, prefix: &str) -> Result<Vec<String>>;

    /// 获取对象元数据
    async fn head(&self, key: &str) -> Result<Option<ObjectMetadata>>;
}

/// 对象元数据
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectMetadata {
    /// 对象大小（字节）
    pub size: u64,
    /// 最后修改时间（毫秒时间戳）
    pub last_modified: i64,
    /// 内容类型
    pub content_type: Option<String>,
    /// ETag
    pub etag: Option<String>,
    /// 自定义元数据
    pub metadata: HashMap<String, String>,
}

/// 数据库构建器
pub struct DatabaseBuilder {
    config: DatabaseConfig,
}

impl DatabaseBuilder {
    /// 创建新的数据库构建器
    pub fn new(db_type: DatabaseType) -> Self {
        Self {
            config: DatabaseConfig {
                db_type,
                ..Default::default()
            },
        }
    }

    /// 设置绑定名称
    pub fn binding_name(mut self, name: String) -> Self {
        self.config.binding_name = name;
        self
    }

    /// 设置数据库 ID
    pub fn db_id(mut self, id: String) -> Self {
        self.config.db_id = Some(id);
        self
    }

    /// 设置缓存配置
    pub fn cache(mut self, max_size: usize, ttl_secs: u64) -> Self {
        self.config.max_cache_size = max_size;
        self.config.cache_ttl_secs = ttl_secs;
        self
    }

    /// 构建数据库配置
    pub fn build(self) -> DatabaseConfig {
        self.config
    }
}
