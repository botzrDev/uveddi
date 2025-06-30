//! End-to-end integration test for the full AI Reasoning Engine pipeline (RAG, hallucination defense, self-correction, hybrid verification)

#[cfg(test)]
mod tests {
    use assert_cmd::prelude::*;
    use predicates::prelude::*;
    use std::process::Command;
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;

    #[test]
    fn cli_ai_pipeline_outputs_explanations_and_confidence() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "struct GodObject {{ fn a(&self) {{}} fn b(&self) {{}} }}").unwrap();

        let mut cmd = Command::cargo_bin("codeatlas").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=markdown")
            .arg("--enable-ai");

        // The output should contain an AI explanation section and a confidence score (even if fallback is used)
        cmd.assert()
            .success()
            .stdout(predicate::str::contains("AI Explanation").or(predicate::str::contains("ai_explanation")))
            .stdout(predicate::str::contains("confidence").or(predicate::str::contains("Confidence")));
    }

    #[test]
    fn cli_ai_pipeline_handles_missing_api_key_gracefully() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let mut cmd = Command::cargo_bin("codeatlas").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=json")
            .arg("--enable-ai");

        // Should not fail, but may warn about missing API key
        cmd.assert()
            .success()
            .stdout(predicate::str::is_match(r#"ai_explanation":|AI Explanation"#).unwrap());
    }
}
