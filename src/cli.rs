use clap::{Parser, Subcommand};

#[derive(Debug, Parser)]
#[command(
    name = "cc-switch",
    version,
    about = "Cross-platform Claude Code profile switcher",
    arg_required_else_help = true
)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Debug, Subcommand)]
pub enum Commands {
    /// List available profiles.
    List,
    /// Show the current profile matched against the target settings file.
    Current,
    /// Switch to a named profile.
    Use { name: String },
    /// Rotate to the next profile in filename order.
    Next,
    /// Validate paths, directories, and target settings state.
    Doctor,
}
