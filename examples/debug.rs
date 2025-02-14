//! An example with debug logging enabled.

// You can execute this example with `cargo run --example debug`

use tracing::subscriber::SetGlobalDefaultError;
use transformrs::chat;
use transformrs::Message;
use transformrs::Provider;

/// Initialize logging with the given level.
pub fn init_subscriber(level: tracing::Level) -> Result<(), SetGlobalDefaultError> {
    let subscriber = tracing_subscriber::FmtSubscriber::builder()
        .with_max_level(level)
        .with_writer(std::io::stderr)
        .without_time()
        .with_target(false)
        .finish();
    tracing::subscriber::set_global_default(subscriber)
}

#[tokio::main]
async fn main() {
    init_subscriber(tracing::Level::DEBUG).unwrap();
    let messages = vec![
        Message::from_str("system", "You are a helpful assistant."),
        Message::from_str("user", "This is a test. Please respond with 'hello world'."),
    ];
    let keys = transformrs::load_keys(".env");
    let provider = Provider::DeepInfra;
    let key = keys.for_provider(&provider).unwrap();
    let model = "meta-llama/Llama-3.3-70B-Instruct";
    let resp = chat::chat_completion(&provider, &key, model, &messages)
        .await
        .unwrap()
        .structured()
        .unwrap();
    println!("{}", resp.choices[0].message.content);
}

// output:
// DEBUG Requesting chat: {"messages":[{"content":"You are a helpful assistant.","role":"system"},{"content":"This is a test. Please respond with 'hello world'.","role":"user"}],"model":"meta-llama/Llama-3.3-70B-Instruct","stream":false}
// DEBUG connecting to 38.101.151.19:443
// DEBUG connected to 38.101.151.19:443
// DEBUG pooling idle connection for ("https", api.deepinfra.com)
// hello world
