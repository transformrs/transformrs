extern crate transformrs;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use transformrs::text_to_image::Image;
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

    let encoded = &resp.parsed().unwrap().images[0];
    let image = encoded.base64_decode().unwrap();
    let mut file = File::create("tests/tmp.jpg").unwrap();
    file.write_all(&image.image).unwrap();
}

async fn text_to_image_helper(
    provider: Provider,
    model: &str,
    prompt: &str,
) -> Result<Image, Box<dyn Error + Send + Sync>> {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).expect("no key");
    let mut config = transformrs::text_to_image::TTIConfig::default();
    config.model = model.to_string();
    let resp = transformrs::text_to_image::text_to_image(&key, config, prompt)
        .await
        .unwrap();

    let encoded = &resp.parsed().unwrap().images[0];
    let image = encoded.base64_decode().unwrap();
    Ok(image)
}

#[tokio::test]
async fn test_text_to_image_deepinfra() {
    text_to_image_helper(
        Provider::DeepInfra,
        "black-forest-labs/FLUX-1-schnell",
        "A beautiful sunset over a calm ocean.",
    )
    .await
    .unwrap();
}
