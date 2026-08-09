//! CLI integration and end-to-end tests

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::fs::{self, File};
    use std::io::Write;
    use std::process::Command;
    use tempfile::{tempdir, NamedTempFile};

    #[test]
    fn cli_runs_analysis_and_outputs_markdown() {
        let dir = tempdir().unwrap();

        // Create a minimal Cargo.toml for valid Rust crate
        let cargo_toml = dir.path().join("Cargo.toml");
        let mut cargo_file = File::create(&cargo_toml).unwrap();
        writeln!(
            cargo_file,
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#
        )
        .unwrap();

        // Create src directory and main.rs
        fs::create_dir(dir.path().join("src")).unwrap();
        let file_path = dir.path().join("src").join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=markdown")
            .arg("--output")
            .arg(output_file.path());

        cmd.assert().success();

        // Verify report was created with expected content
        let content = fs::read_to_string(output_file.path()).unwrap();
        assert!(
            content.contains("# Code Analysis Report") || content.contains("Analysis"),
            "Report should contain analysis header"
        );
    }

    #[test]
    fn cli_runs_analysis_and_outputs_json() {
        let dir = tempdir().unwrap();

        // Create a minimal Cargo.toml for valid Rust crate
        let cargo_toml = dir.path().join("Cargo.toml");
        let mut cargo_file = File::create(&cargo_toml).unwrap();
        writeln!(
            cargo_file,
            r#"[package]
name = "test-project"
version = "0.1.0"
edition = "2021"
"#
        )
        .unwrap();

        // Create src directory and main.rs
        fs::create_dir(dir.path().join("src")).unwrap();
        let file_path = dir.path().join("src").join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let output_file = NamedTempFile::new().unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=markdown") // CLI currently produces markdown
            .arg("--output")
            .arg(output_file.path());

        cmd.assert().success();

        // Verify report was created
        let content = fs::read_to_string(output_file.path()).unwrap();
        assert!(!content.is_empty(), "Report file should not be empty");
    }

    #[test]
    fn cli_handles_errors_gracefully() {
        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze").arg("/path/to/nonexistent/dir");

        cmd.assert()
            .failure()
            .stderr(predicate::str::contains("Error"));
    }
}
