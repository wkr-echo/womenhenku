// OpenAI 兼容 HTTP 客户端
//
// 支持 SSE 流式解析，兼容 OpenAI / DeepSeek / Ollama / vLLM 等。
// 通过 Tauri Event 推送流式内容到前端。

use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::time::Duration;

/// OpenAI Chat Completion 请求（最小子集）
#[derive(Debug, Serialize)]
struct ChatRequest {
    model: String,
    messages: Vec<Message>,
    stream: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    max_tokens: Option<i32>,
    #[serde(skip_serializing_if = "Option::is_none")]
    temperature: Option<f64>,
}

#[derive(Debug, Serialize)]
struct Message {
    role: String,
    content: String,
}

/// SSE 流式响应中的 delta 块
#[derive(Debug, Deserialize)]
struct ChunkDelta {
    #[serde(default)]
    content: String,
}

#[derive(Debug, Deserialize)]
struct ChunkChoice {
    delta: ChunkDelta,
    #[serde(default)]
    finish_reason: Option<String>,
}

/// SSE 流式响应块
#[derive(Debug, Deserialize)]
struct ChatChunk {
    choices: Vec<ChunkChoice>,
    #[serde(default)]
    usage: Option<serde_json::Value>,
}

/// Token 用量
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: i64,
    pub completion_tokens: i64,
}

/// 传递给前端的 AI 流式事件
#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct AiStreamEvent {
    pub task_id: i64,
    pub entry_id: i64,
    pub content: String,
    pub is_done: bool,
    pub agent_type: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
}

/// AI 客户端错误
#[derive(Debug, thiserror::Error)]
pub enum AiClientError {
    #[error("HTTP 请求失败: {0}")]
    Http(String),
    #[error("API 返回错误: {0}")]
    Api(String),
    #[error("SSE 解析错误: {0}")]
    SseParse(String),
    #[error("超时: {0}")]
    Timeout(String),
    #[error("取消")]
    Cancelled,
}

/// OpenAI 兼容 AI 客户端
pub struct AiClient {
    http_client: Client,
}

impl AiClient {
    pub fn new() -> Self {
        let http_client = Client::builder()
            .timeout(Duration::from_secs(120))
            .build()
            .expect("Failed to build HTTP client");
        Self { http_client }
    }

