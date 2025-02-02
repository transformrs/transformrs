extern crate aiapi;

use aiapi::openai;
use aiapi::Message;
use aiapi::Provider;
use futures_util::stream::StreamExt;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct-Turbo";

#[tokio::test]
async fn test_chat_completion_stream_duration() {
    let messages = vec![
        Message {
            role: "system".to_string(),
            content: "You are a helpful assistant.".to_string(),
        },
        Message {
            role: "user".to_string(),
            content: "Tell a joke about a car.".to_string(),
        },
    ];
    let keys = aiapi::read_keys();
    let key = keys.for_provider(&Provider::DeepInfra).unwrap();
    let resp = openai::chat_completion(&key, &MODEL, true, &messages).await;
    let resp = resp.unwrap();
    let mut stream = openai::chat_completion_stream(resp).await.unwrap();
    let mut content = String::new();
    let mut timestamps = Vec::new();
    while let Some(json) = stream.next().await {
        let timestamp = std::time::SystemTime::now();
        let chunk = openai::chat_completion_stream_content(json.unwrap())
            .await
            .unwrap();
        content += &chunk;
        println!("{}", chunk);
        timestamps.push(timestamp);
    }
    let first_timestamp = timestamps.first().unwrap();
    let last_timestamp = timestamps.last().unwrap();
    // The car joke is probably one or two sentences, which should take at least some ms.
    let expected_duration = std::time::Duration::from_millis(100);
    let total_duration = last_timestamp.duration_since(*first_timestamp).unwrap();
    assert!(
        total_duration >= expected_duration,
        "Streaming response took {} ms, expected at least {} ms",
        total_duration.as_millis(),
        expected_duration.as_millis()
    );
}
