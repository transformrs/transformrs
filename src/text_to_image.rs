//! Text-to-image.
//!
//! Functionality related to text-to-image.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use base64::prelude::*;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;

/// Configuration for text-to-image.
#[derive(Debug, Serialize, Deserialize)]
pub struct TTIConfig {
    pub model: String,
    pub steps: Option<u32>,
    pub cfg_scale: Option<u32>,
    pub height: Option<u32>,
    pub width: Option<u32>,
}

impl Default for TTIConfig {
    fn default() -> Self {
        Self {
            model: "FLUX.1-dev".to_string(),
            steps: Some(10),
            cfg_scale: Some(3),
            height: Some(128),
            width: Some(128),
        }
    }
}

fn address(key: &Key, model: &str) -> String {
    match key.provider {
        Provider::Hyperbolic => format!("{}/v1/image/generation", key.provider.domain()),
        Provider::DeepInfra => format!("{}/v1/inference/{}", key.provider.domain(), model),
        _ => format!("{}/v1/image/generation", key.provider.domain()),
    }
}

#[derive(Debug, Deserialize)]
pub struct Base64Image {
    pub index: u64,
    pub random_seed: Option<u64>,
    pub image: String,
}

pub struct Image {
    pub filetype: String,
    pub image: Vec<u8>,
}

impl Base64Image {
    pub fn base64_decode(&self) -> Result<Image, Box<dyn Error + Send + Sync>> {
        let re = regex::Regex::new(r"^data:image/(\w+);base64,").unwrap();
        let filetype = match re
            .captures(&self.image)
            .map(|cap| cap.get(1).unwrap().as_str())
        {
            Some("png") => "png",
            Some("jpg" | "jpeg") => "jpg",
            _ => "unknown",
        };
        let image = re.replace(&self.image, "").to_string();
        let bytes = BASE64_STANDARD.decode(image.as_bytes()).expect("no decode");
        Ok(Image {
            filetype: filetype.to_string(),
            image: bytes,
        })
    }
}

#[derive(Debug, Deserialize)]
pub struct Images {
    pub images: Vec<Base64Image>,
}

pub async fn text_to_image(
    key: &Key,
    config: TTIConfig,
    prompt: &str,
) -> Result<Images, Box<dyn Error + Send + Sync>> {
    let address = address(key, &config.model);
    let mut body = serde_json::json!({
        "model_name": config.model,
        "prompt": prompt,
    });
    if let Some(steps) = config.steps {
        body["steps"] = serde_json::Value::from(steps);
    }
    if let Some(cfg_scale) = config.cfg_scale {
        body["cfg_scale"] = serde_json::Value::from(cfg_scale);
    }
    if let Some(height) = config.height {
        body["height"] = serde_json::Value::from(height);
    }
    if let Some(width) = config.width {
        body["width"] = serde_json::Value::from(width);
    }
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(request_headers(key)?)
        .json(&body)
        .send()
        .await?;
    let text = resp.text().await?;
    let json: Images = if key.provider == Provider::DeepInfra {
        let value: Value = serde_json::from_str(&text)?;
        let image = value["images"][0].clone();
        let images: Vec<Base64Image> = vec![Base64Image {
            index: 0,
            random_seed: None,
            image: image.as_str().unwrap().to_string(),
        }];
        Images { images }
    } else {
        match serde_json::from_str(&text) {
            Ok(json) => json,
            Err(e) => {
                panic!("{e} in response:\n{text}");
            }
        }
    };
    Ok(json)
}
