mod config;
mod pipeline;
mod provider;

use anyhow::Result;
use clap::{Parser, Subcommand};
use config::Config;
use owo_colors::OwoColorize;
use std::path::PathBuf;

/// agentflow — kết nối nhiều AI agent thành một pipeline xử lý task.
#[derive(Parser)]
#[command(name = "agentflow", version, about)]
struct Cli {
    /// Đường dẫn file config TOML.
    #[arg(short, long, default_value = "config.toml", global = true)]
    config: PathBuf,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Chạy pipeline với một task.
    Run {
        /// Nội dung task cần xử lý.
        task: String,
        /// Ghi báo cáo đầy đủ ra file (markdown).
        #[arg(short, long)]
        output: Option<PathBuf>,
    },
    /// In ra các agent và provider đang cấu hình.
    Agents,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Nạp biến môi trường từ .env nếu có (chứa API key).
    let _ = dotenvy::dotenv();

    let cli = Cli::parse();
    let cfg = Config::load(&cli.config)?;

    match cli.command {
        Commands::Agents => {
            println!("{}", "Pipeline hiện tại:".bold());
            for (i, agent) in cfg.agents.iter().enumerate() {
                let p = &cfg.providers[&agent.provider];
                println!(
                    "  {}. {} {}",
                    i + 1,
                    agent.name.cyan().bold(),
                    format!("-> provider '{}' ({:?})", agent.provider, p).dimmed()
                );
            }
        }
        Commands::Run { task, output } => {
            println!("{} {}", "Task:".bold(), task);
            let results = pipeline::run(&cfg, &task).await?;

            if let Some(path) = output {
                let report = build_report(&task, &results);
                std::fs::write(&path, report)?;
                println!("\n{} {}", "Đã ghi báo cáo:".green().bold(), path.display());
            }

            println!("\n{}", "Hoàn tất pipeline.".green().bold());
        }
    }

    Ok(())
}

/// Tạo báo cáo markdown đầy đủ từ kết quả pipeline.
fn build_report(task: &str, results: &[pipeline::StageResult]) -> String {
    let mut s = String::new();
    s.push_str(&format!("# Báo cáo agentflow\n\n**Task:** {task}\n\n"));
    s.push_str(&format!(
        "_Tạo lúc: {}_\n",
        chrono::Local::now().format("%Y-%m-%d %H:%M:%S")
    ));
    for r in results {
        s.push_str(&format!(
            "\n---\n\n## {} ({})\n\n{}\n",
            r.agent, r.provider_label, r.output
        ));
    }
    s
}
