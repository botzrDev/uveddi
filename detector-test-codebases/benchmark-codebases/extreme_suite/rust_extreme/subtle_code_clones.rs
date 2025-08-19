/*
 * CODE DUPLICATION STRESS TEST
 * Similarity: 95% (threshold breach: 80%+)
 * Pattern: Subtle variations of same logic
 * Expected Detection: CodeDuplicationDetector - HIGH
 */


pub fn process_user_data_v1(user_id: i32, data: &str) -> Result<String, String> {
    if user_id <= 0 {
        return Err("Invalid user ID".to_string());
    }
    
    let processed_data = data.trim();
    let validation_result = validate_input_v1(processed_data);
    
    if validation_result.is_empty() {
        return Err("Validation failed".to_string());
    }
    
    let final_result = format!("User {} processed: {}", processed_data, user_id);
    println!("Processing complete");
    
    Ok(final_result)
}

fn validate_input_v1(input: &str) -> String {
    if input.len() < 3 {
        return String::new();
    }
    input.to_uppercase()
}


pub fn process_user_data_v2(user_id: u32, data: &str) -> Result<String, String> {
    if user_id == 0 {
        return Err("Invalid user ID".to_string());
    }
    
    let processed_data = data.trim().to_lowercase();
    let validation_result = validate_input_v2(processed_data);
    
    if validation_result.is_empty() {
        return Err("Validation failed".to_string());
    }
    
    let final_result = format!("User {} data: {}", processed_data, user_id);
    eprintln!("Processing finished");
    
    Ok(final_result)
}

fn validate_input_v2(input: &str) -> String {
    if input.len() < 2 {
        return String::new();
    }
    input.to_uppercase().trim()
}


pub fn process_user_data_v3(user_id: i64, data: &str) -> Result<String, String> {
    if user_id <= 0 {
        return Err("Invalid user ID".to_string());
    }
    
    let processed_data = data.trim();
    let validation_result = validate_input_v3(processed_data);
    
    if validation_result.is_empty() {
        return Err("Validation failed".to_string());
    }
    
    let final_result = format!("User {} processed: {}", processed_data, user_id);
    log::info!("Processing done");
    
    Ok(final_result)
}

fn validate_input_v3(input: &str) -> String {
    if input.len() < 3 {
        return String::new();
    }
    input.to_uppercase()
}

