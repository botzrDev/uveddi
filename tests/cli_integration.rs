//! CLI integration and end-to-end tests

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn cli_runs_analysis_and_outputs_markdown() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=markdown");

        cmd.assert()
            .success()
            .stdout(predicate::str::contains("# Uveddi Analysis Report"));
    }

    #[test]
    fn cli_runs_analysis_and_outputs_json() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=json");

        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(r#""run_id":"#).unwrap());
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