    /// 发送非流式请求（用于连接验证）
    pub async fn validate(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
    ) -> Result<bool, AiClientError> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));
        let model = if model.is_empty() { "gpt-3.5-turbo" } else { model };

        let body = json!({
            "model": model,
            "messages": [{"role": "user", "content": "hi"}],
            "max_tokens": 1,
            "stream": false,
        });

        let mut req = self.http_client.post(&url).json(&body);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let resp = req
            .send()
            .await
            .map_err(|e| AiClientError::Http(e.to_string()))?;

        if resp.status().is_success() {
            Ok(true)
        } else if resp.status().as_u16() == 401 {
            Ok(false) // API Key 无效
        } else {
            let status = resp.status().as_u16();
            let text = resp.text().await.unwrap_or_default();
            Err(AiClientError::Api(format!("HTTP {}: {}", status, text)))
        }
    }

    /// 发送流式 Chat Completion 请求
    /// 通过 callback_fn 将每个 SSE chunk 的内容推送给调用者
    pub async fn stream_chat(
        &self,
        base_url: &str,
        api_key: &str,
        model: &str,
        system_prompt: &str,
        user_prompt: &str,
        mut on_content: impl FnMut(&str),
        mut on_done: impl FnMut(Option<TokenUsage>, Option<String>),
    ) -> Result<(), AiClientError> {
        let url = format!("{}/chat/completions", base_url.trim_end_matches('/'));

        let body = ChatRequest {
            model: model.to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: system_prompt.to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: user_prompt.to_string(),
                },
            ],
            stream: true,
            max_tokens: None,
            temperature: Some(0.3),
        };

        let mut req = self.http_client.post(&url).json(&body);
        if !api_key.is_empty() {
            req = req.header("Authorization", format!("Bearer {}", api_key));
        }

        let response = req
            .send()
            .await
            .map_err(|e| {
                if e.is_timeout() {
                    AiClientError::Timeout(e.to_string())
                } else {
                    AiClientError::Http(e.to_string())
                }
            })?;

        if !response.status().is_success() {
            let status = response.status().as_u16();
            let text = response.text().await.unwrap_or_default();
            return Err(AiClientError::Api(format!("HTTP {}: {}", status, text)));
        }

        // 解析 SSE 流
        let mut full_content = String::new();
        let mut final_usage: Option<TokenUsage> = None;

        let mut stream = response.bytes_stream();

        use futures_util::StreamExt;
        let mut buffer = String::new();

        while let Some(chunk_result) = stream.next().await {
            let chunk = chunk_result.map_err(|e| AiClientError::Http(e.to_string()))?;
            let text = String::from_utf8_lossy(&chunk);
            buffer.push_str(&text);

            // 按行处理 SSE 事件
            while let Some(line_end) = buffer.find('\n') {
                let line = buffer[..line_end].trim().to_string();
                buffer = buffer[line_end + 1..].to_string();

                if line.is_empty() {
                    continue; // SSE 空行（分隔符）
                }

                if line == "data: [DONE]" {
                    break;
                }

                if let Some(data) = line.strip_prefix("data: ") {
                    // 尝试解析 JSON
                    match serde_json::from_str::<ChatChunk>(data) {
                        Ok(chunk_data) => {
                            for choice in &chunk_data.choices {
                                let delta = &choice.delta.content;
                                if !delta.is_empty() {
                                    full_content.push_str(delta);
                                    on_content(delta);
                                }
                            }
                            // 检查是否有 usage 信息（部分 API 在最后一个 chunk 返回）
                            if let Some(usage_val) = &chunk_data.usage {
                                if let (Some(pt), Some(ct)) = (
                                    usage_val.get("prompt_tokens").and_then(|v| v.as_i64()),
                                    usage_val.get("completion_tokens").and_then(|v| v.as_i64()),
                                ) {
                                    final_usage = Some(TokenUsage {
                                        prompt_tokens: pt,
                                        completion_tokens: ct,
                                    });
                                }
                            }
                        }
                        Err(e) => {
                            tracing::warn!("SSE 解析警告: {} (data: {})", e, data.chars().take(100).collect::<String>());
                            // 不中断流，继续解析下一行
                        }
                    }
                }
            }
        }

        on_done(final_usage, None);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// 测试：验证 ChatRequest 序列化格式正确
    #[test]
    fn test_chat_request_serialization() {
        let req = ChatRequest {
            model: "gpt-4".to_string(),
            messages: vec![
                Message {
                    role: "system".to_string(),
                    content: "You are helpful.".to_string(),
                },
                Message {
                    role: "user".to_string(),
                    content: "Hello".to_string(),
                },
            ],
            stream: true,
            max_tokens: None,
            temperature: Some(0.3),
        };

        let json = serde_json::to_string(&req).unwrap();
        assert!(json.contains("\"stream\":true"));
        assert!(json.contains("\"model\":\"gpt-4\""));
        assert!(json.contains("\"role\":\"system\""));
        assert!(json.contains("\"role\":\"user\""));
    }

    /// 测试：验证 SSE chunk 反序列化
    #[test]
    fn test_sse_chunk_deserialization() {
        let json = r#"{"choices":[{"delta":{"content":"Hello"},"finish_reason":null}]}"#;
        let chunk: ChatChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].delta.content, "Hello");
        assert!(chunk.choices[0].finish_reason.is_none());
    }

    /// 测试：验证带 usage 的最终 chunk
    #[test]
    fn test_sse_chunk_with_usage() {
        let json = r#"{"choices":[{"delta":{"content":""},"finish_reason":"stop"}],"usage":{"prompt_tokens":50,"completion_tokens":30}}"#;
        let chunk: ChatChunk = serde_json::from_str(json).unwrap();
        assert_eq!(chunk.choices[0].finish_reason, Some("stop".to_string()));
        let usage = chunk.usage.unwrap();
        assert_eq!(usage["prompt_tokens"], 50);
        assert_eq!(usage["completion_tokens"], 30);
    }
}
