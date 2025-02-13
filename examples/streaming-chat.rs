use futures_util::stream::StreamExt;
use std::io::Write;
use transformrs::chat;
use transformrs::Message;
use transformrs::Provider;

#[tokio::main]
async fn main() {
    let messages = vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str(
            "user",
            "Give a one paragraph summary of the history of the internet.",
        ),
    ];
    let keys = transformrs::load_keys(".env");
    let provider = Provider::DeepInfra;
    let key = keys.for_provider(&provider).unwrap();
    let model = "meta-llama/Llama-3.3-70B-Instruct";
    let mut stream = chat::stream_chat_completion(&provider, &key, model, &messages)
        .await
        .unwrap();
    while let Some(resp) = stream.next().await {
        print!(
            "{}",
            resp.choices[0].delta.content.clone().unwrap_or_default()
        );
        // Ensure the output is printed immediately.
        std::io::stdout().flush().unwrap();
    }
}

// output:
// hello
//  world
