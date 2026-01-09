/// Live integration test with Ollama
/// This test is ignored by default and only runs when Ollama is available
/// Run with: cargo test test_ollama_live -- --ignored --test-threads=1

use std::time::Duration;

#[test]
#[ignore]
fn test_ollama_connection() {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(2))
        .build()
        .unwrap();

    let response = client
        .get("http://localhost:11434/api/version")
        .send();

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success(), "Ollama should return 200 OK");
            println!("✓ Ollama is running and accessible");
        }
        Err(e) => {
            panic!("Ollama is not running or not accessible: {}", e);
        }
    }
}

#[test]
#[ignore]
fn test_ollama_generate_simple() {
    let client = reqwest::blocking::Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap();

    let request = serde_json::json!({
        "model": "qwen2.5-coder:14b",
        "prompt": "Say 'test' and nothing else",
        "stream": false,
        "options": {
            "temperature": 0.0
        }
    });

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send();

    match response {
        Ok(resp) => {
            assert!(resp.status().is_success());
            let json: serde_json::Value = resp.json().unwrap();
            let generated_text = json["response"].as_str().unwrap();
            println!("✓ Generated response: {}", generated_text);
            assert!(!generated_text.is_empty());
        }
        Err(e) => {
            panic!("Failed to generate with Ollama: {}", e);
        }
    }
}
