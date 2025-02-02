extern crate aiapi;

use aiapi::openai;
use aiapi::Message;
use aiapi::Provider;
use futures_util::stream::StreamExt;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct-Turbo";

#[tokio::test]
async fn test_chat_completion_no_stream() {
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
    let keys = aiapi::read_keys();
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let resp = openai::chat_completion(&key, &MODEL, &messages)
        .await
        .unwrap();
    println!("{:?}", resp);
    assert_eq!(resp.object, "chat.completion");
    assert_eq!(resp.choices.len(), 1);
    assert_eq!(resp.choices[0].message.content, "hello world");
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
    let keys = aiapi::read_keys();
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let mut stream = openai::chat_completion_stream(&key, &MODEL, &messages)
        .await
        .unwrap();
    let mut content = String::new();
    while let Some(resp) = stream.next().await {
        let resp = resp.unwrap();
        assert_eq!(resp.choices.len(), 1);
        let chunk = resp.choices[0].delta.content.clone().unwrap_or_default();
        content += &chunk;
    }
    assert_eq!(content, "hello world");
}
