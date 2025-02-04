use crate::request_headers;
use crate::Key;
use crate::Message;
use crate::Provider;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;
use std::pin::Pin;

fn address(key: &Key) -> String {
    if key.provider == Provider::OpenAI {
        format!("{}/v1/chat/completions", key.provider.domain())
    } else {
        format!("{}/v1/openai/chat/completions", key.provider.domain())
    }
}

async fn request_chat_completion(
    key: &Key,
    model: &str,
    stream: bool,
    messages: &[Message],
) -> Result<Response, Box<dyn Error + Send + Sync>> {
    let address = address(key);
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
) -> Result<ChatCompletion, Box<dyn Error + Send + Sync>> {
    let stream = false;
    let resp = request_chat_completion(key, model, stream, messages).await?;
    let json = resp.json::<ChatCompletion>().await?;
    Ok(json)
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Delta {
    pub role: Option<String>,
    pub content: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChunkChoice {
    pub index: u64,
    pub delta: Delta,
    pub finish_reason: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionChunk {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<ChunkChoice>,
}

/// Convert a streaming response into an iterator of JSON messages.
/// Each message represents a complete chunk from the stream.
pub async fn stream_chat_completion(
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<
    Pin<Box<dyn Stream<Item = Result<ChatCompletionChunk, Box<dyn Error + Send + Sync>>> + Send>>,
    Box<dyn Error + Send + Sync>,
> {
    let resp = request_chat_completion(key, model, true, messages).await?;
    let stream = resp
        .bytes_stream()
        .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>)
        .flat_map(|chunk_result| {
            let chunk = chunk_result.unwrap();
            let text = String::from_utf8_lossy(&chunk);
            // Split on "data: " prefix and filter empty lines
            let results: Vec<_> = text
                .lines()
                .filter(|line| !line.is_empty())
                .filter_map(|line| {
                    if let Some(json_str) = line.strip_prefix("data: ") {
                        if json_str == "[DONE]" {
                            return None;
                        }
                        if json_str.is_empty() {
                            return None;
                        }
                        println!("json_str: {}", json_str);
                        Some(
                            serde_json::from_str::<ChatCompletionChunk>(json_str)
                                .map_err(|e| Box::new(e) as Box<dyn Error + Send + Sync>),
                        )
                    } else {
                        None
                    }
                })
                .collect();

            futures::stream::iter(results)
        });

    Ok(Box::pin(stream))
}
