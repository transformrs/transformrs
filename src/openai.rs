use crate::Key;
use crate::Message;
use crate::Provider;
use reqwest;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde_json::Value;

/// Chat completion for OpenAI-compatible providers.
///
/// To get just the text, use
pub async fn openai_chat_completion(
    key: &Key,
    provider: &Provider,
    messages: &[Message],
) -> Result<Value, Box<dyn std::error::Error + Send + Sync>> {
    let address = format!("{}chat/completions", provider.url());
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", key.key))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    let body = serde_json::json!({
        "model": "meta-llama/Llama-3.3-70B-Instruct-Turbo",
        "messages": messages
    });
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    let json = resp.json::<Value>().await?;
    Ok(json)
}

pub fn openai_chat_completion_content(
    json: &Value,
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
