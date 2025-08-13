//! Test fixture data for comprehensive testing
//! 
//! This module provides realistic test data for all supported languages
//! and various architectural patterns for testing purposes.

use std::collections::HashMap;
use std::path::PathBuf;

/// Test fixture manager for organizing test data
pub struct TestFixtures {
    pub rust_fixtures: RustFixtures,
    pub python_fixtures: PythonFixtures,
    pub javascript_fixtures: JavaScriptFixtures,
    pub typescript_fixtures: TypeScriptFixtures,
    pub multi_language_fixtures: MultiLanguageFixtures,
}

impl TestFixtures {
    pub fn new() -> Self {
        Self {
            rust_fixtures: RustFixtures::new(),
            python_fixtures: PythonFixtures::new(),
            javascript_fixtures: JavaScriptFixtures::new(),
            typescript_fixtures: TypeScriptFixtures::new(),
            multi_language_fixtures: MultiLanguageFixtures::new(),
        }
    }
}

/// Rust-specific test fixtures
pub struct RustFixtures {
    pub god_objects: HashMap<&'static str, &'static str>,
    pub dead_code_examples: HashMap<&'static str, &'static str>,
    pub circular_dependencies: HashMap<&'static str, &'static str>,
    pub clean_code_examples: HashMap<&'static str, &'static str>,
    pub performance_test_code: HashMap<&'static str, &'static str>,
}

