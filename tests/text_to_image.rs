extern crate transformrs;

use std::fs::File;
use std::io::Write;
use transformrs::Provider;

#[tokio::test]
async fn test_text_to_image() {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&Provider::Hyperbolic).expect("no key");
    let mut config = transformrs::text_to_image::TTIConfig::default();
    config.model = "FLUX.1-dev".to_string();
    let prompt = "A beautiful sunset over a calm ocean.";
    let resp = transformrs::text_to_image::text_to_image(&key, config, prompt)
        .await
        .unwrap();
    println!("resp: {:?}", resp);

    assert!(false);
}
