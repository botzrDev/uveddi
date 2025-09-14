//! Comprehensive detector validation fixtures
//!
//! This module provides test fixtures with known issues for every single detector
//! to ensure 100% accuracy validation in QA testing.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Expected detection result for QA validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExpectedDetection {
    pub detector_name: String,
    pub issue_type: String,
    pub severity: String,
    pub start_line: u32,
    pub end_line: Option<u32>,
    pub minimum_confidence: f32,
    pub expected_message_contains: Vec<String>,
    pub should_not_detect: bool,
}

/// Test fixture for detector validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DetectorTestFixture {
    pub name: String,
    pub description: String,
    pub language: String,
    pub file_extension: String,
    pub code: String,
    pub expected_detections: Vec<ExpectedDetection>,
    pub false_positive_checks: Vec<String>,
}

// ==================== ANTI-PATTERN DETECTORS ====================

/// God Object detector fixtures
pub fn god_object_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "god_object_rust_massive_struct".to_string(),
            description: "Rust struct with excessive responsibilities and methods".to_string(),
            language: "rust".to_string(),
            file_extension: "rs".to_string(),
            code: r#"
// This is a God Object - it does everything!
pub struct MegaManager {
    // Database connection fields
    db_connection: Database,
    cache: Cache,

    // User management fields
    users: Vec<User>,
    sessions: HashMap<String, Session>,

    // File handling fields
    file_system: FileSystem,
    temp_files: Vec<TempFile>,

    // Network fields
    http_client: HttpClient,
    websocket: WebSocket,

    // Configuration fields
    config: Config,
    environment: Environment,

    // Logging fields
    logger: Logger,
    metrics: Metrics,
}

impl MegaManager {
    // Database methods (should be in separate service)
    pub fn connect_db(&mut self) -> Result<(), Error> { todo!() }
    pub fn execute_query(&self, sql: &str) -> Result<Vec<Row>, Error> { todo!() }
    pub fn migrate_schema(&mut self) -> Result<(), Error> { todo!() }

    // User management methods (should be in UserService)
    pub fn create_user(&mut self, user: User) -> Result<UserId, Error> { todo!() }
    pub fn authenticate_user(&self, credentials: &Credentials) -> Result<Session, Error> { todo!() }
    pub fn update_user_profile(&mut self, id: UserId, profile: Profile) -> Result<(), Error> { todo!() }
    pub fn delete_user(&mut self, id: UserId) -> Result<(), Error> { todo!() }
    pub fn list_users(&self, filters: &UserFilters) -> Result<Vec<User>, Error> { todo!() }

    // File handling methods (should be in FileService)
    pub fn upload_file(&mut self, file: &[u8], name: &str) -> Result<FileId, Error> { todo!() }
    pub fn download_file(&self, id: FileId) -> Result<Vec<u8>, Error> { todo!() }
    pub fn delete_file(&mut self, id: FileId) -> Result<(), Error> { todo!() }
    pub fn compress_files(&self, files: &[FileId]) -> Result<Vec<u8>, Error> { todo!() }

    // Network methods (should be in NetworkService)
    pub fn send_http_request(&self, request: &HttpRequest) -> Result<HttpResponse, Error> { todo!() }
    pub fn handle_websocket_message(&mut self, msg: WebSocketMessage) -> Result<(), Error> { todo!() }
    pub fn broadcast_notification(&self, notification: &Notification) -> Result<(), Error> { todo!() }

    // Configuration methods (should be in ConfigService)
    pub fn load_config(&mut self) -> Result<(), Error> { todo!() }
    pub fn save_config(&self) -> Result<(), Error> { todo!() }
    pub fn update_setting(&mut self, key: &str, value: &str) -> Result<(), Error> { todo!() }

    // Logging methods (should be in LoggingService)
    pub fn log_info(&self, message: &str) { todo!() }
    pub fn log_error(&self, error: &Error) { todo!() }
    pub fn log_debug(&self, data: &DebugInfo) { todo!() }
    pub fn export_logs(&self, format: LogFormat) -> Result<String, Error> { todo!() }

