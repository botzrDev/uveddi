use std::{fs, io::Write, path::Path};
use std::fs::File;

/// Generate realistic test data for benchmarking and testing the analysis engine
fn main() {
    println!("Generating benchmark data...");
    
    // Create base directory
    let base_dir = Path::new("target/benchmark-data");
    fs::create_dir_all(base_dir).unwrap();
    
    // Generate synthetic data for each supported language
    generate_rust_project(base_dir.join("rust-project"));
    generate_python_project(base_dir.join("python-project"));
    generate_javascript_project(base_dir.join("javascript-project"));
    
    println!("Benchmark data generation complete!");
}

/// Generate a synthetic Rust project with various anti-patterns
fn generate_rust_project(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src")).unwrap();
    
    // Create Cargo.toml
    let mut cargo_toml = File::create(dir.join("Cargo.toml")).unwrap();
    cargo_toml.write_all(b"[package]
name = \"test-project\"
version = \"0.1.0\"
edition = \"2021\"

[dependencies]
serde = { version = \"1.0\", features = [\"derive\"] }
serde_json = \"1.0\"
tokio = { version = \"1.0\", features = [\"full\"] }
reqwest = \"0.11\"
").unwrap();
    
    // Create main.rs with global state (anti-pattern)
    let mut main_rs = File::create(dir.join("src/main.rs")).unwrap();
    main_rs.write_all(b"mod god_object;
mod cyclic_a;
mod cyclic_b;
mod magic_values;

// Global state anti-pattern
static mut GLOBAL_CONFIG: Option<Config> = None;

struct Config {
    api_key: String,
    endpoint: String,
    timeout: u64,
}

fn init_config() {
    unsafe {
        GLOBAL_CONFIG = Some(Config {
            api_key: \"1234567890\".to_string(), // Magic value anti-pattern
            endpoint: \"https://api.example.com\".to_string(),
            timeout: 30,
        });
    }
}

fn get_config() -> &'static Config {
    unsafe {
        GLOBAL_CONFIG.as_ref().unwrap()
    }
}

fn main() {
    init_config();
    
    // Using global state
    let config = get_config();
    println!(\"Using API endpoint: {}\", config.endpoint);
    
    // Using god object
    let mut manager = god_object::SystemManager::new();
    manager.start_service();
    manager.process_data();
    
    // Using cyclic modules
    cyclic_a::function_a();
}
").unwrap();
    
    // Create a god object (anti-pattern)
    let mut god_rs = File::create(dir.join("src/god_object.rs")).unwrap();
    god_rs.write_all(b"// God Object anti-pattern
pub struct SystemManager {
    config: String,
    database_connection: String,
    cache: Vec<String>,
    logger: String,
    http_client: String,
    authentication: bool,
    users: Vec<String>,
    items: Vec<String>,
    orders: Vec<String>,
    payments: Vec<String>,
    shipping: Vec<String>,
    reports: Vec<String>,
    notifications: Vec<String>,
    metrics: Vec<f64>,
    state: String,
}

impl SystemManager {
    pub fn new() -> Self {
        Self {
            config: String::new(),
            database_connection: String::new(),
            cache: Vec::new(),
            logger: String::new(),
            http_client: String::new(),
            authentication: false,
            users: Vec::new(),
            items: Vec::new(),
            orders: Vec::new(),
            payments: Vec::new(),
            shipping: Vec::new(),
            reports: Vec::new(),
            notifications: Vec::new(),
            metrics: Vec::new(),
            state: String::new(),
        }
    }
    
    // Too many methods in one class
    pub fn start_service(&mut self) {
        self.state = \"running\".to_string();
        println!(\"Service started\");
    }
    
    pub fn stop_service(&mut self) {
        self.state = \"stopped\".to_string();
        println!(\"Service stopped\");
    }
    
    pub fn process_data(&self) {
        println!(\"Processing data\");
    }
    
    pub fn authenticate_user(&mut self, _username: &str, _password: &str) -> bool {
        self.authentication = true;
        true
    }
    
    pub fn add_user(&mut self, user: String) {
        self.users.push(user);
    }
    
    pub fn remove_user(&mut self, user: &str) {
        self.users.retain(|u| u != user);
    }
    
    pub fn add_item(&mut self, item: String) {
        self.items.push(item);
    }
    
    pub fn create_order(&mut self, _user: &str, _items: Vec<String>) {
        self.orders.push(\"new order\".to_string());
    }
    
    pub fn process_payment(&mut self, _order_id: &str, _amount: f64) {
        self.payments.push(\"payment\".to_string());
    }
    
    pub fn ship_order(&mut self, _order_id: &str) {
        self.shipping.push(\"shipped\".to_string());
    }
    
    pub fn generate_report(&mut self) {
        self.reports.push(\"report\".to_string());
    }
    
    pub fn send_notification(&mut self, _user: &str, _message: &str) {
        self.notifications.push(\"notification\".to_string());
    }
    
    pub fn collect_metrics(&mut self, value: f64) {
        self.metrics.push(value);
    }
    
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }
}
").unwrap();
    
    // Create cyclic dependency (anti-pattern)
    let mut cyclic_a = File::create(dir.join("src/cyclic_a.rs")).unwrap();
    cyclic_a.write_all(b"// Part of a cyclic dependency
pub mod sub_a {
    pub struct A {
        pub value: i32,
    }
}

// Cyclic dependency with cyclic_b
pub fn function_a() {
    println!(\"Function A called\");
    crate::cyclic_b::function_b();
}
").unwrap();
    
    let mut cyclic_b = File::create(dir.join("src/cyclic_b.rs")).unwrap();
    cyclic_b.write_all(b"// Part of a cyclic dependency
use crate::cyclic_a::sub_a::A;

pub fn function_b() {
    println!(\"Function B called\");
    let a = A { value: 42 };
    println!(\"Value from A: {}\", a.value);
}
").unwrap();
    
    // Create magic values (anti-pattern)
    let mut magic_rs = File::create(dir.join("src/magic_values.rs")).unwrap();
    magic_rs.write_all(b"// Magic values anti-pattern
pub fn calculate_tax(amount: f64) -> f64 {
    // Magic value: 0.07 (tax rate)
    amount * 0.07
}

pub fn check_status(status_code: i32) -> bool {
    // Magic values: status codes
    if status_code == 200 || status_code == 201 || status_code == 204 {
        return true;
    }
    false
}

pub fn process_request(request_type: &str) -> u32 {
    // Magic strings and timeout values
    match request_type {
        \"fast\" => 30,     // 30 seconds timeout
        \"standard\" => 60, // 60 seconds timeout
        \"thorough\" => 300, // 300 seconds timeout
        _ => 120,          // Default timeout
    }
}
").unwrap();
}

/// Generate a synthetic Python project with anti-patterns
fn generate_python_project(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src")).unwrap();
    
    // Create requirements.txt
    let mut requirements = File::create(dir.join("requirements.txt")).unwrap();
    requirements.write_all(b"requests==2.28.1
flask==2.2.2
sqlalchemy==1.4.41
").unwrap();
    
    // Create main.py with various anti-patterns
    let mut main_py = File::create(dir.join("src/main.py")).unwrap();
    main_py.write_all(b"# Python main with anti-patterns
from src.god_class import SystemManager
from src.cyclic_a import function_a
import os

# Global state anti-pattern
GLOBAL_CONFIG = {
    'api_key': '1234567890', # Magic value
    'endpoint': 'https://api.example.com',
    'timeout': 30
}

def main():
    print(f\"Using API endpoint: {GLOBAL_CONFIG['endpoint']}\")
    
    # Using god class
    manager = SystemManager()
    manager.start_service()
    manager.process_data()
    
    # Using cyclic modules
    function_a()

if __name__ == \"__main__\":
    main()
").unwrap();
    
    // Create god class (anti-pattern)
    let mut god_py = File::create(dir.join("src/god_class.py")).unwrap();
    god_py.write_all(b"# God Class anti-pattern
class SystemManager:
    def __init__(self):
        self.config = {}
        self.database_connection = None
        self.cache = []
        self.logger = None
        self.http_client = None
        self.authentication = False
        self.users = []
        self.items = []
        self.orders = []
        self.payments = []
        self.shipping = []
        self.reports = []
        self.notifications = []
        self.metrics = []
        self.state = \"\"
    
    # Too many methods in one class
    def start_service(self):
        self.state = \"running\"
        print(\"Service started\")
    
    def stop_service(self):
        self.state = \"stopped\"
        print(\"Service stopped\")
    
    def process_data(self):
        print(\"Processing data\")
    
    def authenticate_user(self, username, password):
        self.authentication = True
        return True
    
    def add_user(self, user):
        self.users.append(user)
    
    def remove_user(self, user):
        if user in self.users:
            self.users.remove(user)
    
    def add_item(self, item):
        self.items.append(item)
    
    def create_order(self, user, items):
        self.orders.append(\"new order\")
    
    def process_payment(self, order_id, amount):
        self.payments.append(\"payment\")
    
    def ship_order(self, order_id):
        self.shipping.append(\"shipped\")
    
    def generate_report(self):
        self.reports.append(\"report\")
    
    def send_notification(self, user, message):
        self.notifications.append(\"notification\")
    
    def collect_metrics(self, value):
        self.metrics.append(value)
    
    def clear_cache(self):
        self.cache.clear()
").unwrap();
    
    // Create cyclic dependency (anti-pattern)
    fs::create_dir_all(dir.join("src/subpackage")).unwrap();
    
    let mut cyclic_a = File::create(dir.join("src/cyclic_a.py")).unwrap();
    cyclic_a.write_all(b"# Part of a cyclic dependency
from src.cyclic_b import function_b

class A:
    def __init__(self):
        self.value = 42

def function_a():
    print(\"Function A called\")
    function_b()
").unwrap();
    
    let mut cyclic_b = File::create(dir.join("src/cyclic_b.py")).unwrap();
    cyclic_b.write_all(b"# Part of a cyclic dependency
from src.cyclic_a import A

def function_b():
    print(\"Function B called\")
    a = A()
    print(f\"Value from A: {a.value}\")
").unwrap();
}

/// Generate a synthetic JavaScript project with anti-patterns
fn generate_javascript_project(dir: impl AsRef<Path>) {
    let dir = dir.as_ref();
    fs::create_dir_all(dir.join("src")).unwrap();
    
    // Create package.json
    let mut package_json = File::create(dir.join("package.json")).unwrap();
    package_json.write_all(b"{
  \"name\": \"test-project\",
  \"version\": \"1.0.0\",
  \"description\": \"Test project with anti-patterns\",
  \"main\": \"src/index.js\",
  \"dependencies\": {
    \"express\": \"^4.18.2\",
    \"axios\": \"^1.3.4\",
    \"lodash\": \"^4.17.21\"
  }
}
").unwrap();
    
    // Create index.js with various anti-patterns
    let mut index_js = File::create(dir.join("src/index.js")).unwrap();
    index_js.write_all(b"// JavaScript main with anti-patterns
const { SystemManager } = require('./godClass');
const { functionA } = require('./cyclicA');

// Global state anti-pattern
const GLOBAL_CONFIG = {
    apiKey: '1234567890', // Magic value
    endpoint: 'https://api.example.com',
    timeout: 30
};

function main() {
    console.log(`Using API endpoint: ${GLOBAL_CONFIG.endpoint}`);
    
    // Using god class
    const manager = new SystemManager();
    manager.startService();
    manager.processData();
    
    // Using cyclic modules
    functionA();
}

main();
").unwrap();
    
    // Create god class (anti-pattern)
    let mut god_js = File::create(dir.join("src/godClass.js")).unwrap();
    god_js.write_all(b"// God Class anti-pattern
class SystemManager {
    constructor() {
        this.config = {};
        this.databaseConnection = null;
        this.cache = [];
        this.logger = null;
        this.httpClient = null;
        this.authentication = false;
        this.users = [];
        this.items = [];
        this.orders = [];
        this.payments = [];
        this.shipping = [];
        this.reports = [];
        this.notifications = [];
        this.metrics = [];
        this.state = '';
    }
    
    // Too many methods in one class
    startService() {
        this.state = 'running';
        console.log('Service started');
    }
    
    stopService() {
        this.state = 'stopped';
        console.log('Service stopped');
    }
    
    processData() {
        console.log('Processing data');
    }
    
    authenticateUser(username, password) {
        this.authentication = true;
        return true;
    }
    
    addUser(user) {
        this.users.push(user);
    }
    
    removeUser(user) {
        this.users = this.users.filter(u => u !== user);
    }
    
    addItem(item) {
        this.items.push(item);
    }
    
    createOrder(user, items) {
        this.orders.push('new order');
    }
    
    processPayment(orderId, amount) {
        this.payments.push('payment');
    }
    
    shipOrder(orderId) {
        this.shipping.push('shipped');
    }
    
    generateReport() {
        this.reports.push('report');
    }
    
    sendNotification(user, message) {
        this.notifications.push('notification');
    }
    
    collectMetrics(value) {
        this.metrics.push(value);
    }
    
    clearCache() {
        this.cache = [];
    }
}

module.exports = { SystemManager };
").unwrap();
    
    // Create cyclic dependency (anti-pattern)
    let mut cyclic_a = File::create(dir.join("src/cyclicA.js")).unwrap();
    cyclic_a.write_all(b"// Part of a cyclic dependency
const { functionB } = require('./cyclicB');

class A {
    constructor() {
        this.value = 42;
    }
}

function functionA() {
    console.log('Function A called');
    functionB();
}

module.exports = { A, functionA };
").unwrap();
    
    let mut cyclic_b = File::create(dir.join("src/cyclicB.js")).unwrap();
    cyclic_b.write_all(b"// Part of a cyclic dependency
const { A } = require('./cyclicA');

function functionB() {
    console.log('Function B called');
    const a = new A();
    console.log(`Value from A: ${a.value}`);
}

module.exports = { functionB };
").unwrap();
}
