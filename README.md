# Ternwire Rust SDK

Async Rust client for the Ternwire transactional email API. Uses reqwest with
rustls, serde_json, and a Tokio-compatible runtime.

## Install

In your project's `Cargo.toml`:

```toml
[dependencies]
ternwire = "0.1"
serde_json = "1"
tokio = { version = "1", features = ["macros", "rt-multi-thread"] }
```

## Usage

```rust
use serde_json::json;
use ternwire::Client;

let client = Client::new(api_key, api_origin)?;
let message = client.submit_email(&json!({
    "from": "notifications@your-verified-domain.com",
    "to": ["recipient@example.com"],
    "subject": "Your receipt",
    "text": "Thanks for your order."
}), "order-123-receipt").await?;
```

Use inside an async function returning a compatible `Result`. The complete
`examples/send.rs` supplies the runtime and environment configuration.

All 17 operations use snake_case and return `Result<serde_json::Value, ternwire::Error>`.
Bodies must be JSON objects with the API's camelCase fields. Template deletion
returns `Value::Null`. The client can be cloned and reused. Use
`Client::with_timeout(api_key, api_origin, duration)` to override the 30-second timeout.
`list_email_templates(cursor: Option<&str>, limit: Option<u32>)` returns one page of
summaries; pass the previous `page.nextCursor` as `cursor` to fetch the next page.

Match `Error::Api { status, body, raw_body, headers }` for HTTP failures.
Transport, JSON, input, and response-shape failures use separate variants.

For webhook verification:

```rust
let valid = ternwire::verify_webhook(
    signature_header, raw_body_bytes, signing_secret,
    std::time::Duration::from_secs(300),
);
```

To run the example later, set `TERNWIRE_API_KEY`, `TERNWIRE_API_ORIGIN`,
`TERNWIRE_FROM`, and `TERNWIRE_TO`, then run `cargo run --example send`.
It submits an email.

Use `https://ternwire.com` as the API origin. Read the full documentation at
[ternwire.com/docs](https://ternwire.com/docs), or contact support@ternwire.com.
Released under the MIT License.
