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
        let file_path = dir.path().join("god_object.rs");
        let code = r#"
        struct GodObject {
            a: i32,
            b: i32,
            c: i32,
        }
        impl GodObject {
            fn m1(&self) {}
            fn m2(&self) {}
            fn m3(&self) {}
            fn m4(&self) {}
            fn m5(&self) {}
            fn m6(&self) {}
            fn m7(&self) {}
            fn m8(&self) {}
            fn m9(&self) {}
            fn m10(&self) {}
            fn m11(&self) {}
            fn m12(&self) {}
            fn m13(&self) {}
            fn m14(&self) {}
            fn m15(&self) {}
            fn m16(&self) {}
            fn m17(&self) {}
            fn m18(&self) {}
            fn m19(&self) {}
            fn m20(&self) {}
            fn m21(&self) {}
        }
        "#;
        File::create(&file_path).unwrap().write_all(code.as_bytes()).unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
        cmd.arg("analyze")
            .arg(dir.path())
            .arg("--output-format=markdown")
            .arg("--enable-ai");

        // Print output for debugging
        let output = cmd.output().expect("Failed to run uveddi");
        let stdout = String::from_utf8_lossy(&output.stdout);
        println!("CLI OUTPUT:\n{}", stdout);
        assert!(stdout.contains("AI Explanation") || stdout.contains("ai_explanation"), "Expected AI explanation in output, got: {}", stdout);
        assert!(stdout.contains("confidence") || stdout.contains("Confidence"), "Expected confidence in output, got: {}", stdout);
    }

    #[test]
    fn cli_ai_pipeline_handles_missing_api_key_gracefully() {
        let dir = tempdir().unwrap();
        let file_path = dir.path().join("main.rs");
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "fn main() {{}}").unwrap();

        let mut cmd = Command::cargo_bin("uveddi").unwrap();
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
