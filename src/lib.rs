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

pub struct Key {
    pub provider: Provider,
    pub key: String,
}

pub fn read_key() -> Key {
    let mut env_content = String::new();
    if let Ok(mut file) = File::open(".env") {
        file.read_to_string(&mut env_content)
            .expect("Failed to read .env file");
    } else {
        panic!("Error: .env file not found");
    }

    let key = env_content
        .lines()
        .find(|line| line.starts_with("DEEPINFRA_KEY="))
        .and_then(|line| line.split('=').nth(1))
        .map(|key| key.trim().to_string())
        .unwrap_or_else(|| {
            println!("Error: DEEPINFRA_KEY not found in .env file");
            String::new()
        });

    Key {
        provider: Provider::DeepInfra,
        key,
    }
}
