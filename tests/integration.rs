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

#[test]
fn init_installs_archunit_test_at_correct_path() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = Command::new(ddd_run_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("run ddd-run init");
    assert!(status.success());

    let archunit = target.join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    assert!(archunit.is_file(), "expected ArchUnit template at {}", archunit.display());

    let content = fs::read_to_string(&archunit).unwrap();
    assert!(content.contains("@ArchTest"), "ArchUnit template must contain @ArchTest");
    assert!(content.contains("layered_dependencies"), "must include layered architecture rule");
    assert!(content.contains("usecase_pure_of_frameworks"), "must include usecase purity rule");
}

#[test]
fn init_minimal_skips_archunit() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    let status = Command::new(ddd_run_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("run ddd-run init --minimal");
    assert!(status.success());

    let archunit = target.join("src").join("test").join("java")
        .join("architecture").join("CleanArchitectureTest.java");
    assert!(!archunit.exists(), "minimal mode must not install ArchUnit");
}

#[test]
fn status_reports_archunit_present_after_init() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(ddd_run_bin())
        .args(["init", "--dir"])
        .arg(target)
        .status()
        .expect("init failed");

    let output = Command::new(ddd_run_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status failed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("CleanArchitectureTest.java"),
        "status output must mention CleanArchitectureTest.java; got:\n{}",
        stdout
    );
    assert!(
        stdout.contains("harness is complete"),
        "status should report complete after init; got:\n{}",
        stdout
    );
}

#[test]
fn status_flags_missing_archunit_when_only_skills_installed() {
    let tmp = tempfile::tempdir().expect("tempdir");
    let target = tmp.path();

    Command::new(ddd_run_bin())
        .args(["init", "--minimal", "--dir"])
        .arg(target)
        .status()
        .expect("init --minimal failed");

    let output = Command::new(ddd_run_bin())
        .args(["status", "--dir"])
        .arg(target)
        .output()
        .expect("status failed");

    let stdout = String::from_utf8(output.stdout).unwrap();
    assert!(
        stdout.contains("some assets are missing"),
        "status should report missing assets; got:\n{}",
        stdout
    );
}
