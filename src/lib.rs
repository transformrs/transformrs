pub mod models;
pub mod openai;
pub mod text_to_image;
pub mod text_to_speech;

use base64::prelude::BASE64_STANDARD;
use base64::Engine;
use reqwest::header::HeaderMap;
use reqwest::header::HeaderValue;
use serde::Deserialize;
use serde::Serialize;
use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::Read;

pub(crate) fn request_headers(key: &Key) -> Result<HeaderMap, Box<dyn Error + Send + Sync>> {
    let mut headers = HeaderMap::new();
    headers.insert(
        "Authorization",
        HeaderValue::from_str(&format!("Bearer {}", key.key))?,
    );
    headers.insert("Content-Type", HeaderValue::from_str("application/json")?);
    Ok(headers)
}

pub(crate) fn openai_base_url(provider: &Provider) -> String {
    match provider {
        Provider::Groq => format!("{}/openai/v1", provider.domain()),
        Provider::OpenAI => format!("{}/v1", provider.domain()),
        Provider::Hyperbolic => format!("{}/v1", provider.domain()),
        Provider::Google => format!("{}/v1beta/openai", provider.domain()),
        Provider::Other(domain) => domain.clone(),
        _ => format!("{}/v1/openai", provider.domain()),
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Api {
    /// The OpenAI API. Most providers (partially) support this.
    OpenAI,
    /// The DeepInfra API. This is their non-OpenAI-compatible API.
    DeepInfra,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Provider {
    Amazon,
    Azure,
    Cerebras,
    DeepInfra,
    Fireworks,
    FriendliAI,
    Google,
    Groq,
    Hyperbolic,
    Nebius,
    Novita,
    OpenAI,
    /// Another OpenAI-compatible provider.
    ///
    /// For example, "https://api.deepinfra.com".
    Other(String),
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Provider {
    pub fn domain(&self) -> String {
        match self {
            Provider::Amazon => "https://api.amazon.com",
            Provider::Azure => "https://api.azure.com",
            Provider::Cerebras => "https://api.cerebras.ai",
            Provider::DeepInfra => "https://api.deepinfra.com",
            Provider::Fireworks => "https://api.fireworks.ai",
            Provider::FriendliAI => "https://api.friendli.ai",
            Provider::Google => "https://generativelanguage.googleapis.com",
            Provider::Groq => "https://api.groq.com",
            Provider::Hyperbolic => "https://api.hyperbolic.xyz",
            Provider::Nebius => "https://api.nebi.us",
            Provider::Novita => "https://api.novita.ai",
            Provider::OpenAI => "https://api.openai.com",
            Provider::Other(domain) => domain,
        }
        .to_string()
    }
    pub fn key_name(&self) -> String {
        match self {
            Provider::Other(_) => "OTHER_KEY".to_string(),
            _ => self.to_string().to_uppercase() + "_KEY",
        }
    }
}

#[derive(Clone, Debug, Deserialize)]
pub enum SubContent {
    TextContent { text: String },
    ImageUrlContent { image_url: String },
}

impl SubContent {
    pub fn new(r#type: &str, text: &str) -> Self {
        match r#type {
            "text" => Self::TextContent {
                text: text.to_string(),
            },
            "image_url" => Self::ImageUrlContent {
                image_url: text.to_string(),
            },
            _ => panic!("Invalid subcontent type: {}", r#type),
        }
    }
}

impl Serialize for SubContent {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            SubContent::TextContent { text } => serializer.serialize_str(text),
            SubContent::ImageUrlContent { image_url } => {
                let json = serde_json::json!({
                    "type": "image_url",
                    "image_url": {
                        "url": image_url
                    }
                });
                json.serialize(serializer)
            }
        }
    }
}

#[derive(Clone, Debug)]
pub enum Content {
    Text(String),
    Collection(Vec<SubContent>),
}

impl std::fmt::Display for Content {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Content::Text(text) => write!(f, "{}", text),
            Content::Collection(items) => {
                write!(f, "{items:?}")
            }
        }
    }
}

impl Serialize for Content {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Content::Text(text) => serializer.serialize_str(text),
            Content::Collection(items) => items.serialize(serializer),
        }
    }
}

impl<'de> Deserialize<'de> for Content {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let value = serde_json::Value::deserialize(deserializer)?;
        if let serde_json::Value::String(text) = value {
            Ok(Content::Text(text))
        } else if let serde_json::Value::Array(items) = value {
            let subcontent = items
                .into_iter()
                .map(SubContent::deserialize)
                .collect::<Result<Vec<_>, _>>()
                .unwrap();
            Ok(Content::Collection(subcontent))
        } else {
            Err(serde::de::Error::custom("Invalid content format"))
        }
    }
}

#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct Message {
    pub role: String,
    pub content: Content,
}

impl Message {
    pub fn from_str(role: &str, text: &str) -> Self {
        Self {
            role: role.to_string(),
            content: Content::Text(text.to_string()),
        }
    }
    pub fn from_image_url(role: &str, image_url: &str) -> Self {
        Self {
            role: role.to_string(),
            content: Content::Collection(vec![SubContent::ImageUrlContent {
                image_url: image_url.to_string(),
            }]),
        }
    }
    pub fn from_image_bytes(role: &str, image_type: &str, image: &[u8]) -> Self {
        let base64 = BASE64_STANDARD.encode(image);
        let image_url = format!("data:image/{image_type};base64,{base64}");
        Self::from_image_url(role, &image_url)
    }
}

#[derive(Clone, Debug)]
pub struct Key {
    pub provider: Provider,
    pub key: String,
}

#[derive(Clone, Debug)]
pub struct Keys {
    keys: Vec<Key>,
}

impl Keys {
    pub fn for_provider(&self, provider: &Provider) -> Option<Key> {
        fn finder(provider: &Provider, key: &Key) -> bool {
            match provider {
                Provider::Other(_domain) => matches!(&key.provider, Provider::Other(_)),
                _ => key.provider == *provider,
            }
        }

        self.keys.iter().find(|key| finder(provider, key)).cloned()
    }
}

fn load_env_file(path: &str) -> HashMap<String, String> {
    let mut env_content = String::new();
    if let Ok(mut file) = File::open(path) {
        file.read_to_string(&mut env_content)
            .expect("Failed to read .env file");
    }
    env_content
        .lines()
        .filter_map(|line| {
            let mut parts = line.split('=');
            if let (Some(key), Some(value)) = (parts.next(), parts.next()) {
                Some((key.to_string(), value.to_string()))
            } else {
                None
            }
        })
        .collect()
}

/// Load the keys from either the .env file or environment variables.
pub fn load_keys(path: &str) -> Keys {
    let env_map = load_env_file(path);

    let mut keys = vec![];

    let providers = [
        Provider::Amazon,
        Provider::Azure,
        Provider::DeepInfra,
        Provider::Fireworks,
        Provider::FriendliAI,
        Provider::Google,
        Provider::Groq,
        Provider::Hyperbolic,
        Provider::Nebius,
        Provider::Novita,
        Provider::OpenAI,
        Provider::Other("".to_string()),
    ];
    for provider in providers {
        if let Ok(key_value) = std::env::var(provider.key_name()) {
            keys.push(Key {
                provider: provider.clone(),
                key: key_value,
            });
        } else if let Some(key_value) = env_map.get(&provider.key_name()) {
            keys.push(Key {
                provider: provider.clone(),
                key: key_value.to_string(),
            });
        }
    }
    Keys { keys }
}
