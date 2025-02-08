extern crate transformrs;

use futures_util::stream::StreamExt;
use std::error::Error;
use transformrs::openai;
use transformrs::Key;
use transformrs::Message;
use transformrs::Provider;
use transformrs::Content;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct";

fn canonicalize_content(content: &Content) -> String {
    match content {
        Content::Text(text) => {
            let lower = text.to_lowercase();
            lower.trim().trim_end_matches('.')
        },
        Content::Collection(_) => panic!("Collection not supported"),
    }.to_lowercase()
        .trim()
        .trim_end_matches('.')
        .to_string()
}

fn hello_messages() -> Vec<Message> {
    vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str("user", "This is a test. Please respond with 'hello world'."),
    ]
}

async fn test_chat_completion_no_stream(
    messages: Vec<Message>,
    provider: Provider,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let messages = messages.clone();
    let resp = openai::chat_completion(&key, model, &messages).await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        }
    };
    println!("{:?}", resp);
    assert_eq!(resp.object, "chat.completion");
    assert_eq!(resp.choices.len(), 1);
    let content = resp.choices[0].message.content.clone();
    assert_eq!(canonicalize_content(&content), "hello world");
    Ok(())
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra() {
    test_chat_completion_no_stream(hello_messages(), Provider::DeepInfra, MODEL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra_image() {
    let messages = vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str("user", "This is a test. Please respond with 'hello world'."),
    ];
    test_chat_completion_no_stream(messages, Provider::DeepInfra, MODEL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra_error() {
    let out = test_chat_completion_no_stream(hello_messages(), Provider::DeepInfra, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_chat_completion_no_stream_hyperbolic() {
    test_chat_completion_no_stream(hello_messages(), Provider::Hyperbolic, MODEL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_hyperbolic_error() {
    let out = test_chat_completion_no_stream(hello_messages(), Provider::Hyperbolic, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("allowed now, your model foo"));
}

#[tokio::test]
async fn test_chat_completion_no_stream_google() {
    test_chat_completion_no_stream(hello_messages(), Provider::Google, "gemini-1.5-flash")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai() {
    test_chat_completion_no_stream(hello_messages(), Provider::OpenAI, "gpt-4o-mini")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai_error() {
    let out = test_chat_completion_no_stream(hello_messages(), Provider::OpenAI, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("does not exist"));
}

async fn chat_completion_stream_helper(
    key: &Key,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let messages =  hello_messages();
    let mut stream = openai::stream_chat_completion(&key, model, &messages)
        .await
        .unwrap();
    let mut content = String::new();
    while let Some(resp) = stream.next().await {
        assert_eq!(resp.choices.len(), 1);
        let chunk = resp.choices[0].delta.content.clone().unwrap_or_default();
        content += &chunk;
    }
    assert_eq!(canonicalize_content(&content), "hello world");
    Ok(())
}

#[tokio::test]
async fn test_chat_completion_stream_deepinfra() {
    let key = transformrs::load_keys(".env")
        .for_provider(&Provider::DeepInfra)
        .unwrap();
    chat_completion_stream_helper(&key, MODEL).await.unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_google() {
    let key = transformrs::load_keys(".env")
        .for_provider(&Provider::Google)
        .unwrap();
    chat_completion_stream_helper(&key, "gemini-1.5-flash")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_hyperbolic() {
    let key = transformrs::load_keys(".env")
        .for_provider(&Provider::Hyperbolic)
        .unwrap();
    chat_completion_stream_helper(&key, MODEL).await.unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_openai() {
    let key = transformrs::load_keys(".env")
        .for_provider(&Provider::OpenAI)
        .unwrap();
    chat_completion_stream_helper(&key, "gpt-4o-mini")
        .await
        .unwrap();
}