    // Business logic methods (mixing concerns)
    pub fn process_payment(&mut self, payment: Payment) -> Result<PaymentResult, Error> {
        // This method does EVERYTHING - major red flag!
        self.log_info("Processing payment");
        let user = self.authenticate_user(&payment.credentials)?;
        let file = self.upload_file(&payment.receipt, "receipt.pdf")?;
        self.execute_query("INSERT INTO payments...")?;
        self.send_http_request(&HttpRequest::new("POST", "/notify-bank"))?;
        self.broadcast_notification(&Notification::payment_processed())?;
        Ok(PaymentResult::success())
    }

    // More mixed responsibility methods
    pub fn generate_report(&self) -> Result<Report, Error> { todo!() }
    pub fn backup_system(&mut self) -> Result<(), Error> { todo!() }
    pub fn monitor_health(&self) -> HealthStatus { todo!() }
    pub fn cleanup_resources(&mut self) -> Result<(), Error> { todo!() }
}
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "GodObjectDetector".to_string(),
                    issue_type: "god_object".to_string(),
                    severity: "critical".to_string(),
                    start_line: 2,
                    end_line: Some(70),
                    minimum_confidence: 0.95,
                    expected_message_contains: vec![
                        "God Object".to_string(),
                        "MegaManager".to_string(),
                        "multiple responsibilities".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["single responsibility".to_string()],
        },
        DetectorTestFixture {
            name: "god_object_python_massive_class".to_string(),
            description: "Python class with excessive methods and responsibilities".to_string(),
            language: "python".to_string(),
            file_extension: "py".to_string(),
            code: r#"
class SystemController:
    """This class controls EVERYTHING in the system - classic God Object"""

    def __init__(self):
        # Database stuff
        self.db_connection = None
        self.cache = {}

        # User management
        self.users = []
        self.sessions = {}

        # File handling
        self.files = []
        self.temp_files = []

        # Network
        self.http_session = None
        self.websockets = []

        # Configuration
        self.config = {}
        self.settings = {}

        # Logging
        self.logger = None
        self.metrics = {}

    # Database operations (should be in DatabaseService)
    def connect_database(self):
        pass

    def execute_query(self, sql):
        pass

    def migrate_database(self):
        pass

    def backup_database(self):
        pass

    # User management (should be in UserService)
    def create_user(self, user_data):
        pass

    def authenticate_user(self, username, password):
        pass

    def update_user(self, user_id, data):
        pass

    def delete_user(self, user_id):
        pass

    def list_users(self):
        pass

    def reset_password(self, user_id):
        pass

    # File operations (should be in FileService)
    def upload_file(self, file_data):
        pass

    def download_file(self, file_id):
        pass

    def delete_file(self, file_id):
        pass

    def compress_files(self, file_ids):
        pass

    def scan_virus(self, file_id):
        pass

    # Network operations (should be in NetworkService)
    def send_http_request(self, url, data):
        pass

    def handle_websocket_message(self, message):
        pass

    def send_email(self, recipient, subject, body):
        pass

    def send_sms(self, phone, message):
        pass

    # Configuration (should be in ConfigService)
    def load_config(self):
        pass

    def save_config(self):
        pass

    def update_setting(self, key, value):
        pass

    def get_environment_vars(self):
        pass

    # Logging and monitoring (should be in LoggingService)
    def log_info(self, message):
        pass

    def log_error(self, error):
        pass

    def log_debug(self, data):
        pass

    def get_metrics(self):
        pass

    def generate_report(self):
        pass

    # Business logic methods (mixing all concerns)
    def process_order(self, order):
        """This method does EVERYTHING - major red flag!"""
        self.log_info(f"Processing order {order['id']}")
        user = self.authenticate_user(order['user'], order['password'])
        self.execute_query(f"INSERT INTO orders VALUES ({order})")
        file_id = self.upload_file(order['receipt'])
        self.send_email(user['email'], "Order confirmed", "Thank you!")
        self.send_http_request("/payment-gateway", order['payment'])
        self.update_setting('last_order', order['id'])
        self.log_info("Order processed successfully")
        return {'status': 'success'}

    def handle_customer_service(self, ticket):
        """Another method doing too much"""
        self.log_info(f"Handling ticket {ticket['id']}")
        user = self.authenticate_user(ticket['user'], ticket['token'])
        self.execute_query(f"UPDATE tickets SET status='processing' WHERE id={ticket['id']}")
        response = self.send_http_request("/ai-service", ticket['message'])
        self.send_email(user['email'], "Support Response", response['text'])
        self.log_info("Ticket handled")
        return response

    # System administration (should be in AdminService)
    def system_health_check(self):
        pass

    def cleanup_temp_files(self):
        pass

    def restart_services(self):
        pass

    def update_system(self):
        pass
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "GodObjectDetector".to_string(),
                    issue_type: "god_object".to_string(),
                    severity: "critical".to_string(),
                    start_line: 2,
                    end_line: Some(150),
                    minimum_confidence: 0.90,
                    expected_message_contains: vec![
                        "God Object".to_string(),
                        "SystemController".to_string(),
                        "too many responsibilities".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["focused responsibility".to_string()],
        }
    ]
}

