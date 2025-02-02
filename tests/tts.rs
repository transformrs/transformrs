extern crate transformrs;

use transformrs::Api;
use transformrs::Provider;

#[tokio::test]
async fn test_tts() {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let config = transformrs::tts::TTSConfig::default();
    let resp = transformrs::tts::tts(&key, &Api::DeepInfra, config, "Hello, world!")
        .await
        .unwrap();
    assert_eq!(resp.output_format, "mp3");
    let bytes = resp.as_bytes().unwrap();
    assert!(bytes.len() > 0);
}
