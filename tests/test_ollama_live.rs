/// Live integration test with Anthropic Claude Haiku
/// This test is ignored by default and only runs when ANTHROPIC_API_KEY is set
/// Run with: cargo test test_ollama_live -- --ignored --test-threads=1

use std::time::Duration;

#[test]
#[ignore]
fn test_anthropic_api_key_present() {
    let api_key = std::env::var("ANTHROPIC_API_KEY");
    if api_key.is_err() {
        panic!("ANTHROPIC_API_KEY environment variable not set");
    }
    println!("\u2713 ANTHROPIC_API_KEY is set");
}

#[test]
#[ignore]
fn test_anthropic_haiku_generate_simple() {
    let api_key = std::env::var("ANTHROPIC_API_KEY")
        .expect("ANTHROPIC_API_KEY environment variable not set");
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let request = serde_json::json!({
        "model": "claude-3-haiku-20240307",
        "max_tokens": 32,
        "messages": [
            {"role": "user", "content": "Say 'test' and nothing else."}
        ]
    });

    let response = client
        .post("https://api.anthropic.com/v1/messages")
        .header("x-api-key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&request)
        .send();

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success(), "Anthropic API should return 200 OK");
            let json: serde_json::Value = resp.json().unwrap();
            let content = json["content"][0]["text"].as_str().unwrap_or("");
            println!("\u2713 Generated response: {}", content);
            assert_eq!(content.trim(), "test");
        }
        Err(e) => {
            panic!("Failed to generate with Anthropic Claude Haiku: {}", e);
        }
    }
}
