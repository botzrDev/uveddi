//! Code samples for testing anti-pattern detection

/// Example of a God Object in Rust - too many responsibilities
pub const GOD_OBJECT_RUST: &str = r#"
use std::collections::{HashMap, HashSet};
use std::fs::File;
use std::io::{Read, Write};
use serde::{Serialize, Deserialize};

/// This struct has too many responsibilities - it's a God Object
pub struct UserManager {
    // User management fields
    users: HashMap<u32, User>,
    active_sessions: HashMap<String, Session>,

    // Authentication fields
    password_hashes: HashMap<u32, String>,
    failed_login_attempts: HashMap<u32, u32>,

    // Caching fields
    user_cache: HashMap<u32, CachedUser>,
    session_cache: HashMap<String, CachedSession>,

    // Logging fields
    audit_log: Vec<AuditEntry>,
    error_log: Vec<ErrorEntry>,

    // Configuration fields
    config: SystemConfig,
    feature_flags: HashMap<String, bool>,

    // External service fields
    email_service: EmailService,
    sms_service: SmsService,
    notification_service: NotificationService,

    // Database fields
    database_connection: DatabaseConnection,
    connection_pool: ConnectionPool,

    // Analytics fields
    usage_metrics: HashMap<String, u64>,
    performance_metrics: PerformanceTracker,
}

impl UserManager {
    // User management methods
    pub fn create_user(&mut self, user_data: UserData) -> Result<User, UserError> {
        // Complex user creation logic
        todo!()
    }

    pub fn update_user(&mut self, user_id: u32, updates: UserUpdates) -> Result<(), UserError> {
        // User update logic
        todo!()
    }

    pub fn delete_user(&mut self, user_id: u32) -> Result<(), UserError> {
        // User deletion logic
        todo!()
    }

    pub fn get_user(&self, user_id: u32) -> Option<&User> {
        // User retrieval logic
        todo!()
    }

    pub fn list_users(&self, filter: UserFilter) -> Vec<&User> {
        // User listing logic
        todo!()
    }

    // Authentication methods
    pub fn authenticate(&mut self, username: &str, password: &str) -> Result<Session, AuthError> {
        // Authentication logic
        todo!()
    }

    pub fn validate_session(&self, session_token: &str) -> Result<&Session, AuthError> {
        // Session validation logic
        todo!()
    }

    pub fn logout(&mut self, session_token: &str) -> Result<(), AuthError> {
        // Logout logic
        todo!()
    }

    pub fn reset_password(&mut self, user_id: u32, new_password: &str) -> Result<(), AuthError> {
        // Password reset logic
        todo!()
    }

    pub fn check_password_strength(&self, password: &str) -> PasswordStrength {
        // Password validation logic
        todo!()
    }

    // Caching methods
    pub fn cache_user(&mut self, user: &User) {
        // User caching logic
        todo!()
    }

    pub fn invalidate_user_cache(&mut self, user_id: u32) {
        // Cache invalidation logic
        todo!()
    }

    pub fn warm_cache(&mut self) {
        // Cache warming logic
        todo!()
    }

    // Logging methods
    pub fn log_user_action(&mut self, user_id: u32, action: UserAction) {
        // Audit logging logic
        todo!()
    }

    pub fn log_error(&mut self, error: &str) {
        // Error logging logic
        todo!()
    }

    pub fn export_audit_log(&self) -> String {
        // Audit log export logic
        todo!()
    }

    // Configuration methods
    pub fn update_config(&mut self, config: SystemConfig) {
        // Configuration update logic
        todo!()
    }

    pub fn get_feature_flag(&self, flag_name: &str) -> bool {
        // Feature flag retrieval logic
        todo!()
    }

    pub fn set_feature_flag(&mut self, flag_name: &str, enabled: bool) {
        // Feature flag setting logic
        todo!()
    }

    // Notification methods
    pub fn send_email(&self, user_id: u32, subject: &str, body: &str) -> Result<(), NotificationError> {
        // Email sending logic
        todo!()
    }

    pub fn send_sms(&self, user_id: u32, message: &str) -> Result<(), NotificationError> {
        // SMS sending logic
        todo!()
    }

