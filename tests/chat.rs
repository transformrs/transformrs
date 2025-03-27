extern crate transformrs;

mod common;

use futures_util::stream::StreamExt;
use std::error::Error;
use transformrs::chat;
use transformrs::Content;
use transformrs::Key;
use transformrs::Message;
use transformrs::Provider;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct";

fn canonicalize_content(content: &Content) -> String {
    content
        .to_string()
        .to_lowercase()
        .trim()
        .trim_end_matches('.')
        .trim_end_matches('!')
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
    expected: Option<&str>,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    common::init_tracing();
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).expect("no key found");
    let messages = messages.clone();
    let resp = chat::chat_completion(&provider, &key, model, &messages).await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        }
    };
    let json = resp.raw_value()?;
    println!("json: {json}");
    let resp = resp.structured()?;
    assert_eq!(resp.object, "chat.completion");
    assert_eq!(resp.choices.len(), 1);
    let content = resp.choices[0].message.content.clone();
    if let Some(expected) = expected {
        assert_eq!(canonicalize_content(&content), expected);
    }
    Ok(())
}

async fn test_hello_chat_completion_no_stream(
    provider: Provider,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    test_chat_completion_no_stream(hello_messages(), provider, model, Some("hello world")).await
}

async fn test_image_url_chat_completion_no_stream(
    provider: Provider,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let image_url = "https://transformrs.org/sunset.jpg";
    let messages = vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str("user", "Describe this image in one sentence."),
        Message::from_image_url("user", image_url),
    ];
    test_chat_completion_no_stream(messages, provider, model, None).await
}

async fn test_image_chat_completion_no_stream(
    provider: Provider,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    let image = include_bytes!("sunset.jpg");
    let messages = vec![
        Message::from_str("user", "Describe this image in one sentence."),
        Message::from_image_bytes("user", "jpeg", image),
    ];
    test_chat_completion_no_stream(messages, provider, model, None).await
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra() {
    test_hello_chat_completion_no_stream(Provider::DeepInfra, MODEL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra_error() {
    let out = test_hello_chat_completion_no_stream(Provider::DeepInfra, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra_image() {
    let model = "meta-llama/Llama-3.2-11B-Vision-Instruct";
    test_image_url_chat_completion_no_stream(Provider::DeepInfra, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_deepinfra_image_base64() {
    let model = "meta-llama/Llama-3.2-11B-Vision-Instruct";
    test_image_chat_completion_no_stream(Provider::DeepInfra, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_groq() {
    let model = "llama3-70b-8192";
    test_hello_chat_completion_no_stream(Provider::Groq, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_groq_image() {
    let model = "llama-3.2-11b-vision-preview";
    test_image_chat_completion_no_stream(Provider::Groq, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_groq_error() {
    let out = test_hello_chat_completion_no_stream(Provider::Groq, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("does not exist"));
}

fn hyperbolic_model() -> &'static str {
    "meta-llama/Meta-Llama-3.1-70B-Instruct"
}

#[tokio::test]
async fn test_chat_completion_no_stream_hyperbolic() {
    let model = &hyperbolic_model();
    test_hello_chat_completion_no_stream(Provider::Hyperbolic, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_hyperbolic_error() {
    let out = test_hello_chat_completion_no_stream(Provider::Hyperbolic, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("allowed now, your model foo"));
}

#[tokio::test]
async fn test_chat_completion_no_stream_google() {
    test_hello_chat_completion_no_stream(Provider::Google, "gemini-2.0-flash-lite")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai() {
    test_hello_chat_completion_no_stream(Provider::OpenAI, "gpt-4o-mini")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai_error() {
    let out = test_hello_chat_completion_no_stream(Provider::OpenAI, "foo").await;
    assert!(out.is_err());
    let err = out.unwrap_err();
    println!("{}", err);
    assert!(err.to_string().contains("does not exist"));
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai_image() {
    test_image_url_chat_completion_no_stream(Provider::OpenAI, "gpt-4o-mini")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_no_stream_openai_compatible() {
    let provider = Provider::OpenAICompatible("https://api.deepinfra.com/v1/openai".to_string());
    test_hello_chat_completion_no_stream(provider, MODEL)
        .await
        .unwrap();
}

async fn chat_completion_stream_helper(
    provider: &Provider,
    key: &Key,
    model: &str,
) -> Result<(), Box<dyn Error + Send + Sync>> {
    common::init_tracing();
    let messages = hello_messages();
    let mut stream = chat::stream_chat_completion(provider, key, model, &messages)
        .await
        .unwrap();
    let mut content = String::new();
    while let Some(resp) = stream.next().await {
        assert_eq!(resp.choices.len(), 1);
        let chunk = resp.choices[0].delta.content.clone().unwrap_or_default();
        content += &chunk;
    }
    let content = Content::Text(content);
    assert_eq!(canonicalize_content(&content), "hello world");
    Ok(())
}

#[tokio::test]
async fn test_chat_completion_stream_deepinfra() {
    let provider = Provider::DeepInfra;
    let key = transformrs::load_keys(".env")
        .for_provider(&provider)
        .unwrap();
    chat_completion_stream_helper(&provider, &key, MODEL)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_google() {
    let provider = Provider::Google;
    let key = transformrs::load_keys(".env")
        .for_provider(&provider)
        .unwrap();
    chat_completion_stream_helper(&provider, &key, "gemini-2.0-flash-lite")
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_hyperbolic() {
    let provider = Provider::Hyperbolic;
    let model = &hyperbolic_model();
    let key = transformrs::load_keys(".env")
        .for_provider(&provider)
        .unwrap();
    chat_completion_stream_helper(&provider, &key, model)
        .await
        .unwrap();
}

#[tokio::test]
async fn test_chat_completion_stream_openai() {
    let provider = Provider::OpenAI;
    let key = transformrs::load_keys(".env")
        .for_provider(&provider)
        .unwrap();
    chat_completion_stream_helper(&provider, &key, "gpt-4o-mini")
        .await
        .unwrap();
}