impl RustFixtures {
    pub fn new() -> Self {
        let mut god_objects = HashMap::new();
        let mut dead_code_examples = HashMap::new();
        let mut circular_dependencies = HashMap::new();
        let mut clean_code_examples = HashMap::new();
        let mut performance_test_code = HashMap::new();
        
        // God Object Examples
        god_objects.insert("massive_struct", r#"
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

/// A god object that handles too many responsibilities
pub struct MassiveApplicationManager {
    // Database management
    database_connections: Vec<String>,
    cached_queries: HashMap<String, String>,
    
    // File system management
    open_files: Vec<File>,
    file_cache: HashMap<PathBuf, String>,
    
    // Network management
    network_connections: Vec<String>,
    api_cache: HashMap<String, serde_json::Value>,
    
    // UI state management
    window_positions: HashMap<String, (i32, i32)>,
    theme_settings: HashMap<String, String>,
    
    // Configuration management
    config_values: HashMap<String, String>,
    environment_variables: HashMap<String, String>,
    
    // Logging and monitoring
    log_buffer: Vec<String>,
    metrics: HashMap<String, f64>,
    
    // Business logic state
    user_sessions: HashMap<String, UserSession>,
    business_rules: Vec<BusinessRule>,
}

impl MassiveApplicationManager {
    pub fn new() -> Self {
        Self {
            database_connections: Vec::new(),
            cached_queries: HashMap::new(),
            open_files: Vec::new(),
            file_cache: HashMap::new(),
            network_connections: Vec::new(),
            api_cache: HashMap::new(),
            window_positions: HashMap::new(),
            theme_settings: HashMap::new(),
            config_values: HashMap::new(),
            environment_variables: HashMap::new(),
            log_buffer: Vec::new(),
            metrics: HashMap::new(),
            user_sessions: HashMap::new(),
            business_rules: Vec::new(),
        }
    }
    
    // Database methods (should be separate service)
    pub fn connect_database(&mut self, connection_string: String) { /* ... */ }
    pub fn execute_query(&mut self, query: String) -> Result<String, String> { Ok("".to_string()) }
    pub fn cache_query_result(&mut self, query: String, result: String) { /* ... */ }
    pub fn get_cached_query(&self, query: &str) -> Option<&String> { None }
    
    // File system methods (should be separate service)
    pub fn open_file(&mut self, path: PathBuf) -> Result<(), std::io::Error> { Ok(()) }
    pub fn read_file_cached(&mut self, path: PathBuf) -> Result<String, std::io::Error> { Ok("".to_string()) }
    pub fn write_file(&mut self, path: PathBuf, content: String) -> Result<(), std::io::Error> { Ok(()) }
    pub fn close_all_files(&mut self) { /* ... */ }
    
    // Network methods (should be separate service)
    pub fn make_http_request(&mut self, url: String) -> Result<String, String> { Ok("".to_string()) }
    pub fn cache_api_response(&mut self, url: String, response: serde_json::Value) { /* ... */ }
    pub fn get_cached_api_response(&self, url: &str) -> Option<&serde_json::Value> { None }
    
    // UI methods (should be separate service)
    pub fn set_window_position(&mut self, window: String, x: i32, y: i32) { /* ... */ }
    pub fn get_window_position(&self, window: &str) -> Option<(i32, i32)> { None }
    pub fn set_theme_setting(&mut self, key: String, value: String) { /* ... */ }
    pub fn apply_theme(&mut self, theme_name: String) { /* ... */ }
    
    // Configuration methods (should be separate service)
    pub fn load_config(&mut self, config_path: PathBuf) -> Result<(), String> { Ok(()) }
    pub fn save_config(&self, config_path: PathBuf) -> Result<(), String> { Ok(()) }
    pub fn get_config_value(&self, key: &str) -> Option<&String> { None }
    pub fn set_config_value(&mut self, key: String, value: String) { /* ... */ }
    
    // Logging methods (should be separate service)
    pub fn log_message(&mut self, level: String, message: String) { /* ... */ }
    pub fn get_recent_logs(&self, count: usize) -> Vec<&String> { Vec::new() }
    pub fn clear_log_buffer(&mut self) { /* ... */ }
    
    // Metrics methods (should be separate service)
    pub fn record_metric(&mut self, name: String, value: f64) { /* ... */ }
    pub fn get_metric(&self, name: &str) -> Option<f64> { None }
    pub fn get_all_metrics(&self) -> &HashMap<String, f64> { &self.metrics }
    
    // Business logic methods (should be separate service)
    pub fn create_user_session(&mut self, user_id: String) -> String { "session_id".to_string() }
    pub fn validate_business_rules(&self, data: &BusinessData) -> Result<(), String> { Ok(()) }
    pub fn process_business_transaction(&mut self, transaction: BusinessTransaction) -> Result<(), String> { Ok(()) }
    
    // The main method that tries to do everything
    pub fn handle_everything(&mut self, request: ApplicationRequest) -> Result<ApplicationResponse, String> {
        // Log the request
        self.log_message("INFO".to_string(), format!("Handling request: {:?}", request));
        
        // Load configuration
        self.load_config(PathBuf::from("config.toml"))?;
        
        // Connect to database
        self.connect_database("postgresql://localhost/app".to_string());
        
        // Make API calls
        let api_response = self.make_http_request("https://api.example.com/data".to_string())?;
        
        // Process files
        self.read_file_cached(PathBuf::from("data.txt"))?;
        
        // Update UI
        self.set_window_position("main".to_string(), 100, 100);
        self.apply_theme("dark".to_string());
        
        // Record metrics
        self.record_metric("requests_processed".to_string(), 1.0);
        
        // Create user session
        let session_id = self.create_user_session(request.user_id.clone());
        
        // Validate business rules
        self.validate_business_rules(&request.business_data)?;
        
        // Process transaction
        self.process_business_transaction(request.transaction)?;
        
        // Cache results
        self.cache_query_result("SELECT * FROM users".to_string(), "user_data".to_string());
        
        Ok(ApplicationResponse {
            success: true,
            session_id,
            message: "Request processed successfully".to_string(),
        })
    }
}

// Supporting types for the god object
#[derive(Debug)]
pub struct UserSession {
    pub user_id: String,
    pub session_id: String,
    pub created_at: std::time::SystemTime,
}

#[derive(Debug)]
pub struct BusinessRule {
    pub id: String,
    pub condition: String,
    pub action: String,
}

#[derive(Debug)]
pub struct BusinessData {
    pub field1: String,
    pub field2: i32,
    pub field3: Vec<String>,
}

#[derive(Debug)]
pub struct BusinessTransaction {
    pub id: String,
    pub transaction_type: String,
    pub amount: f64,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug)]
pub struct ApplicationRequest {
    pub user_id: String,
    pub business_data: BusinessData,
    pub transaction: BusinessTransaction,
}

#[derive(Debug)]
pub struct ApplicationResponse {
    pub success: bool,
    pub session_id: String,
    pub message: String,
}
"#);

        // Dead Code Examples
        dead_code_examples.insert("unused_functions", r#"
// Active code that is used
pub fn active_function() -> i32 {
    42
}

pub fn another_active_function() -> String {
    active_function().to_string()
}

// Dead code that is never called
fn unused_private_function() -> bool {
    true
}

pub fn unused_public_function() -> Vec<String> {
    vec!["never".to_string(), "called".to_string()]
}

fn another_unused_function(param: i32) -> i32 {
    param * 2 + unused_helper_function()
}

fn unused_helper_function() -> i32 {
    999
}

// Dead structs and implementations
struct UnusedStruct {
    field1: String,
    field2: i32,
}

impl UnusedStruct {
    fn new(field1: String, field2: i32) -> Self {
        Self { field1, field2 }
    }
    
    fn unused_method(&self) -> String {
        format!("{}: {}", self.field1, self.field2)
    }
}

// Dead enums
enum UnusedEnum {
    Variant1,
    Variant2(String),
    Variant3 { field: i32 },
}

// Dead traits
trait UnusedTrait {
    fn unused_trait_method(&self) -> String;
}

// Dead constants
const UNUSED_CONSTANT: i32 = 123;
const ANOTHER_UNUSED_CONSTANT: &str = "unused";

// Dead macros
macro_rules! unused_macro {
    ($x:expr) => {
        $x * 2
    };
}

// Dead modules
mod unused_module {
    pub fn unused_module_function() -> String {
        "never called".to_string()
    }
    
    pub struct UnusedModuleStruct {
        pub value: i32,
    }
}

// Mixed usage - some functions used, others not
pub fn mixed_usage_entry() -> String {
    used_helper().to_string()
}

fn used_helper() -> i32 {
    42
}

fn unused_helper_in_mixed() -> i32 {
    99  // This helper is never called
}

// Generic dead code
fn unused_generic_function<T>(param: T) -> T {
    param
}

struct UnusedGenericStruct<T> {
    data: T,
}

impl<T> UnusedGenericStruct<T> {
    fn new(data: T) -> Self {
        Self { data }
    }
}
"#);

        // Circular Dependencies
        circular_dependencies.insert("module_a", r#"
// Module A depends on Module B
use crate::module_b::StructB;

pub struct StructA {
    pub value: i32,
    pub b_instance: Option<Box<StructB>>,
}

impl StructA {
    pub fn new(value: i32) -> Self {
        Self {
            value,
            b_instance: None,
        }
    }
    
    pub fn create_b(&mut self) -> StructB {
        StructB::new_from_a(self.value)
    }
    
    pub fn set_b(&mut self, b: StructB) {
        self.b_instance = Some(Box::new(b));
    }
}

pub fn create_circular_reference() -> (StructA, StructB) {
    let mut a = StructA::new(42);
    let b = a.create_b();
    a.set_b(b.clone());
    (a, b)
}
"#);

        circular_dependencies.insert("module_b", r#"
// Module B depends on Module A (creates circular dependency)
use crate::module_a::StructA;

#[derive(Clone)]
pub struct StructB {
    pub name: String,
    pub a_instance: Option<Box<StructA>>,
}

impl StructB {
    pub fn new(name: String) -> Self {
        Self {
            name,
            a_instance: None,
        }
    }
    
    pub fn new_from_a(value: i32) -> Self {
        Self {
            name: format!("B_from_A_{}", value),
            a_instance: None,
        }
    }
    
    pub fn create_a(&mut self) -> StructA {
        StructA::new(self.name.len() as i32)
    }
    
    pub fn set_a(&mut self, a: StructA) {
        self.a_instance = Some(Box::new(a));
    }
}

pub fn create_reverse_reference() -> (StructB, StructA) {
    let mut b = StructB::new("test".to_string());
    let a = b.create_a();
    b.set_a(a.clone());
    (b, a)
}
"#);

        // Clean Code Examples
        clean_code_examples.insert("well_structured", r#"
//! Well-structured code example demonstrating good architectural patterns

use std::collections::HashMap;
use std::sync::Arc;

/// Represents a user in the system
#[derive(Debug, Clone)]
pub struct User {
    pub id: String,
    pub name: String,
    pub email: String,
}

/// Repository trait for data access abstraction
pub trait UserRepository {
    fn find_by_id(&self, id: &str) -> Option<User>;
    fn find_by_email(&self, email: &str) -> Option<User>;
    fn save(&mut self, user: User) -> Result<(), String>;
    fn delete(&mut self, id: &str) -> Result<(), String>;
}

/// In-memory implementation of UserRepository
pub struct InMemoryUserRepository {
    users: HashMap<String, User>,
}

impl InMemoryUserRepository {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
        }
    }
}

impl UserRepository for InMemoryUserRepository {
    fn find_by_id(&self, id: &str) -> Option<User> {
        self.users.get(id).cloned()
    }
    
    fn find_by_email(&self, email: &str) -> Option<User> {
        self.users.values()
            .find(|user| user.email == email)
            .cloned()
    }
    
    fn save(&mut self, user: User) -> Result<(), String> {
        self.users.insert(user.id.clone(), user);
        Ok(())
    }
    
    fn delete(&mut self, id: &str) -> Result<(), String> {
        if self.users.remove(id).is_some() {
            Ok(())
        } else {
            Err(format!("User with id {} not found", id))
        }
    }
}

/// Service layer for business logic
pub struct UserService {
    repository: Arc<dyn UserRepository + Send + Sync>,
}

impl UserService {
    pub fn new(repository: Arc<dyn UserRepository + Send + Sync>) -> Self {
        Self { repository }
    }
    
    pub fn create_user(&self, name: String, email: String) -> Result<User, String> {
        // Validate email format
        if !email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        // Check if user already exists
        if self.repository.find_by_email(&email).is_some() {
            return Err("User with this email already exists".to_string());
        }
        
        // Create new user
        let user = User {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            email,
        };
        
        // Note: In real implementation, we would handle the mutable repository differently
        // This is simplified for the example
        Ok(user)
    }
    
    pub fn get_user(&self, id: &str) -> Result<User, String> {
        self.repository.find_by_id(id)
            .ok_or_else(|| format!("User with id {} not found", id))
    }
    
    pub fn update_user_email(&self, id: &str, new_email: String) -> Result<User, String> {
        if !new_email.contains('@') {
            return Err("Invalid email format".to_string());
        }
        
        let mut user = self.get_user(id)?;
        user.email = new_email;
        
        // Note: In real implementation, we would save the updated user
        Ok(user)
    }
}

/// Configuration for the application
#[derive(Debug, Clone)]
pub struct AppConfig {
    pub database_url: String,
    pub port: u16,
    pub log_level: String,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            database_url: "memory://".to_string(),
            port: 8080,
            log_level: "info".to_string(),
        }
    }
}

/// Application factory for dependency injection
pub struct AppFactory;

impl AppFactory {
    pub fn create_user_service(config: &AppConfig) -> UserService {
        let repository = Arc::new(InMemoryUserRepository::new());
        UserService::new(repository)
    }
    
    pub fn create_app_config() -> AppConfig {
        AppConfig::default()
    }
}

// Example of good error handling
#[derive(Debug, thiserror::Error)]
pub enum UserError {
    #[error("User not found: {id}")]
    NotFound { id: String },
    
    #[error("Invalid email format: {email}")]
    InvalidEmail { email: String },
    
    #[error("User already exists: {email}")]
    AlreadyExists { email: String },
    
    #[error("Database error: {message}")]
    Database { message: String },
}

// Example of good testing structure
#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_user_creation() {
        let mut repo = InMemoryUserRepository::new();
        let user = User {
            id: "1".to_string(),
            name: "Test User".to_string(),
            email: "test@example.com".to_string(),
        };
        
        assert!(repo.save(user.clone()).is_ok());
        assert_eq!(repo.find_by_id("1"), Some(user));
    }
    
