//! `cx-switch` 独立入口：只提供 Codex 预设切换命令。

use anyhow::Result;

fn main() -> Result<()> {
    cc_switch::run_codex()
}
