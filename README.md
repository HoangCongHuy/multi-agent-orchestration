# agentflow

Tool nối nhiều AI agent thành một **pipeline** xử lý một task. Mỗi agent là một
vai trò (tìm hiểu → lên kế hoạch → review → thực hiện) và có thể gán **model
tùy ý** (opus, haiku, gpt, hoặc gọi CLI codex/claude). Agent sau nhận task gốc
cộng với toàn bộ output của các agent trước.

## Cài đặt

1. Cài Rust (một lần): tải từ https://rustup.rs hoặc `winget install Rustlang.Rustup`
2. Tạo key:
   ```powershell
   copy .env.example .env   # rồi điền API key vào .env
   ```
3. Build:
   ```powershell
   cargo build --release
   ```

## Dùng

```powershell
# Xem pipeline đang cấu hình
cargo run -- agents

# Chạy một task
cargo run -- run "Viết một REST API quản lý todo bằng Rust"

# Lưu báo cáo đầy đủ ra file markdown
cargo run -- run "..." --output bao-cao.md

# Dùng config khác
cargo run -- --config khac.toml run "..."
```

Sau khi `cargo build --release`, binary nằm ở `target/release/agentflow.exe`,
chạy trực tiếp: `agentflow run "..."`.

## Tùy biến

Mọi thứ nằm trong `config.toml`:

- **Đổi model của một agent**: sửa field `provider` của agent đó.
- **Thêm provider**: thêm block `[providers.<tên>]`.
  - `type = "anthropic"` / `"openai"`: gọi API trực tiếp.
  - `type = "cli"`: chạy lệnh có sẵn (`claude`, `codex`...) qua subprocess.
- **Thêm/bớt/đổi thứ tự agent**: sửa các block `[[agents]]`. Pipeline chạy
  đúng theo thứ tự khai báo.

## Kiến trúc

```
src/
  main.rs          # CLI (clap): lệnh run / agents
  config.rs        # đọc & validate config.toml
  pipeline.rs      # điều phối: chạy agent tuần tự, nối context
  provider/
    mod.rs         # trait Provider + factory
    anthropic.rs   # Anthropic Messages API
    openai.rs      # OpenAI Chat Completions API
    cli.rs         # chạy CLI qua subprocess
```

Mở rộng provider mới = thêm 1 file cài trait `Provider` và 1 nhánh trong
`provider::build`.
