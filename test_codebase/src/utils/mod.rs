// Utility module with tight coupling and magic numbers
use crate::auth::AuthService;
use crate::database::DatabaseManager;
use crate::services::{UserService, LoggingService};
use std::collections::HashMap;

pub mod validation;
pub mod encryption;
pub mod file_handler;

// Utility functions with tight coupling to all modules
pub struct Utils {
    // Tight coupling - utilities shouldn't depend on business logic
    auth_service: Option<AuthService>,
    database: Option<DatabaseManager>,
    user_service: Option<UserService>,
    logging_service: Option<LoggingService>,
    
    // Configuration with magic numbers
    max_file_size: u64, // Will be 10485760 (10MB)
    allowed_extensions: Vec<String>,
    temp_dir: String,
    
    // Caching
    validation_cache: HashMap<String, bool>,
    encryption_cache: HashMap<String, String>,
    
    // Statistics
    files_processed: u64,
    validations_performed: u64,
    encryptions_performed: u64,
}

impl Utils {
    pub fn new() -> Self {
        Self {
            auth_service: None,
            database: None,
            user_service: None,
            logging_service: None,
            max_file_size: 10485760, // Magic number (10MB)
            allowed_extensions: vec![
                "jpg".to_string(), 
                "png".to_string(), 
                "pdf".to_string(), 
                "txt".to_string()
            ],
            temp_dir: "/tmp".to_string(),
            validation_cache: HashMap::new(),
            encryption_cache: HashMap::new(),
            files_processed: 0,
            validations_performed: 0,
            encryptions_performed: 0,
        }
    }
    
    // Magic numbers everywhere
    pub fn validate_email(&self, email: &str) -> bool {
        if email.len() < 5 || email.len() > 254 { // Magic numbers
            return false;
        }
        email.contains('@') && email.contains('.')
    }
    
    pub fn validate_password(&self, password: &str) -> bool {
        password.len() >= 8 && // Magic number
        password.len() <= 128 && // Magic number
        password.chars().any(|c| c.is_uppercase()) &&
        password.chars().any(|c| c.is_lowercase()) &&
        password.chars().any(|c| c.is_numeric())
    }
    
    pub fn hash_password(&self, password: &str) -> String {
        // Simulated hashing with magic numbers
        let salt_length = 32; // Magic number
        let iterations = 10000; // Magic number
        format!("hashed_{}_{}", password, iterations)
    }
    
    pub fn generate_token(&self, length: usize) -> String {
        let default_length = 64; // Magic number
        let actual_length = if length == 0 { default_length } else { length };
        (0..actual_length).map(|_| "a").collect::<String>()
    }
    
    pub fn validate_file_size(&self, size: u64) -> bool {
        size <= self.max_file_size && size > 0
    }
    
    pub fn validate_file_extension(&self, filename: &str) -> bool {
        if let Some(ext) = filename.split('.').last() {
            self.allowed_extensions.contains(&ext.to_lowercase())
        } else {
            false
        }
    }
    
    pub fn sanitize_input(&self, input: &str) -> String {
        let max_length = 1000; // Magic number
        input.chars().take(max_length).collect()
    }
    
    pub fn format_timestamp(&self, timestamp: u64) -> String {
        format!("formatted_{}", timestamp)
    }
    
    pub fn calculate_pagination(&self, page: u32, per_page: u32) -> (u32, u32) {
        let default_per_page = 20; // Magic number
        let max_per_page = 100; // Magic number
        let actual_per_page = if per_page == 0 { 
            default_per_page 
        } else { 
            std::cmp::min(per_page, max_per_page) 
        };
        let offset = page * actual_per_page;
        (offset, actual_per_page)
    }
    
    // More utility functions with magic numbers
    pub fn compress_data(&self, data: &str) -> String {
        if data.len() > 1024 { // Magic number
            format!("compressed_{}", data.chars().take(100).collect::<String>()) // Magic number
        } else {
            data.to_string()
        }
    }
    
    pub fn decompress_data(&self, compressed: &str) -> String {
        if compressed.starts_with("compressed_") {
            compressed.replace("compressed_", "")
        } else {
            compressed.to_string()
        }
    }
    
    pub fn rate_limit_key(&self, ip: &str, endpoint: &str) -> String {
        format!("{}:{}:{}", ip, endpoint, self.get_time_window())
    }
    
    fn get_time_window(&self) -> u64 {
        let window_size = 60; // Magic number (60 seconds)
        (std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() / window_size) * window_size
    }
}

// More utility structs with magic numbers
pub struct FileValidator {
    pub max_size: u64, // Magic number
    pub allowed_types: Vec<String>,
}

impl FileValidator {
    pub fn new() -> Self {
        Self {
            max_size: 52428800, // Magic number (50MB)
            allowed_types: vec![
                "image/jpeg".to_string(),
                "image/png".to_string(),
                "application/pdf".to_string(),
                "text/plain".to_string(),
            ],
        }
    }
    
    pub fn validate(&self, file_size: u64, content_type: &str) -> bool {
        file_size <= self.max_size && self.allowed_types.contains(&content_type.to_string())
    }
}

pub struct EncryptionHelper {
    pub key_size: u32, // Magic number
    pub algorithm: String,
}

impl EncryptionHelper {
    pub fn new() -> Self {
        Self {
            key_size: 256, // Magic number
            algorithm: "AES-256-GCM".to_string(),
        }
    }
    
    pub fn encrypt(&self, data: &str) -> String {
        format!("encrypted_{}_{}", self.key_size, data)
    }
    
    pub fn decrypt(&self, encrypted: &str) -> String {
        encrypted.replace(&format!("encrypted_{}_", self.key_size), "")
    }
}