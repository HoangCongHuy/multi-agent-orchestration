mod anthropic;
mod cli;
mod openai;

use crate::config::ProviderConfig;
use anyhow::Result;
use async_trait::async_trait;

/// Trừu tượng cho "một cách gọi model". API hay CLI đều cài trait này.
#[async_trait]
pub trait Provider: Send + Sync {
    /// Gửi system + user prompt, trả về text model sinh ra.
    async fn complete(&self, system: &str, user: &str) -> Result<String>;
    /// Mô tả ngắn để in ra log (vd "anthropic:claude-opus-4-8").
    fn label(&self) -> String;
}

/// Tạo provider cụ thể từ config.
pub fn build(cfg: &ProviderConfig) -> Result<Box<dyn Provider>> {
    Ok(match cfg {
        ProviderConfig::Anthropic {
            model,
            max_tokens,
            api_key_env,
        } => Box::new(anthropic::AnthropicProvider::new(
            model.clone(),
            *max_tokens,
            api_key_env,
        )?),
        ProviderConfig::Openai {
            model,
            max_tokens,
            api_key_env,
            base_url,
        } => Box::new(openai::OpenaiProvider::new(
            model.clone(),
            *max_tokens,
            api_key_env,
            base_url.clone(),
        )?),
        ProviderConfig::Cli {
            command,
            args,
            prompt_via,
        } => Box::new(cli::CliProvider::new(
            command.clone(),
            args.clone(),
            prompt_via.clone(),
        )),
    })
}
