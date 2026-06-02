use super::Provider;
use anyhow::{Context, Result};
use async_trait::async_trait;
use tokio::io::AsyncWriteExt;
use tokio::process::Command;

/// Gọi một CLI có sẵn (claude, codex...) qua subprocess.
/// Tiện khi bạn đã đăng nhập sẵn các tool đó và không muốn quản lý API key.
pub struct CliProvider {
    command: String,
    args: Vec<String>,
    /// "stdin": đẩy prompt qua stdin. "arg": nối prompt thành 1 tham số cuối.
    prompt_via: String,
}

impl CliProvider {
    pub fn new(command: String, args: Vec<String>, prompt_via: String) -> Self {
        Self {
            command,
            args,
            prompt_via,
        }
    }
}

#[async_trait]
impl Provider for CliProvider {
    async fn complete(&self, system: &str, user: &str) -> Result<String> {
        // Các CLI thường không tách system/user, nên ta gộp lại.
        let prompt = format!("{system}\n\n---\n\n{user}");

        let mut cmd = Command::new(&self.command);
        cmd.args(&self.args);
        cmd.stdout(std::process::Stdio::piped());
        cmd.stderr(std::process::Stdio::piped());

        match self.prompt_via.as_str() {
            "arg" => {
                cmd.arg(&prompt);
                cmd.stdin(std::process::Stdio::null());
            }
            "stdin" => {
                cmd.stdin(std::process::Stdio::piped());
            }
            other => anyhow::bail!("prompt_via không hợp lệ: '{other}' (dùng 'stdin' hoặc 'arg')"),
        }

        let mut child = cmd
            .spawn()
            .with_context(|| format!("Không chạy được lệnh '{}'", self.command))?;

        if self.prompt_via == "stdin" {
            let mut stdin = child
                .stdin
                .take()
                .context("Không mở được stdin của subprocess")?;
            stdin
                .write_all(prompt.as_bytes())
                .await
                .context("Ghi prompt vào stdin thất bại")?;
            drop(stdin); // đóng stdin để CLI biết hết input
        }

        let output = child
            .wait_with_output()
            .await
            .context("Chờ subprocess kết thúc thất bại")?;

        if !output.status.success() {
            let err = String::from_utf8_lossy(&output.stderr);
            anyhow::bail!("CLI '{}' lỗi (exit {}): {err}", self.command, output.status);
        }

        let text = String::from_utf8_lossy(&output.stdout).trim().to_string();
        if text.is_empty() {
            anyhow::bail!("CLI '{}' trả về rỗng", self.command);
        }
        Ok(text)
    }

    fn label(&self) -> String {
        format!("cli:{}", self.command)
    }
}
