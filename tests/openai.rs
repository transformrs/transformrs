extern crate aiapi;

use aiapi::openai;
use aiapi::Message;
use futures_util::stream::StreamExt;
use serde_json::Value;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct-Turbo";

#[tokio::test]
async fn test_chat_completion() {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "This is a test. Please respond with 'hello world'.".to_string(),
        },
    ];
    let key = aiapi::read_key();
    let resp = openai::chat_completion(&key, &MODEL, false, &messages).await;
    let resp = resp.unwrap();
    let json = resp.json::<Value>().await.unwrap();
    let content = openai::chat_completion_content(json).await;
    let content = content.unwrap();
    assert_eq!(content, "hello world");
}

#[tokio::test]
async fn test_chat_completion_stream() {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "This is a test. Please respond with 'hello world'.".to_string(),
        },
    ];
    let key = aiapi::read_key();
    let resp = openai::chat_completion(&key, MODEL, true, &messages).await;
    let resp = resp.unwrap();
    let mut stream = openai::chat_completion_stream(resp).await.unwrap();
    let mut content = String::new();
    while let Some(json) = stream.next().await {
        let chunk = openai::chat_completion_stream_content(json.unwrap())
            .await
            .unwrap();
        content += &chunk;
    }
    assert_eq!(content, "hello world");
}
