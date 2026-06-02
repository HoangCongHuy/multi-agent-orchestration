use super::Provider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use serde_json::json;

/// Gọi OpenAI Chat Completions API (hoặc endpoint tương thích).
pub struct OpenaiProvider {
    model: String,
    max_tokens: u32,
    api_key: String,
    base_url: String,
    client: reqwest::Client,
}

impl OpenaiProvider {
    pub fn new(
        model: String,
        max_tokens: u32,
        api_key_env: &str,
        base_url: String,
    ) -> Result<Self> {
        let api_key = std::env::var(api_key_env)
            .with_context(|| format!("Thiếu biến môi trường {api_key_env} cho provider openai"))?;
        Ok(Self {
            model,
            max_tokens,
            api_key,
            base_url: base_url.trim_end_matches('/').to_string(),
            client: reqwest::Client::new(),
        })
    }
}

#[async_trait]
impl Provider for OpenaiProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let body = json!({
            "model": self.model,
            "max_tokens": self.max_tokens,
            "messages": [
                { "role": "system", "content": system },
                { "role": "user", "content": user },
            ],
        });

        let resp = self
            .client
            .post(format!("{}/chat/completions", self.base_url))
            .bearer_auth(&self.api_key)
            .json(&body)
            .send()
            .await
            .context("Gọi OpenAI API thất bại")?;

        let status = resp.status();
        let val: serde_json::Value = resp.json().await.context("Response OpenAI không phải JSON")?;
        if !status.is_success() {
            anyhow::bail!("OpenAI API lỗi {status}: {val}");
        }

        let text = val["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or_default()
            .to_string();

        if text.is_empty() {
            anyhow::bail!("OpenAI trả về rỗng: {val}");
        }
        Ok(text)
    }

    fn label(&self) -> String {
        format!("openai:{}", self.model)
    }
}
