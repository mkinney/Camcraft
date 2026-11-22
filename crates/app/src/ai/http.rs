//! The native HTTP client for AI providers (#1945). Every request passes
//! the endpoint laws (`endpoint::admit`) before anything resolves, then
//! the resolver re-checks the addresses: offline-only, or plain `http`,
//! reaches loopback addresses alone. No redirect is followed (a local
//! server could otherwise bounce a call off the machine), no proxy is
//! read from the environment, and no error message carries the key.
//!
//! Provenance: our own policy over the `reqwest` crate's public API; the
//! auth headers are each provider's documented ones (`x-api-key`,
//! `Authorization: Bearer`, `x-goog-api-key`).

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use serde_json::Value;

use crate::mcp::provider::Provider;

use super::endpoint::{self, Endpoint, Scheme};
use super::probe::{self, Capabilities};
use super::secret::ApiKey;
use super::wire::{self, ChatRequest};

/// How long one call may take: connect, then the whole answer.
#[derive(Clone, Copy, Debug)]
pub struct Timeouts {
    pub connect: Duration,
    pub total: Duration,
}

impl Default for Timeouts {
    fn default() -> Self {
        Self {
            connect: Duration::from_secs(10),
            total: Duration::from_secs(300),
        }
    }
}

/// The client: the offline switch and the timeouts. A connection is built
/// per request, so each carries its own loopback rule.
#[derive(Clone, Copy, Debug)]
pub struct Client {
    pub offline_only: bool,
    pub timeouts: Timeouts,
}

/// The longest provider error text kept in a message.
const ERROR_TEXT: usize = 400;

impl Client {
    #[must_use]
    pub fn new(offline_only: bool) -> Self {
        Self {
            offline_only,
            timeouts: Timeouts::default(),
        }
    }

