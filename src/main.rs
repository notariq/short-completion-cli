use reqwest::Client;
use std::fs;
use serde_json::Value;
use clap::Parser;

#[derive(Parser, Debug)]
#[command(version, about = "Chat Completion CLI", long_about = None)]
struct Args {
    /// The prompt to send to the AI model
    #[arg(short, long)]
    prompt: String,

    /// Show usage information
    #[arg(short, long, default_value_t = false)]
    usage: bool,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Args = Args::parse();
    println!("Q: {}", args.prompt);

    let request_body = build_request_body(&args.prompt);

    // Fetch API token from environment variables
    // let api_token = env::var("GROQ_API_TOKEN").expect("API token not found in environment variables");
    let api_token: String = parse_api_key("config.json")?;

    // Send the request and handle the response
    match send_request(request_body, &api_token).await {
        Ok(response_json) => {
            // Extract and print the "content"
            if let Some(content) = response_json
                .get("choices")
                .and_then(|choices| choices.get(0))
                .and_then(|choice| choice.get("message"))
                .and_then(|message| message.get("content"))
            {
                println!("A: {}", content);
            }

            // Print "usage" if the flag is set
            if args.usage {
                if let Some(usage) = response_json.get("usage") {
                    println!("Usage: {:#?}", usage);
                }
            }
        }
        Err(e) => eprintln!("Error: {}", e),
    }

    Ok(())
}

/// Builds the JSON body for the API request
fn build_request_body(prompt: &str) -> Value {
    serde_json::json!({
        "model": "llama-3.1-70b-versatile",
        "messages": [
            {
                "role": "user",
                "content": prompt
            },
            {
                "role": "system",
                "content": "Answer concisely, focusing on insight, accuracy, and relevance. Explain in one to three sentences only, in other words only short answer while ensuring the response remains clear and compelling."
            }
        ]
    })
}

/// Sends a request to the AI model API
async fn send_request(body: Value, api_token: &str) -> Result<Value, Box<dyn std::error::Error>> {
    let client = Client::new();

    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Content-Type", "application/json")
        .header("Authorization", format!("Bearer {}", api_token))
        .json(&body)
        .send()
        .await?;

    // Ensure the response status is OK
    if response.status().is_success() {
        // Parse the response body as JSON
        let json: Value = response.json().await?;
        Ok(json)
    } else {
        Err(format!(
            "Request failed with status: {}",
            response.status()
        )
        .into())
    }
}

/// Reads and parses the API key from a JSON file
fn parse_api_key(file_path: &str) -> Result<String, Box<dyn std::error::Error>> {
    // Read the JSON file
    let config_content = fs::read_to_string(file_path)?;
    // Parse the JSON file
    let config_json: Value = serde_json::from_str(&config_content)?;
    // Extract the API key
    if let Some(api_key) = config_json.get("GROQ_API_KEY").and_then(|v| v.as_str()) {
        Ok(api_key.to_string())
    } else {
        Err("API key not found in config.json".into())
    }
}