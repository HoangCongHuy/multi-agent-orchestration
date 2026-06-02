use anyhow::{Context, Result};
use serde::Deserialize;
use std::collections::HashMap;
use std::path::Path;

/// Toàn bộ cấu hình của tool, nạp từ file TOML.
#[derive(Debug, Deserialize)]
pub struct Config {
    /// Khai báo các provider (cách gọi model), key = tên tự đặt.
    pub providers: HashMap<String, ProviderConfig>,
    /// Danh sách agent, chạy theo đúng thứ tự khai báo.
    pub agents: Vec<AgentConfig>,
}

/// Một provider: hoặc gọi API trực tiếp, hoặc chạy CLI có sẵn.
#[derive(Debug, Deserialize, Clone)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum ProviderConfig {
    /// Anthropic Messages API (opus, sonnet, haiku).
    Anthropic {
        model: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        /// Tên biến môi trường chứa API key (mặc định ANTHROPIC_API_KEY).
        #[serde(default = "default_anthropic_key_env")]
        api_key_env: String,
    },
    /// OpenAI Chat Completions API (gpt, codex...).
    Openai {
        model: String,
        #[serde(default = "default_max_tokens")]
        max_tokens: u32,
        #[serde(default = "default_openai_key_env")]
        api_key_env: String,
        /// Cho phép trỏ tới endpoint tương thích OpenAI khác.
        #[serde(default = "default_openai_base")]
        base_url: String,
    },
    /// Gọi một CLI có sẵn trên máy (claude, codex...) qua subprocess.
    Cli {
        /// Lệnh để chạy, ví dụ "claude" hoặc "codex".
        command: String,
        /// Tham số cố định truyền vào lệnh.
        #[serde(default)]
        args: Vec<String>,
        /// Cách đưa prompt vào: "stdin" hoặc "arg".
        #[serde(default = "default_prompt_via")]
        prompt_via: String,
    },
}

/// Một agent trong pipeline.
#[derive(Debug, Deserialize, Clone)]
pub struct AgentConfig {
    /// Tên hiển thị, ví dụ "researcher".
    pub name: String,
    /// Trỏ tới key của một provider đã khai báo ở trên.
    pub provider: String,
    /// System prompt định nghĩa vai trò của agent.
    pub system: String,
}

fn default_max_tokens() -> u32 {
    4096
}
fn default_anthropic_key_env() -> String {
    "ANTHROPIC_API_KEY".into()
}
fn default_openai_key_env() -> String {
    "OPENAI_API_KEY".into()
}
fn default_openai_base() -> String {
    "https://api.openai.com/v1".into()
}
fn default_prompt_via() -> String {
    "stdin".into()
}

impl Config {
    pub fn load(path: &Path) -> Result<Self> {
        let text = std::fs::read_to_string(path)
            .with_context(|| format!("Không đọc được file config: {}", path.display()))?;
        let cfg: Config =
            toml::from_str(&text).with_context(|| "File config TOML không hợp lệ")?;
        cfg.validate()?;
        Ok(cfg)
    }

    /// Kiểm tra mỗi agent đều trỏ tới một provider có thật.
    fn validate(&self) -> Result<()> {
        if self.agents.is_empty() {
            anyhow::bail!("Config không có agent nào trong pipeline");
        }
        for agent in &self.agents {
            if !self.providers.contains_key(&agent.provider) {
                anyhow::bail!(
                    "Agent '{}' trỏ tới provider '{}' không tồn tại",
                    agent.name,
                    agent.provider
                );
            }
        }
        Ok(())
    }
}