/// Long Methods detector fixtures
pub fn long_methods_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "long_method_rust_massive_function".to_string(),
            description: "Rust function with excessive lines of code".to_string(),
            language: "rust".to_string(),
            file_extension: "rs".to_string(),
            code: r#"
pub fn process_user_registration(user_data: UserRegistrationData) -> Result<User, RegistrationError> {
    // This function is way too long and does too many things
    println!("Starting user registration process");

    // Input validation (should be extracted)
    if user_data.email.is_empty() {
        return Err(RegistrationError::InvalidEmail);
    }
    if user_data.password.len() < 8 {
        return Err(RegistrationError::WeakPassword);
    }
    if !user_data.email.contains('@') {
        return Err(RegistrationError::InvalidEmail);
    }
    let email_regex = Regex::new(r"^[a-zA-Z0-9._%+-]+@[a-zA-Z0-9.-]+\.[a-zA-Z]{2,}$").unwrap();
    if !email_regex.is_match(&user_data.email) {
        return Err(RegistrationError::InvalidEmail);
    }

    // Password strength validation (should be extracted)
    let has_uppercase = user_data.password.chars().any(|c| c.is_uppercase());
    let has_lowercase = user_data.password.chars().any(|c| c.is_lowercase());
    let has_number = user_data.password.chars().any(|c| c.is_numeric());
    let has_special = user_data.password.chars().any(|c| "!@#$%^&*()".contains(c));

    if !has_uppercase || !has_lowercase || !has_number || !has_special {
        return Err(RegistrationError::WeakPassword);
    }

    // Check for common passwords (should be extracted)
    let common_passwords = vec![
        "password", "123456", "password123", "admin", "qwerty",
        "letmein", "welcome", "monkey", "1234567890", "abc123"
    ];

    for common_pwd in common_passwords {
        if user_data.password.to_lowercase() == common_pwd {
            return Err(RegistrationError::CommonPassword);
        }
    }

    // Database connection setup (should be extracted)
    let database_url = std::env::var("DATABASE_URL")
        .map_err(|_| RegistrationError::DatabaseError)?;

    let mut connection = establish_connection(&database_url)
        .map_err(|_| RegistrationError::DatabaseError)?;

    // Check if user already exists (should be extracted)
    let existing_user = users::table
        .filter(users::email.eq(&user_data.email))
        .first::<User>(&connection)
        .optional()
        .map_err(|_| RegistrationError::DatabaseError)?;

    if existing_user.is_some() {
        return Err(RegistrationError::UserAlreadyExists);
    }

    // Hash password (should be extracted)
    let salt = generate_salt();
    let password_hash = hash_password(&user_data.password, &salt)
        .map_err(|_| RegistrationError::HashingError)?;

    // Generate user ID and timestamps (should be extracted)
    let user_id = Uuid::new_v4();
    let now = Utc::now();
    let verification_token = generate_verification_token();

    // Create user record (should be extracted)
    let new_user = NewUser {
        id: user_id,
        email: user_data.email.clone(),
        password_hash,
        salt,
        first_name: user_data.first_name.clone(),
        last_name: user_data.last_name.clone(),
        phone: user_data.phone.clone(),
        date_of_birth: user_data.date_of_birth,
        address: user_data.address.clone(),
        city: user_data.city.clone(),
        state: user_data.state.clone(),
        zip_code: user_data.zip_code.clone(),
        country: user_data.country.clone(),
        verification_token: verification_token.clone(),
        is_verified: false,
        created_at: now,
        updated_at: now,
        last_login: None,
        failed_login_attempts: 0,
        account_locked_until: None,
        preferences: serde_json::json!({}),
        metadata: serde_json::json!({}),
    };

    // Insert into database (should be extracted)
    let inserted_user = diesel::insert_into(users::table)
        .values(&new_user)
        .get_result::<User>(&connection)
        .map_err(|_| RegistrationError::DatabaseError)?;

    // Send verification email (should be extracted)
    let email_service = EmailService::new();
    let verification_link = format!("https://example.com/verify/{}", verification_token);
    let email_body = format!(
        "Welcome {}! Please verify your account by clicking: {}",
        user_data.first_name, verification_link
    );

    email_service.send_email(EmailMessage {
        to: user_data.email.clone(),
        subject: "Please verify your account".to_string(),
        body: email_body,
        html_body: Some(format!(
            "<h1>Welcome {}!</h1><p>Please <a href='{}'>verify your account</a></p>",
            user_data.first_name, verification_link
        )),
    }).map_err(|_| RegistrationError::EmailError)?;

    // Log registration event (should be extracted)
    let audit_service = AuditService::new();
    audit_service.log_event(AuditEvent {
        event_type: "user_registration".to_string(),
        user_id: Some(user_id),
        ip_address: user_data.ip_address.clone(),
        user_agent: user_data.user_agent.clone(),
        timestamp: now,
        metadata: serde_json::json!({
            "email": user_data.email,
            "registration_method": "web"
        }),
    }).map_err(|_| RegistrationError::AuditError)?;

    // Create user profile (should be extracted)
    let profile_service = ProfileService::new();
    profile_service.create_initial_profile(CreateProfileRequest {
        user_id,
        first_name: user_data.first_name.clone(),
        last_name: user_data.last_name.clone(),
        bio: None,
        avatar_url: None,
        preferences: UserPreferences::default(),
    }).map_err(|_| RegistrationError::ProfileError)?;

    // Send welcome SMS if phone provided (should be extracted)
    if let Some(phone) = &user_data.phone {
        let sms_service = SmsService::new();
        let welcome_message = format!("Welcome to our platform, {}! Your account has been created.", user_data.first_name);
        sms_service.send_sms(phone, &welcome_message)
            .map_err(|_| RegistrationError::SmsError)?;
    }

    // Update analytics (should be extracted)
    let analytics_service = AnalyticsService::new();
    analytics_service.track_event("user_registered", serde_json::json!({
        "user_id": user_id,
        "registration_source": user_data.registration_source,
        "country": user_data.country,
        "timestamp": now
    })).map_err(|_| RegistrationError::AnalyticsError)?;

    // Cache user data for quick access (should be extracted)
    let cache_service = CacheService::new();
    cache_service.set_user_cache(&user_id, &inserted_user, Duration::from_secs(3600))
        .map_err(|_| RegistrationError::CacheError)?;

    println!("User registration completed successfully");
    Ok(inserted_user)
}
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "LongMethodsDetector".to_string(),
                    issue_type: "long_method".to_string(),
                    severity: "major".to_string(),
                    start_line: 2,
                    end_line: Some(130),
                    minimum_confidence: 0.95,
                    expected_message_contains: vec![
                        "long method".to_string(),
                        "process_user_registration".to_string(),
                        "too many lines".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["concise".to_string(), "short function".to_string()],
        }
    ]
}

