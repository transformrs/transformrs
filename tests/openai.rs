extern crate transformrs;

use futures_util::stream::StreamExt;
use transformrs::openai;
use transformrs::Message;
use transformrs::Provider;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct-Turbo";

#[tokio::test]
async fn test_chat_completion_no_stream() {
    let providers = vec![
        (Provider::DeepInfra, MODEL),
        (Provider::OpenAI, "gpt-4o-mini"),
    ];
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
    let keys = transformrs::read_keys();
    for (provider, model) in providers {
        let key = keys.for_provider(&provider).unwrap();
        let resp = openai::chat_completion(&key, model, &messages)
            .await
            .unwrap();
        println!("{:?}", resp);
        assert_eq!(resp.object, "chat.completion");
        assert_eq!(resp.choices.len(), 1);
        assert_eq!(resp.choices[0].message.content, "hello world");
    }
}

#[tokio::test]
async fn test_chat_completion_stream() {
    let providers = vec![
        (Provider::DeepInfra, MODEL),
        (Provider::OpenAI, "gpt-4o-mini"),
    ];
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
    let keys = transformrs::read_keys();
    for (provider, model) in providers {
        let key = keys.for_provider(&provider).unwrap();
        let mut stream = openai::chat_completion_stream(&key, model, &messages)
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
}
