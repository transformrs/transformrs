pub mod openai;

use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;

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

fn read_key(content: &str, name: &str) -> String {
    content
        .lines()
        .find(|line| line.starts_with(name))
        .and_then(|line| line.split('=').nth(1))
        .map(|key| key.trim().to_string())
        .unwrap_or_else(|| {
            println!("Error: DEEPINFRA_KEY not found in .env file");
            String::new()
        })
}

pub fn read_keys() -> Keys {
    let mut env_content = String::new();
    if let Ok(mut file) = File::open(".env") {
        file.read_to_string(&mut env_content)
            .expect("Failed to read .env file");
    } else {
        panic!("Error: .env file not found");
    }

    let mut keys = vec![];
    for line in env_content.lines() {
        if line.starts_with(&Provider::OpenAI.key_name()) {
            keys.push(Key {
                provider: Provider::OpenAI,
                key: read_key(line, &Provider::OpenAI.key_name()),
            });
        } else if line.starts_with(&Provider::DeepInfra.key_name()) {
            keys.push(Key {
                provider: Provider::DeepInfra,
                key: read_key(line, &Provider::DeepInfra.key_name()),
            });
        }
    }
    Keys { keys }
}
