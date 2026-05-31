//! cc-switch 入口点。仅委托给库 crate 的 `run()`。
//!
//! 将 CLI 解析与业务逻辑分离到 `lib.rs`，这样集成测试可以直接调用
//! `cc_switch::run()` 而无需通过 `main`。

use anyhow::Result;

fn main() -> Result<()> {
    cc_switch::run()
}
