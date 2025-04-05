use crate::request_headers;
use crate::Key;
use crate::Message;
use crate::Provider;
use async_stream::stream;
use bytes::Bytes;
use futures::Stream;
use futures::StreamExt;
use reqwest;
use reqwest::Response;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;
use std::pin::Pin;

fn address(provider: &Provider) -> String {
    let base_url = crate::openai_base_url(provider);
    format!("{}/chat/completions", base_url)
}

async fn request_chat_completion(
    provider: &Provider,
    key: &Key,
    model: &str,
    stream: bool,
    messages: &[Message],
) -> Result<Response, Box<dyn Error + Send + Sync>> {
    let address = address(provider);
    let body = serde_json::json!({
        "model": model,
        "messages": messages,
        "stream": stream,
    });
    let client = if provider == &Provider::Google {
        // Without this, the request will fail with 400 INVALID_ARGUMENT.
        // According to the docs, a 400 error is returned when the request body
        // is malformed.  Why rustls tls fixes this, I do not know.
        reqwest::Client::builder().use_rustls_tls().build()?
    } else {
        reqwest::Client::new()
    };
    tracing::debug!("Requesting chat: {body}");
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

/// Response from the OpenAI API.
///
/// This is a wrapper around the `serde_json::Value` which can either be
/// extracted as a structured object or left as a raw value. Allowing clients to
/// extract the unstructured response is done to allow for access to fields that
/// might be added in the future, handle edge cases, or custom processing.
///
/// You might think while reading, why not keep it simple and that's a good
/// point.  DeepSeek R1 had a great observation about this: "API design is like
/// walking the tightrope. The challenge is to build constraints that empower,
/// not confine."
pub struct ChatCompletionResponse {
    status: u16,
    resp: Bytes,
}

impl ChatCompletionResponse {
    pub fn bytes(&self) -> &Bytes {
        &self.resp
    }
    pub fn raw_value(&self) -> Result<Value, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_slice::<Value>(&self.resp)?)
    }
    pub fn structured(&self) -> Result<ChatCompletion, Box<dyn Error + Send + Sync>> {
        let json = self.raw_value()?;
        let text = json.to_string();
        if text.is_empty() {
            return Err(
                format!("Received empty response with status code: {}", self.status).into(),
            );
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
}

pub async fn chat_completion(
    provider: &Provider,
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<ChatCompletionResponse, Box<dyn Error + Send + Sync>> {
    let stream = false;
    let resp = request_chat_completion(provider, key, model, stream, messages).await?;
    let status = resp.status();
    let chat_completion_response = ChatCompletionResponse {
        status: status.into(),
        resp: resp.bytes().await?,
    };
    Ok(chat_completion_response)
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

fn process_line(line: &str) -> Option<ChatCompletionChunk> {
    if line.is_empty() {
        return None;
    }

    if let Some(json_str) = line.strip_prefix("data: ") {
        if json_str == "[DONE]" {
            return None;
        }
        serde_json::from_str::<ChatCompletionChunk>(json_str).ok()
    } else {
        None
    }
}

pub async fn stream_chat_completion(
    provider: &Provider,
    key: &Key,
    model: &str,
    messages: &[Message],
) -> Result<Pin<Box<dyn Stream<Item = ChatCompletionChunk> + Send>>, Box<dyn Error + Send + Sync>> {
    let resp = request_chat_completion(provider, key, model, true, messages).await?;

    let stream = stream! {
        let mut buffer = String::new();
        let mut byte_stream = resp.bytes_stream();

        while let Some(chunk) = byte_stream.next().await {
            let chunk = match chunk {
                Ok(c) => c,
                Err(_) => break,
            };

            let mut current_text = String::from_utf8_lossy(&chunk).to_string();

            if !buffer.is_empty() {
                current_text = format!("{buffer}{current_text}");
                buffer.clear();
            }
            let mut lines = current_text.split_inclusive('\n').peekable();

            while let Some(line) = lines.next() {
                let is_last_line = lines.peek().is_none() && !current_text.ends_with('\n');
                if is_last_line {
                    buffer.push_str(line);
                    continue;
                }
                if let Some(chunk) = process_line(line) {
                    yield chunk;
                }
            }
        }

        if !buffer.is_empty() {
            if let Some(chunk) = process_line(&buffer) {
                yield chunk;
            }
        }
    };

    Ok(Box::pin(stream))
}