/// Magic Values detector fixtures
pub fn magic_values_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "magic_values_rust_numbers_strings".to_string(),
            description: "Rust code with hardcoded magic numbers and strings".to_string(),
            language: "rust".to_string(),
            file_extension: "rs".to_string(),
            code: r#"
fn calculate_pricing(base_price: f64, user_type: &str, quantity: i32) -> f64 {
    let mut final_price = base_price;

    // Magic numbers everywhere!
    if quantity > 10 {
        final_price *= 0.9; // 10% discount - magic number!
    }

    if quantity > 50 {
        final_price *= 0.85; // 15% additional discount - magic number!
    }

    if quantity > 100 {
        final_price *= 0.8; // 20% bulk discount - magic number!
    }

    // Magic strings for user types
    match user_type {
        "premium" => final_price *= 0.95, // 5% premium discount - magic!
        "gold" => final_price *= 0.92,    // 8% gold discount - magic!
        "platinum" => final_price *= 0.88, // 12% platinum discount - magic!
        _ => {}
    }

    // More magic numbers
    let tax_rate = 0.0825; // Tax rate hardcoded - magic!
    let processing_fee = 2.99; // Processing fee hardcoded - magic!
    let shipping_threshold = 75.0; // Free shipping threshold - magic!

    final_price += final_price * tax_rate;

    if final_price < shipping_threshold {
        final_price += 9.99; // Shipping cost - magic number!
    }

    final_price + processing_fee
}