    #[test]
    fn test_user_service_validation() {
        let repo = Arc::new(InMemoryUserRepository::new());
        let service = UserService::new(repo);
        
        // Test invalid email
        let result = service.create_user("Test".to_string(), "invalid-email".to_string());
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid email format"));
        
        // Test valid user creation
        let result = service.create_user("Test".to_string(), "test@example.com".to_string());
        assert!(result.is_ok());
    }
}
"#);

        // Performance Test Code
        performance_test_code.insert("large_data_structures", r#"
//! Performance test code with large data structures

use std::collections::{HashMap, Vec};

/// Large data structure for performance testing
pub struct LargeDataStructure {
    pub primary_data: Vec<DataItem>,
    pub index_by_id: HashMap<String, usize>,
    pub index_by_category: HashMap<String, Vec<usize>>,
    pub metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DataItem {
    pub id: String,
    pub category: String,
    pub value: f64,
    pub data: Vec<u8>,
    pub tags: Vec<String>,
    pub metadata: HashMap<String, String>,
}

impl LargeDataStructure {
    pub fn new() -> Self {
        Self {
            primary_data: Vec::new(),
            index_by_id: HashMap::new(),
            index_by_category: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            primary_data: Vec::with_capacity(capacity),
            index_by_id: HashMap::with_capacity(capacity),
            index_by_category: HashMap::new(),
            metadata: HashMap::new(),
        }
    }
    
    pub fn add_item(&mut self, item: DataItem) {
        let index = self.primary_data.len();
        
        // Update ID index
        self.index_by_id.insert(item.id.clone(), index);
        
        // Update category index
        self.index_by_category
            .entry(item.category.clone())
            .or_insert_with(Vec::new)
            .push(index);
        
        // Add to primary data
        self.primary_data.push(item);
    }
    
    pub fn get_by_id(&self, id: &str) -> Option<&DataItem> {
        self.index_by_id.get(id)
            .and_then(|&index| self.primary_data.get(index))
    }
    
    pub fn get_by_category(&self, category: &str) -> Vec<&DataItem> {
        self.index_by_category.get(category)
            .map(|indices| {
                indices.iter()
                    .filter_map(|&index| self.primary_data.get(index))
                    .collect()
            })
            .unwrap_or_default()
    }
    
    pub fn bulk_process(&mut self) -> ProcessingResult {
        let mut result = ProcessingResult::new();
        
        for (index, item) in self.primary_data.iter_mut().enumerate() {
            // Simulate complex processing
            item.value *= 1.1;
            item.data.extend_from_slice(&[index as u8; 100]);
            
            // Update result statistics
            result.processed_count += 1;
            result.total_value += item.value;
            
            if item.tags.len() > 5 {
                result.complex_items += 1;
            }
        }
        
        result.average_value = result.total_value / result.processed_count as f64;
        result
    }
    
    pub fn memory_intensive_operation(&self) -> Vec<ProcessedData> {
        self.primary_data.iter()
            .map(|item| ProcessedData {
                id: item.id.clone(),
                processed_value: item.value * 2.0,
                derived_data: vec![0u8; 1024], // 1KB per item
                computed_hash: self.compute_hash(&item.data),
            })
            .collect()
    }
    
    fn compute_hash(&self, data: &[u8]) -> u64 {
        // Simple hash computation for testing
        data.iter().map(|&b| b as u64).sum()
    }
}

#[derive(Debug)]
pub struct ProcessingResult {
    pub processed_count: usize,
    pub total_value: f64,
    pub average_value: f64,
    pub complex_items: usize,
}

impl ProcessingResult {
    pub fn new() -> Self {
        Self {
            processed_count: 0,
            total_value: 0.0,
            average_value: 0.0,
            complex_items: 0,
        }
    }
}

#[derive(Debug)]
pub struct ProcessedData {
    pub id: String,
    pub processed_value: f64,
    pub derived_data: Vec<u8>,
    pub computed_hash: u64,
}

/// Performance test helper functions
pub mod performance_helpers {
    use super::*;
    use std::time::Instant;
    
    pub fn create_large_dataset(size: usize) -> LargeDataStructure {
        let mut structure = LargeDataStructure::with_capacity(size);
        
        for i in 0..size {
            let item = DataItem {
                id: format!("item_{}", i),
                category: format!("category_{}", i % 10),
                value: (i as f64) * 1.5,
                data: vec![i as u8; 256],
                tags: (0..=(i % 10)).map(|j| format!("tag_{}", j)).collect(),
                metadata: {
                    let mut map = HashMap::new();
                    map.insert("created_at".to_string(), format!("2023-01-01T{}:00:00Z", i % 24));
                    map.insert("priority".to_string(), (i % 5).to_string());
                    map
                },
            };
            structure.add_item(item);
        }
        
        structure
    }
    
    pub fn benchmark_operation<F, R>(name: &str, operation: F) -> (R, std::time::Duration)
    where
        F: FnOnce() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        
        println!("Benchmark '{}': {:?}", name, duration);
        (result, duration)
    }
    
    pub fn memory_usage_test(iterations: usize) -> Vec<usize> {
        let mut memory_usage = Vec::new();
        
        for i in 0..iterations {
            let dataset = create_large_dataset(1000 * (i + 1));
            
            // Simulate memory usage measurement
            let estimated_memory = dataset.primary_data.len() * std::mem::size_of::<DataItem>()
                + dataset.index_by_id.len() * (std::mem::size_of::<String>() + std::mem::size_of::<usize>());
            
            memory_usage.push(estimated_memory);
        }
        
        memory_usage
    }
}
"#);

        Self {
            god_objects,
            dead_code_examples,
            circular_dependencies,
            clean_code_examples,
            performance_test_code,
        }
    }
}

