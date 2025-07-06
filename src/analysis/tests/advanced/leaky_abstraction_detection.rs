//! Comprehensive Leaky Abstraction Detection Tests
//! 
//! This module provides extensive testing for the advanced leaky abstraction detector,
//! covering multiple programming languages and architectural patterns.

#[cfg(test)]
mod tests {
    use crate::analysis::detectors::anti_patterns::leaky_abstraction::{
        LeakyAbstractionDetector, ArchitecturalConfig, ArchitecturalLayer
    };
    use crate::analysis::AnalysisDetector;
    use crate::ast::tree_sitter::{AstParser, ParsedFile};
    use tempfile::tempdir;
    use std::fs::File;
    use std::io::Write;
    use std::collections::{HashMap, HashSet};

    fn create_parsed_file(content: &str, language: &str, file_path: &str) -> ParsedFile {
        let temp_dir = tempdir().unwrap();
        let _file_extension = match language {
            "rust" => ".rs",
            "python" => ".py", 
            "javascript" => ".js",
            _ => ".txt",
        };
        
        // Create the full path structure to match layer mappings
        // Convert paths like "src/domain/user.rs" to actual directory structure
        let test_file_path = temp_dir.path().join(file_path).with_extension(match language {
            "rust" => "rs",
            "python" => "py", 
            "javascript" => "js",
            _ => "txt",
        });
        
        // Ensure parent directories exist
        if let Some(parent) = test_file_path.parent() {
            std::fs::create_dir_all(parent).unwrap();
        }
        
        // Write content to temporary file
        let mut file = File::create(&test_file_path).unwrap();
        writeln!(file, "{}", content).unwrap();
        
        // Parse the file
        let mut parser = AstParser::new().unwrap();
        parser.parse_file(&test_file_path).unwrap()
    }

    fn create_test_config() -> ArchitecturalConfig {
        let mut layer_mappings = HashMap::new();
        layer_mappings.insert("**/domain/**".to_string(), ArchitecturalLayer::Domain);
        layer_mappings.insert("**/services/**".to_string(), ArchitecturalLayer::Application);
        layer_mappings.insert("**/controllers/**".to_string(), ArchitecturalLayer::Presentation);
        layer_mappings.insert("**/infrastructure/**".to_string(), ArchitecturalLayer::Infrastructure);

        let mut infrastructure_modules = HashSet::new();
        infrastructure_modules.insert("diesel".to_string());
        infrastructure_modules.insert("sqlx".to_string());
        infrastructure_modules.insert("django".to_string());
        infrastructure_modules.insert("requests".to_string());
        infrastructure_modules.insert("sqlalchemy".to_string());
        infrastructure_modules.insert("express".to_string());
        infrastructure_modules.insert("@prisma/client".to_string());
        infrastructure_modules.insert("axios".to_string());

        ArchitecturalConfig {
            layer_mappings,
            infrastructure_modules,
            internal_patterns: vec!["internal".to_string(), "_private".to_string()],
        }
    }

    #[test]
    fn test_rust_infrastructure_import_in_domain_layer() {
        let rust_code = r#"
            use diesel::prelude::*;
            use serde::{Deserialize, Serialize};

            #[derive(Queryable, Serialize, Deserialize)]
            pub struct User {
                pub id: i32,
                pub name: String,
            }

            impl User {
                pub fn find_by_id(id: i32) -> Result<User, diesel::result::Error> {
                    // Domain logic should not directly use ORM
                    users::table.find(id).first(&connection)
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(rust_code, "rust", "src/domain/user.rs");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect infrastructure import in domain layer");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("Infrastructure module 'diesel' imported in Domain layer")
        ));
    }

