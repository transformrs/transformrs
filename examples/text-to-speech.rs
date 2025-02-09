use std::fs::File;
use std::io::Write;
use transformrs::Provider;

#[tokio::main]
async fn main() {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let mut config = transformrs::text_to_speech::TTSConfig::default();
    config.preset_voice = Some("am_echo".to_string());
    let msg = "Hello, world! This is a test of the TTS API.";
    let model = "hexgrad/Kokoro-82M".to_string();
    let resp = transformrs::text_to_speech::tts(&key, config, &model, msg)
        .await
        .unwrap()
        .structured()
        .unwrap();
    let bytes = resp.base64_decode().unwrap();
    let ext = resp.output_format;
    let mut file = File::create(format!("test.{ext}")).unwrap();
    file.write_all(&bytes).unwrap();
}
