// Test code with known anti-patterns for diagram generation
use std::collections::HashMap;

// God Object anti-pattern
pub struct MegaService {
    users: HashMap<u64, User>,
    products: HashMap<u64, Product>,
    orders: HashMap<u64, Order>,
    logger: Logger,
    database: Database,
    email_service: EmailService,
    payment_processor: PaymentProcessor,
    cache: Cache,
    metrics: Metrics,
    configuration: Configuration,
}

impl MegaService {
    pub fn new() -> Self {
        Self {
            users: HashMap::new(),
            products: HashMap::new(),
            orders: HashMap::new(),
            logger: Logger::new(),
            database: Database::new(),
            email_service: EmailService::new(),
            payment_processor: PaymentProcessor::new(),
            cache: Cache::new(),
            metrics: Metrics::new(),
            configuration: Configuration::new(),
        }
    }

    // Too many methods in one class
    pub fn create_user(&mut self, user: User) -> Result<u64, String> {
        self.logger.log("Creating user");
        self.metrics.increment("user_created");
        let id = self.generate_user_id();
        self.users.insert(id, user);
        self.database.save_user(id);
        Ok(id)
    }

    pub fn get_user(&self, id: u64) -> Option<&User> {
        self.users.get(&id)
    }

    pub fn delete_user(&mut self, id: u64) -> Result<(), String> {
        self.users.remove(&id);
        self.database.delete_user(id);
        Ok(())
    }

    pub fn create_product(&mut self, product: Product) -> Result<u64, String> {
        self.logger.log("Creating product");
        let id = self.generate_product_id();
        self.products.insert(id, product);
        self.database.save_product(id);
        Ok(id)
    }

    pub fn get_product(&self, id: u64) -> Option<&Product> {
        self.products.get(&id)
    }

    pub fn create_order(&mut self, order: Order) -> Result<u64, String> {
        self.logger.log("Creating order");
        let id = self.generate_order_id();
        self.orders.insert(id, order);
        self.process_payment(id)?;
        self.send_confirmation_email(id)?;
        self.database.save_order(id);
        Ok(id)
    }

    pub fn process_payment(&self, order_id: u64) -> Result<(), String> {
        self.payment_processor.process(order_id)
    }

    pub fn send_confirmation_email(&self, order_id: u64) -> Result<(), String> {
        self.email_service.send_confirmation(order_id)
    }

    pub fn generate_user_id(&self) -> u64 {
        42 // Magic number anti-pattern
    }

    pub fn generate_product_id(&self) -> u64 {
        100 // Magic number anti-pattern
    }

    pub fn generate_order_id(&self) -> u64 {
        200 // Magic number anti-pattern
    }

    pub fn get_configuration_value(&self, key: &str) -> String {
        self.configuration.get(key)
    }

    pub fn cache_get(&self, key: &str) -> Option<String> {
        self.cache.get(key)
    }

    pub fn cache_set(&mut self, key: String, value: String) {
        self.cache.set(key, value);
    }

    pub fn log_metric(&self, name: &str, value: f64) {
        self.metrics.record(name, value);
    }

    pub fn health_check(&self) -> bool {
        self.database.is_connected()
    }
}

// Tightly coupled classes
pub struct User {
    pub id: u64,
    pub name: String,
    pub email: String,
    pub orders: Vec<Order>, // Tight coupling
}

pub struct Product {
    pub id: u64,
    pub name: String,
    pub price: f64,
    pub orders: Vec<Order>, // Tight coupling
}

pub struct Order {
    pub id: u64,
    pub user: User, // Tight coupling
    pub products: Vec<Product>, // Tight coupling
    pub total: f64,
}

// Circular dependency
impl User {
    pub fn create_order(&self, products: Vec<Product>) -> Order {
        Order {
            id: 0,
            user: self.clone(),
            products,
            total: 0.0,
        }
    }
}

impl Order {
    pub fn get_user(&self) -> &User {
        &self.user
    }
}

// Stub implementations for compilation
#[derive(Clone)]
pub struct Logger;
impl Logger {
    pub fn new() -> Self { Logger }
    pub fn log(&self, _msg: &str) {}
}

pub struct Database;
impl Database {
    pub fn new() -> Self { Database }
    pub fn save_user(&self, _id: u64) {}
    pub fn delete_user(&self, _id: u64) {}
    pub fn save_product(&self, _id: u64) {}
    pub fn save_order(&self, _id: u64) {}
    pub fn is_connected(&self) -> bool { true }
}

pub struct EmailService;
impl EmailService {
    pub fn new() -> Self { EmailService }
    pub fn send_confirmation(&self, _order_id: u64) -> Result<(), String> { Ok(()) }
}

pub struct PaymentProcessor;
impl PaymentProcessor {
    pub fn new() -> Self { PaymentProcessor }
    pub fn process(&self, _order_id: u64) -> Result<(), String> { Ok(()) }
}

pub struct Cache;
impl Cache {
    pub fn new() -> Self { Cache }
    pub fn get(&self, _key: &str) -> Option<String> { None }
    pub fn set(&mut self, _key: String, _value: String) {}
}

pub struct Metrics;
impl Metrics {
    pub fn new() -> Self { Metrics }
    pub fn increment(&self, _name: &str) {}
    pub fn record(&self, _name: &str, _value: f64) {}
}

pub struct Configuration;
impl Configuration {
    pub fn new() -> Self { Configuration }
    pub fn get(&self, _key: &str) -> String { String::new() }
}

fn main() {
    let mut service = MegaService::new();
    let user = User {
        id: 1,
        name: "Test User".to_string(),
        email: "test@example.com".to_string(),
        orders: Vec::new(),
    };
    let _result = service.create_user(user);
}