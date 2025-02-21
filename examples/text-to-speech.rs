//! An example with text-to-speech.

// You can execute this example with `cargo run --example text-to-speech`

use serde_json::json;
use std::collections::HashMap;
use std::fs::File;
use std::io::Write;
use transformrs::Provider;

#[tokio::main]
async fn main() {
    let keys = transformrs::load_keys(".env");
    let provider = Provider::DeepInfra;
    let key = keys.for_provider(&provider).unwrap();
    let mut other = HashMap::new();
    other.insert("seed".to_string(), json!(42));
    let config = transformrs::text_to_speech::TTSConfig {
        voice: Some("american_male".to_string()),
        output_format: Some("mp3".to_string()),
        other: Some(other),
        ..Default::default()
    };
    let msg = "Hello, world! This is a test of the TTS API.";
    let model = Some("Zyphra/Zonos-v0.1-hybrid");
    let resp = transformrs::text_to_speech::tts(&key, &config, model, msg)
        .await
        .unwrap()
        .structured()
        .unwrap();
    let bytes = resp.audio.clone();
    let ext = resp.file_format;
    let mut file = File::create(format!("example.{ext}")).unwrap();
    file.write_all(&bytes).unwrap();
}
