use std::fs::File;
use std::io::Read;

enum Api {
    OpenAI,
}

struct Key {
    api: Api,
    key: String,
}

fn read_key() -> Key {
    let mut env_content = String::new();
    if let Ok(mut file) = File::open(".env") {
        file.read_to_string(&mut env_content)
            .expect("Failed to read .env file");
    } else {
        panic!("Error: .env file not found");
    }

    let key = env_content
        .lines()
        .find(|line| line.starts_with("API_KEY="))
        .and_then(|line| line.split('=').nth(1))
        .map(|key| key.trim().to_string())
        .unwrap_or_else(|| {
            println!("Error: API_KEY not found in .env file");
            String::new()
        });

    Key {
        api: Api::OpenAI,
        key,
    }
}

fn main() {
    println!("Hello, world!");
    let key = read_key();
}