    pub fn send_push_notification(&self, user_id: u32, notification: PushNotification) -> Result<(), NotificationError> {
        // Push notification logic
        todo!()
    }

    // Database methods
    pub fn save_user_to_db(&self, user: &User) -> Result<(), DatabaseError> {
        // Database save logic
        todo!()
    }

    pub fn load_user_from_db(&self, user_id: u32) -> Result<User, DatabaseError> {
        // Database load logic
        todo!()
    }

    pub fn backup_database(&self) -> Result<(), DatabaseError> {
        // Database backup logic
        todo!()
    }

    // Analytics methods
    pub fn track_user_event(&mut self, user_id: u32, event: UserEvent) {
        // Event tracking logic
        todo!()
    }

    pub fn generate_usage_report(&self) -> UsageReport {
        // Usage report generation logic
        todo!()
    }

    pub fn get_performance_metrics(&self) -> PerformanceReport {
        // Performance metrics logic
        todo!()
    }
}

// Supporting types (normally these would be in separate modules)
#[derive(Debug)]
pub struct User { id: u32, username: String, email: String }
#[derive(Debug)]
pub struct Session { token: String, user_id: u32, expires_at: std::time::SystemTime }
#[derive(Debug)]
pub struct UserData { username: String, email: String, password: String }
#[derive(Debug)]
pub struct UserUpdates { email: Option<String>, username: Option<String> }
#[derive(Debug)]
pub struct UserFilter { active_only: bool, search_term: Option<String> }
#[derive(Debug)]
pub struct CachedUser { user: User, cached_at: std::time::SystemTime }
#[derive(Debug)]
pub struct CachedSession { session: Session, cached_at: std::time::SystemTime }
#[derive(Debug)]
pub struct AuditEntry { user_id: u32, action: String, timestamp: std::time::SystemTime }
#[derive(Debug)]
pub struct ErrorEntry { error: String, timestamp: std::time::SystemTime }
#[derive(Debug)]
pub struct SystemConfig { max_login_attempts: u32, session_timeout: u64 }
#[derive(Debug)]
pub struct EmailService;
#[derive(Debug)]
pub struct SmsService;
#[derive(Debug)]
pub struct NotificationService;
#[derive(Debug)]
pub struct DatabaseConnection;
#[derive(Debug)]
pub struct ConnectionPool;
#[derive(Debug)]
pub struct PerformanceTracker;
#[derive(Debug)]
pub struct UserAction;
#[derive(Debug)]
pub struct UserEvent;
#[derive(Debug)]
pub struct PushNotification;
#[derive(Debug)]
pub struct UsageReport;
#[derive(Debug)]
pub struct PerformanceReport;
#[derive(Debug)]
pub enum PasswordStrength { Weak, Medium, Strong }
#[derive(Debug)]
pub enum UserError { NotFound, ValidationError, DatabaseError }
#[derive(Debug)]
pub enum AuthError { InvalidCredentials, SessionExpired, TooManyAttempts }
#[derive(Debug)]
pub enum NotificationError { ServiceUnavailable, InvalidRecipient }
#[derive(Debug)]
pub enum DatabaseError { ConnectionFailed, QueryFailed }
"#;

