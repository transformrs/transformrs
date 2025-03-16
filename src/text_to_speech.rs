//! Text-to-speech.
//!
//! Functionality related to text-to-speech.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use base64::prelude::*;
use bytes::Bytes;
use reqwest;
use serde::Deserialize;
use serde::Serialize;
use serde_json::json;
use serde_json::Value;
use std::collections::HashMap;
use std::error::Error;

/// Text-to-speech config
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct TTSConfig {
    pub output_format: Option<String>,
    pub voice: Option<String>,
    pub speed: Option<f64>,
    pub language_code: Option<String>,
    pub seed: Option<u64>,
    pub other: Option<HashMap<String, Value>>,
}

fn is_openai_compatible(provider: &Provider) -> bool {
    matches!(provider, Provider::OpenAICompatible(_))
}

fn address(
    provider: &Provider,
    key: &Key,
    model: Option<&str>,
    config: &TTSConfig,
) -> String {
    if provider == &Provider::ElevenLabs {
        let voice = config.voice.as_ref().expect("voice is required for ElevenLabs");
        if let Some(output_format) = &config.output_format {
            format!(
                "{}/v1/text-to-speech/{voice}?{output_format}",
                provider.domain()
            )
        } else {
            format!("{}/v1/text-to-speech/{voice}", provider.domain())
        }
    } else if provider == &Provider::DeepInfra {
        let model = model.unwrap_or("hexgrad/Kokoro-82M");
        format!("{}/v1/inference/{}", provider.domain(), model)
    } else if provider == &Provider::Hyperbolic {
        format!("{}/v1/audio/generation", provider.domain())
    } else if provider == &Provider::OpenAI {
        format!("{}/v1/audio/speech", provider.domain())
    } else if let Provider::OpenAICompatible(domain) = &provider {
        format!("{domain}/v1/audio/speech")
    } else if provider == &Provider::Google {
        let domain = "https://texttospeech.googleapis.com";
        let path = "/v1beta1/text:synthesize";
        format!("{domain}{path}?key={}", key.key)
    } else {
        panic!("Unsupported TTS provider: {}", provider);
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
    pub fn decode_speech(
        audio: &str,
        provider: &Provider,
        output_format: Option<&str>,
    ) -> Result<Bytes, Box<dyn Error + Send + Sync>> {
        let stripped = if provider == &Provider::DeepInfra {
            let output_format = output_format.expect("no output format");
            tracing::debug!("Decoding DeepInfra speech with output format: {output_format}");
            let deepinfra_prefix = match output_format {
                "mp3" => "data:audio/mp3;base64,",
                "opus" => "data:audio/ogg; codec=\"opus\";base64,",
                _ => panic!("Unsupported output format: {}", output_format),
            };
            match audio.strip_prefix(deepinfra_prefix) {
                Some(stripped) => stripped,
                None => panic!("prefix '{deepinfra_prefix}' not found"),
            }
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
    pub fn bytes(&self) -> &Bytes {
        &self.resp
    }
    pub fn raw_value(&self) -> Result<Value, Box<dyn Error + Send + Sync>> {
        Ok(serde_json::from_slice::<Value>(&self.resp)?)
    }
    pub fn structured(&self) -> Result<Speech, Box<dyn Error + Send + Sync>> {
        if self.provider == Provider::ElevenLabs {
            return Ok(Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: self.resp.clone(),
            });
        } else if self.provider == Provider::DeepInfra {
            let resp = self.raw_value()?;
            tracing::debug!("Response: {resp}");
            if resp.get("detail").is_some() {
                return Err(format!("DeepInfra returned an error: {}", resp["detail"]).into());
            }
            let audio = resp["audio"].as_str().expect("no audio in resp");
            let output_format = resp["output_format"].as_str().unwrap().to_string();
            let out = Speech {
                request_id: Some(resp["request_id"].as_str().unwrap().to_string()),
                file_format: output_format.to_string(),
                audio: Speech::decode_speech(audio, &self.provider, Some(&output_format))?,
            };
            Ok(out)
        } else if self.provider == Provider::Hyperbolic {
            let resp = self.raw_value()?;
            tracing::debug!("Response: {resp}");
            let audio = &resp["audio"].as_str().unwrap();
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: Speech::decode_speech(audio, &self.provider, None)?,
            };
            Ok(out)
        } else if self.provider == Provider::OpenAI || is_openai_compatible(&self.provider) {
            let audio = self.resp.clone();
            if let Ok(resp) = serde_json::from_slice::<Value>(&self.resp) {
                tracing::debug!("Response: {resp}");
                if resp.get("error").is_some() {
                    return Err(resp["error"].to_string().into());
                }
            }
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio,
            };
            Ok(out)
        } else if self.provider == Provider::Google {
            let resp = self.raw_value()?;
            tracing::debug!("Response: {resp}");
            if resp.get("error").is_some() {
                return Err(resp["error"].to_string().into());
            }
            let audio = &resp["audioContent"].as_str().expect("audioContent");
            let _timepoints = &resp["timepoints"].as_array().unwrap();
            let out = Speech {
                request_id: None,
                file_format: "mp3".to_string(),
                audio: Speech::decode_speech(audio, &self.provider, None)?,
            };
            Ok(out)
        } else {
            panic!("Unsupported TTS provider: {}", self.provider);
        }
    }
}

