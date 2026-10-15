use serde_json::json;
use std::env;
use ternwire::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let api_key = env::var("TERNWIRE_API_KEY")?;
    let client = match env::var("TERNWIRE_API_ORIGIN") {
        Ok(origin) => Client::new(api_key, &origin)?,
        Err(_) => Client::from_api_key(api_key)?,
    };
    let accepted = client.submit_email(&json!({
        "from": env::var("TERNWIRE_FROM")?,
        "to": [env::var("TERNWIRE_TO")?],
        "subject": "Your receipt",
        "text": "Thanks for your order."
    }), "order-123-receipt").await?;
    println!("{accepted}");
    Ok(())
}
