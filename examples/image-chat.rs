use transformrs::chat;
use transformrs::Message;
use transformrs::Provider;

#[tokio::main]
async fn main() {
    let messages = vec![
        Message::from_str("user", "Describe this image in one short sentence."),
        // To pass a local image, use `Message::from_image_bytes`, for example:
        // Message::from_image_bytes("user", "jpeg", include_bytes!("sunset.jpg")),
        Message::from_image_url("user", "https://transformrs.org/sunset.jpg"),
    ];
    let keys = transformrs::load_keys(".env");
    let provider = Provider::DeepInfra;
    let key = keys.for_provider(&provider).unwrap();
    let model = "meta-llama/Llama-3.2-11B-Vision-Instruct";
    let resp = chat::chat_completion(&provider, &key, model, &messages)
        .await
        .unwrap()
        .structured()
        .unwrap();
    println!("{}", resp.choices[0].message.content);
}

// output:
// The image depicts a red and orange sunset over soft water.
