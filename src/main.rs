//! ddd-run: A CLI to bootstrap DDD + Superpowers harness for Claude Code projects.

use anyhow::Result;
use clap::{Parser, Subcommand};
use colored::*;

mod commands;

#[derive(Parser, Debug)]
#[command(name = "ddd-run")]
#[command(about = "Bootstrap DDD + Superpowers harness for Claude Code projects", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand, Debug)]
enum Commands {
    /// Initialize DDD harness in the current directory
    Init {
        /// Overwrite existing files
        #[arg(short, long)]
        force: bool,

        /// Only install skills, skip CLAUDE.md / DOMAIN.md / README generation
        #[arg(short, long)]
        minimal: bool,

        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },

    /// Check if the current project has the DDD harness properly installed
    Status {
        /// Target directory (default: current directory)
        #[arg(short, long, default_value = ".")]
        dir: String,
    },
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Init {
            force,
            minimal,
            dir,
        } => {
            commands::init::run(&dir, force, minimal)?;
        }
        Commands::Status { dir } => {
            commands::status::run(&dir)?;
        }
    }

    Ok(())
}

/// Print a success message with a green checkmark.
pub fn success(msg: &str) {
    println!("{} {}", "✓".green().bold(), msg);
}

/// Print an info message.
pub fn info(msg: &str) {
    println!("{} {}", "ℹ".blue().bold(), msg);
}

/// Print a warning message.
pub fn warn(msg: &str) {
    println!("{} {}", "⚠".yellow().bold(), msg);
}

/// Print a skipped message.
pub fn skip(msg: &str) {
    println!("{} {}", "↷".bright_black().bold(), msg.bright_black());
}
