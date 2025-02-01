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
    let result = openai::chat_completion(&key, &provider, &messages).await;
    let response = result.unwrap();
    println!("Response: {:?}", response);
    let content = openai::chat_completion_content(&response).unwrap();
    assert_eq!(content, "hello");
}
