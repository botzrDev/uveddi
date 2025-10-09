// God Object Anti-pattern Example - Should be detected
pub struct MegaController {
    database_connection: String,
    file_system: String,
    network_handler: String,
    user_interface: String,
    logging_system: String,
    validation_engine: String,
    security_manager: String,
    cache_system: String,
    configuration: String,
}

impl MegaController {
    // This struct has too many responsibilities - classic God Object
    pub fn handle_user_login(&self) { /* Login logic */ }
    pub fn validate_input(&self) { /* Input validation */ }
    pub fn log_activity(&self) { /* Logging */ }
    pub fn cache_data(&self) { /* Caching */ }
    pub fn handle_database(&self) { /* Database operations */ }
    pub fn manage_files(&self) { /* File system operations */ }
    pub fn send_network_request(&self) { /* Network operations */ }
    pub fn update_ui(&self) { /* UI updates */ }
    pub fn manage_security(&self) { /* Security operations */ }
    pub fn handle_configuration(&self) { /* Configuration management */ }
    pub fn process_payments(&self) { /* Payment processing */ }
    pub fn generate_reports(&self) { /* Report generation */ }
    pub fn manage_users(&self) { /* User management */ }
    pub fn handle_notifications(&self) { /* Notification system */ }
    pub fn backup_system(&self) { /* Backup operations */ }
    
    // More methods to make it exceed thresholds...
    pub fn method1(&self) { /* */ }
    pub fn method2(&self) { /* */ }
    pub fn method3(&self) { /* */ }
    pub fn method4(&self) { /* */ }
    pub fn method5(&self) { /* */ }
    pub fn method6(&self) { /* */ }
    pub fn method7(&self) { /* */ }
    pub fn method8(&self) { /* */ }
    pub fn method9(&self) { /* */ }
    pub fn method10(&self) { /* */ }
}

// Dead code - should be detected
fn never_used_function() {
    println!("This function is never called");
}

fn another_unused_function() {
    let _unused_var = "This is dead code";
}

// Magic numbers - should be detected
pub fn calculate_tax(amount: f64) -> f64 {
    amount * 0.08 // Magic number 0.08
}

pub fn get_max_connections() -> i32 {
    100 // Magic number 100
}

pub fn process_data() {
    let timeout = 5000; // Magic number 5000
    let buffer_size = 8192; // Magic number 8192
    println!("Processing with timeout: {} and buffer: {}", timeout, buffer_size);
}