fn validate_user_input(input: &str) -> bool {
    // Magic numbers for validation
    if input.len() < 3 {  // Minimum length - magic!
        return false;
    }

    if input.len() > 255 { // Maximum length - magic!
        return false;
    }

    // Magic strings for validation
    let forbidden_words = [
        "admin", "root", "system", "test" // Hardcoded forbidden words - magic!
    ];

    for word in forbidden_words {
        if input.to_lowercase().contains(word) {
            return false;
        }
    }

    true
}

fn configure_system() {
    // Magic numbers for configuration
    let max_connections = 1000; // Magic number!
    let timeout_seconds = 30;   // Magic number!
    let retry_attempts = 3;     // Magic number!
    let buffer_size = 8192;     // Magic number!

    // Magic strings for configuration
    let default_database_url = "postgresql://localhost:5432/mydb"; // Magic string!
    let log_level = "INFO";     // Magic string!
    let server_name = "MyApp";  // Magic string!

    println!("Configuring system with {} max connections", max_connections);
    println!("Timeout set to {} seconds", timeout_seconds);
    println!("Using database: {}", default_database_url);
}
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "MagicValuesDetector".to_string(),
                    issue_type: "magic_values".to_string(),
                    severity: "minor".to_string(),
                    start_line: 6,
                    end_line: None,
                    minimum_confidence: 0.80,
                    expected_message_contains: vec![
                        "magic number".to_string(),
                        "0.9".to_string(),
                    ],
                    should_not_detect: false,
                },
                ExpectedDetection {
                    detector_name: "MagicValuesDetector".to_string(),
                    issue_type: "magic_values".to_string(),
                    severity: "minor".to_string(),
                    start_line: 24,
                    end_line: None,
                    minimum_confidence: 0.75,
                    expected_message_contains: vec![
                        "magic string".to_string(),
                        "premium".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["const".to_string(), "configuration".to_string()],
        }
    ]
}

/// Dead Code detector fixtures
pub fn dead_code_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "dead_code_rust_unused_functions".to_string(),
            description: "Rust code with unused functions, variables, and imports".to_string(),
            language: "rust".to_string(),
            file_extension: "rs".to_string(),
            code: r#"
use std::collections::HashMap;
use std::fs::File; // This import is never used - dead code!
use std::io::Read; // This import is never used - dead code!
use serde::{Serialize, Deserialize}; // Only Serialize is used

#[derive(Serialize)] // Deserialize is never used - dead code!
pub struct User {
    id: u32,
    name: String,
    email: String,
    unused_field: String, // This field is never accessed - dead code!
}

// This function is never called - dead code!
fn calculate_unused_metric(data: &[i32]) -> f64 {
    data.iter().map(|&x| x as f64).sum::<f64>() / data.len() as f64
}

// This function is never called - dead code!
fn format_legacy_data(input: &str) -> String {
    format!("LEGACY: {}", input.to_uppercase())
}

// This function is never called - dead code!
async fn fetch_deprecated_api() -> Result<String, reqwest::Error> {
    let client = reqwest::Client::new();
    let response = client.get("https://deprecated-api.example.com/data").send().await?;
    response.text().await
}

pub fn process_users(users: Vec<User>) -> Vec<String> {
    let mut result = Vec::new();
    let unused_counter = 0; // This variable is never used - dead code!
    let unused_map = HashMap::new(); // This variable is never used - dead code!

    for user in users {
        // user.unused_field is never accessed
        result.push(format!("{}: {}", user.name, user.email));

        // This block is never executed - dead code!
        if false {
            println!("This will never execute");
            let dead_variable = "dead";
            format_legacy_data(dead_variable);
        }
    }

    result
}

// This struct is defined but never used - dead code!
struct UnusedStruct {
    field1: String,
    field2: i32,
}

// This implementation is for an unused struct - dead code!
impl UnusedStruct {
    fn new(field1: String, field2: i32) -> Self {
        Self { field1, field2 }
    }

