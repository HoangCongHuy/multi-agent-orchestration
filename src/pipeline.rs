use crate::config::Config;
use crate::provider::{self, Provider};
use anyhow::Result;
use owo_colors::OwoColorize;

/// Kết quả một agent chạy xong.
pub struct StageResult {
    pub agent: String,
    pub provider_label: String,
    pub output: String,
}

/// Chạy toàn bộ pipeline: từng agent nhận task gốc + output các agent trước.
pub async fn run(cfg: &Config, task: &str) -> Result<Vec<StageResult>> {
    let mut results: Vec<StageResult> = Vec::new();

    for agent in &cfg.agents {
        // provider đã được validate là tồn tại ở Config::load.
        let provider_cfg = &cfg.providers[&agent.provider];
        let provider: Box<dyn Provider> = provider::build(provider_cfg)?;
        let label = provider.label();

        println!(
            "\n{} {} {}",
            "▶".cyan().bold(),
            agent.name.bold(),
            format!("({label})").dimmed()
        );

        let user_prompt = build_user_prompt(task, &results);
        let output = provider.complete(&agent.system, &user_prompt).await?;

        println!("{}", output.dimmed());

        results.push(StageResult {
            agent: agent.name.clone(),
            provider_label: label,
            output,
        });
    }

    Ok(results)
}

/// Ghép task gốc + output tích lũy của các agent trước thành prompt cho agent kế tiếp.
fn build_user_prompt(task: &str, prior: &[StageResult]) -> String {
    let mut s = String::new();
    s.push_str("# TASK GỐC\n");
    s.push_str(task);
    s.push('\n');

    if !prior.is_empty() {
        s.push_str("\n# KẾT QUẢ TỪ CÁC AGENT TRƯỚC\n");
        for r in prior {
            s.push_str(&format!("\n## {}\n{}\n", r.agent, r.output));
        }
    }

    s.push_str("\nHãy thực hiện đúng vai trò của bạn dựa trên thông tin trên.");
    s
}
