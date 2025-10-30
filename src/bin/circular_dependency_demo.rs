//! Circular Dependency Resolution Demo
//!
//! This tool demonstrates the circular dependency resolution implementation
//! as part of UV-105. It shows how the new interfaces break circular dependencies
//! and enable clean architecture.

use std::sync::Arc;
use uveddi::core::interfaces::{
    events::{AnalysisEvent, AstData, AstEvent, AstMetadata, DomainEvent, EventBus},
    persistence::{
        AnalysisRunDomain, DomainIssue, IssueSeverity, MockPersistenceProvider, PersistenceProvider,
    },
};

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🔄 Circular Dependency Resolution Demo (UV-105)");
    println!("==============================================\n");

    // Demonstrate Phase 1.1: Persistence Interface
    println!("📊 Phase 1.1: Persistence Interface Demo");
    demonstrate_persistence_interface().await?;

    println!("\n🎪 Phase 1.2: Event System Demo");
    demonstrate_event_system().await?;

    println!("\n✅ Circular Dependencies Successfully Resolved!");
    println!("   • Analysis ↔ Database cycle eliminated via PersistenceProvider interface");
    println!("   • AST ↔ Analysis cycle eliminated via Event Bus communication");
    println!("   • Clean architecture boundaries established");

    Ok(())
}

/// Demonstrates the persistence interface that breaks Analysis ↔ Database circular dependency
async fn demonstrate_persistence_interface() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Creating mock persistence provider...");
    let persistence = Arc::new(MockPersistenceProvider::new());

    // Create some sample domain issues (no direct database dependency)
    let issues = vec![
        DomainIssue::new(
            "god_object_detector".to_string(),
            "architectural".to_string(),
            IssueSeverity::High,
            "Class has too many responsibilities".to_string(),
            "The AnalysisEngine class violates Single Responsibility Principle".to_string(),
            "src/analysis/engine.rs".to_string(),
            45,
        ),
        DomainIssue::new(
            "circular_dependency_detector".to_string(),
            "architectural".to_string(),
            IssueSeverity::Critical,
            "Circular dependency detected".to_string(),
            "analysis ↔ database circular dependency found".to_string(),
            "src/analysis/services.rs".to_string(),
            123,
        ),
    ];

    println!(
        "  Saving {} issues via persistence interface...",
        issues.len()
    );
    persistence.save_issues(issues).await?;

    // Create and save analysis run
    let run = AnalysisRunDomain::new(Some(1));
    let run_id = persistence.save_analysis_run(run).await?;
    println!("  Created analysis run with ID: {}", run_id);

    // Get statistics
    let stats = persistence.get_issue_stats().await?;
    println!("  Issue statistics: {} total issues", stats.total_issues);

    println!("  ✅ Persistence interface working - no circular dependency!");

    Ok(())
}

/// Demonstrates the event system that breaks AST ↔ Analysis circular dependency
async fn demonstrate_event_system() -> Result<(), Box<dyn std::error::Error>> {
    println!("  Creating event bus...");
    let event_bus = Arc::new(EventBus::new());

    // Subscribe to events
    let mut subscriber = event_bus.subscribe();

    // Simulate AST parsing event (no direct analysis dependency)
    let ast_data = AstData {
        syntax_tree: serde_json::json!({
            "type": "source_file",
            "children": [
                {"type": "function_item", "name": "analyze"}
            ]
        }),
        metadata: AstMetadata {
            language: "rust".to_string(),
            file_size: 1024,
            node_count: 45,
            depth: 6,
            is_valid: true,
        },
    };

    let ast_event = AstEvent::file_parsed(
        "src/main.rs".to_string(),
        ast_data,
        std::time::Duration::from_millis(50),
    );

    println!("  Publishing AST event...");
    event_bus.publish(DomainEvent::Ast(ast_event))?;

    // Simulate analysis completion event
    let analysis_event =
        AnalysisEvent::analysis_completed(1, 2, 10, std::time::Duration::from_secs(5));

    println!("  Publishing Analysis event...");
    event_bus.publish(DomainEvent::Analysis(analysis_event))?;

    // Demonstrate event reception (in real implementation, components would handle these)
    println!("  Listening for events...");

    // Use a timeout to avoid blocking indefinitely
    tokio::select! {
        event_result = subscriber.recv() => {
            match event_result {
                Ok(event) => {
                    match event {
                        DomainEvent::Ast(_) => println!("    📝 Received AST event - components can react without direct coupling"),
                        DomainEvent::Analysis(_) => println!("    🔍 Received Analysis event - loose coupling achieved"),
                        _ => {}
                    }
                }
                Err(e) => println!("    Error receiving event: {}", e),
            }
        }
        _ = tokio::time::sleep(std::time::Duration::from_millis(100)) => {
            println!("    ⏰ Event processing complete");
        }
    }

    println!("  ✅ Event system working - components decoupled!");

    Ok(())
}

/// Example of how analysis components would use the new interfaces
#[allow(dead_code)]
struct ExampleAnalysisService {
    persistence: Arc<
        dyn PersistenceProvider<Error = uveddi::core::interfaces::persistence::PersistenceError>,
    >,
    event_bus: Arc<EventBus>,
}

#[allow(dead_code)]
impl ExampleAnalysisService {
    pub fn new(
        persistence: Arc<
            dyn PersistenceProvider<
                Error = uveddi::core::interfaces::persistence::PersistenceError,
            >,
        >,
        event_bus: Arc<EventBus>,
    ) -> Self {
        Self {
            persistence,
            event_bus,
        }
    }

    /// Example analysis method that uses dependency injection
    pub async fn analyze_file(&self, file_path: &str) -> Result<(), Box<dyn std::error::Error>> {
        // 1. Publish analysis started event (no direct AST dependency)
        let event = AnalysisEvent::analysis_started(1, file_path.to_string());
        self.event_bus.publish(DomainEvent::Analysis(event))?;

        // 2. Perform analysis (would be done by actual detectors)
        let issue = DomainIssue::new(
            "example_detector".to_string(),
            "architectural".to_string(),
            IssueSeverity::Medium,
            "Example issue".to_string(),
            "This is an example issue".to_string(),
            file_path.to_string(),
            1,
        );

        // 3. Save results via persistence interface (no direct database dependency)
        self.persistence.save_issues(vec![issue]).await?;

        // 4. Publish completion event
        let completion_event =
            AnalysisEvent::analysis_completed(1, 1, 1, std::time::Duration::from_millis(100));
        self.event_bus
            .publish(DomainEvent::Analysis(completion_event))?;

        Ok(())
    }
}
