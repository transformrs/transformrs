use std::fs::File;
use std::io::Write;
use transformrs::Provider;

#[tokio::main]
async fn main() {
    let keys = transformrs::load_keys(".env");
    let provider = Provider::DeepInfra;
    let key = keys.for_provider(&provider).unwrap();
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.voice = Some("am_echo".to_string());
    config.output_format = Some("mp3".to_string());
    let msg = "Hello, world! This is a test of the TTS API.";
    let model = Some("hexgrad/Kokoro-82M");
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