    /// POST `body` as JSON to `path` under `ep`, with `auth` as the auth
    /// header when given, and read a JSON answer. A non-2xx status is an
    /// error naming the status and the provider's message.
    pub fn post_json(
        &self,
        ep: &Endpoint,
        path: &str,
        headers: &[(String, String)],
        auth: Option<(&'static str, String)>,
        body: &Value,
    ) -> Result<Value, String> {
        self.post_json_cancellable(ep, path, headers, auth, body, &AtomicBool::new(false))
    }

    /// As `post_json`, but drops the active connection when cancelled.
    pub fn post_json_cancellable(
        &self,
        ep: &Endpoint,
        path: &str,
        headers: &[(String, String)],
        auth: Option<(&'static str, String)>,
        body: &Value,
        cancel: &AtomicBool,
    ) -> Result<Value, String> {
        endpoint::admit(ep, self.offline_only)?;
        if cancel.load(Ordering::Relaxed) {
            return Err("cancelled".into());
        }
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(|e| format!("HTTP runtime unavailable: {e}"))?;
        runtime.block_on(async {
            let request = self.post_async(ep, path, headers, auth.as_ref(), body);
            tokio::select! {
                result = request => result,
                () = async {
                    while !cancel.load(Ordering::Relaxed) {
                        tokio::time::sleep(Duration::from_millis(20)).await;
                    }
                } => Err("cancelled".into()),
            }
        })
    }

    async fn post_async(
        &self,
        ep: &Endpoint,
        path: &str,
        headers: &[(String, String)],
        auth: Option<&(&'static str, String)>,
        body: &Value,
    ) -> Result<Value, String> {
        let client = reqwest::Client::builder()
            .redirect(reqwest::redirect::Policy::none())
            .no_proxy()
            .connect_timeout(self.timeouts.connect)
            .timeout(self.timeouts.total)
            .dns_resolver(std::sync::Arc::new(CheckedResolver {
                loopback_only: self.offline_only || ep.scheme == Scheme::Http,
            }))
            .user_agent(concat!("camcraft/", env!("CARGO_PKG_VERSION")))
            .build()
            .map_err(|_| "HTTP client unavailable".to_owned())?;
        let mut req = client
            .post(ep.url(path))
            .header("content-type", "application/json");
        for (name, value) in headers {
            req = req.header(name, value);
        }
        if let Some((name, value)) = auth {
            req = req.header(*name, value);
        }
        // Do not expose transport diagnostics: they can include header values.
        let resp = req
            .body(body.to_string())
            .send()
            .await
            .map_err(|_| format!("{} could not be reached", ep.host))?;
        let status = resp.status();
        let text = resp
            .text()
            .await
            .map_err(|_| format!("{} answer not read", ep.host))?;
        if !status.is_success() {
            let secret = auth.map(|(name, value)| {
                if name.eq_ignore_ascii_case("authorization") {
                    value.strip_prefix("Bearer ").unwrap_or(value.as_str())
                } else {
                    value.as_str()
                }
            });
            return Err(format!(
                "{} answered HTTP {}: {}",
                ep.host,
                status.as_u16(),
                provider_message(&text, secret)
            ));
        }
        serde_json::from_str(&text).map_err(|e| format!("{} answer is not JSON ({e})", ep.host))
    }
}

/// Resolve inside the cancellable future and enforce the address policy before connecting.
struct CheckedResolver {
    loopback_only: bool,
}

impl reqwest::dns::Resolve for CheckedResolver {
    fn resolve(&self, name: reqwest::dns::Name) -> reqwest::dns::Resolving {
        let loopback_only = self.loopback_only;
        Box::pin(async move {
            let addrs = tokio::net::lookup_host((name.as_str(), 0)).await?.collect();
            let checked =
                endpoint::admit_addrs(addrs, loopback_only).map_err(std::io::Error::other)?;
            Ok(Box::new(checked.into_iter()) as reqwest::dns::Addrs)
        })
    }
}

/// Redact before truncating, including keys that cross the text limit.
fn provider_message(text: &str, secret: Option<&str>) -> String {
    let parsed: Option<Value> = serde_json::from_str(text).ok();
    let msg = parsed
        .as_ref()
        .and_then(|v| {
            v["error"]["message"]
                .as_str()
                .or_else(|| v["error"].as_str())
                .map(str::to_owned)
        })
        .unwrap_or_else(|| text.to_owned());
    let redacted = match secret.filter(|key| !key.is_empty()) {
        Some(key) => msg.replace(key, "[redacted]"),
        None => msg,
    };
    redacted.chars().take(ERROR_TEXT).collect()
}

/// The auth header for `provider`, per its documentation.
#[must_use]
pub fn auth_header(provider: Provider, key: &ApiKey) -> (&'static str, String) {
    match provider {
        Provider::Anthropic => ("x-api-key", key.expose().to_owned()),
        Provider::OpenAiChat | Provider::OpenAiResponses => {
            ("authorization", format!("Bearer {}", key.expose()))
        }
        Provider::Gemini => ("x-goog-api-key", key.expose().to_owned()),
    }
}

/// Send one chat step and return the provider's response envelope.
pub fn send_chat(
    client: &Client,
    ep: &Endpoint,
    req: &ChatRequest<'_>,
    key: Option<&ApiKey>,
    cancel: &AtomicBool,
) -> Result<Value, String> {
    if cancel.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }
    let w = wire::build(req)?;
    let auth = key.map(|k| auth_header(req.provider, k));
    let out = client.post_json_cancellable(ep, &w.path, &w.headers, auth, &w.body, cancel)?;
    if cancel.load(Ordering::Relaxed) {
        return Err("cancelled".into());
    }
    Ok(out)
}

/// Ask an Ollama server what `model` can do. `ep` is the chat base
/// (`.../v1`); the probe goes to `/api/show` on the same host and port.
pub fn probe_ollama(client: &Client, ep: &Endpoint, model: &str) -> Result<Capabilities, String> {
    probe_ollama_cancellable(client, ep, model, &AtomicBool::new(false))
}

/// Probe with the same cancellation policy as chat requests.
pub fn probe_ollama_cancellable(
    client: &Client,
    ep: &Endpoint,
    model: &str,
    cancel: &AtomicBool,
) -> Result<Capabilities, String> {
    wire::check_model(model)?;
    let root = ep.with_base_path("");
    let answer = client.post_json_cancellable(
        &root,
        "/api/show",
        &[],
        None,
        &probe::show_request(model),
        cancel,
    )?;
    probe::parse_show(&answer)
}
