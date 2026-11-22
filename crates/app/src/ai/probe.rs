//! What a local Ollama model can do (#1945), read from its `/api/show`
//! answer: native tool calls, image input, and the chat fallback mode
//! that follows (native tools, else prompt JSON for a completion model,
//! else read-only Q&A).
//!
//! Provenance: Ollama's public API documentation (`POST /api/show`, the
//! `capabilities` list); older servers without that list are read from
//! the model template's `.Tools` variable, as that documentation shows.

use serde_json::{Value, json};

use crate::mcp::provider::FallbackMode;

/// A local model's capabilities.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Capabilities {
    pub tools: bool,
    pub vision: bool,
    pub mode: FallbackMode,
}

/// The `/api/show` request body for `model`.
#[must_use]
pub fn show_request(model: &str) -> Value {
    json!({ "model": model })
}

/// Read an `/api/show` answer.
pub fn parse_show(answer: &Value) -> Result<Capabilities, String> {
    if !answer.is_object() {
        return Err("Ollama /api/show did not answer an object".into());
    }
    if let Some(e) = answer.get("error").and_then(Value::as_str) {
        return Err(format!("Ollama /api/show: {e}"));
    }
    let (tools, vision, completion) = match answer.get("capabilities").and_then(Value::as_array) {
        Some(caps) => {
            let has = |c: &str| caps.iter().any(|v| v.as_str() == Some(c));
            (has("tools"), has("vision"), has("completion"))
        }
        None => {
            let template = answer.get("template").and_then(Value::as_str).unwrap_or_default();
            (template.contains(".Tools"), false, true)
        }
    };
    Ok(Capabilities { tools, vision, mode: FallbackMode::for_capabilities(tools, completion) })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn capabilities_pick_the_fallback() {
        let c = parse_show(&json!({"capabilities":["completion","tools","vision"]})).expect("ok");
        assert_eq!((c.tools, c.vision, c.mode), (true, true, FallbackMode::Native));
        let c = parse_show(&json!({"capabilities":["completion"]})).expect("ok");
        assert_eq!(c.mode, FallbackMode::PromptJson);
        let c = parse_show(&json!({"capabilities":["embedding"]})).expect("ok");
        assert_eq!(c.mode, FallbackMode::ReadOnly);
        let c = parse_show(&json!({"template":"{{ if .Tools }}x{{ end }}"})).expect("ok");
        assert!(c.tools);
        assert!(parse_show(&json!({"error":"model 'x' not found"})).is_err());
    }
}
