extern crate transformrs;

use std::fs::File;
use std::io::Write;
use transformrs::Provider;

#[tokio::test]
async fn test_tts() {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.preset_voice = Some("am_echo".to_string());
    let msg = "Hello, world! This is a test of the TTS API.";
    let model = "hexgrad/Kokoro-82M".to_string();
    let resp = transformrs::text_to_speech::tts(&key, config, &model, msg)
        .await
        .unwrap();
    let resp = resp.structured().unwrap();
    assert_eq!(resp.output_format, "mp3");
    let bytes = resp.base64_decode().unwrap();
    assert!(bytes.len() > 0);

    // Can be used to manually verify the output.
    let mut file = File::create("tests/tmp.mp3").unwrap();
    file.write_all(&bytes).unwrap();
}
