//! Text-to-speech.
//!
//! Functionality related to text-to-speech.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use base64::prelude::*;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct TTSConfig {
    pub output_format: Option<String>,
    pub preset_voice: Option<String>,
    pub speed: Option<f32>,
}

impl Default for TTSConfig {
    fn default() -> Self {
        Self {
            output_format: Some("mp3".to_string()),
            preset_voice: None,
            speed: None,
        }
    }
}

fn address(key: &Key, model: Option<&str>) -> String {
    if key.provider == Provider::DeepInfra {
        let model = model.unwrap_or("hexgrad/Kokoro-82M");
        format!("{}/v1/inference/{}", key.provider.domain(), model)
    } else if key.provider == Provider::Hyperbolic {
        format!("{}/v1/audio/generation", key.provider.domain())
    } else {
        panic!("Unsupported TTS provider: {}", key.provider);
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Speech {
    pub request_id: Option<String>,
    pub output_format: String,
    pub audio: String,
}

impl Speech {
    /// Convert the base64 encoded audio to bytes.
    ///
    /// These bytes can then, for example, be written to a file.
    pub fn base64_decode(&self) -> Result<Vec<u8>, Box<dyn Error + Send + Sync>> {
        let audio = self
            .audio
            .strip_prefix("data:audio/mp3;base64,")
            .unwrap_or(&self.audio);
        let bytes = BASE64_STANDARD.decode(audio).expect("no decode");
        Ok(bytes)
    }
}

pub struct SpeechResponse {
    provider: Provider,
    resp: Value,
}

impl SpeechResponse {
    pub fn raw(&self) -> &Value {
        &self.resp
    }
    pub fn structured(&self) -> Result<Speech, Box<dyn Error + Send + Sync>> {
        if self.provider == Provider::Hyperbolic {
            let out = Speech {
                request_id: None,
                output_format: "mp3".to_string(),
                audio: self.resp["audio"].to_string(),
            };
            Ok(out)
        } else {
            let json = match serde_json::from_value(self.resp.clone()) {
                Ok(json) => json,
                Err(e) => {
                    return Err(format!("{e} in response:\n{}", self.resp).into());
                }
            };
            Ok(json)
        }
    }
}

pub async fn tts(
    key: &Key,
    config: &TTSConfig,
    model: Option<&str>,
    text: &str,
) -> Result<SpeechResponse, Box<dyn Error + Send + Sync>> {
    let address = address(key, model);
    let mut body = serde_json::json!({
        "text": text,
    });
    if let Some(model) = &model {
        body["model"] = serde_json::Value::String(model.to_string());
    }
    if let Some(output_format) = &config.output_format {
        body["output_format"] = serde_json::Value::String(output_format.clone());
    }
    if let Some(preset_voice) = &config.preset_voice {
        body["preset_voice"] = serde_json::Value::String(preset_voice.clone());
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
    let speech_response = SpeechResponse {
        provider: key.provider.clone(),
        resp: resp.json::<Value>().await?,
    };
    Ok(speech_response)
}
