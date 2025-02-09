//! Models.
//!
//! Functionality related to requesting available models.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use reqwest;
use serde_json::Value;
use std::error::Error;

fn address(key: &Key) -> String {
    let base_url = match key.provider {
        Provider::Groq => format!("{}/openai/v1", key.provider.domain()),
        Provider::OpenAI => format!("{}/v1", key.provider.domain()),
        Provider::Hyperbolic => format!("{}/v1", key.provider.domain()),
        Provider::Google => format!("{}/v1beta/openai", key.provider.domain()),
        _ => format!("{}/v1/openai", key.provider.domain()),
    };
    format!("{}/models", base_url)
}

pub struct ModelsResponse {
    resp: Value,
}

impl ModelsResponse {
    pub fn raw(&self) -> &Value {
        &self.resp
    }
}

pub async fn models(key: &Key) -> Result<ModelsResponse, Box<dyn Error + Send + Sync>> {
    let address = address(key);
    let client = reqwest::Client::new();
    let resp = client
        .get(address)
        .headers(request_headers(key)?)
        .send()
        .await?;
    let models_response = ModelsResponse {
        resp: resp.json::<Value>().await?,
    };
    Ok(models_response)
}
