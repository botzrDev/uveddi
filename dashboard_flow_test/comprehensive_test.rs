//! Comprehensive test file designed to trigger all detectors

use std::collections::HashMap;

// God Object - many methods and fields (should trigger GodObjectDetector)
pub struct UserManager {
    pub users: HashMap<String, User>,
    pub sessions: HashMap<String, Session>,
    pub permissions: HashMap<String, Vec<String>>,
    pub roles: HashMap<String, Role>,
    pub groups: HashMap<String, Group>,
    pub audit_log: Vec<AuditEntry>,
    pub config: SystemConfig,
    pub cache: CacheManager,
    pub database_pool: ConnectionPool,
}

impl UserManager {
    pub fn create_user(&self) {}
    pub fn delete_user(&self) {}
    pub fn update_user(&self) {}
    pub fn authenticate_user(&self) {}
    pub fn authorize_action(&self) {}
    pub fn assign_role(&self) {}
    pub fn remove_role(&self) {}
    pub fn create_group(&self) {}
    pub fn add_to_group(&self) {}
    pub fn remove_from_group(&self) {}
    pub fn audit_action(&self) {}
    pub fn cleanup_sessions(&self) {}
    pub fn backup_data(&self) {}
    pub fn restore_data(&self) {}
    pub fn generate_report(&self) {}
    pub fn send_notification(&self) {}
    pub fn log_event(&self) {}
    pub fn validate_permission(&self) {}
    pub fn cache_result(&self) {}
    pub fn invalidate_cache(&self) {}
}

// Code Duplication - identical functions (should trigger CodeDuplicationDetector)
fn process_payment_v1(amount: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err("Invalid amount".to_string());
    }
    let fee = amount * 0.03;
    let total = amount + fee;
    let transaction_id = format!("tx_{}", rand::random::<u64>());
    println!("Processing payment: ${:.2}", total);
    Ok(transaction_id)
}

fn process_payment_v2(amount: f64) -> Result<String, String> {
    if amount <= 0.0 {
        return Err("Invalid amount".to_string());
    }
    let fee = amount * 0.03;
    let total = amount + fee;
    let transaction_id = format!("tx_{}", rand::random::<u64>());
    println!("Processing payment: ${:.2}", total);
    Ok(transaction_id)
}

// Dead Code - unused functions (should trigger DeadCodeDetector)
fn unused_helper_function() {
    println!("This function is never called");
}

fn another_unused_function(data: &str) -> String {
    format!("Processed: {}", data)
}

// Large Class - many lines and methods (should trigger LargeClassDetector)
pub struct DataProcessor {
    data: Vec<String>,
    config: ProcessingConfig,
    cache: HashMap<String, ProcessedData>,
}

impl DataProcessor {
    pub fn new() -> Self {
        Self {
            data: Vec::new(),
            config: ProcessingConfig::default(),
            cache: HashMap::new(),
        }
    }
    
    pub fn process_batch(&mut self) {
        // Large method with many lines
        for item in &self.data {
            let processed = self.preprocess(item);
            let validated = self.validate(&processed);
            let transformed = self.transform(&validated);
            let enriched = self.enrich(&transformed);
            let formatted = self.format(&enriched);
            let stored = self.store(&formatted);
            self.update_cache(&stored);
            self.log_processing(&stored);
            self.notify_completion(&stored);
            self.cleanup_temp_data(&stored);
        }
    }
    
    fn preprocess(&self, data: &str) -> String { data.to_string() }
    fn validate(&self, data: &str) -> String { data.to_string() }
    fn transform(&self, data: &str) -> String { data.to_string() }
    fn enrich(&self, data: &str) -> String { data.to_string() }
    fn format(&self, data: &str) -> String { data.to_string() }
    fn store(&self, data: &str) -> String { data.to_string() }
    fn update_cache(&mut self, data: &str) {}
    fn log_processing(&self, data: &str) {}
    fn notify_completion(&self, data: &str) {}
    fn cleanup_temp_data(&self, data: &str) {}
}

// Long Method - method with many lines (should trigger LongMethodsDetector)
pub fn very_long_computation(input: Vec<i32>) -> i32 {
    let mut result = 0;
    let mut temp1 = 0;
    let mut temp2 = 0;
    let mut temp3 = 0;
    let mut temp4 = 0;
    let mut temp5 = 0;
    
    for (i, value) in input.iter().enumerate() {
        temp1 = value * 2;
        temp2 = temp1 + i as i32;
        temp3 = temp2 * temp2;
        temp4 = temp3 / (i as i32 + 1);
        temp5 = temp4 % 100;
        result += temp5;
        
        if result > 1000 {
            result -= 500;
        } else if result < -1000 {
            result += 500;
        }
        
        if i % 10 == 0 {
            result *= 2;
        }
        
        if i % 20 == 0 {
            result /= 3;
        }
        
        println!("Step {}: temp1={}, temp2={}, temp3={}, temp4={}, temp5={}, result={}", 
                 i, temp1, temp2, temp3, temp4, temp5, result);
    }
    
    result
}

// Magic Values - hardcoded constants (should trigger MagicValuesDetector)
pub fn configuration_example() {
    let timeout = 5000; // Magic number
    let max_retries = 3; // Magic number
    let buffer_size = 8192; // Magic number
    let cache_ttl = 300; // Magic number
    let rate_limit = 100; // Magic number
    
    println!("Config: timeout={}, retries={}, buffer={}, ttl={}, rate={}", 
             timeout, max_retries, buffer_size, cache_ttl, rate_limit);
}

// Tight Coupling - dependencies between modules (should trigger TightCouplingDetector)
pub struct OrderService {
    payment_processor: PaymentProcessor,
    inventory_manager: InventoryManager,
    notification_service: NotificationService,
    audit_logger: AuditLogger,
}

impl OrderService {
    pub fn process_order(&self, order: Order) {
        self.inventory_manager.reserve_items(&order);
        self.payment_processor.charge_customer(&order);
        self.notification_service.send_confirmation(&order);
        self.audit_logger.log_order(&order);
    }
}

// Supporting structs
#[derive(Default)]
struct ProcessingConfig;
struct ProcessedData;
struct User;
struct Session;
struct Role;
struct Group;
struct AuditEntry;
struct SystemConfig;
struct CacheManager;
struct ConnectionPool;
struct Order;
struct PaymentProcessor;
struct InventoryManager;
struct NotificationService;
struct AuditLogger;

impl PaymentProcessor {
    fn charge_customer(&self, _order: &Order) {}
}

impl InventoryManager {
    fn reserve_items(&self, _order: &Order) {}
}

impl NotificationService {
    fn send_confirmation(&self, _order: &Order) {}
}

impl AuditLogger {
    fn log_order(&self, _order: &Order) {}
}
