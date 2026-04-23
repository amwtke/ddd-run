//! `ddd-run status` — check whether the harness is properly installed.

use anyhow::{Context, Result};
use colored::*;
use std::path::{Path, PathBuf};

pub fn run(target_dir: &str) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!(
        "{} {}",
        "📋".bold(),
        "ddd-run harness status".bold().cyan()
    );
    println!("  {} {}", "→ target:".dimmed(), target.display());
    println!();

    let mut all_ok = true;

    println!("{}", "Skills".bold());
    all_ok &= check(&target, ".claude/skills/ddd-storm/SKILL.md");
    all_ok &= check(&target, ".claude/skills/ddd-model/SKILL.md");
    all_ok &= check(&target, ".claude/skills/ddd-spec/SKILL.md");

    println!();
    println!("{}", "Harness documents".bold());
    all_ok &= check(&target, "CLAUDE.md");
    all_ok &= check(&target, "DOMAIN.md");
    all_ok &= check(&target, "README-DDD-HARNESS.md");

    println!();
    println!("{}", "Working directories".bold());
    all_ok &= check_dir(&target, "docs/ddd");
    all_ok &= check_dir(&target, "docs/specs");

    println!();
    if all_ok {
        println!("{} {}", "✓".green().bold(), "harness is complete.".green());
    } else {
        println!(
            "{} {}",
            "✗".red().bold(),
            "some assets are missing. Run `ddd-run init` to install.".red()
        );
    }
    println!();

    Ok(())
}

fn check(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_file() {
        println!("  {} {}", "✓".green(), rel);
        true
    } else {
        println!("  {} {}", "✗".red(), rel.red());
        false
    }
}

fn check_dir(target: &Path, rel: &str) -> bool {
    let p = target.join(rel);
    if p.is_dir() {
        println!("  {} {}/", "✓".green(), rel);
        true
    } else {
        println!("  {} {}/", "✗".red(), rel.red());
        false
    }
}
