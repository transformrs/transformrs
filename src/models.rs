//! Models.
//!
//! Functionality related to requesting available models.

use crate::request_headers;
use crate::Key;
use crate::Provider;
use reqwest;
use serde::Deserialize;
use serde_json::Value;
use std::error::Error;

fn address(provider: &Provider) -> String {
    let base_url = crate::openai_base_url(provider);
    format!("{}/models", base_url)
}

pub struct ModelsResponse {
    resp: Value,
}

#[derive(Debug, Deserialize)]
pub struct Model {
    pub id: String,
}

#[derive(Debug, Deserialize)]
pub struct Models {
    pub models: Vec<Model>,
}

impl Models {
    pub fn contains(&self, id: &str) -> bool {
        self.models.iter().any(|model| model.id == id)
    }
}

impl ModelsResponse {
    pub fn raw(&self) -> &Value {
        &self.resp
    }
    pub fn structured(&self) -> Result<Models, Box<dyn Error + Send + Sync>> {
        let data = self.resp.get("data").unwrap().as_array().unwrap();
        Ok(Models {
            models: data
                .iter()
                .map(|model| serde_json::from_value(model.clone()).unwrap())
                .collect(),
        })
    }
}

pub async fn models(
    provider: &Provider,
    key: &Key,
) -> Result<ModelsResponse, Box<dyn Error + Send + Sync>> {
    let address = address(provider);
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
