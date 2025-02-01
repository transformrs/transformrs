extern crate aiapi;

use aiapi::openai;
use aiapi::Message;
use aiapi::Provider;

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
    let content = openai::chat_completion_content(resp).await;
    let content = content.unwrap();
    assert_eq!(content, "hello");
}
