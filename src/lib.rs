pub mod openai;

use serde::Deserialize;
use serde::Serialize;
use std::fs::File;
use std::io::Read;

pub enum Api {
    /// The OpenAI API. Most providers (partially) support this.
    OpenAI,
    /// The DeepInfra API. This is the non-OpenAI-compatible API.
    DeepInfra,
}

#[derive(Clone, Debug, PartialEq)]
pub enum Provider {
    OpenAI,
    DeepInfra,
}

impl Provider {
    pub fn url(&self, api: &Api) -> String {
        match self {
            Provider::OpenAI => "https://api.openai.com/v1/",
            Provider::DeepInfra => match api {
                Api::OpenAI => "https://api.deepinfra.com/v1/openai/",
                Api::DeepInfra => "https://api.deepinfra.com/v1/",
            },
        }
        .to_string()
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
        if line.starts_with("OPENAI_KEY=") {
            keys.push(Key {
                provider: Provider::OpenAI,
                key: read_key(line, "OPENAI_KEY"),
            });
        } else if line.starts_with("DEEPINFRA_KEY=") {
            keys.push(Key {
                provider: Provider::DeepInfra,
                key: read_key(line, "DEEPINFRA_KEY"),
            });
        }
    }
    Keys { keys }
}
