extern crate aiapi;

use aiapi::openai;
use aiapi::Message;
use aiapi::Provider;
use futures_util::stream::StreamExt;
use serde_json::Value;

#[tokio::test]
async fn test_chat_completion() {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "This is a test. Please respond with 'hello'.".to_string(),
        },
    ];
    let key = aiapi::read_key();
    let provider = Provider::DeepInfra;
    let model = "meta-llama/Llama-3.3-70B-Instruct-Turbo";
    let resp = openai::chat_completion(&key, &provider, model, false, &messages).await;
    let resp = resp.unwrap();
    let json = resp.json::<Value>().await.unwrap();
    let content = openai::chat_completion_content(json).await;
    let content = content.unwrap();
    assert_eq!(content, "hello");
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
            content: "This is a test. Please respond with 'hello'.".to_string(),
        },
    ];
    let key = aiapi::read_key();
    let provider = Provider::DeepInfra;
    let model = "meta-llama/Llama-3.3-70B-Instruct-Turbo";
    let resp = openai::chat_completion(&key, &provider, model, true, &messages).await;
    let resp = resp.unwrap();
    let mut stream = openai::chat_completion_stream(resp).await.unwrap();
    while let Some(json) = stream.next().await {
        let content = openai::chat_completion_stream_content(json.unwrap())
            .await
            .unwrap();
        println!("Chunk: {:?}", content);
    }
    assert!(false);
}
