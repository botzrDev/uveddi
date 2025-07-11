//! Human-in-the-loop verification workflow

use crate::ai::types::AiSuggestion;
use std::io::{self, Write};

/// CLI workflow for human review of AI suggestions
pub fn request_human_verification_cli(suggestion: &AiSuggestion) -> bool {
    println!("\n===== AI Architectural Suggestion =====");
    println!("Title: {}", suggestion.title);
    println!("Description: {}", suggestion.description);
    println!("Explanation: {}", suggestion.explanation);
    println!("Refactoring: {}", suggestion.refactoring);
    println!("Confidence: {}", suggestion.confidence.as_ref().unwrap_or("unknown"));
    println!("======================================");
    println!("\nDo you accept this suggestion? [y]es / [n]o / [c]larify: ");
    print!("> ");
    io::stdout().flush().unwrap();
    let mut input = String::new();
    io::stdin().read_line(&mut input).unwrap();
    match input.trim().to_lowercase().as_str() {
        "y" | "yes" => {
            log_user_feedback(&suggestion, "accepted");
            true
        },
        "n" | "no" => {
            log_user_feedback(&suggestion, "rejected");
            false
        },
        "c" | "clarify" => {
            log_user_feedback(&suggestion, "clarify");
            false
        },
        _ => {
            println!("Invalid input. Assuming rejection.");
            log_user_feedback(&suggestion, "invalid");
            false
        }
    }
}

/// Log user feedback to a file (append mode)
pub fn log_user_feedback(suggestion: &AiSuggestion, action: &str) {
    use std::fs::OpenOptions;
    use std::io::Write;
    if let Ok(mut file) = OpenOptions::new().create(true).append(true).open("ai_suggestion_feedback.log") {
        let log_entry = format!(
            "{} | {} | {} | {}\n",
            chrono::Utc::now().to_rfc3339(),
            action,
            suggestion.title,
            suggestion.confidence.as_ref().unwrap_or("unknown")
        );
        let _ = file.write_all(log_entry.as_bytes());
    }
}