fn tts_headers(provider: &Provider, key: &Key) -> Result<HeaderMap, Box<dyn Error + Send + Sync>> {
    let headers = if provider == &Provider::Google {
        let mut headers = request_headers(key)?;
        headers.remove("Authorization");
        headers
    } else if provider == &Provider::ElevenLabs {
        let mut headers = request_headers(key)?;
        headers.insert("xi-api-key", HeaderValue::from_str(&key.key)?);
        headers.remove("Authorization");
        headers
    } else {
        request_headers(key)?
    };
    Ok(headers)
}

fn tts_body(config: &TTSConfig, provider: &Provider, model: Option<&str>, text: &str) -> Value {
    if provider == &Provider::ElevenLabs {
        let mut body = json!({});
        body["text"] = Value::String(text.to_string());
        if let Some(model) = &model {
            body["model_id"] = Value::String(model.to_string());
        }
        if let Some(language_code) = &config.language_code {
            body["language_code"] = Value::String(language_code.clone());
        }
        if let Some(_speed) = &config.speed {
            panic!("Set speed for ElevenLabs via stored settings for voice.");
        }
        if let Some(seed) = &config.seed {
            body["seed"] = Value::String(seed.to_string());
        }
        return body;
    }
    let mut body = json!({});
    if provider == &Provider::OpenAI || is_openai_compatible(provider) {
        body["input"] = Value::String(text.to_string());
    } else if provider == &Provider::Google {
        body["input"] = json!({
            "text": text.to_string()
        });
    } else {
        body["text"] = Value::String(text.to_string());
    }
    if let Some(model) = &model {
        body["model"] = Value::String(model.to_string());
    }
    if let Some(voice) = &config.voice {
        if provider == &Provider::OpenAI || is_openai_compatible(provider) {
            body["voice"] = Value::String(voice.clone());
        } else if provider == &Provider::Google {
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
        } else if provider == &Provider::DeepInfra {
            body["preset_voice"] = Value::String(voice.clone());
        } else {
            panic!("Unsupported TTS provider: {}", provider);
        }
    }
    if let Some(speed) = config.speed {
        body["speed"] = Value::from(speed);
    }
    if let Some(output_format) = &config.output_format {
        body["output_format"] = Value::String(output_format.clone());
    }
    if let Some(other) = &config.other {
        for (key, value) in other {
            body[key] = value.clone();
        }
    }
    body
}

pub async fn tts(
    provider: &Provider,
    key: &Key,
    config: &TTSConfig,
    model: Option<&str>,
    text: &str,
) -> Result<SpeechResponse, Box<dyn Error + Send + Sync>> {
    let address = address(provider, key, model, config);
    let headers = tts_headers(provider, key)?;
    let body = tts_body(config, provider, model, text);
    tracing::debug!("Requesting {address} for text-to-speech with {body}");
    let client = reqwest::Client::new();
    let resp = client
        .post(address)
        .headers(headers)
        .json(&body)
        .send()
        .await?;
    let speech_response = SpeechResponse {
        provider: provider.clone(),
        resp: resp.bytes().await?,
    };
    Ok(speech_response)
}
