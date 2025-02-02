pub mod openai;
pub mod tts;

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

#[derive(Clone, Debug, PartialEq)]
pub enum Api {
    /// The OpenAI API. Most providers (partially) support this.
    OpenAI,
    /// The DeepInfra API. This is their non-OpenAI-compatible API.
    DeepInfra,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Provider {
    OpenAI,
    DeepInfra,
    Groq,
    Azure,
    Amazon,
    TogetherAI,
    Fireworks,
    FriendliAI,
    Hyperbolic,
    Nebius,
    Novita,
}

impl std::fmt::Display for Provider {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl Provider {
    pub fn url(&self, api: &Api) -> String {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1/",
            Provider::DeepInfra => match api {
                Api::OpenAI => "https://api.deepinfra.com/v1/openai/",
                Api::DeepInfra => "https://api.deepinfra.com/v1/",
            },
            Provider::Groq => "https://api.groq.com/openai/v1/",
            Provider::Azure => "https://api.azure.com/v1/",
            Provider::Amazon => "https://api.amazon.com/v1/",
            Provider::TogetherAI => "https://api.together.ai/v1/",
            Provider::Fireworks => "https://api.fireworks.ai/v1/",
            Provider::FriendliAI => "https://api.friendli.ai/v1/",
            Provider::Hyperbolic => "https://api.hyperbolic.xyz/v1/",
            Provider::Nebius => "https://api.nebi.us/v1/",
            Provider::Novita => "https://api.novita.ai/v1/",
        }
        .to_string()
    }
    pub fn key_name(&self) -> String {
        self.to_string().to_uppercase() + "_KEY"
    }
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Message {
    pub role: String,
    pub content: String,
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
        self.keys
            .iter()
            .find(|key| key.provider == *provider)
            .cloned()
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
                Some((key.trim().to_string(), value.trim().to_string()))
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

    // Loop through each provider
    for provider in [Provider::OpenAI, Provider::DeepInfra] {
        // First check environment variables
        if let Ok(key_value) = std::env::var(provider.key_name()) {
            keys.push(Key {
                provider: provider.clone(),
                key: key_value,
            });
        }
        // Then check .env file
        else if let Some(key_value) = env_map.get(&provider.key_name()) {
            keys.push(Key {
                provider: provider.clone(),
                key: key_value.to_string(),
            });
        }
    }
    Keys { keys }
}
