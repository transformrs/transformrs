use crate::Key;
use crate::Message;
use crate::Provider;
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use reqwest::Response;
use serde_json::Value;

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
    resp: Response,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let json = resp.json::<Value>().await?;
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
