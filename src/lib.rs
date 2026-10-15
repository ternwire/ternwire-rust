use regex::Regex;
use reqwest::{header::HeaderMap, Method, Url};
use serde_json::Value;
use std::net::IpAddr;
use std::time::Duration;

mod webhooks;
pub use webhooks::verify_webhook;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("{0}")]
    InvalidInput(String),
    #[error("Ternwire API returned HTTP {status}")]
    Api {
        status: u16,
        raw_body: String,
        body: Option<Value>,
        headers: HeaderMap,
    },
    #[error("Ternwire transport failed: {0}")]
    Transport(#[from] reqwest::Error),
    #[error("Invalid Ternwire JSON response: {0}")]
    Json(#[from] serde_json::Error),
    #[error("Ternwire API returned a non-object JSON response")]
    InvalidResponse,
}

/// The production API origin, used by [`Client::from_api_key`].
pub const DEFAULT_BASE_URL: &str = "https://ternwire.com";

#[derive(Clone)]
pub struct Client {
    http: reqwest::Client,
    api_key: String,
    base_url: String,
}

// Url has already lowercased domain names, canonicalized IPv4 and bracketed IPv6.
fn is_loopback_host(host: &str) -> bool {
    host == "localhost"
        || host.ends_with(".localhost")
        || host
            .trim_start_matches('[')
            .trim_end_matches(']')
            .parse::<IpAddr>()
            .is_ok_and(|ip| ip.is_loopback())
}

impl Client {
    /// Builds a client for the production API at [`DEFAULT_BASE_URL`].
    ///
    /// ```
    /// let client = ternwire::Client::from_api_key("em_test_example")?;
    /// let local = ternwire::Client::new("em_test_example", "http://localhost:3001")?;
    /// # Ok::<(), ternwire::Error>(())
    /// ```
    pub fn from_api_key(api_key: impl Into<String>) -> Result<Self, Error> {
        Self::new(api_key, DEFAULT_BASE_URL)
    }

    /// Builds a client for the API at `base_url`, such as a local or staging origin.
    pub fn new(api_key: impl Into<String>, base_url: &str) -> Result<Self, Error> {
        Self::with_timeout(api_key, base_url, Duration::from_secs(30))
    }

    pub fn with_timeout(api_key: impl Into<String>, base_url: &str, timeout: Duration) -> Result<Self, Error> {
        let api_key = api_key.into();
        if !Regex::new(r"\Aem_(live|test)_[A-Za-z0-9_-]+\z").unwrap().is_match(&api_key) {
            return Err(Error::InvalidInput("Use an em_live_ or em_test_ API key".into()));
        }
        let origin = Url::parse(base_url).map_err(|_| Error::InvalidInput("Invalid base_url".into()))?;
        if !matches!(origin.scheme(), "http" | "https") || origin.host_str().is_none()
            || !origin.username().is_empty() || origin.password().is_some()
            || origin.path() != "/" || origin.query().is_some() || origin.fragment().is_some() {
            return Err(Error::InvalidInput("base_url must be an HTTP(S) origin without /v1".into()));
        }
        let loopback = origin.host_str().is_some_and(is_loopback_host);
        if origin.scheme() == "http" && !loopback {
            return Err(Error::InvalidInput(
                "base_url must use https unless it points at localhost or a loopback address".into(),
            ));
        }
        if timeout.is_zero() {
            return Err(Error::InvalidInput("timeout must be positive".into()));
        }
        let mut http = reqwest::Client::builder()
            .timeout(timeout)
            .redirect(reqwest::redirect::Policy::none())
            .retry(reqwest::retry::never());
        if loopback {
            // A proxy cannot reach this machine's loopback, and would receive the API key.
            http = http.no_proxy();
        }
        let http = http.build()?;
        Ok(Self { http, api_key, base_url: base_url.trim_end_matches('/').into() })
    }

    fn id(value: &str) -> Result<&str, Error> {
        if !Regex::new(r"\A[a-z0-9][a-z0-9_-]{7,127}\z").unwrap().is_match(value) {
            return Err(Error::InvalidInput("Invalid Ternwire resource ID".into()));
        }
        Ok(value)
    }

    fn query(pairs: &[(&str, Option<String>)]) -> String {
        let mut query = String::new();
        for (name, value) in pairs {
            let Some(value) = value else { continue };
            query.push(if query.is_empty() { '?' } else { '&' });
            query.push_str(name);
            query.push('=');
            for byte in value.bytes() {
                if byte.is_ascii_alphanumeric() || b"-._~".contains(&byte) {
                    query.push(byte as char);
                } else {
                    query.push_str(&format!("%{byte:02X}"));
                }
            }
        }
        query
    }

    // BEGIN GENERATED OPERATIONS
    pub async fn submit_email(&self, input: &Value, idempotency_key: &str) -> Result<Value, Error> {
        self.request(Method::POST, "/v1/emails", 202, Some(input), Some(idempotency_key)).await
    }

    pub async fn get_message(&self, message_id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/emails/", Self::id(message_id)?].concat(), 200, None, None).await
    }

    pub async fn create_email_template(&self, input: &Value) -> Result<Value, Error> {
        self.request(Method::POST, "/v1/templates", 201, Some(input), None).await
    }

    pub async fn list_email_templates(&self, cursor: Option<&str>, limit: Option<u32>) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/templates", &Self::query(&[("cursor", cursor.map(|value| value.to_string())), ("limit", limit.map(|value| value.to_string()))])].concat(), 200, None, None).await
    }

    pub async fn get_email_template(&self, template_id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/templates/", Self::id(template_id)?].concat(), 200, None, None).await
    }

    pub async fn replace_email_template(&self, template_id: &str, input: &Value) -> Result<Value, Error> {
        self.request(Method::PUT, &["/v1/templates/", Self::id(template_id)?].concat(), 200, Some(input), None).await
    }

    pub async fn delete_email_template(&self, template_id: &str) -> Result<Value, Error> {
        self.request(Method::DELETE, &["/v1/templates/", Self::id(template_id)?].concat(), 204, None, None).await
    }

    pub async fn create_attachment_upload_slot(&self, input: &Value) -> Result<Value, Error> {
        self.request(Method::POST, "/v1/attachments", 201, Some(input), None).await
    }

    pub async fn get_attachment(&self, attachment_id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/attachments/", Self::id(attachment_id)?].concat(), 200, None, None).await
    }

    pub async fn complete_attachment_upload(&self, attachment_id: &str) -> Result<Value, Error> {
        self.request(Method::POST, &["/v1/attachments/", Self::id(attachment_id)?, "/complete"].concat(), 202, None, None).await
    }

    pub async fn create_sending_domain(&self, input: &Value) -> Result<Value, Error> {
        self.request(Method::POST, "/v1/domains", 201, Some(input), None).await
    }

    pub async fn get_sending_domain(&self, domain_id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/domains/", Self::id(domain_id)?].concat(), 200, None, None).await
    }

    pub async fn delete_sending_domain(&self, domain_id: &str) -> Result<Value, Error> {
        self.request(Method::DELETE, &["/v1/domains/", Self::id(domain_id)?].concat(), 202, None, None).await
    }

    pub async fn create_webhook_endpoint(&self, input: &Value) -> Result<Value, Error> {
        self.request(Method::POST, "/v1/webhook-endpoints", 201, Some(input), None).await
    }

    pub async fn rotate_webhook_endpoint_signing_secret(&self, endpoint_id: &str) -> Result<Value, Error> {
        self.request(Method::POST, &["/v1/webhook-endpoints/", Self::id(endpoint_id)?, "/rotate-signing-secret"].concat(), 200, None, None).await
    }

    pub async fn get_webhook_attempt(&self, attempt_id: &str) -> Result<Value, Error> {
        self.request(Method::GET, &["/v1/webhook-attempts/", Self::id(attempt_id)?].concat(), 200, None, None).await
    }

    pub async fn replay_webhook_attempt(&self, attempt_id: &str) -> Result<Value, Error> {
        self.request(Method::POST, &["/v1/webhook-attempts/", Self::id(attempt_id)?, "/replay"].concat(), 202, None, None).await
    }
    // END GENERATED OPERATIONS

    fn build_request(&self, method: Method, path: &str, body: Option<&Value>, idempotency_key: Option<&str>) -> Result<reqwest::Request, Error> {
        let mut request = self.http.request(method, format!("{}{path}", self.base_url))
            .bearer_auth(&self.api_key).header("accept", "application/json");
        if let Some(key) = idempotency_key {
            if key.is_empty() || key.len() > 128 || !key.bytes().all(|b| (33..=126).contains(&b)) {
                return Err(Error::InvalidInput("idempotency_key must contain 1-128 visible ASCII characters".into()));
            }
            request = request.header("idempotency-key", key);
        }
        if let Some(body) = body {
            if !body.is_object() {
                return Err(Error::InvalidInput("Request body must be a JSON object".into()));
            }
            request = request.json(body);
        }
        Ok(request.build()?)
    }

    async fn request(&self, method: Method, path: &str, expected_status: u16, body: Option<&Value>, idempotency_key: Option<&str>) -> Result<Value, Error> {
        let response = self.http.execute(self.build_request(method, path, body, idempotency_key)?).await?;
        let status = response.status().as_u16();
        let headers = response.headers().clone();
        let raw_body = response.text().await?;
        if status != expected_status {
            let body = serde_json::from_str(&raw_body).ok();
            return Err(Error::Api { status, raw_body, body, headers });
        }
        if status == 204 { return Ok(Value::Null); }
        let payload: Value = serde_json::from_str(&raw_body)?;
        if !payload.is_object() { return Err(Error::InvalidResponse); }
        Ok(payload)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{BufRead, BufReader, Write};
    use std::net::TcpListener;
    use std::sync::{Arc, Mutex};

    const API_KEY: &str = "em_test_origincheck";

    fn record_authorizations(listener: TcpListener) -> Arc<Mutex<Vec<String>>> {
        let seen = Arc::new(Mutex::new(Vec::new()));
        let recorder = seen.clone();
        std::thread::spawn(move || {
            let page = r#"{"data":[],"page":{"nextCursor":null,"hasMore":false}}"#;
            for stream in listener.incoming() {
                let stream = stream.unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() { break; }
                    if let Some((name, value)) = line.split_once(": ") {
                        if name.eq_ignore_ascii_case("authorization") { recorder.lock().unwrap().push(value.trim().to_string()); }
                    }
                }
                write!(&stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}", page.len()).unwrap();
            }
        });
        seen
    }

    #[test]
    fn rejects_cleartext_non_loopback_origins() {
        for base_url in [
            "http://api.example.com",
            "http://10.0.0.5:3001",
            "http://api:3001",
            "http://localhost.example.com",
            "http://127.0.0.1.nip.io",
            "http://[::2]",
        ] {
            match Client::new(API_KEY, base_url) {
                Err(Error::InvalidInput(message)) => assert!(message.contains("must use https"), "{base_url}: {message}"),
                other => panic!("{base_url}: expected an https error, got {:?}", other.err()),
            }
        }
    }

    #[test]
    fn accepts_https_and_loopback_origins() {
        for base_url in [
            "https://api.example.com",
            "http://localhost:3001",
            "http://LOCALHOST:3001",
            "http://api.localhost:3001",
            "http://127.10.20.30",
            "http://[::1]:3001",
        ] {
            assert!(Client::new(API_KEY, base_url).is_ok(), "{base_url}");
        }
    }

    #[test]
    fn default_origin_unless_one_is_passed() {
        let target = |client: Client| client.build_request(Method::GET, "/v1/templates", None, None).unwrap().url().to_string();
        assert_eq!(target(Client::from_api_key(API_KEY).unwrap()), "https://ternwire.com/v1/templates");
        assert_eq!(target(Client::new(API_KEY, "http://localhost:3001").unwrap()), "http://localhost:3001/v1/templates");
        assert!(matches!(Client::new(API_KEY, ""), Err(Error::InvalidInput(_))));
    }

    #[tokio::test]
    async fn loopback_client_sends_the_key() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let (stream, _) = listener.accept().unwrap();
            let mut reader = BufReader::new(stream.try_clone().unwrap());
            let mut authorization = None;
            loop {
                let mut line = String::new();
                reader.read_line(&mut line).unwrap();
                if line == "\r\n" || line.is_empty() { break; }
                if let Some((name, value)) = line.split_once(": ") {
                    if name.eq_ignore_ascii_case("authorization") { authorization = Some(value.trim().to_string()); }
                }
            }
            (&stream).write_all(b"HTTP/1.1 204 No Content\r\nConnection: close\r\n\r\n").unwrap();
            authorization
        });
        let client = Client::new(API_KEY, &base_url).unwrap();
        client.delete_email_template("tpl_00000001").await.unwrap();
        assert_eq!(server.join().unwrap(), Some(format!("Bearer {API_KEY}")));
    }

    #[tokio::test]
    async fn list_email_templates_sends_cursor_and_limit() {
        let listener = TcpListener::bind("127.0.0.1:0").unwrap();
        let base_url = format!("http://{}", listener.local_addr().unwrap());
        let server = std::thread::spawn(move || {
            let page = r#"{"data":[],"page":{"nextCursor":null,"hasMore":false}}"#;
            let mut targets = Vec::new();
            for _ in 0..2 {
                let (stream, _) = listener.accept().unwrap();
                let mut reader = BufReader::new(stream.try_clone().unwrap());
                let mut request_line = String::new();
                reader.read_line(&mut request_line).unwrap();
                targets.push(request_line.split(' ').nth(1).unwrap().to_string());
                loop {
                    let mut line = String::new();
                    reader.read_line(&mut line).unwrap();
                    if line == "\r\n" || line.is_empty() { break; }
                }
                write!(&stream, "HTTP/1.1 200 OK\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{page}", page.len()).unwrap();
            }
            targets
        });
        let client = Client::new(API_KEY, &base_url).unwrap();
        let result = client.list_email_templates(Some("c+/= x"), Some(10)).await.unwrap();
        client.list_email_templates(None, None).await.unwrap();
        assert_eq!(result["page"]["hasMore"], Value::Bool(false));
        assert_eq!(server.join().unwrap(), ["/v1/templates?cursor=c%2B%2F%3D%20x&limit=10", "/v1/templates"]);
    }

    #[tokio::test]
    async fn loopback_client_ignores_environment_proxies() {
        // Proxy variables are process-wide, so set them only in a child running this test alone.
        if std::env::var_os("TERNWIRE_PROXY_CHILD").is_none() {
            let output = std::process::Command::new(std::env::current_exe().unwrap())
                .args(["--exact", "tests::loopback_client_ignores_environment_proxies"])
                .env("TERNWIRE_PROXY_CHILD", "1")
                .output()
                .unwrap();
            assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stdout));
            return;
        }
        let api = TcpListener::bind("127.0.0.1:0").unwrap();
        let port = api.local_addr().unwrap().port();
        let proxy = TcpListener::bind("127.0.0.1:0").unwrap();
        let proxy_url = format!("http://{}", proxy.local_addr().unwrap());
        for name in ["http_proxy", "HTTP_PROXY", "all_proxy", "ALL_PROXY"] { std::env::set_var(name, &proxy_url); }
        for name in ["no_proxy", "NO_PROXY"] { std::env::remove_var(name); }
        let (api_seen, proxy_seen) = (record_authorizations(api), record_authorizations(proxy));
        for host in ["127.0.0.1", "localhost", "api.localhost"] {
            let client = Client::new(API_KEY, &format!("http://{host}:{port}")).unwrap();
            client.list_email_templates(None, None).await.unwrap();
        }
        assert_eq!(*proxy_seen.lock().unwrap(), Vec::<String>::new());
        assert_eq!(*api_seen.lock().unwrap(), vec![format!("Bearer {API_KEY}"); 3]);
    }
}
