extern crate transformrs;

mod common;

use serde_json::json;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Write;
use transformrs::text_to_speech::Speech;
use transformrs::text_to_speech::TTSConfig;
use transformrs::Provider;

/// Ensure that calling clone compiles.
/// 
/// This is to double-check that `Clone` is not removed for `TTSConfig`.
#[allow(dead_code)]
fn test_tts_clone() {
    let config = TTSConfig::default();
    let _ = config.clone();
}

async fn tts_helper(
    provider: &Provider,
    config: &TTSConfig,
    model: Option<&str>,
) -> Result<Speech, Box<dyn Error + Send + Sync>> {
    common::init_tracing();
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let msg = "Hello, world!";
    let resp = transformrs::text_to_speech::tts(&key, config, model, msg)
        .await
        .unwrap();
    resp.structured()
}

#[tokio::test]
async fn test_tts_deepinfra() {
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.voice = Some("am_echo".to_string());
    config.output_format = Some("mp3".to_string());
    let model = Some("hexgrad/Kokoro-82M");
    let provider = Provider::DeepInfra;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    assert_eq!(speech.file_format, "mp3");
    let bytes = speech.audio.clone();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp-deepinfra.mp3").unwrap();
    file.write_all(&bytes).unwrap();
}

#[tokio::test]
async fn test_tts_deepinfra_error() {
    let config = transformrs::text_to_speech::TTSConfig::default();
    let model = Some("foobar");
    let provider = Provider::DeepInfra;
    let speech = tts_helper(&provider, &config, model).await;
    let err = speech.unwrap_err();
    assert!(err.to_string().contains("Model is not available"));
}

#[tokio::test]
async fn test_tts_deepinfra_opus() {
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.voice = Some("am_echo".to_string());
    config.output_format = Some("opus".to_string());
    let model = Some("hexgrad/Kokoro-82M");
    let provider = Provider::DeepInfra;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    assert_eq!(speech.file_format, "opus");
    let bytes = speech.audio.clone();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp-deepinfra.opus").unwrap();
    file.write_all(&bytes).unwrap();
}

#[tokio::test]
async fn test_tts_deepinfra_other() {
    let mut other = HashMap::new();
    other.insert("seed".to_string(), json!(42));
    let config = transformrs::text_to_speech::TTSConfig {
        output_format: Some("mp3".to_string()),
        other: Some(other),
        ..Default::default()
    };
    let model = Some("foobar");
    let provider = Provider::DeepInfra;
    let speech = tts_helper(&provider, &config, model).await;
    let err = speech.unwrap_err();
    assert!(err.to_string().contains("Model is not available"));
}

#[tokio::test]
async fn test_tts_hyperbolic() {
    let config = transformrs::text_to_speech::TTSConfig::default();
    let model = None;
    let provider = Provider::Hyperbolic;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    let mut file = File::create("tests/tmp").unwrap();
    file.write_all(&speech.audio.clone()).unwrap();
    let bytes = speech.audio.clone();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp-hyperbolic.mp3").unwrap();
    file.write_all(&bytes).unwrap();
}

#[tokio::test]
async fn test_tts_openai() {
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.voice = Some("alloy".to_string());
    let model = Some("tts-1");
    let provider = Provider::OpenAI;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    let mut file = File::create("tests/tmp-openai.mp3").unwrap();
    file.write_all(&speech.audio.clone()).unwrap();
}

#[tokio::test]
async fn test_tts_openai_error() {
    let config = transformrs::text_to_speech::TTSConfig::default();
    let model = Some("foobar");
    let provider = Provider::OpenAI;
    let speech = tts_helper(&provider, &config, model).await;
    let err = speech.unwrap_err();
    println!("err: {}", err);
    assert!(err.to_string().contains("model_not_found"));
}

#[tokio::test]
async fn test_tts_google() {
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.voice = Some("en-US-Studio-Q".to_string());
    config.language_code = Some("en-US".to_string());
    let model = None;
    let provider = Provider::Google;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    let mut file = File::create("tests/tmp-google.mp3").unwrap();
    file.write_all(&speech.audio.clone()).unwrap();
}

#[tokio::test]
async fn test_tts_google_error() {
    let config = transformrs::text_to_speech::TTSConfig::default();
    let model = Some("foobar");
    let provider = Provider::Google;
    let speech = tts_helper(&provider, &config, model).await;
    let err = speech.unwrap_err();
    println!("err: {}", err);
    assert!(err.to_string().contains("INVALID_ARGUMENT"));
}