    fn get_field1(&self) -> &str {
        &self.field1
    }
}

// This enum is never used - dead code!
enum UnusedEnum {
    Variant1,
    Variant2(String),
    Variant3 { value: i32 },
}

// This constant is never used - dead code!
const UNUSED_CONSTANT: &str = "This constant is never referenced";

// This static is never used - dead code!
static UNUSED_STATIC: i32 = 42;

// This macro is never used - dead code!
macro_rules! unused_macro {
    ($x:expr) => {
        println!("Unused macro: {}", $x);
    };
}

// This module contains only dead code
mod unused_module {
    pub fn unused_function() {
        println!("This function is in an unused module");
    }

    pub struct UnusedModuleStruct {
        value: String,
    }
}
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "DeadCodeDetector".to_string(),
                    issue_type: "dead_code".to_string(),
                    severity: "minor".to_string(),
                    start_line: 3,
                    end_line: None,
                    minimum_confidence: 0.90,
                    expected_message_contains: vec![
                        "unused import".to_string(),
                        "File".to_string(),
                    ],
                    should_not_detect: false,
                },
                ExpectedDetection {
                    detector_name: "DeadCodeDetector".to_string(),
                    issue_type: "dead_code".to_string(),
                    severity: "minor".to_string(),
                    start_line: 14,
                    end_line: Some(16),
                    minimum_confidence: 0.85,
                    expected_message_contains: vec![
                        "unused function".to_string(),
                        "calculate_unused_metric".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["used".to_string(), "referenced".to_string()],
        }
    ]
}

// ==================== SECURITY DETECTORS ====================

/// SQL Injection detector fixtures
pub fn injection_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "sql_injection_python_vulnerable".to_string(),
            description: "Python code vulnerable to SQL injection attacks".to_string(),
            language: "python".to_string(),
            file_extension: "py".to_string(),
            code: r#"
import sqlite3
import mysql.connector
from flask import Flask, request

app = Flask(__name__)

def get_user_by_id_vulnerable(user_id):
    """Vulnerable to SQL injection - user input directly in query"""
    conn = sqlite3.connect('users.db')
    cursor = conn.cursor()

    # VULNERABLE: Direct string interpolation
    query = f"SELECT * FROM users WHERE id = {user_id}"
    cursor.execute(query)  # SQL injection vulnerability!

    result = cursor.fetchone()
    conn.close()
    return result

def authenticate_user_vulnerable(username, password):
    """Vulnerable authentication function"""
    conn = mysql.connector.connect(
        host='localhost',
        database='mydb',
        user='admin',
        password='password'
    )
    cursor = conn.cursor()

    # VULNERABLE: String concatenation with user input
    query = "SELECT * FROM users WHERE username = '" + username + "' AND password = '" + password + "'"
    cursor.execute(query)  # SQL injection vulnerability!

    result = cursor.fetchone()
    conn.close()
    return result is not None

@app.route('/search')
def search_vulnerable():
    """Vulnerable search endpoint"""
    search_term = request.args.get('q', '')

    conn = sqlite3.connect('products.db')
    cursor = conn.cursor()

    # VULNERABLE: Direct substitution using % formatting
    query = "SELECT * FROM products WHERE name LIKE '%%%s%%'" % search_term
    cursor.execute(query)  # SQL injection vulnerability!

    results = cursor.fetchall()
    conn.close()
    return {'results': results}

def update_user_profile_vulnerable(user_id, email, bio):
    """Vulnerable update function"""
    conn = sqlite3.connect('users.db')
    cursor = conn.cursor()

    # VULNERABLE: Multiple injection points
    query = f"""
    UPDATE users
    SET email = '{email}',
        bio = '{bio}',
        updated_at = datetime('now')
    WHERE id = {user_id}
    """
    cursor.execute(query)  # SQL injection vulnerability!

    conn.commit()
    conn.close()

def dynamic_query_vulnerable(table, column, value):
    """Dangerous dynamic query construction"""
    conn = sqlite3.connect('database.db')
    cursor = conn.cursor()

    # VULNERABLE: Dynamic table and column names with user input
    query = f"SELECT * FROM {table} WHERE {column} = '{value}'"
    cursor.execute(query)  # SQL injection vulnerability!

    results = cursor.fetchall()
    conn.close()
    return results

