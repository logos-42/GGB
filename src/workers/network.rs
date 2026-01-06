//! Workers网络适配器
//! 
//! 提供Cloudflare Workers兼容的网络功能

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// 网络配置
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerNetworkConfig {
    /// 是否启用WebSocket
    pub enable_websocket: bool,
    /// 最大连接数
    pub max_connections: usize,
    /// 连接超时时间（毫秒）
    pub connection_timeout_ms: u64,
    /// 是否启用压缩
    pub enable_compression: bool,
    /// 是否启用加密
    pub enable_encryption: bool,
}

/// 网络适配器
pub struct WorkerNetworkAdapter {
    config: WorkerNetworkConfig,
}

impl WorkerNetworkAdapter {
    /// 创建新的网络适配器
    pub fn new(config: WorkerNetworkConfig) -> Result<Self> {
        Ok(Self { config })
    }
    
    /// 发送HTTP请求
    pub async fn send_http_request(&self, request: HttpRequest) -> Result<HttpResponse> {
        // 在Workers环境中，使用fetch API
        #[cfg(feature = "workers")]
        {
            use worker::Url;
            
            let url = Url::parse(&request.url)
                .map_err(|e| anyhow::anyhow!("URL解析失败: {}", e))?;
            
            let mut init = worker::RequestInit::new();
            init.with_method(request.method.into());
            
            if let Some(body) = request.body {
                init.with_body(Some(worker::Body::from(body)));
            }
            
            if !request.headers.is_empty() {
                let mut headers = worker::Headers::new();
                for (key, value) in request.headers {
                    headers.set(&key, &value)?;
                }
                init.with_headers(headers);
            }
            
            let worker_request = worker::Request::new_with_init(&url, &init)?;
            let response = worker::Fetch::Request(worker_request).send().await?;
            
            let status = response.status_code();
            let headers = response.headers().to_json()?;
            let body = response.bytes().await?;
            
            Ok(HttpResponse {
                status,
                headers,
                body: Some(body),
            })
        }
        
        #[cfg(not(feature = "workers"))]
        {
            // 非Workers环境使用reqwest
            let client = reqwest::Client::new();
            let mut req_builder = client.request(request.method.into(), &request.url);
            
            for (key, value) in request.headers {
                req_builder = req_builder.header(key, value);
            }
            
            if let Some(body) = request.body {
                req_builder = req_builder.body(body);
            }
            
            let response = req_builder.send().await?;
            let status = response.status().as_u16();
            let headers = response
                .headers()
                .iter()
                .map(|(k, v)| (k.to_string(), v.to_str().unwrap_or("").to_string()))
                .collect();
            let body = response.bytes().await?;
            
            Ok(HttpResponse {
                status,
                headers,
                body: Some(body.to_vec()),
            })
        }
    }
    
    /// 建立WebSocket连接
    pub async fn connect_websocket(&self, url: &str) -> Result<WebSocketConnection> {
        #[cfg(feature = "workers")]
        {
            use worker::WebSocket;
            
            let ws = WebSocket::connect(url).await?;
            Ok(WebSocketConnection::Worker(ws))
        }
        
        #[cfg(not(feature = "workers"))]
        {
            use tokio_tungstenite::connect_async;
            
            let (ws_stream, _) = connect_async(url).await?;
            Ok(WebSocketConnection::Native(ws_stream))
        }
    }
    
    /// 发送WebSocket消息
    pub async fn send_websocket_message(
        &self,
        connection: &mut WebSocketConnection,
        message: WebSocketMessage,
    ) -> Result<()> {
        match connection {
            #[cfg(feature = "workers")]
            WebSocketConnection::Worker(ws) => {
                match message {
                    WebSocketMessage::Text(text) => ws.send_with_str(&text).await?,
                    WebSocketMessage::Binary(data) => ws.send_with_bytes(&data).await?,
                }
            }
            
            #[cfg(not(feature = "workers"))]
            WebSocketConnection::Native(ws) => {
                use tokio_tungstenite::tungstenite::Message;
                
                let msg = match message {
                    WebSocketMessage::Text(text) => Message::Text(text),
                    WebSocketMessage::Binary(data) => Message::Binary(data),
                };
                
                ws.send(msg).await?;
            }
        }
        
        Ok(())
    }
    
    /// 接收WebSocket消息
    pub async fn receive_websocket_message(
        &self,
        connection: &mut WebSocketConnection,
    ) -> Result<Option<WebSocketMessage>> {
        match connection {
            #[cfg(feature = "workers")]
            WebSocketConnection::Worker(ws) => {
                // Workers WebSocket API可能需要不同的处理方式
                // 这里简化处理
                Ok(None)
            }
            
            #[cfg(not(feature = "workers"))]
            WebSocketConnection::Native(ws) => {
                use tokio_tungstenite::tungstenite::Message;
                
                match ws.next().await {
                    Some(Ok(msg)) => match msg {
                        Message::Text(text) => Ok(Some(WebSocketMessage::Text(text))),
                        Message::Binary(data) => Ok(Some(WebSocketMessage::Binary(data))),
                        _ => Ok(None),
                    },
                    _ => Ok(None),
                }
            }
        }
    }
}

// 类型定义
#[derive(Debug, Clone)]
pub struct HttpRequest {
    pub url: String,
    pub method: HttpMethod,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub struct HttpResponse {
    pub status: u16,
    pub headers: Vec<(String, String)>,
    pub body: Option<Vec<u8>>,
}

#[derive(Debug, Clone)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Patch,
}

#[derive(Debug)]
pub enum WebSocketConnection {
    #[cfg(feature = "workers")]
    Worker(worker::WebSocket),
    #[cfg(not(feature = "workers"))]
    Native(tokio_tungstenite::WebSocketStream<
        tokio_tungstenite::MaybeTlsStream<tokio::net::TcpStream>,
    >),
}

#[derive(Debug, Clone)]
pub enum WebSocketMessage {
    Text(String),
    Binary(Vec<u8>),
}

// 转换实现
impl From<HttpMethod> for worker::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => worker::Method::Get,
            HttpMethod::Post => worker::Method::Post,
            HttpMethod::Put => worker::Method::Put,
            HttpMethod::Delete => worker::Method::Delete,
            HttpMethod::Patch => worker::Method::Patch,
        }
    }
}

#[cfg(not(feature = "workers"))]
impl From<HttpMethod> for reqwest::Method {
    fn from(method: HttpMethod) -> Self {
        match method {
            HttpMethod::Get => reqwest::Method::GET,
            HttpMethod::Post => reqwest::Method::POST,
            HttpMethod::Put => reqwest::Method::PUT,
            HttpMethod::Delete => reqwest::Method::DELETE,
            HttpMethod::Patch => reqwest::Method::PATCH,
        }
    }
}
