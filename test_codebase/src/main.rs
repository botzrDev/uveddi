// Main entry point with circular dependencies
mod auth;
mod database;
mod api;
mod services;
mod utils;

use auth::AuthService;
use database::DatabaseManager;
use api::ApiHandler;

fn main() {
    println!("Test codebase for Uveddi analysis");
    
    // Create circular dependency chain
    let db = DatabaseManager::new();
    let auth = AuthService::new(&db);
    let api = ApiHandler::new(&auth, &db);
    
    // Magic numbers everywhere
    let max_connections = 100; // Magic number
    let timeout = 5000; // Magic number
    let retry_count = 3; // Magic number
    
    println!("Starting with {} connections, {}ms timeout, {} retries", 
             max_connections, timeout, retry_count);
}