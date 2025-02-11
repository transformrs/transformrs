//! Text-to-speech.
//!
//! Functionality related to text-to-speech.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use base64::prelude::*;
use bytes::Bytes;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::error::Error;

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct TTSConfig {
    pub output_format: Option<String>,
    pub voice: Option<String>,
    pub speed: Option<f32>,
    pub language_code: Option<String>,
}

fn address(key: &Key, model: Option<&str>) -> String {
    if key.provider == Provider::DeepInfra {
        let model = model.unwrap_or("hexgrad/Kokoro-82M");
        format!("{}/v1/inference/{}", key.provider.domain(), model)
    } else if key.provider == Provider::Hyperbolic {
        format!("{}/v1/audio/generation", key.provider.domain())
    } else if key.provider == Provider::OpenAI {
        format!("{}/v1/audio/speech", key.provider.domain())
    } else if key.provider == Provider::Google {
        let domain = "https://texttospeech.googleapis.com";
        let path = "/v1beta1/text:synthesize";
        format!("{domain}{path}?key={}", key.key)
    } else {
        panic!("Unsupported TTS provider: {}", key.provider);
    }
}

#[derive(Debug)]
pub struct Speech {
    pub request_id: Option<String>,
    pub file_format: String,
    pub audio: Bytes,
}

impl Speech {
    /// Convert the base64 encoded audio to bytes.
    ///
    /// These bytes can then, for example, be written to a file.
    pub fn base64_decode(
        audio: &str,
        provider: &Provider,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let stripped = if provider == &Provider::DeepInfra {
            let deepinfra_prefix = "data:audio/mp3;base64,";
            audio.strip_prefix(deepinfra_prefix).unwrap()
        } else {
            audio
        };
        let bytes = BASE64_STANDARD.decode(stripped).expect("no decode");
        Ok(Bytes::from(bytes))
    }
}

pub struct SpeechResponse {
    provider: Provider,
    resp: Bytes,
}

impl SpeechResponse {
    pub fn raw(&self) -> &Bytes {
        &self.resp
    }
    pub fn structured(&self) -> Result<Speech, Box<dyn Error + Send + Sync>> {
        if self.provider == Provider::DeepInfra {
            let resp = serde_json::from_slice::<Value>(&self.resp).unwrap();
            let audio = resp["audio"].as_str().unwrap();
            let out = Speech {
                request_id: Some(resp["request_id"].as_str().unwrap().to_string()),
                file_format: resp["output_format"].as_str().unwrap().to_string(),
                audio: Speech::base64_decode(audio, &self.provider)?,
            };
            Ok(out)
        } else if self.provider == Provider::Hyperbolic {
            let resp = serde_json::from_slice::<Value>(&self.resp).unwrap();
            let audio = &resp["audio"].as_str().unwrap();
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: Speech::base64_decode(audio, &self.provider)?,
            };
            Ok(out)
        } else if self.provider == Provider::OpenAI {
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: self.resp.clone(),
            };
            Ok(out)
        } else if self.provider == Provider::Google {
            let resp = serde_json::from_slice::<Value>(&self.resp).unwrap();
            if resp.get("error").is_some() {
                panic!("Google returned an error: {}", resp["error"]);
            }
            let audio = &resp["audioContent"].as_str().expect("audioContent");
            let _timepoints = &resp["timepoints"].as_array().unwrap();
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: Speech::base64_decode(audio, &self.provider)?,
            };
            Ok(out)
        } else {
            panic!("Unsupported TTS provider: {}", self.provider);
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
    let mut body = json!({});
    if key.provider == Provider::OpenAI {
        body["input"] = Value::String(text.to_string());
    } else if key.provider == Provider::Google {
        body["input"] = json!({
            "text": text.to_string()
        });
    } else {
        body["text"] = Value::String(text.to_string());
    }
    if let Some(model) = &model {
        body["model"] = Value::String(model.to_string());
    }
    if let Some(output_format) = &config.output_format {
        body["output_format"] = Value::String(output_format.clone());
    }
    if let Some(voice) = &config.voice {
        if key.provider == Provider::OpenAI {
            body["voice"] = Value::String(voice.clone());
        } else if key.provider == Provider::Google {
            body["voice"] = json!({
                "name": voice.clone()
            });
            if let Some(language_code) = &config.language_code {
                body["voice"]["languageCode"] = Value::String(language_code.clone());
            }
            body["audioConfig"] = json!({
                "audioEncoding": "LINEAR16",
                "pitch": 0,
                "speakingRate": 1
            });
        } else if key.provider == Provider::DeepInfra {
            body["preset_voice"] = Value::String(voice.clone());
        } else {
            panic!("Unsupported TTS provider: {}", key.provider);
        }
    }
    if let Some(speed) = config.speed {
        body["speed"] = Value::from(speed);
    }
    println!("body: {:?}", body);
    let headers = if key.provider == Provider::Google {
        let mut headers = request_headers(key)?;
        headers.remove("Authorization");
        headers
    } else {
        request_headers(key)?
    };
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    let speech_response = SpeechResponse {
        provider: key.provider.clone(),
        resp: resp.bytes().await?,
    };
    Ok(speech_response)
}
