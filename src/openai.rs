use crate::Key;
use crate::Message;
use crate::Provider;
use futures::Stream;
use futures::StreamExt;
use futures::TryStreamExt;
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Response;
use serde_json::Value;
use std::pin::Pin;

fn request_headers(key: &Key) -> Result<HeaderMap, Box<dyn std::error::Error + Send + Sync>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", key.key))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    Ok(headers)
}

pub async fn chat_completion(
    key: &Key,
    provider: &Provider,
    model: &str,
    stream: bool,
    messages: &[Message],
) -> Result<Response, Box<dyn std::error::Error + Send + Sync>> {
    let address = format!("{}chat/completions", provider.url());
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

/// Get the content of a non-streaming chat completion.
pub async fn chat_completion_content(
    json: Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let content = json
        .get("choices")
        .expect("expected choices")
        .get(0)
        .unwrap()
        .get("message")
        .unwrap()
        .get("content")
        .unwrap();
    Ok(content.as_str().unwrap().to_string())
}

pub async fn chat_completion_stream_content(
    json: Value,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let content = json
        .get("choices")
        .expect("expected choices")
        .get(0)
        .unwrap()
        .get("delta")
        .expect("expected delta")
        .get("content")
        .expect("expected content");
    Ok(content.as_str().unwrap().to_string())
}

/// Convert a streaming response into an iterator of JSON messages.
/// Each message represents a complete chunk from the stream.
pub async fn chat_completion_stream(
    resp: Response,
) -> Result<
    Pin<Box<dyn Stream<Item = Result<Value, Box<dyn std::error::Error + Send + Sync>>> + Send>>,
    Box<dyn std::error::Error + Send + Sync>,
> {
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
                            println!("Json: {:?}", json_str);
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