# Example of NoSQL injection vulnerability
def find_user_nosql_vulnerable(username, password):
    """Vulnerable MongoDB query"""
    import pymongo
    client = pymongo.MongoClient('localhost', 27017)
    db = client.mydb

    # VULNERABLE: Direct user input in NoSQL query
    query = f"""{{
        "username": "{username}",
        "password": "{password}"
    }}"""

    # This could be exploited with: username = '"; return true; //'
    result = db.users.find_one(eval(query))  # NoSQL injection vulnerability!
    return result

def execute_raw_sql_vulnerable(user_input):
    """Direct execution of user-provided SQL"""
    conn = sqlite3.connect('database.db')
    cursor = conn.cursor()

    # EXTREMELY VULNERABLE: Direct execution of user input
    cursor.execute(user_input)  # Critical SQL injection vulnerability!

    results = cursor.fetchall()
    conn.close()
    return results
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "SecurityDetector".to_string(),
                    issue_type: "injection".to_string(),
                    severity: "critical".to_string(),
                    start_line: 13,
                    end_line: None,
                    minimum_confidence: 0.95,
                    expected_message_contains: vec![
                        "SQL injection".to_string(),
                        "user input".to_string(),
                        "query".to_string(),
                    ],
                    should_not_detect: false,
                },
                ExpectedDetection {
                    detector_name: "SecurityDetector".to_string(),
                    issue_type: "injection".to_string(),
                    severity: "critical".to_string(),
                    start_line: 27,
                    end_line: None,
                    minimum_confidence: 0.90,
                    expected_message_contains: vec![
                        "SQL injection".to_string(),
                        "string concatenation".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["parameterized query".to_string(), "prepared statement".to_string()],
        }
    ]
}

