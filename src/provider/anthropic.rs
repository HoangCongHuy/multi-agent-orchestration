use super::Provider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

/// Gọi Anthropic Messages API.
pub struct AnthropicProvider {
    model: String,
    max_tokens: u32,
    api_key: String,
    client: reqwest::Client,
}

impl AnthropicProvider {
    pub fn new(model: String, max_tokens: u32, api_key_env: &str) -> Result<Self> {
        let api_key = std::env::var(api_key_env).with_context(|| {
            format!("Thiếu biến môi trường {api_key_env} cho provider anthropic")
        })?;
        Ok(Self {
            model,
            max_tokens,
            api_key,
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for AnthropicProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "system": system,
            "messages": [{ "role": "user", "content": user }],
        });

        let resp = self
            .client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .header("content-type", "application/json")
            .json(&body)
            .send()
            .await
            .context("Gọi Anthropic API thất bại")?;

        let status = resp.status();
        let val: serde_json::Value = resp.json().await.context("Response Anthropic không phải JSON")?;
        if !status.is_success() {
            anyhow::bail!("Anthropic API lỗi {status}: {val}");
        }

        // content là mảng các block; gộp text của các block type=text.
        let text = val["content"]
            .as_array()
            .map(|blocks| {
                blocks
                    .iter()
                    .filter_map(|b| b["text"].as_str())
                    .collect::<Vec<_>>()
                    .join("")
            })
            .unwrap_or_default();

        if text.is_empty() {
            anyhow::bail!("Anthropic trả về rỗng: {val}");
        }
        Ok(text)
    }

    fn label(&self) -> String {
        format!("anthropic:{}", self.model)
    }
}