/// Example of Data Clumps anti-pattern in Python
pub const DATA_CLUMPS_PYTHON: &str = r#"
class UserService:
    """Service with data clumps - same parameter groups appear repeatedly"""

    def create_user_profile(self, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Creates a user profile with address information"""
        # Implementation here
        pass

    def update_user_contact(self, user_id, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Updates user contact and address information"""
        # Implementation here
        pass

    def send_user_notification(self, first_name, last_name, email, phone, street, city, state, zip_code, country, message):
        """Sends notification to user at their address"""
        # Implementation here
        pass

    def validate_user_data(self, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Validates user personal and address data"""
        # Implementation here
        pass

class OrderService:
    """Another service with the same data clumps"""

    def create_shipping_order(self, order_id, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Creates a shipping order with customer details"""
        # Same parameter clump appears here
        pass

    def calculate_shipping_cost(self, weight, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Calculates shipping cost based on address"""
        # Same parameter clump appears here
        pass

    def track_shipment(self, tracking_id, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Tracks shipment to customer address"""
        # Same parameter clump appears here
        pass

class PaymentService:
    """Payment service also using the same data clumps"""

    def process_payment(self, amount, currency, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Processes payment with billing address"""
        # Same parameter clump appears here
        pass

    def send_receipt(self, transaction_id, first_name, last_name, email, phone, street, city, state, zip_code, country):
        """Sends receipt to customer"""
        # Same parameter clump appears here
        pass

# The correct approach would be to create classes for the data clumps:
class PersonalInfo:
    def __init__(self, first_name, last_name, email, phone):
        self.first_name = first_name
        self.last_name = last_name
        self.email = email
        self.phone = phone

class Address:
    def __init__(self, street, city, state, zip_code, country):
        self.street = street
        self.city = city
        self.state = state
        self.zip_code = zip_code
        self.country = country

class Customer:
    def __init__(self, personal_info, address):
        self.personal_info = personal_info
        self.address = address

# Then services would use the Customer class instead of individual parameters
"#;

/// Example of Cyclic Dependencies in JavaScript
pub const CYCLIC_DEPENDENCIES_JS: &str = r#"
// File: userService.js
const orderService = require('./orderService');
const paymentService = require('./paymentService');

class UserService {
    constructor() {
        this.orders = [];
        this.payments = [];
    }

    createUser(userData) {
        // User creation logic
        const user = { id: Date.now(), ...userData };

        // Creates cyclic dependency - UserService depends on OrderService
        orderService.initializeUserOrders(user.id);

        return user;
    }

    getUserOrders(userId) {
        // Another dependency on OrderService
        return orderService.getOrdersByUserId(userId);
    }

    processUserPayment(userId, amount) {
        // Dependency on PaymentService
        return paymentService.processPayment(userId, amount);
    }
}

module.exports = new UserService();

// File: orderService.js
const userService = require('./userService'); // Cyclic dependency!
const paymentService = require('./paymentService');

class OrderService {
    constructor() {
        this.orders = [];
    }

    createOrder(orderData) {
        const order = { id: Date.now(), ...orderData };

        // Creates cyclic dependency - OrderService depends on UserService
        const user = userService.getUserById(orderData.userId);
        order.userEmail = user.email;

        this.orders.push(order);
        return order;
    }

    initializeUserOrders(userId) {
        // Called by UserService, creating circular call
        const userOrders = this.getOrdersByUserId(userId);
        return userOrders;
    }

    getOrdersByUserId(userId) {
        return this.orders.filter(order => order.userId === userId);
    }

    processOrderPayment(orderId) {
        const order = this.orders.find(o => o.id === orderId);
        if (order) {
            // Another dependency
            return paymentService.processPayment(order.userId, order.total);
        }
    }
}

module.exports = new OrderService();

// File: paymentService.js
const userService = require('./userService'); // Another cyclic dependency!
const orderService = require('./orderService'); // And another one!

class PaymentService {
    constructor() {
        this.payments = [];
    }

    processPayment(userId, amount) {
        // Creates cyclic dependency - PaymentService depends on UserService
        const user = userService.getUserById(userId);

        const payment = {
            id: Date.now(),
            userId,
            amount,
            userEmail: user.email,
            timestamp: new Date()
        };

        this.payments.push(payment);

        // Another cyclic dependency with OrderService
        orderService.updateOrderPaymentStatus(payment.orderId, 'paid');

        return payment;
    }

    getPaymentsByUserId(userId) {
        return this.payments.filter(payment => payment.userId === userId);
    }

    refundPayment(paymentId) {
        const payment = this.payments.find(p => p.id === paymentId);
        if (payment) {
            // Yet another cyclic dependency
            userService.notifyUser(payment.userId, 'refund_processed');
        }
    }
}

module.exports = new PaymentService();

// The correct approach would be to:
// 1. Extract shared data models into separate modules
// 2. Use dependency injection
// 3. Create a clear layered architecture
// 4. Use events/observers instead of direct dependencies
"#;

/// Example of clean, well-structured Rust code
pub const CLEAN_RUST_CODE: &str = r#"
//! Clean, well-structured user management module
//!
//! This module demonstrates good separation of concerns,
//! single responsibility principle, and clean architecture.

use std::collections::HashMap;
use uuid::Uuid;
use chrono::{DateTime, Utc};

/// Represents a user in the system
#[derive(Debug, Clone)]
pub struct User {
    pub id: Uuid,
    pub username: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// User repository trait for data persistence
pub trait UserRepository {
    type Error;

    async fn create(&self, user: User) -> Result<User, Self::Error>;
    async fn find_by_id(&self, id: Uuid) -> Result<Option<User>, Self::Error>;
    async fn find_by_username(&self, username: &str) -> Result<Option<User>, Self::Error>;
    async fn update(&self, user: User) -> Result<User, Self::Error>;
    async fn delete(&self, id: Uuid) -> Result<bool, Self::Error>;
}

/// User service for business logic
pub struct UserService<R: UserRepository> {
    repository: R,
}

impl<R: UserRepository> UserService<R> {
    pub fn new(repository: R) -> Self {
        Self { repository }
    }

    pub async fn create_user(&self, username: String, email: String) -> Result<User, R::Error> {
        let user = User {
            id: Uuid::new_v4(),
            username,
            email,
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        self.repository.create(user).await
    }

    pub async fn get_user(&self, id: Uuid) -> Result<Option<User>, R::Error> {
        self.repository.find_by_id(id).await
    }

    pub async fn update_user_email(&self, id: Uuid, new_email: String) -> Result<Option<User>, R::Error> {
        if let Some(mut user) = self.repository.find_by_id(id).await? {
            user.email = new_email;
            user.updated_at = Utc::now();
            Ok(Some(self.repository.update(user).await?))
        } else {
            Ok(None)
        }
    }
}

/// Email validation utility
pub struct EmailValidator;

impl EmailValidator {
    pub fn is_valid(email: &str) -> bool {
        email.contains('@') && email.contains('.')
    }
}

/// Username validation utility
pub struct UsernameValidator;

impl UsernameValidator {
    pub fn is_valid(username: &str) -> bool {
        username.len() >= 3 && username.len() <= 50 && username.chars().all(char::is_alphanumeric)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct MockUserRepository {
        users: HashMap<Uuid, User>,
    }

    impl MockUserRepository {
        fn new() -> Self {
            Self {
                users: HashMap::new(),
            }
        }
    }

    #[async_trait::async_trait]
    impl UserRepository for MockUserRepository {
        type Error = &'static str;

        async fn create(&self, user: User) -> Result<User, Self::Error> {
            Ok(user)
        }

        async fn find_by_id(&self, _id: Uuid) -> Result<Option<User>, Self::Error> {
            Ok(None)
        }

        async fn find_by_username(&self, _username: &str) -> Result<Option<User>, Self::Error> {
            Ok(None)
        }

        async fn update(&self, user: User) -> Result<User, Self::Error> {
            Ok(user)
        }

        async fn delete(&self, _id: Uuid) -> Result<bool, Self::Error> {
            Ok(true)
        }
    }

    #[tokio::test]
    async fn test_create_user() {
        let repository = MockUserRepository::new();
        let service = UserService::new(repository);

        let result = service.create_user("testuser".to_string(), "test@example.com".to_string()).await;
        assert!(result.is_ok());

        let user = result.unwrap();
        assert_eq!(user.username, "testuser");
        assert_eq!(user.email, "test@example.com");
    }

    #[test]
    fn test_email_validation() {
        assert!(EmailValidator::is_valid("test@example.com"));
        assert!(!EmailValidator::is_valid("invalid-email"));
    }

    #[test]
    fn test_username_validation() {
        assert!(UsernameValidator::is_valid("testuser"));
        assert!(!UsernameValidator::is_valid("a")); // too short
        assert!(!UsernameValidator::is_valid("user@name")); // invalid characters
    }
}
"#;

/// Example of clean Python code
pub const CLEAN_PYTHON_CODE: &str = r#"
"""
Clean, well-structured user management module.

This module demonstrates good separation of concerns,
single responsibility principle, and clean architecture.
"""

from abc import ABC, abstractmethod
from datetime import datetime
from typing import Optional, List
from uuid import UUID, uuid4
from dataclasses import dataclass


@dataclass
class User:
    """Represents a user in the system."""
    id: UUID
    username: str
    email: str
    created_at: datetime
    updated_at: datetime


class UserRepository(ABC):
    """Abstract repository for user data persistence."""

    @abstractmethod
    async def create(self, user: User) -> User:
        """Create a new user."""
        pass

    @abstractmethod
    async def find_by_id(self, user_id: UUID) -> Optional[User]:
        """Find user by ID."""
        pass

    @abstractmethod
    async def find_by_username(self, username: str) -> Optional[User]:
        """Find user by username."""
        pass

    @abstractmethod
    async def update(self, user: User) -> User:
        """Update an existing user."""
        pass

    @abstractmethod
    async def delete(self, user_id: UUID) -> bool:
        """Delete a user."""
        pass


class UserService:
    """Service for user business logic."""

    def __init__(self, repository: UserRepository):
        self.repository = repository

    async def create_user(self, username: str, email: str) -> User:
        """Create a new user with validation."""
        if not EmailValidator.is_valid(email):
            raise ValueError("Invalid email format")

        if not UsernameValidator.is_valid(username):
            raise ValueError("Invalid username")

        user = User(
            id=uuid4(),
            username=username,
            email=email,
            created_at=datetime.utcnow(),
            updated_at=datetime.utcnow()
        )

        return await self.repository.create(user)

    async def get_user(self, user_id: UUID) -> Optional[User]:
        """Get user by ID."""
        return await self.repository.find_by_id(user_id)

    async def update_user_email(self, user_id: UUID, new_email: str) -> Optional[User]:
        """Update user's email address."""
        if not EmailValidator.is_valid(new_email):
            raise ValueError("Invalid email format")

        user = await self.repository.find_by_id(user_id)
        if not user:
            return None

        user.email = new_email
        user.updated_at = datetime.utcnow()

        return await self.repository.update(user)


class EmailValidator:
    """Utility class for email validation."""

    @staticmethod
    def is_valid(email: str) -> bool:
        """Check if email format is valid."""
        return '@' in email and '.' in email and len(email) > 5


class UsernameValidator:
    """Utility class for username validation."""

    @staticmethod
    def is_valid(username: str) -> bool:
        """Check if username is valid."""
        return (
            3 <= len(username) <= 50 and
            username.isalnum()
        )


# Example of clean testing
import unittest
from unittest.mock import AsyncMock


class MockUserRepository(UserRepository):
    """Mock repository for testing."""

    def __init__(self):
        self.users = {}

    async def create(self, user: User) -> User:
        self.users[user.id] = user
        return user

    async def find_by_id(self, user_id: UUID) -> Optional[User]:
        return self.users.get(user_id)

    async def find_by_username(self, username: str) -> Optional[User]:
        for user in self.users.values():
            if user.username == username:
                return user
        return None

    async def update(self, user: User) -> User:
        self.users[user.id] = user
        return user

    async def delete(self, user_id: UUID) -> bool:
        if user_id in self.users:
            del self.users[user_id]
            return True
        return False


class TestUserService(unittest.TestCase):
    """Test cases for UserService."""

    def setUp(self):
        self.repository = MockUserRepository()
        self.service = UserService(self.repository)

    async def test_create_user(self):
        """Test user creation."""
        user = await self.service.create_user("testuser", "test@example.com")

        self.assertEqual(user.username, "testuser")
        self.assertEqual(user.email, "test@example.com")
        self.assertIsNotNone(user.id)

    def test_email_validation(self):
        """Test email validation."""
        self.assertTrue(EmailValidator.is_valid("test@example.com"))
        self.assertFalse(EmailValidator.is_valid("invalid-email"))

    def test_username_validation(self):
        """Test username validation."""
        self.assertTrue(UsernameValidator.is_valid("testuser"))
        self.assertFalse(UsernameValidator.is_valid("a"))  # too short
        self.assertFalse(UsernameValidator.is_valid("user@name"))  # invalid chars


if __name__ == '__main__':
    unittest.main()
"#;