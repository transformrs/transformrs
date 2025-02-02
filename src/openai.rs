use crate::Api;
use crate::Key;
use crate::Message;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::pin::Pin;

const API: Api = Api::OpenAI;

fn request_headers(key: &Key) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", key.key))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    Ok(headers)
}

async fn request_chat_completion(
    key: &Key,
    model: &str,
    stream: bool,
    messages: &[Message],
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    // Using the OpenAI-compatible API.
    let address = format!("{}chat/completions", key.provider.url(&API));
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": stream,
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(request_headers(key)?)
        .json(&body)
        .send()
        .await?;
    Ok(resp)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Choice {
    pub index: u64,
    pub message: Message,
    pub logprobs: Option<String>,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Usage {
    pub prompt_tokens: u64,
    pub completion_tokens: u64,
    pub total_tokens: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub service_tier: Option<String>,
    pub usage: Usage,
}

pub async fn chat_completion(
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<ChatCompletion, Box<dyn std::error::Error + Send + Sync>> {
    let stream = false;
    let resp = request_chat_completion(key, model, stream, messages).await?;
    let json = resp.json::<ChatCompletion>().await?;
    Ok(json)
}

/// Convert a streaming response into an iterator of JSON messages.
/// Each message represents a complete chunk from the stream.
pub async fn chat_completion_stream(
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<
    Pin<Box<dyn Stream<Item = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
    let resp = request_chat_completion(key, model, true, messages).await?;
    let stream = resp
        .bytes_stream()
        .map_err(|e| Box::new(e) as Box<dyn std::error::Error + Send + Sync>)
        .flat_map(|chunk_result| {
            let chunk = chunk_result.unwrap();
            let text = String::from_utf8_lossy(&chunk);
            // Split on "data: " prefix and filter empty lines
            let results: Vec<_> =
                text.lines()
                    .filter(|line| !line.is_empty())
                    .filter_map(|line| {
                        if let Some(stripped) = line.strip_prefix("data: ") {
                            let json_str = stripped;
                            if json_str == "[DONE]" {
                                return None;
                            }
                            Some(serde_json::from_str::<Value>(json_str).map_err(|e| {
                                Box::new(e) as Box<dyn std::error::Error + Send + Sync>
                            }))
                        } else {
                            None
                        }
                    })
                    .collect();

            futures::stream::iter(results)
        });

    Ok(Box::pin(stream))
}

pub async fn chat_completion_stream_content(
    json: Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let choice = json
        .get("choices")
        .expect("expected choices")
        .get(0)
        .unwrap();
    if choice.get("finish_reason") != Some(&Value::Null) {
        return Ok("".to_string());
    }
    let content = choice
        .get("delta")
        .expect("expected delta")
        .get("content")
        .expect("expected content");
    Ok(content.as_str().unwrap().to_string())
}
