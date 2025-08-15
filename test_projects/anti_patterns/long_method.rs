// Long Method anti-pattern test
fn extremely_long_method(data: &str) -> String {
    let mut result = String::new();
    
    // Step 1: Parse input
    if data.is_empty() {
        return "empty".to_string();
    }
    
    // Step 2: Validate data
    let lines: Vec<&str> = data.split('\n').collect();
    if lines.len() > 100 {
        return "too many lines".to_string();
    }
    
    // Step 3: Process each line
    for line in lines {
        if line.trim().is_empty() {
            continue;
        }
        
        // Sub-step 3.1: Parse line components
        let parts: Vec<&str> = line.split(',').collect();
        if parts.len() < 2 {
            continue;
        }
        
        // Sub-step 3.2: Validate each part
        let mut valid_parts = Vec::new();
        for part in parts {
            let trimmed = part.trim();
            if trimmed.len() > 0 && trimmed.len() < 50 {
                valid_parts.push(trimmed);
            }
        }
        
        // Sub-step 3.3: Transform data
        if valid_parts.len() >= 2 {
            let first = valid_parts[0].to_uppercase();
            let second = valid_parts[1].to_lowercase();
            let combined = format!("{}:{}", first, second);
            
            // Sub-step 3.4: Apply business logic
            if combined.contains("error") {
                result.push_str(&format!("ERROR: {}\n", combined));
            } else if combined.contains("warning") {
                result.push_str(&format!("WARN: {}\n", combined));
            } else if combined.contains("info") {
                result.push_str(&format!("INFO: {}\n", combined));
            } else {
                result.push_str(&format!("DEBUG: {}\n", combined));
            }
        }
    }
    
    // Step 4: Post-process results
    let mut final_result = String::new();
    let result_lines: Vec<&str> = result.split('\n').collect();
    let mut error_count = 0;
    let mut warning_count = 0;
    let mut info_count = 0;
    let mut debug_count = 0;
    
    // Count message types
    for line in &result_lines {
        if line.starts_with("ERROR:") {
            error_count += 1;
        } else if line.starts_with("WARN:") {
            warning_count += 1;
        } else if line.starts_with("INFO:") {
            info_count += 1;
        } else if line.starts_with("DEBUG:") {
            debug_count += 1;
        }
    }
    
    // Add summary header
    final_result.push_str(&format!("SUMMARY: {} errors, {} warnings, {} info, {} debug\n", 
                                   error_count, warning_count, info_count, debug_count));
    
    // Add separator
    final_result.push_str("=" .repeat(50));
    final_result.push('\n');
    
    // Add all messages
    for line in result_lines {
        if !line.trim().is_empty() {
            final_result.push_str(line);
            final_result.push('\n');
        }
    }
    
    // Step 5: Final validation and cleanup
    if final_result.len() > 10000 {
        final_result = final_result[..10000].to_string();
        final_result.push_str("... [TRUNCATED]");
    }
    
    // Remove trailing newlines
    while final_result.ends_with('\n') {
        final_result.pop();
    }
    
    // Add timestamp if needed
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    
    final_result.push_str(&format!("\nProcessed at: {}", timestamp));
    
    final_result
}