/// Python-specific test fixtures
pub struct PythonFixtures {
    pub god_objects: HashMap<&'static str, &'static str>,
    pub dead_code_examples: HashMap<&'static str, &'static str>,
    pub clean_code_examples: HashMap<&'static str, &'static str>,
}

impl PythonFixtures {
    pub fn new() -> Self {
        let mut god_objects = HashMap::new();
        let mut dead_code_examples = HashMap::new();
        let mut clean_code_examples = HashMap::new();
        
        god_objects.insert("massive_class", r#"
"""
God Object example in Python - handles too many responsibilities
"""
import json
import sqlite3
import requests
import logging
from typing import Dict, List, Any, Optional
from datetime import datetime

class MassiveApplicationManager:
    """
    A god object that violates the Single Responsibility Principle
    by handling database, network, file operations, UI state, and business logic
    """
    
    def __init__(self):
        # Database management
        self.db_connections = []
        self.query_cache = {}
        
        # File system management
        self.open_files = {}
        self.file_cache = {}
        
        # Network management
        self.http_sessions = {}
        self.api_cache = {}
        
        # UI state management
        self.window_states = {}
        self.ui_components = {}
        
        # Configuration management
        self.config = {}
        self.environment_vars = {}
        
        # Logging and monitoring
        self.log_buffer = []
        self.metrics = {}
        
        # Business logic state
        self.user_sessions = {}
        self.business_rules = []
        
        # Initialize logger
        self.logger = logging.getLogger(__name__)
    
    # Database methods (should be separate service)
    def connect_database(self, connection_string: str) -> bool:
        """Connect to database"""
        try:
            conn = sqlite3.connect(connection_string)
            self.db_connections.append(conn)
            return True
        except Exception as e:
            self.log_error(f"Database connection failed: {e}")
            return False
    
    def execute_query(self, query: str, params: tuple = ()) -> List[Dict]:
        """Execute database query"""
        if not self.db_connections:
            raise RuntimeError("No database connection available")
        
        conn = self.db_connections[0]
        cursor = conn.cursor()
        cursor.execute(query, params)
        
        columns = [description[0] for description in cursor.description]
        results = [dict(zip(columns, row)) for row in cursor.fetchall()]
        
        # Cache the result
        cache_key = f"{query}:{params}"
        self.query_cache[cache_key] = results
        
        return results
    
    def cache_query_result(self, query: str, result: List[Dict]) -> None:
        """Cache query result"""
        self.query_cache[query] = result
    
    # File system methods (should be separate service)
    def read_file(self, file_path: str) -> str:
        """Read file with caching"""
        if file_path in self.file_cache:
            return self.file_cache[file_path]
        
        try:
            with open(file_path, 'r') as f:
                content = f.read()
                self.file_cache[file_path] = content
                return content
        except Exception as e:
            self.log_error(f"Failed to read file {file_path}: {e}")
            raise
    
    def write_file(self, file_path: str, content: str) -> bool:
        """Write content to file"""
        try:
            with open(file_path, 'w') as f:
                f.write(content)
                self.file_cache[file_path] = content
                return True
        except Exception as e:
            self.log_error(f"Failed to write file {file_path}: {e}")
            return False
    
    # Network methods (should be separate service)
    def make_http_request(self, url: str, method: str = 'GET', data: Dict = None) -> Dict:
        """Make HTTP request"""
        try:
            session = requests.Session()
            response = session.request(method, url, json=data)
            response.raise_for_status()
            
            result = response.json()
            
            # Cache the response
            cache_key = f"{method}:{url}:{json.dumps(data) if data else ''}"
            self.api_cache[cache_key] = result
            
            return result
        except Exception as e:
            self.log_error(f"HTTP request failed: {e}")
            raise
    
    def get_cached_api_response(self, url: str, method: str = 'GET', data: Dict = None) -> Optional[Dict]:
        """Get cached API response"""
        cache_key = f"{method}:{url}:{json.dumps(data) if data else ''}"
        return self.api_cache.get(cache_key)
    
    # UI methods (should be separate service)
    def set_window_state(self, window_id: str, state: Dict) -> None:
        """Set window state"""
        self.window_states[window_id] = state
    
    def get_window_state(self, window_id: str) -> Optional[Dict]:
        """Get window state"""
        return self.window_states.get(window_id)
    
    def register_ui_component(self, component_id: str, component: Any) -> None:
        """Register UI component"""
        self.ui_components[component_id] = component
    
    def update_ui_component(self, component_id: str, properties: Dict) -> bool:
        """Update UI component properties"""
        if component_id in self.ui_components:
            component = self.ui_components[component_id]
            for key, value in properties.items():
                setattr(component, key, value)
            return True
        return False
    
    # Configuration methods (should be separate service)
    def load_config(self, config_path: str) -> bool:
        """Load configuration from file"""
        try:
            content = self.read_file(config_path)
            self.config = json.loads(content)
            return True
        except Exception as e:
            self.log_error(f"Failed to load config: {e}")
            return False
    
    def save_config(self, config_path: str) -> bool:
        """Save configuration to file"""
        try:
            content = json.dumps(self.config, indent=2)
            return self.write_file(config_path, content)
        except Exception as e:
            self.log_error(f"Failed to save config: {e}")
            return False
    
    def get_config_value(self, key: str, default: Any = None) -> Any:
        """Get configuration value"""
        return self.config.get(key, default)
    
    def set_config_value(self, key: str, value: Any) -> None:
        """Set configuration value"""
        self.config[key] = value
    
    # Logging methods (should be separate service)
    def log_message(self, level: str, message: str) -> None:
        """Log message"""
        timestamp = datetime.now().isoformat()
        log_entry = f"[{timestamp}] {level}: {message}"
        self.log_buffer.append(log_entry)
        
        # Also log to Python logger
        getattr(self.logger, level.lower(), self.logger.info)(message)
    
    def log_error(self, message: str) -> None:
        """Log error message"""
        self.log_message("ERROR", message)
    
    def log_info(self, message: str) -> None:
        """Log info message"""
        self.log_message("INFO", message)
    
    def get_recent_logs(self, count: int = 100) -> List[str]:
        """Get recent log entries"""
        return self.log_buffer[-count:]
    
    # Metrics methods (should be separate service)
    def record_metric(self, name: str, value: float, tags: Dict[str, str] = None) -> None:
        """Record metric value"""
        timestamp = datetime.now().isoformat()
        metric_entry = {
            'name': name,
            'value': value,
            'timestamp': timestamp,
            'tags': tags or {}
        }
        
        if name not in self.metrics:
            self.metrics[name] = []
        
        self.metrics[name].append(metric_entry)
    
    def get_metric_values(self, name: str) -> List[float]:
        """Get all values for a metric"""
        if name in self.metrics:
            return [entry['value'] for entry in self.metrics[name]]
        return []
    
    def get_metric_average(self, name: str) -> Optional[float]:
        """Get average value for a metric"""
        values = self.get_metric_values(name)
        return sum(values) / len(values) if values else None
    
    # Business logic methods (should be separate service)
    def create_user_session(self, user_id: str, session_data: Dict = None) -> str:
        """Create user session"""
        import uuid
        session_id = str(uuid.uuid4())
        
        session = {
            'user_id': user_id,
            'session_id': session_id,
            'created_at': datetime.now().isoformat(),
            'data': session_data or {}
        }
        
        self.user_sessions[session_id] = session
        return session_id
    
    def validate_business_rules(self, data: Dict) -> List[str]:
        """Validate data against business rules"""
        errors = []
        
        for rule in self.business_rules:
            if not self._evaluate_business_rule(rule, data):
                errors.append(f"Business rule violation: {rule.get('description', 'Unknown rule')}")
        
        return errors
    
    def _evaluate_business_rule(self, rule: Dict, data: Dict) -> bool:
        """Evaluate a single business rule"""
        # Simplified rule evaluation
        rule_type = rule.get('type', 'unknown')
        
        if rule_type == 'required_field':
            field_name = rule.get('field')
            return field_name in data and data[field_name] is not None
        
        if rule_type == 'range_check':
            field_name = rule.get('field')
            min_value = rule.get('min', float('-inf'))
            max_value = rule.get('max', float('inf'))
            
            if field_name in data:
                value = data[field_name]
                return min_value <= value <= max_value
        
        return True
    
    def process_business_transaction(self, transaction_data: Dict) -> Dict:
        """Process business transaction"""
        transaction_id = transaction_data.get('id', 'unknown')
        
        # Log transaction start
        self.log_info(f"Processing transaction: {transaction_id}")
        
        # Record metric
        self.record_metric('transactions_processed', 1.0)
        
        # Validate business rules
        validation_errors = self.validate_business_rules(transaction_data)
        if validation_errors:
            self.log_error(f"Transaction validation failed: {validation_errors}")
            return {
                'success': False,
                'errors': validation_errors,
                'transaction_id': transaction_id
            }
        
        # Simulate processing
        processing_result = {
            'success': True,
            'transaction_id': transaction_id,
            'processed_at': datetime.now().isoformat(),
            'result_data': {'status': 'completed'}
        }
        
        # Log success
        self.log_info(f"Transaction processed successfully: {transaction_id}")
        
        return processing_result
    
    # The main method that tries to do everything
    def handle_application_request(self, request_data: Dict) -> Dict:
        """Main method that handles everything - violates SRP"""
        request_id = request_data.get('id', 'unknown')
        
        try:
            # Log request
            self.log_info(f"Handling application request: {request_id}")
            
            # Load configuration
            if not self.load_config('app_config.json'):
                return {'success': False, 'error': 'Failed to load configuration'}
            
            # Connect to database
            db_url = self.get_config_value('database_url', 'data.db')
            if not self.connect_database(db_url):
                return {'success': False, 'error': 'Failed to connect to database'}
            
            # Make API calls
            api_url = self.get_config_value('external_api_url')
            if api_url:
                try:
                    api_response = self.make_http_request(api_url, 'GET')
                    self.log_info(f"API response received: {len(str(api_response))} characters")
                except Exception as e:
                    self.log_error(f"API call failed: {e}")
            
            # Process files
            input_file = request_data.get('input_file')
            if input_file:
                try:
                    file_content = self.read_file(input_file)
                    self.log_info(f"File processed: {len(file_content)} characters")
                except Exception as e:
                    self.log_error(f"File processing failed: {e}")
            
            # Update UI state
            window_id = request_data.get('window_id', 'main')
            self.set_window_state(window_id, {'last_request': request_id})
            
            # Record metrics
            self.record_metric('requests_handled', 1.0)
            
            # Create user session
            user_id = request_data.get('user_id')
            if user_id:
                session_id = self.create_user_session(user_id, request_data.get('session_data'))
                self.log_info(f"User session created: {session_id}")
            
            # Process business transaction
            transaction_data = request_data.get('transaction')
            transaction_result = None
            if transaction_data:
                transaction_result = self.process_business_transaction(transaction_data)
            
            # Execute database queries
            user_query = "SELECT * FROM users WHERE active = 1"
            try:
                users = self.execute_query(user_query)
                self.log_info(f"Found {len(users)} active users")
            except Exception as e:
                self.log_error(f"Database query failed: {e}")
                users = []
            
            # Prepare response
            response = {
                'success': True,
                'request_id': request_id,
                'processed_at': datetime.now().isoformat(),
                'user_count': len(users),
                'transaction_result': transaction_result,
                'metrics': {
                    'requests_handled': len(self.get_metric_values('requests_handled')),
                    'transactions_processed': len(self.get_metric_values('transactions_processed'))
                }
            }
            
            self.log_info(f"Request processed successfully: {request_id}")
            return response
            
        except Exception as e:
            self.log_error(f"Request processing failed: {e}")
            return {
                'success': False,
                'request_id': request_id,
                'error': str(e),
                'processed_at': datetime.now().isoformat()
            }

# Supporting classes for the god object
class BusinessRule:
    def __init__(self, rule_type: str, description: str, **kwargs):
        self.type = rule_type
        self.description = description
        self.parameters = kwargs
    
    def to_dict(self) -> Dict:
        return {
            'type': self.type,
            'description': self.description,
            **self.parameters
        }
"#);

        dead_code_examples.insert("unused_functions", r#"
"""
Dead code examples in Python
"""

# Active functions that are used
def active_function():
    """This function is called"""
    return 42

def another_active_function():
    """This function calls active_function"""
    return str(active_function())

# Dead code that is never called
def unused_function():
    """This function is never called"""
    return "unused"

def another_unused_function(param):
    """Another unused function"""
    return param * 2 + unused_helper()

def unused_helper():
    """Helper function that's never used"""
    return 999

# Dead classes
class UnusedClass:
    """Class that's never instantiated"""
    
    def __init__(self, value):
        self.value = value
    
    def unused_method(self):
        """Method that's never called"""
        return self.value * 2
    
    @staticmethod
    def unused_static_method():
        """Static method that's never called"""
        return "static"
    
    @classmethod
    def unused_class_method(cls):
        """Class method that's never called"""
        return cls()

# Dead global variables
UNUSED_CONSTANT = 123
ANOTHER_UNUSED_CONSTANT = "unused"

# Dead decorators
def unused_decorator(func):
    """Decorator that's never used"""
    def wrapper(*args, **kwargs):
        print(f"Calling {func.__name__}")
        return func(*args, **kwargs)
    return wrapper

# Dead generators
def unused_generator():
    """Generator that's never used"""
    for i in range(10):
        yield i * 2

# Dead context managers
class UnusedContextManager:
    """Context manager that's never used"""
    
    def __enter__(self):
        print("Entering context")
        return self
    
    def __exit__(self, exc_type, exc_val, exc_tb):
        print("Exiting context")

# Dead async functions
async def unused_async_function():
    """Async function that's never awaited"""
    import asyncio
    await asyncio.sleep(1)
    return "async result"

# Dead exception classes
class UnusedException(Exception):
    """Exception that's never raised"""
    pass

class AnotherUnusedException(ValueError):
    """Another unused exception"""
    def __init__(self, message):
        super().__init__(message)
        self.custom_data = "unused"

# Mixed usage - some used, some not
def mixed_usage_entry():
    """Entry point that uses some helpers"""
    return used_helper()

def used_helper():
    """Helper that is actually used"""
    return 42

def unused_helper_in_mixed():
    """Helper in mixed module that's never used"""
    return 99

# Dead modules and imports
try:
    import unused_module  # This import is never used
except ImportError:
    unused_module = None

# Dead lambda functions
unused_lambda = lambda x: x * 2

# Dead list comprehensions assigned to variables
unused_list_comp = [i * 2 for i in range(100)]

# Dead dictionary
unused_dict = {
    "key1": "value1",
    "key2": "value2",
    "key3": unused_function  # References dead function
}
"#);

        clean_code_examples.insert("well_structured", r#"
"""
Well-structured Python code demonstrating good architectural patterns
"""
from abc import ABC, abstractmethod
from typing import List, Optional, Dict, Any
from dataclasses import dataclass
from datetime import datetime
import uuid

@dataclass
class User:
    """Represents a user in the system"""
    id: str
    name: str
    email: str
    created_at: datetime

    @classmethod
    def create(cls, name: str, email: str) -> 'User':
        """Factory method to create a new user"""
        return cls(
            id=str(uuid.uuid4()),
            name=name,
            email=email,
            created_at=datetime.now()
        )

class UserRepository(ABC):
    """Abstract repository for user data access"""
    
    @abstractmethod
    def find_by_id(self, user_id: str) -> Optional[User]:
        """Find user by ID"""
        pass
    
    @abstractmethod
    def find_by_email(self, email: str) -> Optional[User]:
        """Find user by email"""
        pass
    
    @abstractmethod
    def save(self, user: User) -> None:
        """Save user"""
        pass
    
    @abstractmethod
    def delete(self, user_id: str) -> bool:
        """Delete user by ID"""
        pass

class InMemoryUserRepository(UserRepository):
    """In-memory implementation of UserRepository"""
    
    def __init__(self):
        self._users: Dict[str, User] = {}
    
    def find_by_id(self, user_id: str) -> Optional[User]:
        """Find user by ID"""
        return self._users.get(user_id)
    
    def find_by_email(self, email: str) -> Optional[User]:
        """Find user by email"""
        for user in self._users.values():
            if user.email == email:
                return user
        return None
    
    def save(self, user: User) -> None:
        """Save user"""
        self._users[user.id] = user
    
    def delete(self, user_id: str) -> bool:
        """Delete user by ID"""
        if user_id in self._users:
            del self._users[user_id]
            return True
        return False

class EmailValidator:
    """Email validation service"""
    
    @staticmethod
    def is_valid(email: str) -> bool:
        """Validate email format"""
        return '@' in email and '.' in email.split('@')[1]

class UserService:
    """Service layer for user business logic"""
    
    def __init__(self, repository: UserRepository, email_validator: EmailValidator):
        self._repository = repository
        self._email_validator = email_validator
    
    def create_user(self, name: str, email: str) -> User:
        """Create a new user"""
        # Validate input
        if not name.strip():
            raise ValueError("Name cannot be empty")
        
        if not self._email_validator.is_valid(email):
            raise ValueError("Invalid email format")
        
        # Check if user already exists
        existing_user = self._repository.find_by_email(email)
        if existing_user:
            raise ValueError("User with this email already exists")
        
        # Create and save user
        user = User.create(name.strip(), email.lower())
        self._repository.save(user)
        return user
    
    def get_user(self, user_id: str) -> User:
        """Get user by ID"""
        user = self._repository.find_by_id(user_id)
        if not user:
            raise ValueError(f"User with ID {user_id} not found")
        return user
    
    def update_user_email(self, user_id: str, new_email: str) -> User:
        """Update user email"""
        if not self._email_validator.is_valid(new_email):
            raise ValueError("Invalid email format")
        
        user = self.get_user(user_id)
        
        # Check if email is already taken by another user
        existing_user = self._repository.find_by_email(new_email)
        if existing_user and existing_user.id != user_id:
            raise ValueError("Email is already taken by another user")
        
        # Update email
        user.email = new_email.lower()
        self._repository.save(user)
        return user
    
    def delete_user(self, user_id: str) -> None:
        """Delete user"""
        user = self.get_user(user_id)  # Verify user exists
        self._repository.delete(user_id)

@dataclass
class AppConfig:
    """Application configuration"""
    database_url: str = "memory://"
    port: int = 8080
    log_level: str = "INFO"
    debug: bool = False

class AppFactory:
    """Factory for creating application components"""
    
    @staticmethod
    def create_user_service(config: AppConfig) -> UserService:
        """Create user service with dependencies"""
        repository = InMemoryUserRepository()
        email_validator = EmailValidator()
        return UserService(repository, email_validator)
    
    @staticmethod
    def create_config() -> AppConfig:
        """Create default configuration"""
        return AppConfig()

# Custom exceptions for better error handling
class UserError(Exception):
    """Base exception for user-related errors"""
    pass

class UserNotFoundError(UserError):
    """Raised when user is not found"""
    
    def __init__(self, user_id: str):
        super().__init__(f"User with ID {user_id} not found")
        self.user_id = user_id

class DuplicateEmailError(UserError):
    """Raised when email already exists"""
    
    def __init__(self, email: str):
        super().__init__(f"User with email {email} already exists")
        self.email = email

class InvalidEmailError(UserError):
    """Raised when email format is invalid"""
    
    def __init__(self, email: str):
        super().__init__(f"Invalid email format: {email}")
        self.email = email

# Example of proper testing structure
import unittest

class TestUserService(unittest.TestCase):
    """Test cases for UserService"""
    
    def setUp(self):
        """Set up test dependencies"""
        self.repository = InMemoryUserRepository()
        self.email_validator = EmailValidator()
        self.service = UserService(self.repository, self.email_validator)
    
    def test_create_user_success(self):
        """Test successful user creation"""
        user = self.service.create_user("John Doe", "john@example.com")
        
        self.assertIsNotNone(user.id)
        self.assertEqual(user.name, "John Doe")
        self.assertEqual(user.email, "john@example.com")
        self.assertIsInstance(user.created_at, datetime)
    
    def test_create_user_invalid_email(self):
        """Test user creation with invalid email"""
        with self.assertRaises(ValueError) as context:
            self.service.create_user("John Doe", "invalid-email")
        
        self.assertIn("Invalid email format", str(context.exception))
    
    def test_create_user_duplicate_email(self):
        """Test user creation with duplicate email"""
        # Create first user
        self.service.create_user("John Doe", "john@example.com")
        
        # Try to create second user with same email
        with self.assertRaises(ValueError) as context:
            self.service.create_user("Jane Doe", "john@example.com")
        
        self.assertIn("already exists", str(context.exception))
    
    def test_get_user_not_found(self):
        """Test getting non-existent user"""
        with self.assertRaises(ValueError) as context:
            self.service.get_user("non-existent-id")
        
        self.assertIn("not found", str(context.exception))

if __name__ == "__main__":
    unittest.main()
"#);

        Self {
            god_objects,
            dead_code_examples,
            clean_code_examples,
        }
    }
}

/// JavaScript-specific test fixtures
pub struct JavaScriptFixtures {
    pub god_objects: HashMap<&'static str, &'static str>,
    pub dead_code_examples: HashMap<&'static str, &'static str>,
    pub clean_code_examples: HashMap<&'static str, &'static str>,
}

impl JavaScriptFixtures {
    pub fn new() -> Self {
        // Implementation similar to above but for JavaScript
        HashMap::new().into()
    }
}

/// TypeScript-specific test fixtures  
pub struct TypeScriptFixtures {
    pub god_objects: HashMap<&'static str, &'static str>,
    pub dead_code_examples: HashMap<&'static str, &'static str>,
    pub clean_code_examples: HashMap<&'static str, &'static str>,
}

impl TypeScriptFixtures {
    pub fn new() -> Self {
        // Implementation similar to above but for TypeScript
        HashMap::new().into()
    }
}

/// Multi-language project fixtures
pub struct MultiLanguageFixtures {
    pub microservices_project: HashMap<&'static str, &'static str>,
    pub monorepo_project: HashMap<&'static str, &'static str>,
    pub cross_language_dependencies: HashMap<&'static str, &'static str>,
}

impl MultiLanguageFixtures {
    pub fn new() -> Self {
        // Implementation for multi-language test scenarios
        Self {
            microservices_project: HashMap::new(),
            monorepo_project: HashMap::new(), 
            cross_language_dependencies: HashMap::new(),
        }
    }
}

/// Helper functions for creating test projects
pub mod test_project_helpers {
    use super::*;
    use std::fs;
    use std::path::Path;
    use tempfile::TempDir;
    
    /// Create a temporary test project with realistic structure
    pub async fn create_test_project(fixtures: &TestFixtures, project_type: &str) -> std::io::Result<TempDir> {
        let temp_dir = TempDir::new()?;
        let project_path = temp_dir.path();
        
        match project_type {
            "rust_with_issues" => create_rust_project_with_issues(project_path, &fixtures.rust_fixtures).await?,
            "python_with_issues" => create_python_project_with_issues(project_path, &fixtures.python_fixtures).await?,
            "clean_rust" => create_clean_rust_project(project_path, &fixtures.rust_fixtures).await?,
            "multi_language" => create_multi_language_project(project_path, fixtures).await?,
            _ => return Err(std::io::Error::new(std::io::ErrorKind::InvalidInput, "Unknown project type")),
        }
        
        Ok(temp_dir)
    }
    
    async fn create_rust_project_with_issues(path: &Path, fixtures: &RustFixtures) -> std::io::Result<()> {
        let src_dir = path.join("src");
        fs::create_dir_all(&src_dir)?;
        
        // Write god object
        if let Some(god_object_code) = fixtures.god_objects.get("massive_struct") {
            fs::write(src_dir.join("god_object.rs"), god_object_code)?;
        }
        
        // Write dead code
        if let Some(dead_code) = fixtures.dead_code_examples.get("unused_functions") {
            fs::write(src_dir.join("dead_code.rs"), dead_code)?;
        }
        
        // Write circular dependencies
        if let Some(module_a) = fixtures.circular_dependencies.get("module_a") {
            fs::write(src_dir.join("module_a.rs"), module_a)?;
        }
        if let Some(module_b) = fixtures.circular_dependencies.get("module_b") {
            fs::write(src_dir.join("module_b.rs"), module_b)?;
        }
        
        // Write main.rs
        let main_content = r#"
mod god_object;
mod dead_code;
mod module_a;
mod module_b;

use god_object::MassiveApplicationManager;

fn main() {
    let mut manager = MassiveApplicationManager::new();
    let request = god_object::ApplicationRequest {
        user_id: "test_user".to_string(),
        business_data: god_object::BusinessData {
            field1: "test".to_string(),
            field2: 42,
            field3: vec!["tag1".to_string(), "tag2".to_string()],
        },
        transaction: god_object::BusinessTransaction {
            id: "tx_123".to_string(),
            transaction_type: "payment".to_string(),
            amount: 100.0,
            metadata: std::collections::HashMap::new(),
        },
    };
    
    match manager.handle_everything(request) {
        Ok(response) => println!("Success: {:?}", response),
        Err(error) => eprintln!("Error: {}", error),
    }
}
"#;
        fs::write(src_dir.join("main.rs"), main_content)?;
        
        // Write Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-project-with-issues"
version = "0.1.0"
edition = "2021"

[dependencies]
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"
"#;
        fs::write(path.join("Cargo.toml"), cargo_toml)?;
        
        Ok(())
    }
    
    async fn create_python_project_with_issues(path: &Path, fixtures: &PythonFixtures) -> std::io::Result<()> {
        // Write god object
        if let Some(god_object_code) = fixtures.god_objects.get("massive_class") {
            fs::write(path.join("god_object.py"), god_object_code)?;
        }
        
        // Write dead code
        if let Some(dead_code) = fixtures.dead_code_examples.get("unused_functions") {
            fs::write(path.join("dead_code.py"), dead_code)?;
        }
        
        // Write main script
        let main_content = r#"
#!/usr/bin/env python3
"""
Main script demonstrating usage of problematic code
"""

from god_object import MassiveApplicationManager
import dead_code

def main():
    """Main function"""
    manager = MassiveApplicationManager()
    
    # Use the god object
    request_data = {
        'id': 'req_123',
        'user_id': 'user_456',
        'input_file': 'test_input.txt',
        'window_id': 'main_window',
        'transaction': {
            'id': 'tx_789',
            'type': 'payment',
            'amount': 100.0
        },
        'session_data': {
            'theme': 'dark',
            'language': 'en'
        }
    }
    
    result = manager.handle_application_request(request_data)
    print(f"Request result: {result}")
    
    # This calls active code (not dead)
    print(f"Active function result: {dead_code.another_active_function()}")

if __name__ == "__main__":
    main()
"#;
        fs::write(path.join("main.py"), main_content)?;
        
        // Write requirements.txt
        let requirements = r#"
requests>=2.25.0
sqlite3
"#;
        fs::write(path.join("requirements.txt"), requirements)?;
        
        Ok(())
    }
    
    async fn create_clean_rust_project(path: &Path, fixtures: &RustFixtures) -> std::io::Result<()> {
        let src_dir = path.join("src");
        fs::create_dir_all(&src_dir)?;
        
        // Write clean code
        if let Some(clean_code) = fixtures.clean_code_examples.get("well_structured") {
            fs::write(src_dir.join("lib.rs"), clean_code)?;
        }
        
        // Write main.rs
        let main_content = r#"
use test_clean_project::{AppFactory, User};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let config = AppFactory::create_app_config();
    let user_service = AppFactory::create_user_service(&config);
    
    // Create a user
    let user = user_service.create_user(
        "John Doe".to_string(),
        "john@example.com".to_string()
    )?;
    
    println!("Created user: {:?}", user);
    
    // Get the user
    let retrieved_user = user_service.get_user(&user.id)?;
    println!("Retrieved user: {:?}", retrieved_user);
    
    Ok(())
}
"#;
        fs::write(src_dir.join("main.rs"), main_content)?;
        
        // Write Cargo.toml
        let cargo_toml = r#"
[package]
name = "test-clean-project"
version = "0.1.0"
edition = "2021"

[dependencies]
uuid = { version = "1.0", features = ["v4"] }
thiserror = "1.0"
"#;
        fs::write(path.join("Cargo.toml"), cargo_toml)?;
        
        Ok(())
    }
    
    async fn create_multi_language_project(path: &Path, fixtures: &TestFixtures) -> std::io::Result<()> {
        // Create Rust backend
        let rust_dir = path.join("backend");
        create_rust_project_with_issues(&rust_dir, &fixtures.rust_fixtures).await?;
        
        // Create Python scripts
        let python_dir = path.join("scripts");
        fs::create_dir_all(&python_dir)?;
        create_python_project_with_issues(&python_dir, &fixtures.python_fixtures).await?;
        
        // Create JavaScript frontend
        let js_dir = path.join("frontend");
        fs::create_dir_all(&js_dir)?;
        
        let package_json = r#"
{
  "name": "test-frontend",
  "version": "1.0.0",
  "description": "Test frontend with issues",
  "main": "index.js",
  "scripts": {
    "start": "node index.js"
  },
  "dependencies": {
    "express": "^4.18.0",
    "axios": "^0.27.0"
  }
}
"#;
        fs::write(js_dir.join("package.json"), package_json)?;
        
        let js_god_object = r#"
// God object in JavaScript
class UniversalApplicationController {
    constructor() {
        this.dataStore = new Map();
        this.eventHandlers = new Map();
        this.uiComponents = new Map();
        this.networkClients = new Map();
        this.configManager = new Map();
        this.logBuffer = [];
        this.metrics = new Map();
    }
    
    // Data management methods (should be separate service)
    storeData(key, value) {
        this.dataStore.set(key, value);
        this.logMessage('info', `Data stored: ${key}`);
    }
    
    getData(key) {
        const value = this.dataStore.get(key);
        this.logMessage('info', `Data retrieved: ${key}`);
        return value;
    }
    
    // Event handling methods (should be separate service)
    addEventListener(event, handler) {
        if (!this.eventHandlers.has(event)) {
            this.eventHandlers.set(event, []);
        }
        this.eventHandlers.get(event).push(handler);
    }
    
    emitEvent(event, data) {
        const handlers = this.eventHandlers.get(event) || [];
        handlers.forEach(handler => {
            try {
                handler(data);
            } catch (error) {
                this.logMessage('error', `Event handler failed: ${error.message}`);
            }
        });
    }
    
    // UI management methods (should be separate service)
    registerComponent(id, component) {
        this.uiComponents.set(id, component);
        this.logMessage('info', `Component registered: ${id}`);
    }
    
    updateComponent(id, props) {
        const component = this.uiComponents.get(id);
        if (component) {
            Object.assign(component, props);
            this.emitEvent('componentUpdated', { id, props });
        }
    }
    
    // Network methods (should be separate service)
    async makeHttpRequest(url, options = {}) {
        const axios = require('axios');
        try {
            const response = await axios({ url, ...options });
            this.logMessage('info', `HTTP request successful: ${url}`);
            return response.data;
        } catch (error) {
            this.logMessage('error', `HTTP request failed: ${error.message}`);
            throw error;
        }
    }
    
    // Configuration methods (should be separate service)
    setConfig(key, value) {
        this.configManager.set(key, value);
        this.emitEvent('configChanged', { key, value });
    }
    
    getConfig(key, defaultValue = null) {
        return this.configManager.get(key) || defaultValue;
    }
    
    // Logging methods (should be separate service)
    logMessage(level, message) {
        const timestamp = new Date().toISOString();
        const logEntry = `[${timestamp}] ${level.toUpperCase()}: ${message}`;
        this.logBuffer.push(logEntry);
        
        if (this.logBuffer.length > 1000) {
            this.logBuffer = this.logBuffer.slice(-1000);
        }
        
        console.log(logEntry);
    }
    
    // Metrics methods (should be separate service)
    recordMetric(name, value, tags = {}) {
        if (!this.metrics.has(name)) {
            this.metrics.set(name, []);
        }
        
        this.metrics.get(name).push({
            value,
            timestamp: Date.now(),
            tags
        });
    }
    
    // Main method that does everything
    async handleRequest(requestData) {
        const requestId = requestData.id || 'unknown';
        
        try {
            this.logMessage('info', `Handling request: ${requestId}`);
            
            // Store request data
            this.storeData(`request_${requestId}`, requestData);
            
            // Update configuration
            if (requestData.config) {
                for (const [key, value] of Object.entries(requestData.config)) {
                    this.setConfig(key, value);
                }
            }
            
            // Make external API calls
            if (requestData.externalApiUrl) {
                try {
                    const apiResponse = await this.makeHttpRequest(requestData.externalApiUrl);
                    this.storeData(`api_response_${requestId}`, apiResponse);
                } catch (error) {
                    this.logMessage('error', `External API call failed: ${error.message}`);
                }
            }
            
            // Update UI components
            if (requestData.uiUpdates) {
                for (const [componentId, props] of Object.entries(requestData.uiUpdates)) {
                    this.updateComponent(componentId, props);
                }
            }
            
            // Emit events
            this.emitEvent('requestProcessed', { requestId, timestamp: Date.now() });
            
            // Record metrics
            this.recordMetric('requests_processed', 1, { requestId });
            
            // Prepare response
            const response = {
                success: true,
                requestId,
                processedAt: new Date().toISOString(),
                dataStored: this.dataStore.size,
                componentsRegistered: this.uiComponents.size,
                logEntries: this.logBuffer.length
            };
            
            this.logMessage('info', `Request processed successfully: ${requestId}`);
            return response;
            
        } catch (error) {
            this.logMessage('error', `Request processing failed: ${error.message}`);
            return {
                success: false,
                requestId,
                error: error.message,
                processedAt: new Date().toISOString()
            };
        }
    }
}

// Dead code examples
function unusedFunction() {
    return "This function is never called";
}

class UnusedClass {
    constructor(value) {
        this.value = value;
    }
    
    unusedMethod() {
        return this.value * 2;
    }
}

const UNUSED_CONSTANT = 42;

// Export the god object
module.exports = { UniversalApplicationController };
"#;
        fs::write(js_dir.join("app_controller.js"), js_god_object)?;
        
        let index_js = r#"
const { UniversalApplicationController } = require('./app_controller');

async function main() {
    const controller = new UniversalApplicationController();
    
    // Set up some initial configuration
    controller.setConfig('apiUrl', 'https://api.example.com');
    controller.setConfig('theme', 'dark');
    
    // Register some UI components
    controller.registerComponent('header', { title: 'Test App' });
    controller.registerComponent('sidebar', { collapsed: false });
    
    // Handle a test request
    const testRequest = {
        id: 'test_request_1',
        externalApiUrl: 'https://jsonplaceholder.typicode.com/posts/1',
        uiUpdates: {
            header: { title: 'Updated App' },
            sidebar: { collapsed: true }
        },
        config: {
            debugMode: true,
            logLevel: 'debug'
        }
    };
    
    const result = await controller.handleRequest(testRequest);
    console.log('Request result:', result);
}

main().catch(console.error);
"#;
        fs::write(js_dir.join("index.js"), index_js)?;
        
        // Create project README
        let readme = r#"
# Multi-Language Test Project

This project demonstrates various architectural issues across multiple programming languages:

## Structure

- `backend/`: Rust backend with god objects and circular dependencies
- `scripts/`: Python scripts with dead code and god objects  
- `frontend/`: JavaScript frontend with similar issues

## Issues to Detect

1. **God Objects**: Classes/structs that handle too many responsibilities
2. **Dead Code**: Functions, classes, and variables that are never used
3. **Circular Dependencies**: Modules that depend on each other
4. **Tight Coupling**: Components that are too tightly connected
5. **Magic Values**: Hard-coded values that should be constants

## Running

- Backend: `cd backend && cargo run`
- Scripts: `cd scripts && python main.py`
- Frontend: `cd frontend && npm install && npm start`
"#;
        fs::write(path.join("README.md"), readme)?;
        
        Ok(())
    }
}

impl Default for TestFixtures {
    fn default() -> Self {
        Self::new()
    }
}