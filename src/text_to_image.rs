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
            steps: Some(25),
            cfg_scale: Some(3),
            height: Some(1024),
            width: Some(1024),
        }
    }
}

fn address(key: &Key) -> String {
    format!("{}/v1/image/generation", key.provider.domain())
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TTI {
    pub request_id: String,
}

pub async fn text_to_image(
    key: &Key,
    config: TTIConfig,
    prompt: &str,
) -> Result<TTI, Box<dyn Error + Send + Sync>> {
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
    let json = resp.json::<TTI>().await?;
    Ok(json)
}
