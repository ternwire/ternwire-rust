use hmac::{Hmac, Mac};
use regex::Regex;
use sha2::Sha256;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Verify the signature against the original request bytes before parsing JSON.
pub fn verify_webhook(header: &str, body: &[u8], secret: &str, tolerance: Duration) -> bool {
    if secret.is_empty() { return false; }
    let pattern = Regex::new(r"\A\s*t=([1-9][0-9]{0,11})\s*,\s*v1=([a-f0-9]{64})\s*\z").unwrap();
    let Some(parts) = pattern.captures(header) else { return false; };
    let Ok(timestamp) = parts[1].parse::<u64>() else { return false; };
    let Ok(now) = SystemTime::now().duration_since(UNIX_EPOCH) else { return false; };
    if now.as_secs().abs_diff(timestamp) > tolerance.as_secs() { return false; }
    let Ok(signature) = hex::decode(&parts[2]) else { return false; };
    let Ok(mut mac) = Hmac::<Sha256>::new_from_slice(secret.as_bytes()) else { return false; };
    mac.update(parts[1].as_bytes());
    mac.update(b".");
    mac.update(body);
    mac.verify_slice(&signature).is_ok()
}
