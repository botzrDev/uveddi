// Test file for Tight Coupling detection

use std::collections::HashMap;

pub struct Database {
    pub connection: String,
    pub tables: HashMap<String, String>,
}

pub struct UserService {
    pub database: Database,  // Direct dependency - tight coupling
    pub email_service: EmailService,  // Another direct dependency
}

pub struct EmailService {
    pub smtp_server: String,
    pub user_service: Option<Box<UserService>>,  // Circular reference
}

pub struct OrderService {
    pub user_service: UserService,  // Depends on UserService
    pub payment_service: PaymentService,
    pub inventory_service: InventoryService,
}

pub struct PaymentService {
    pub database: Database,  // Multiple services depend on Database directly
    pub order_service: Option<Box<OrderService>>,  // Another circular reference
}

pub struct InventoryService {
    pub database: Database,  // More direct coupling to Database
    pub order_service: Option<Box<OrderService>>,
}

impl UserService {
    pub fn create_user(&mut self, name: String) {
        // Directly manipulates database - tight coupling
        self.database.tables.insert(name.clone(), "user_data".to_string());
        
        // Directly calls email service - tight coupling
        self.email_service.send_welcome_email(&name);
    }
}

impl EmailService {
    pub fn send_welcome_email(&self, name: &str) {
        println!("Sending email to {}", name);
        
        // Access SMTP directly - tight coupling to infrastructure
        let _ = &self.smtp_server;
    }
}

impl OrderService {
    pub fn create_order(&mut self, user: String, item: String) {
        // Direct manipulation of multiple services
        self.user_service.database.tables.insert(user, "order".to_string());
        self.payment_service.database.tables.insert(item.clone(), "payment".to_string());
        self.inventory_service.database.tables.remove(&item);
    }
}