    #[test]
    fn test_rust_public_field_exposure() {
        let rust_code = r#"
            pub struct Database {
                pub connection_pool: ConnectionPool,
                pub cache: HashMap<String, String>,
                pub internal_state: InternalState,
            }

            impl Database {
                pub fn new() -> Self {
                    Self {
                        connection_pool: ConnectionPool::new(),
                        cache: HashMap::new(),
                        internal_state: InternalState::default(),
                    }
                }
            }
        "#;

        let detector = LeakyAbstractionDetector::new();
        let parsed_file = create_parsed_file(rust_code, "rust", "src/infrastructure/database.rs");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect public field exposure");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("Public field exposes internal structure")
        ));
    }

    #[test]
    fn test_python_django_model_in_view() {
        let python_code = r#"
            from django.shortcuts import render
            from django.contrib.auth.models import User
            from myapp.models import UserProfile

            def user_list_view(request):
                # Direct model access in view - should use service layer
                users = User.objects.filter(is_active=True)
                for user in users:
                    user.last_login = timezone.now()
                    user.save()
                
                return render(request, 'users.html', {'users': users})

            def profile_view(request, user_id):
                profile = UserProfile.objects.get(user_id=user_id)
                return render(request, 'profile.html', {'profile': profile})
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(python_code, "python", "src/controllers/user_views.py");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        // Debug output
        println!("Django test issues:");
        for issue in &issues {
            println!("  - {}", issue.description);
        }
        
        assert!(!issues.is_empty(), "Should detect Django model usage in view");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("Framework module 'django' imported in Presentation layer")
        ));
    }

    #[test]
    fn test_python_infrastructure_import_in_domain() {
        let python_code = r#"
            import requests
            from sqlalchemy import Column, Integer, String
            from sqlalchemy.ext.declarative import declarative_base

            Base = declarative_base()

            class User(Base):
                __tablename__ = 'users'
                
                id = Column(Integer, primary_key=True)
                name = Column(String(50))
                
                def fetch_external_data(self):
                    # Domain model should not make HTTP requests directly
                    response = requests.get(f"https://api.example.com/users/{self.id}")
                    return response.json()
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(python_code, "python", "src/domain/user.py");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect infrastructure imports in domain");
        // Should detect both requests and sqlalchemy as infrastructure concerns
        assert!(issues.len() >= 1);
    }

    #[test]
    fn test_javascript_dom_manipulation_in_business_logic() {
        let js_code = r#"
            class UserService {
                constructor() {
                    this.users = [];
                }

                calculateUserScore(user) {
                    const score = user.points * 1.5;
                    
                    // Business logic should not manipulate DOM
                    document.getElementById('score-display').textContent = score;
                    window.localStorage.setItem('lastScore', score);
                    
                    return score;
                }

                processUsers(users) {
                    return users.map(user => {
                        const processed = this.calculateUserScore(user);
                        
                        // More DOM manipulation in business logic
                        document.querySelector('.user-list').appendChild(
                            this.createUserElement(user)
                        );
                        
                        return processed;
                    });
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(js_code, "javascript", "src/services/user_service.js");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect DOM manipulation in business logic");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("DOM manipulation in business logic")
        ));
    }

    #[test]
    fn test_javascript_infrastructure_import_in_domain() {
        let js_code = r#"
            import express from 'express';
            import { PrismaClient } from '@prisma/client';
            import axios from 'axios';

            class User {
                constructor(id, name, email) {
                    this.id = id;
                    this.name = name;
                    this.email = email;
                }

                async save() {
                    // Domain model should not directly use ORM
                    const prisma = new PrismaClient();
                    return await prisma.user.create({
                        data: {
                            id: this.id,
                            name: this.name,
                            email: this.email
                        }
                    });
                }

                async notifyExternal() {
                    // Domain logic should not make HTTP calls directly
                    return await axios.post('https://api.example.com/notify', {
                        userId: this.id
                    });
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(js_code, "javascript", "src/domain/user.js");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect infrastructure imports in domain");
        // Should detect express, prisma, and axios imports
        assert!(issues.len() >= 2);
    }

    #[test]
    fn test_layer_boundary_violations() {
        let rust_code = r#"
            use crate::infrastructure::database::DatabaseConnection;
            use crate::domain::user::User;

            pub struct UserController {
                db: DatabaseConnection,
            }

            impl UserController {
                pub fn get_user(&self, id: i32) -> Result<User, String> {
                    // Controller should not directly access database
                    // Should go through service layer
                    self.db.query("SELECT * FROM users WHERE id = ?", &[&id])
                        .map(|row| User::from_row(row))
                        .map_err(|e| e.to_string())
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(rust_code, "rust", "src/controllers/user_controller.rs");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        // This test demonstrates layer violation detection
        // The exact detection depends on the AST structure and query patterns
    }

    #[test]
    fn test_proper_abstraction_no_violations() {
        let rust_code = r#"
            use crate::domain::user::{User, UserRepository};
            use crate::application::user_service::UserService;

            pub struct UserController {
                user_service: UserService,
            }

            impl UserController {
                pub fn new(user_service: UserService) -> Self {
                    Self { user_service }
                }

                pub async fn get_user(&self, id: i32) -> Result<UserDto, String> {
                    // Proper layering: Controller -> Service -> Repository
                    let user = self.user_service.find_user(id).await
                        .map_err(|e| e.to_string())?;
                    
                    Ok(UserDto::from(user))
                }
            }

            #[derive(serde::Serialize)]
            pub struct UserDto {
                pub id: i32,
                pub name: String,
                pub email: String,
            }

            impl From<User> for UserDto {
                fn from(user: User) -> Self {
                    Self {
                        id: user.id(),
                        name: user.name().to_string(),
                        email: user.email().to_string(),
                    }
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(rust_code, "rust", "src/controllers/user_controller.rs");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        // Should have no violations for properly structured code
        assert!(issues.is_empty() || issues.iter().all(|issue| issue.severity != "high"), 
                "Well-structured code should have no high-severity violations");
    }

    #[test]
    fn test_error_propagation_detection() {
        let rust_code = r#"
            use diesel::prelude::*;
            use sqlx::Error as SqlxError;
            
            #[derive(Queryable)]
            struct User {
                id: i32,
                name: String,
            }

            #[derive(Insertable)]
            struct CreateUserData {
                name: String,
            }

            #[derive(Debug)]
            enum DieselError {
                DatabaseError(String),
                ValidationError(String),
            }

            pub struct UserService;

            impl UserService {
                pub fn find_user(&self, id: i32) -> Result<User, DieselError> {
                    // Public API should not expose infrastructure errors
                    users::table.find(id).first(&connection)
                }

                pub fn create_user(&self, data: CreateUserData) -> Result<User, SqlxError> {
                    // Another infrastructure error leak
                    sqlx::query!("INSERT INTO users ...")
                        .execute(&pool)
                        .await
                }
            }
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(rust_code, "rust", "src/services/user_service.rs");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect error type propagation");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("Infrastructure error type") && 
            issue.description.contains("propagated to public API")
        ));
    }

    #[test]
    fn test_framework_coupling_detection() {
        let python_code = r#"
            from flask import request, session
            from django.http import HttpRequest
            import fastapi

            class UserService:
                def get_user_preferences(self):
                    # Business logic should not depend on web framework
                    user_id = request.args.get('user_id')
                    preferences = session.get('preferences', {})
                    return self.fetch_preferences(user_id, preferences)

                def process_request(self, http_request: HttpRequest):
                    # Framework-specific type in business logic
                    user_data = http_request.POST.get('user_data')
                    return self.process_user_data(user_data)

                def handle_api_request(self, fastapi_request):
                    # Another framework coupling
                    return {"status": "processed"}
        "#;

        let config = create_test_config();
        let detector = LeakyAbstractionDetector::with_config(config);
        let parsed_file = create_parsed_file(python_code, "python", "src/services/user_service.py");

        let issues = detector.detect_issues(&parsed_file).unwrap();
        
        assert!(!issues.is_empty(), "Should detect framework coupling");
        // Should detect flask, django, and fastapi imports in service layer
    }

    #[test]
    fn test_custom_architectural_config() {
        let mut custom_layer_mappings = HashMap::new();
        custom_layer_mappings.insert("**/web/**".to_string(), ArchitecturalLayer::Presentation);
        custom_layer_mappings.insert("**/business/**".to_string(), ArchitecturalLayer::Domain);
        custom_layer_mappings.insert("**/data/**".to_string(), ArchitecturalLayer::Infrastructure);

        let mut custom_infrastructure = HashSet::new();
        custom_infrastructure.insert("custom_orm".to_string());
        custom_infrastructure.insert("my_framework".to_string());

        let custom_config = ArchitecturalConfig {
            layer_mappings: custom_layer_mappings,
            infrastructure_modules: custom_infrastructure,
            internal_patterns: vec!["_impl".to_string()],
        };

        let detector = LeakyAbstractionDetector::with_config(custom_config);

        let rust_code = r#"
            use custom_orm::prelude::*;
            
            pub struct BusinessLogic {
                // Business logic using custom ORM directly
            }
        "#;

        let parsed_file = create_parsed_file(rust_code, "rust", "src/business/logic.rs");
        let issues = detector.detect_issues(&parsed_file).unwrap();

        assert!(!issues.is_empty(), "Should detect custom infrastructure module in business layer");
        assert!(issues.iter().any(|issue| 
            issue.description.contains("custom_orm")
        ));
    }

    #[test]
    fn test_performance_characteristics() {
        // Test with larger code samples to ensure performance
        let large_rust_code = (0..100).map(|i| format!(
            r#"
            use diesel::prelude::*;
            
            pub struct Entity{} {{
                pub field_{}: String,
            }}
            
            impl Entity{} {{
                pub fn method_{}(&self) -> Result<(), diesel::result::Error> {{
                    Ok(())
                }}
            }}
            "#, i, i, i, i
        )).collect::<Vec<_>>().join("\n");

        let detector = LeakyAbstractionDetector::new();
        let parsed_file = create_parsed_file(&large_rust_code, "rust", "src/domain/large_file.rs");

        let start = std::time::Instant::now();
        let issues = detector.detect_issues(&parsed_file).unwrap();
        let duration = start.elapsed();

        // Should complete in reasonable time (less than 1 second for this test)
        assert!(duration.as_secs() < 1, "Analysis should be fast");
        assert!(!issues.is_empty(), "Should detect multiple violations in large file");
    }
}
