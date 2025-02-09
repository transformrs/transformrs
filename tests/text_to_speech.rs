extern crate transformrs;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use transformrs::text_to_speech::Speech;
use transformrs::text_to_speech::TTSConfig;
use transformrs::Provider;

async fn tts_helper(
    provider: &Provider,
    config: &TTSConfig,
    model: Option<&str>,
) -> Result<Speech, Box<dyn Error + Send + Sync>> {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let msg = "Hello, world!";
    let resp = transformrs::text_to_speech::tts(&key, config, model, msg)
        .await
        .unwrap();
    let resp = resp.structured().unwrap();
    Ok(resp)
}

#[tokio::test]
async fn test_tts_deepinfra() {
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.preset_voice = Some("am_echo".to_string());
    let model = Some("hexgrad/Kokoro-82M");
    let provider = Provider::DeepInfra;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    assert_eq!(speech.output_format, "mp3");
    let bytes = speech.base64_decode().unwrap();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp-deepinfra.mp3").unwrap();
    file.write_all(&bytes).unwrap();
}

#[tokio::test]
async fn test_tts_hyperbolic() {
    let config = transformrs::text_to_speech::TTSConfig::default();
    let model = None;
    let provider = Provider::Hyperbolic;
    let speech = tts_helper(&provider, &config, model).await.unwrap();
    let mut file = File::create("tests/tmp").unwrap();
    file.write_all(&speech.audio.to_string().as_bytes())
        .unwrap();
    let bytes = speech.base64_decode().unwrap();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp-hyperbolic.mp3").unwrap();
    file.write_all(&bytes).unwrap();
}
