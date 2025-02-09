extern crate transformrs;

use std::error::Error;
use transformrs::models::models;
use transformrs::models::Models;
use transformrs::Provider;

async fn test_models(provider: Provider) -> Result<Models, Box<dyn Error + Send + Sync>> {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let resp = models(&provider, &key).await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        }
    };
    println!("{:?}", resp.raw());
    let resp = resp.structured();
    Ok(resp.unwrap())
}

#[tokio::test]
async fn test_models_groq() {
    let models = test_models(Provider::Groq).await.unwrap();
    assert!(models.contains("llama3-8b-8192"));
}

#[tokio::test]
async fn test_models_openai() {
    let models = test_models(Provider::OpenAI).await.unwrap();
    assert!(models.contains("gpt-4o"));
}
