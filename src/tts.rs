//! Text-to-speech.
//! 
//! Functionality related to text-to-speech.

use crate::Key;
use crate::Provider;
use crate::request_headers;
use base64::prelude::*;
use crate::Api;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TTSConfig {
    model: String,
    output_format: Option<String>,
    preset_voice: Option<String>,
    speed: Option<f32>,
}

impl TTSConfig {    
    pub fn new(model: &str, output_format: Option<&str>, preset_voice: Option<&str>, speed: Option<f32>) -> Self {
        Self {
            model: model.to_string(),
            output_format: output_format.map(|s| s.to_string()),
            preset_voice: preset_voice.map(|s| s.to_string()),
            speed,
        }
    }
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

fn base64_decode(value: &str) -> String {
    // Base64 may include newlines to be compatible with older versions that
    // have a maximum line length.
    let value = value.replace("\n", "");
    let decoded = BASE64_STANDARD.decode(value.trim()).expect("no decode");
    String::from_utf8(decoded).expect("no utf8")
}

impl TTS {
    /// Convert the base64 encoded audio to bytes.
    /// 
    /// These bytes can then be written to a file or used in other ways.
    pub fn as_bytes(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        // if starts with data:audio/mp3;base64, then remove this
        let audio = if self.audio.starts_with("data:audio/mp3;base64,") {
            &self.audio[22..]
        } else {
            &self.audio
        };
        let bytes = base64_decode(audio);
        Ok(bytes.as_bytes().to_vec())
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