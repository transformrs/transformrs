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
use serde_json::Value;
use std::error::Error;
use std::pin::Pin;

fn address(key: &Key) -> String {
    match key.provider {
        Provider::OpenAI => format!("{}/v1/chat/completions", key.provider.domain()),
        Provider::Hyperbolic => format!("{}/v1/chat/completions", key.provider.domain()),
        Provider::Google => format!("{}/v1beta/openai/chat/completions", key.provider.domain()),
        _ => format!("{}/v1/openai/chat/completions", key.provider.domain()),
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
    let client = if key.provider == Provider::Google {
        // Without this, the request will fail with 400 INVALID_ARGUMENT.
        // According to the docs, a 400 error is returned when the request body
        // is malformed.  Why rustls tls fixes this, I do not know.
        reqwest::Client::builder().use_rustls_tls().build()?
    } else {
        reqwest::Client::new()
    };
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
    pub id: Option<String>,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub system_fingerprint: Option<String>,
    pub choices: Vec<Choice>,
    pub service_tier: Option<String>,
    pub usage: Usage,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ChatCompletionError {
    pub object: Option<String>,
    pub message: String,
}

fn extract_error(body: &Value) -> String {
    if let Some(error) = body.get("error") {
        if let Some(message) = error.get("message") {
            return message
                .as_str()
                .unwrap_or(body.to_string().as_str())
                .to_string();
        }
    }
    if let Some(message) = body.get("message") {
        return message
            .as_str()
            .unwrap_or(body.to_string().as_str())
            .to_string();
    }
    format!("Unknown error: {body}")
}

pub async fn chat_completion(
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<ChatCompletion, Box<dyn Error + Send + Sync>> {
    let stream = false;
    let resp = request_chat_completion(key, model, stream, messages).await?;
    let status = resp.status();
    let text = resp.text().await?;
    if text.is_empty() {
        return Err(format!("Received empty response with status code: {}", status).into());
    }
    let json = match serde_json::from_str::<ChatCompletion>(&text) {
        Ok(json) => json,
        Err(_e) => match serde_json::from_str::<Value>(&text) {
            Ok(error) => return Err(extract_error(&error).into()),
            Err(e) => {
                return Err(format!("Error parsing response: {} in text: '{}'", e, text).into())
            }
        },
    };
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
    pub id: Option<String>,
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
                        // To debug intermittent EOF while parsing string.
                        println!("{}", json_str);
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