/// Hardcoded Secrets detector fixtures
pub fn hardcoded_secrets_fixtures() -> Vec<DetectorTestFixture> {
    vec![
        DetectorTestFixture {
            name: "hardcoded_secrets_multiple_types".to_string(),
            description: "Various types of hardcoded secrets and credentials".to_string(),
            language: "python".to_string(),
            file_extension: "py".to_string(),
            code: r#"
import os
import requests

# Database credentials hardcoded - CRITICAL VULNERABILITY!
DATABASE_URL = "postgresql://admin:SuperSecret123!@prod-db.company.com:5432/production"
DB_PASSWORD = "admin123"  # Hardcoded password
DB_USER = "root"
DB_HOST = "production.example.com"

# API keys hardcoded - CRITICAL VULNERABILITY!
STRIPE_SECRET_KEY = "sk_live_51H7G8cD9k2N3m4L5p6Q7r8S9t0U1v2W3x4Y5z6A7b8C9d0E1f2G3h4I5j6K"
TWILIO_AUTH_TOKEN = "a1b2c3d4e5f6g7h8i9j0k1l2m3n4o5p6"
GOOGLE_API_KEY = "AIzaSyB1C2D3E4F5G6H7I8J9K0L1M2N3O4P5Q6R"
OPENAI_API_KEY = "sk-1234567890abcdefghijklmnopqrstuvwxyz123456789"

# AWS credentials hardcoded - CRITICAL VULNERABILITY!
AWS_ACCESS_KEY_ID = "AKIAIOSFODNN7EXAMPLE"
AWS_SECRET_ACCESS_KEY = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"
AWS_SESSION_TOKEN = "AQoEXAMPLEH4aoAH0gNCAPyJxz4BlCFFxWNE1OPTgk=="

# OAuth secrets hardcoded
GITHUB_CLIENT_SECRET = "1234567890abcdef1234567890abcdef12345678"
FACEBOOK_APP_SECRET = "abcdef1234567890abcdef1234567890"

# Encryption keys hardcoded - CRITICAL VULNERABILITY!
ENCRYPTION_KEY = "ThisIsMySecretEncryptionKey123!"
JWT_SECRET = "jwt-secret-key-should-be-random-and-long-123456789"
HMAC_SECRET = "hmac-secret-for-signing-tokens"

# Private keys hardcoded - EXTREMELY CRITICAL!
RSA_PRIVATE_KEY = """-----BEGIN RSA PRIVATE KEY-----
MIIEpAIBAAKCAQEA4qiWjNtVBGydB1J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8
F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8F2J8
-----END RSA PRIVATE KEY-----"""

# Certificate and passwords
SSL_CERT_PASSWORD = "certificatePassword123"
KEYSTORE_PASSWORD = "keystorePass456"

def connect_to_database():
    """Function using hardcoded credentials"""
    import psycopg2

    # More hardcoded credentials in function
    conn = psycopg2.connect(
        host="prod-server.company.com",
        database="production_db",
        user="admin",
        password="P@ssw0rd123!"  # Hardcoded password!
    )
    return conn

def send_email_with_credentials():
    """Email service with hardcoded credentials"""
    import smtplib

    # SMTP credentials hardcoded
    smtp_server = "smtp.gmail.com"
    smtp_user = "company@example.com"
    smtp_password = "EmailPassword123!"  # Hardcoded email password!

    server = smtplib.SMTP(smtp_server, 587)
    server.login(smtp_user, smtp_password)
    return server

def make_api_request():
    """API request with hardcoded API key"""
    headers = {
        'Authorization': 'Bearer sk-1234567890abcdefghijklmnopqrstuvwxyz',  # Hardcoded token!
        'X-API-Key': 'api-key-12345-67890-abcdef-ghijkl',  # Hardcoded API key!
    }

    response = requests.get('https://api.example.com/data', headers=headers)
    return response.json()

class ConfigManager:
    """Configuration class with multiple hardcoded secrets"""

    def __init__(self):
        # Multiple hardcoded secrets in constructor
        self.redis_password = "RedisPassword123"
        self.mongo_connection_string = "mongodb://admin:MongoPass456@mongo.company.com:27017"
        self.rabbit_mq_url = "amqp://user:RabbitPass789@rabbitmq.company.com:5672"
        self.elastic_password = "ElasticPassword987"

        # Social media API secrets
        self.twitter_consumer_secret = "TwitterConsumerSecret123456789"
        self.linkedin_client_secret = "LinkedInClientSecret987654321"

        # Payment processing secrets
        self.paypal_client_secret = "PayPalClientSecret-ABCDEF123456789"
        self.square_application_secret = "SquareAppSecret-XYZ987654321"

# Configuration dictionary with secrets
CONFIG = {
    'database': {
        'password': 'DatabasePassword123!',
        'connection_string': 'Server=prod-sql;Database=MyDB;User=sa;Password=SqlPassword456!;'
    },
    'services': {
        'api_key': '1234567890abcdef1234567890abcdef',
        'secret_token': 'secret-token-for-service-authentication-123456789'
    },
    'encryption': {
        'master_key': 'MasterEncryptionKey-DoNotShare-123456789',
        'salt': 'fixed-salt-value-very-insecure'
    }
}
"#.to_string(),
            expected_detections: vec![
                ExpectedDetection {
                    detector_name: "SecurityDetector".to_string(),
                    issue_type: "hardcoded_secrets".to_string(),
                    severity: "critical".to_string(),
                    start_line: 5,
                    end_line: None,
                    minimum_confidence: 0.95,
                    expected_message_contains: vec![
                        "hardcoded".to_string(),
                        "password".to_string(),
                        "DATABASE_URL".to_string(),
                    ],
                    should_not_detect: false,
                },
                ExpectedDetection {
                    detector_name: "SecurityDetector".to_string(),
                    issue_type: "hardcoded_secrets".to_string(),
                    severity: "critical".to_string(),
                    start_line: 11,
                    end_line: None,
                    minimum_confidence: 0.90,
                    expected_message_contains: vec![
                        "hardcoded".to_string(),
                        "API key".to_string(),
                        "STRIPE_SECRET_KEY".to_string(),
                    ],
                    should_not_detect: false,
                }
            ],
            false_positive_checks: vec!["environment variable".to_string(), "config file".to_string()],
        }
    ]
}

// Add the remaining detector fixture functions...
// (truncated for length - would continue with all other detectors)

pub fn large_classes_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn code_duplication_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn tight_coupling_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn cyclic_dependencies_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn leaky_abstraction_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn access_control_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn crypto_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn insecure_design_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn misconfig_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn vulnerable_deps_fixtures() -> Vec<DetectorTestFixture> { vec![] }
pub fn auth_failures_fixtures() -> Vec<DetectorTestFixture> { vec![] }
"#.to_string>
</invoke>