//! Workers 时间适配模块
//!
//! Cloudflare Workers 不支持系统时间，需要使用时间钟方法获取当前时间。
//! 本模块提供时间适配功能。

use std::sync::atomic::{AtomicI64, Ordering};
use std::time::Duration;

/// Workers 时间钟
///
/// Cloudflare Workers 环境中没有系统时间，需要通过外部时间钟获取时间。
/// 本结构体提供了时间钟的接口。
#[derive(Debug, Clone)]
pub struct WorkersClock {
    /// 内部时间戳（毫秒）
    timestamp_ms: AtomicI64,
    /// 是否使用真实时间
    use_real_time: bool,
}

impl WorkersClock {
    /// 创建新的 Workers 时间钟
    pub fn new() -> Self {
        Self {
            timestamp_ms: AtomicI64::new(0),
            use_real_time: false,
        }
    }

    /// 创建使用真实时间的时间钟（用于测试）
    pub fn with_real_time() -> Self {
        Self {
            timestamp_ms: AtomicI64::new(
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_millis() as i64
            ),
            use_real_time: true,
        }
    }

    /// 设置当前时间戳（毫秒）
    ///
    /// 在 Workers 环境中，这应该通过外部时间钟（如请求头）设置。
    pub fn set_timestamp(&self, timestamp_ms: i64) {
        self.timestamp_ms.store(timestamp_ms, Ordering::SeqCst);
    }

    /// 获取当前时间戳（毫秒）
    pub fn timestamp_ms(&self) -> i64 {
        if self.use_real_time {
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64
        } else {
            self.timestamp_ms.load(Ordering::SeqCst)
        }
    }

    /// 获取当前时间戳（秒）
    pub fn timestamp_secs(&self) -> i64 {
        self.timestamp_ms() / 1000
    }

    /// 获取当前时间戳（纳秒）
    pub fn timestamp_nanos(&self) -> i64 {
        self.timestamp_ms() * 1_000_000
    }

    /// 检查时间钟是否使用真实时间
    pub fn is_real_time(&self) -> bool {
        self.use_real_time
    }

    /// 增加时间（毫秒）
    ///
    /// 用于测试或模拟时间流逝。
    pub fn advance(&self, millis: i64) {
        self.timestamp_ms.fetch_add(millis, Ordering::SeqCst);
    }

    /// 计算两个时间戳之间的差值（毫秒）
    pub fn elapsed_since(&self, timestamp_ms: i64) -> i64 {
        self.timestamp_ms() - timestamp_ms
    }

    /// 检查是否超时
    pub fn is_timeout(&self, start_timestamp_ms: i64, timeout_ms: i64) -> bool {
        self.elapsed_since(start_timestamp_ms) >= timeout_ms
    }
}

impl Default for WorkersClock {
    fn default() -> Self {
        Self::new()
    }
}

/// 时间戳辅助函数
///
/// 提供 static 时间戳工具函数，无需实例化 WorkersClock。
pub struct TimestampUtils;

impl TimestampUtils {
    /// 从 ISO 8601 字符串解析时间戳（毫秒）
    ///
    /// 示例：`"2024-01-01T00:00:00Z"` -> `1704067200000`
    pub fn parse_iso8601(iso_str: &str) -> Result<i64, String> {
        use chrono::DateTime;

        let dt = DateTime::parse_from_rfc3339(iso_str)
            .map_err(|e| format!("ISO 8601 解析失败: {}", e))?;

        Ok(dt.timestamp_millis())
    }

    /// 将时间戳（毫秒）格式化为 ISO 8601 字符串
    ///
    /// 示例：`1704067200000` -> `"2024-01-01T00:00:00Z"`
    pub fn format_iso8601(timestamp_ms: i64) -> String {
        use chrono::{DateTime, Utc};

        let dt = DateTime::<Utc>::from_timestamp_millis(timestamp_ms)
            .unwrap_or(DateTime::UNIX_EPOCH);

        dt.to_rfc3339()
    }

    /// 从请求头中提取时间戳
    ///
    /// Workers 请求通常包含 `Date` 或 `X-Request-Start-Time` 头。
    pub fn extract_from_headers(headers: &[(String, String)]) -> Option<i64> {
        // 尝试 X-Request-Start-Time（毫秒）
        if let Some((_, value)) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("x-request-start-time")) {
            return value.parse().ok();
        }

        // 尝试 Date（秒）
        if let Some((_, value)) = headers.iter().find(|(k, _)| k.eq_ignore_ascii_case("date")) {
            if let Ok(timestamp_ms) = Self::parse_http_date(value) {
                return Some(timestamp_ms);
            }
        }

        None
    }

    /// 解析 HTTP Date 头格式
    ///
    /// 示例：`"Wed, 21 Oct 2015 07:28:00 GMT"`
    fn parse_http_date(date_str: &str) -> Result<i64, String> {
        use chrono::{DateTime, NaiveDateTime, TimeZone, Utc};

        // HTTP Date 格式：Wed, 21 Oct 2015 07:28:00 GMT
        let dt = DateTime::parse_from_rfc2822(date_str)
            .map_err(|e| format!("HTTP Date 解析失败: {}", e))?;

        Ok(dt.timestamp_millis())
    }

    /// 获取当前 Unix 时间戳（秒）
    ///
    /// 注意：在 Workers 环境中，应该使用 WorkersClock 而不是此函数。
    pub fn now_secs() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64
    }

    /// 获取当前 Unix 时间戳（毫秒）
    ///
    /// 注意：在 Workers 环境中，应该使用 WorkersClock 而不是此函数。
    pub fn now_millis() -> i64 {
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as i64
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_workers_clock_basic() {
        let clock = WorkersClock::new();

        clock.set_timestamp(1704067200000);
        assert_eq!(clock.timestamp_ms(), 1704067200000);
        assert_eq!(clock.timestamp_secs(), 1704067200);
    }

    #[test]
    fn test_workers_clock_advance() {
        let clock = WorkersClock::new();

        clock.set_timestamp(1000);
        clock.advance(500);

        assert_eq!(clock.timestamp_ms(), 1500);
    }

    #[test]
    fn test_elapsed_since() {
        let clock = WorkersClock::new();

        clock.set_timestamp(2000);
        assert_eq!(clock.elapsed_since(1000), 1000);
    }

    #[test]
    fn test_is_timeout() {
        let clock = WorkersClock::new();

        clock.set_timestamp(1500);

        assert!(clock.is_timeout(1000, 400)); // 1500 - 1000 = 500 >= 400
        assert!(!clock.is_timeout(1000, 600)); // 1500 - 1000 = 500 < 600
    }

    #[test]
    fn test_iso8601_parse() {
        let timestamp = TimestampUtils::parse_iso8601("2024-01-01T00:00:00Z").unwrap();
        assert_eq!(timestamp, 1704067200000);
    }

    #[test]
    fn test_iso8601_format() {
        let formatted = TimestampUtils::format_iso8601(1704067200000);
        assert_eq!(formatted, "2024-01-01T00:00:00+00:00");
    }
}
