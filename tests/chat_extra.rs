extern crate transformrs;

use futures_util::stream::StreamExt;
use transformrs::chat;
use transformrs::Message;
use transformrs::Provider;

const MODEL: &str = "meta-llama/Llama-3.3-70B-Instruct-Turbo";

#[tokio::test]
async fn test_chat_completion_stream_duration() {
    // Verify that the stream is passing messages through once they are available.
    let messages = vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str("user", "Tell a joke about a car."),
    ];
    let provider = Provider::DeepInfra;
    let keys = transformrs::load_keys(".env");
    let key = keys.for_provider(&provider).unwrap();
    let mut stream = chat::stream_chat_completion(&provider, &key, &MODEL, &messages)
        .await
        .unwrap();
    let mut content = String::new();
    let mut timestamps = Vec::new();
    while let Some(resp) = stream.next().await {
        let timestamp = std::time::SystemTime::now();
        let chunk = resp.choices[0].delta.content.clone().unwrap_or_default();
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
