//! Text-to-image.
//!
//! Functionality related to text-to-image.

use crate::request_headers;
use crate::Key;
use base64::prelude::*;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
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

fn address(key: &Key) -> String {
    format!("{}/v1/image/generation", key.provider.domain())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Image {
    pub index: u64,
    pub random_seed: Option<u64>,
    pub image: String,
}

impl Image {
    pub fn base64_decode(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let bytes = BASE64_STANDARD
            .decode(self.image.as_bytes())
            .expect("no decode");
        Ok(bytes)
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Images {
    pub images: Vec<Image>,
}

pub async fn text_to_image(
    key: &Key,
    config: TTIConfig,
    prompt: &str,
) -> Result<Images, Box<dyn Error + Send + Sync>> {
    let address = address(key);
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
    let json: Images = match serde_json::from_str(&text) {
        Ok(json) => json,
        Err(e) => {
            panic!("{e} in response:\n{text}");
        }
    };
    Ok(json)
}
