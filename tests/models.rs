extern crate transformrs;

use std::error::Error;
use transformrs::models::models;
use transformrs::Provider;

async fn test_models(provider: Provider) -> Result<(), Box<dyn Error + Send + Sync>> {
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let resp = models(&key).await;
    let resp = match resp {
        Ok(resp) => resp,
        Err(e) => {
            return Err(e);
        }
    };
    let resp = resp.raw();
    println!("{:?}", resp);
    Ok(())
}

#[tokio::test]
async fn test_models_groq() {
    test_models(Provider::Groq).await.unwrap();
    assert!(false);
}
