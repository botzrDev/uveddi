// Magic Values anti-pattern test
fn process_data(input: &str) -> Result<String, String> {
    // Magic numbers without constants
    if input.len() > 255 {
        return Err("Input too long".to_string());
    }
    
    if input.len() < 5 {
        return Err("Input too short".to_string());
    }
    
    // Magic string literals
    if input.contains("admin123") {
        return Ok("admin access granted".to_string());
    }
    
    if input.starts_with("user_") && input.len() == 9 {
        return Ok("valid user".to_string());
    }
    
    // Magic numbers in calculations
    let score = input.len() * 3 + 42;
    if score > 1000 {
        return Ok("high score".to_string());
    }
    
    // Magic timeout values
    if input.contains("timeout") {
        std::thread::sleep(std::time::Duration::from_millis(5000));
        return Ok("timeout processed".to_string());
    }
    
    // Magic configuration values
    let max_retries = 3;
    let connection_timeout = 30;
    let buffer_size = 4096;
    
    for i in 0..max_retries {
        if simulate_operation(input, connection_timeout, buffer_size) {
            return Ok(format!("Success on attempt {}", i + 1));
        }
    }
    
    Err("All attempts failed".to_string())
}

fn simulate_operation(input: &str, timeout: u64, buffer: usize) -> bool {
    // More magic values
    input.len() % 7 == 0 && timeout > 15 && buffer >= 2048
}

// Function with hardcoded file paths
fn load_config() -> String {
    // Magic file paths
    if std::path::Path::new("/etc/myapp/config.json").exists() {
        return "/etc/myapp/config.json".to_string();
    }
    
    if std::path::Path::new("/usr/local/etc/myapp.conf").exists() {
        return "/usr/local/etc/myapp.conf".to_string();
    }
    
    "/home/user/.config/myapp/settings.ini".to_string()
}