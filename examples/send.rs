use serde_json::json;
use std::env;
use ternwire::Client;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new(env::var("TERNWIRE_API_KEY")?, &env::var("TERNWIRE_API_ORIGIN")?)?;
    let accepted = client.submit_email(&json!({
        "from": env::var("TERNWIRE_FROM")?,
        "to": [env::var("TERNWIRE_TO")?],
        "subject": "Your receipt",
        "text": "Thanks for your order."
    }), "order-123-receipt").await?;
    println!("{accepted}");
    Ok(())
}
