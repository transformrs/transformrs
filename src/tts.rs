//! Text-to-speech.
//!
//! Functionality related to text-to-speech.

use crate::request_headers;
use crate::Api;
use crate::Key;
use crate::Provider;
use base64::prelude::*;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TTSConfig {
    pub model: String,
    pub output_format: Option<String>,
    pub preset_voice: Option<String>,
    pub speed: Option<f32>,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            model: "hexgrad/Kokoro-82M".to_string(),
            output_format: Some("mp3".to_string()),
            preset_voice: None,
            speed: None,
        }
    }
}

fn address(key: &Key, api: &Api, model: &str) -> String {
    if key.provider == Provider::DeepInfra {
        format!("{}inference/{}", key.provider.url(api), model)
    } else {
        format!("{}chat/completions", key.provider.url(api))
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TTS {
    pub request_id: String,
    pub output_format: String,
    pub audio: String,
}

impl TTS {
    /// Convert the base64 encoded audio to bytes.
    ///
    /// These bytes can then, for example, be written to a file.
    pub fn as_bytes(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let audio = self
            .audio
            .strip_prefix("data:audio/mp3;base64,")
            .unwrap_or(&self.audio);
        let bytes = BASE64_STANDARD.decode(audio).expect("no decode");
        Ok(bytes)
    }
}

pub async fn tts(
    key: &Key,
    api: &Api,
    config: TTSConfig,
    text: &str,
) -> Result<TTS, Box<dyn Error + Send + Sync>> {
    let address = address(key, api, &config.model);
    let mut body = serde_json::json!({
        "text": text,
        "model": config.model,
    });
    if let Some(output_format) = config.output_format {
        body["output_format"] = serde_json::Value::String(output_format);
    }
    if let Some(preset_voice) = config.preset_voice {
        body["preset_voice"] = serde_json::Value::String(preset_voice);
    }
    if let Some(speed) = config.speed {
        body["speed"] = serde_json::Value::from(speed);
    }
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(request_headers(key)?)
        .json(&body)
        .send()
        .await?;
    let json = resp.json::<TTS>().await?;
    Ok(json)
}
