//! Integration tests for ddd-run.
//! Each test creates a tempdir, runs init/status, and verifies filesystem state.

use std::fs;
use std::process::Command;

/// Path to the cargo-built binary under test.
fn ddd_run_bin() -> std::path::PathBuf {
    // CARGO_BIN_EXE_<name> is set by Cargo for integration tests.
    std::path::PathBuf::from(env!("CARGO_BIN_EXE_ddd-run"))
}

#[test]
fn init_smoke_creates_skill_files() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = Command::new(ddd_run_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("run ddd-run init");
    assert!(status.success(), "ddd-run init failed");

    // Sanity: each skill file exists and is non-empty.
    for skill in &["ddd-storm", "ddd-model", "ddd-spec"] {
        let p = target.join(".claude").join("skills").join(skill).join("SKILL.md");
        assert!(p.is_file(), "missing {}", p.display());
        let content = fs::read_to_string(&p).unwrap();
        assert!(!content.is_empty(), "empty {}", p.display());
    }

    // Root files.
    for f in &["CLAUDE.md", "DOMAIN.md", "README-DDD-HARNESS.md"] {
        assert!(target.join(f).is_file(), "missing {}", f);
    }
}
