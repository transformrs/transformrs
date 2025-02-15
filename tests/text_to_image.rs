extern crate transformrs;

mod common;

use std::error::Error;
use std::fs::File;
use std::io::Write;
use transformrs::text_to_image::Images;
use transformrs::Provider;

#[tokio::test]
async fn text_to_image_hyperbolic() {
    common::init_tracing();
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&Provider::Hyperbolic).expect("no key");
    let mut config = transformrs::text_to_image::TTIConfig::default();
    config.model = "FLUX.1-dev".to_string();
    let prompt = "A beautiful sunset over a calm ocean.";
    let resp = transformrs::text_to_image::text_to_image(&key, config, prompt)
        .await
        .unwrap();

    let encoded = &resp.structured().unwrap().images[0];
    let image = encoded.base64_decode().unwrap();
    let mut file = File::create("tests/tmp.jpg").unwrap();
    file.write_all(&image.image).unwrap();
}

async fn text_to_image_helper(
    provider: Provider,
    model: &str,
    prompt: &str,
) -> Result<Images, Box<dyn Error + Send + Sync>> {
    common::init_tracing();
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).expect("no key");
    let mut config = transformrs::text_to_image::TTIConfig::default();
    config.model = model.to_string();
    let resp = transformrs::text_to_image::text_to_image(&key, config, prompt)
        .await
        .unwrap();

    resp.structured()
}

#[tokio::test]
async fn text_to_image_deepinfra() {
    let resp = text_to_image_helper(
        Provider::DeepInfra,
        "black-forest-labs/FLUX-1-schnell",
        "A beautiful sunset over a calm ocean.",
    )
    .await
    .unwrap();
    resp.images[0].base64_decode().unwrap();
}

#[tokio::test]
async fn text_to_image_deepinfra_error() {
    let resp = text_to_image_helper(
        Provider::DeepInfra,
        "foo",
        "A beautiful sunset over a calm ocean.",
    )
    .await
    .unwrap_err();
    assert!(resp.to_string().contains("Model is not available"));
}
