//! `ddd-run init` command — installs DDD harness assets into the target directory.

use anyhow::{Context, Result};
use colored::*;
use std::fs;
use std::path::{Path, PathBuf};

// Templates embedded at compile time — zero runtime dependency on external files.
const SKILL_DDD_STORM: &str = include_str!("../templates/skills/ddd-storm.md");
const SKILL_DDD_MODEL: &str = include_str!("../templates/skills/ddd-model.md");
const SKILL_DDD_SPEC: &str = include_str!("../templates/skills/ddd-spec.md");
const ROOT_CLAUDE_MD: &str = include_str!("../templates/root/CLAUDE.md");
const ROOT_DOMAIN_MD: &str = include_str!("../templates/root/DOMAIN.md");
const ROOT_README: &str = include_str!("../templates/root/README-DDD-HARNESS.md");

pub fn run(target_dir: &str, force: bool, minimal: bool) -> Result<()> {
    let target = PathBuf::from(target_dir)
        .canonicalize()
        .with_context(|| format!("Failed to resolve target directory: {}", target_dir))?;

    println!();
    println!(
        "{} {}",
        "🛠 ".bold(),
        "ddd-run: installing DDD + Superpowers harness".bold().cyan()
    );
    println!("  {} {}", "→ target:".dimmed(), target.display());
    if force {
        println!("  {} {}", "→ mode:".dimmed(), "--force (will overwrite)".yellow());
    }
    if minimal {
        println!("  {} {}", "→ mode:".dimmed(), "--minimal (skills only)".yellow());
    }
    println!();

    // 1. Install the three skills under .claude/skills/<name>/SKILL.md
    println!("{}", "Installing skills...".bold());
    install_skill(&target, "ddd-storm", SKILL_DDD_STORM, force)?;
    install_skill(&target, "ddd-model", SKILL_DDD_MODEL, force)?;
    install_skill(&target, "ddd-spec", SKILL_DDD_SPEC, force)?;

    // 2. Install root-level docs (unless --minimal)
    if !minimal {
        println!();
        println!("{}", "Installing harness documents...".bold());
        install_root_file(&target, "CLAUDE.md", ROOT_CLAUDE_MD, force)?;
        install_root_file(&target, "DOMAIN.md", ROOT_DOMAIN_MD, force)?;
        install_root_file(&target, "README-DDD-HARNESS.md", ROOT_README, force)?;

        // 3. Create working directories
        println!();
        println!("{}", "Creating working directories...".bold());
        ensure_dir(&target.join("docs").join("ddd"))?;
        ensure_dir(&target.join("docs").join("specs"))?;
        crate::success(&format!("{}", "docs/ddd/   (event storming & modeling notes)"));
        crate::success(&format!("{}", "docs/specs/ (ddd-spec outputs → Superpowers inputs)"));
    }

    // 4. Print next steps
    print_next_steps(minimal);

    Ok(())
}

/// Install a skill as `.claude/skills/<name>/SKILL.md`.
fn install_skill(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let skill_dir = target.join(".claude").join("skills").join(name);
    ensure_dir(&skill_dir)?;
    let path = skill_dir.join("SKILL.md");
    write_file(&path, content, force, &format!(".claude/skills/{}/SKILL.md", name))
}

/// Install a root-level file.
fn install_root_file(target: &Path, name: &str, content: &str, force: bool) -> Result<()> {
    let path = target.join(name);
    write_file(&path, content, force, name)
}

/// Write a file, respecting the --force flag.
fn write_file(path: &Path, content: &str, force: bool, display: &str) -> Result<()> {
    if path.exists() && !force {
        crate::skip(&format!("{} already exists (use --force to overwrite)", display));
        return Ok(());
    }
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)
            .with_context(|| format!("Failed to create parent dir for {}", path.display()))?;
    }
    fs::write(path, content)
        .with_context(|| format!("Failed to write {}", path.display()))?;
    crate::success(display);
    Ok(())
}

/// Make sure a directory exists.
fn ensure_dir(path: &Path) -> Result<()> {
    fs::create_dir_all(path)
        .with_context(|| format!("Failed to create directory {}", path.display()))
}

fn print_next_steps(minimal: bool) {
    println!();
    println!("{}", "━".repeat(60).bright_black());
    println!("{}", "Next steps".bold().green());
    println!("{}", "━".repeat(60).bright_black());

    if minimal {
        println!("  Skills installed. Open Claude Code and try:");
        println!("    {} /ddd-storm <your business description>", "•".cyan());
        println!();
        println!(
            "  {} for full project integration, run without --minimal.",
            "tip:".yellow().bold()
        );
        return;
    }

    println!();
    println!("  1. {} open the generated files:", "Review".cyan());
    println!("       - CLAUDE.md               (project-level AI rules)");
    println!("       - DOMAIN.md               (domain model SSoT, starts empty)");
    println!("       - README-DDD-HARNESS.md   (how to use this harness)");
    println!();
    println!("  2. {} open Claude Code in this directory.", "Launch".cyan());
    println!();
    println!("  3. {} start the DDD workflow:", "Run".cyan());
    println!(
        "       {}",
        "/ddd-storm <your business description>".bold().green()
    );
    println!("       {}", "/ddd-model".bold().green());
    println!(
        "       {}",
        "/ddd-spec <use case>".bold().green()
    );
    println!();
    println!(
        "  4. {} hand off the generated spec to Superpowers for TDD.",
        "Implement".cyan()
    );
    println!();
    println!(
        "  {} see README-DDD-HARNESS.md for the complete workflow.",
        "→".bright_black()
    );
    println!();
}
