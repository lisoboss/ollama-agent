use std::env;
use std::time::Instant;
use anyhow::{Context, Result};
use reqwest::header::{HeaderMap, HeaderValue, CONTENT_TYPE};

#[tokio::main]
async fn main() -> Result<()> {
    // Configure this to point to your local proxy
    let proxy_url = env::var("PROXY_URL").unwrap_or_else(|_| "http://127.0.0.1:11434".to_string());
    
    // Check if API key is provided (optional)
    let api_key = env::var("API_KEY").ok();
    
    println!("Ollama API Client Example");
    println!("Using proxy URL: {}", proxy_url);
    println!("API key: {}", if api_key.is_some() { "provided" } else { "not provided" });
    
    // Create HTTP client
    let client = reqwest::Client::new();
    
    // Test connection to models endpoint
    println!("\n1. Testing connection to models endpoint...");
    let start = Instant::now();
    
    // Create request builder
    let mut request = client.get(format!("{}/api/tags", proxy_url));
    
    // Add API key header if provided
    if let Some(key) = &api_key {
        request = request.header("Authorization", format!("Bearer {}", key));
    }
    
    // Send request
    let resp = request
        .send()
        .await
        .context("Failed to connect to proxy")?;
    
    println!("Status: {}", resp.status());
    if resp.status().is_success() {
        let models = resp.json::<serde_json::Value>().await?;
        println!("Available models: {}", models);
        println!("Request completed in {:?}", start.elapsed());
    } else {
        println!("Error: {}", resp.text().await?);
    }
    
    // Run a simple generation test
    println!("\n2. Testing simple generation...");
    let start = Instant::now();
    
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));
    
    // Add API key if provided
    if let Some(key) = &api_key {
        headers.insert("Authorization", format!("Bearer {}", key).parse().unwrap());
    }
    
    let resp = client.post(format!("{}/api/generate", proxy_url))
        .headers(headers)
        .json(&serde_json::json!({
            "model": "llama2", // Change this to a model you have available
            "prompt": "Write a short poem about rust programming",
            "stream": false
        }))
        .send()
        .await
        .context("Failed to send generation request")?;
    
    println!("Status: {}", resp.status());
    if resp.status().is_success() {
        let response = resp.json::<serde_json::Value>().await?;
        println!("Generated text: {}", response["response"]);
        println!("Request completed in {:?}", start.elapsed());
    } else {
        println!("Error: {}", resp.text().await?);
    }
    
    println!("\nExample completed successfully!");
    Ok(())
}