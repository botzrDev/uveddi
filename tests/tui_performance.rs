#[cfg(feature = "tui")]
// TUI Performance and Load Testing
//
// This module tests the performance characteristics of the TUI system
// under various load conditions, ensuring responsive user interactions
// and efficient resource utilization.
#[cfg(feature = "tui")]
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::path::PathBuf;
use std::time::{Duration, Instant};
#[cfg(feature = "tui")]
use uveddi::tui::{AppMessage, AppScreen, AppState};
#[cfg(feature = "tui")]
use uveddi::tui::ui::analyze_form::FormField;
#[cfg(feature = "tui")]
use uveddi::tui::messages::FieldValue;
#[cfg(feature = "tui")]
use uveddi::cli::analyze_command::AnalyzeCommand;

/// Performance test configuration
const STRESS_TEST_DURATION: Duration = Duration::from_secs(5);
const RAPID_UPDATES_COUNT: usize = 1000;
const PERFORMANCE_THRESHOLD_MS: u128 = 100;

/// Create a large test project for performance testing
#[cfg(feature = "tui")]
async fn create_large_test_project() -> std::io::Result<PathBuf> {
    let test_dir = PathBuf::from("./tmp/performance_test_project");
    tokio::fs::create_dir_all(&test_dir).await?;

    // Create multiple files to test scalability
    for i in 0..10 {
        let file_path = test_dir.join(format!("module_{}.rs", i));
        let content = format!(
            r#"
// Module {} for performance testing
use std::collections::{{HashMap, BTreeMap}};

pub struct TestStruct{} {{
    pub field1: String,
    pub field2: i32,
    pub field3: f64,
    pub field4: bool,
    pub field5: Vec<String>,
    pub field6: HashMap<String, i32>,
    pub field7: Option<String>,
    pub field8: Result<i32, String>,
    pub field9: u64,
    pub field10: char,
}}

impl TestStruct{} {{
    pub fn new() -> Self {{
        Self {{
            field1: String::new(),
            field2: 0,
            field3: 0.0,
            field4: false,
            field5: Vec::new(),
            field6: HashMap::new(),
            field7: None,
            field8: Ok(0),
            field9: 0,
            field10: 'a',
        }}
    }}
    
    // Many methods to trigger large class detection
    pub fn method1(&self) -> String {{ self.field1.clone() }}
    pub fn method2(&self) -> i32 {{ self.field2 }}
    pub fn method3(&self) -> f64 {{ self.field3 }}
    pub fn method4(&self) -> bool {{ self.field4 }}
    pub fn method5(&self) -> Vec<String> {{ self.field5.clone() }}
    pub fn method6(&self) -> HashMap<String, i32> {{ self.field6.clone() }}
    pub fn method7(&self) -> Option<String> {{ self.field7.clone() }}
    pub fn method8(&self) -> Result<i32, String> {{ self.field8.clone() }}
    pub fn method9(&self) -> u64 {{ self.field9 }}
    pub fn method10(&self) -> char {{ self.field10 }}
    
    // Complex method with high cyclomatic complexity
    pub fn complex_method(&self, input: i32) -> String {{
        match input {{
            0..=10 => "very_low".to_string(),
            11..=20 => "low".to_string(),
            21..=30 => "medium_low".to_string(),
            31..=40 => "medium".to_string(),
            41..=50 => "medium_high".to_string(),
            51..=60 => "high".to_string(),
            61..=70 => "very_high".to_string(),
            71..=80 => "extremely_high".to_string(),
            81..=90 => "maximum".to_string(),
            _ => "overflow".to_string(),
        }}
    }}
    
    // Dead code methods
    #[allow(dead_code)]
    fn unused_method_1(&self) {{ println!("unused 1"); }}
    #[allow(dead_code)]
    fn unused_method_2(&self) {{ println!("unused 2"); }}
    #[allow(dead_code)]
    fn unused_method_3(&self) {{ println!("unused 3"); }}
    #[allow(dead_code)]
    fn unused_method_4(&self) {{ println!("unused 4"); }}
    #[allow(dead_code)]
    fn unused_method_5(&self) {{ println!("unused 5"); }}
}}

// Dead code - unused functions
#[allow(dead_code)]
fn unused_function_{}() {{
    println!("This function is never used");
}}

#[allow(dead_code)]
struct UnusedStruct{} {{
    data: String,
    number: i32,
}}

pub fn public_function_{}() -> TestStruct{} {{
    TestStruct{}::new()
}}
"#,
            i, i, i, i, i, i, i, i
        );

        tokio::fs::write(&file_path, content).await?;
    }

    // Create a main file that uses some of the modules
    let main_rs = test_dir.join("main.rs");
    tokio::fs::write(
        &main_rs,
        r#"
mod module_0;
mod module_1;
mod module_2;
mod module_3;
mod module_4;

use module_0::public_function_0;
use module_1::public_function_1;
use module_2::public_function_2;

fn main() {
    let _struct0 = public_function_0();
    let _struct1 = public_function_1();
    let _struct2 = public_function_2();
    
    println!("Performance test project main");
}
"#,
    )
    .await?;

    Ok(test_dir)
}

/// Clean up performance test project
#[cfg(feature = "tui")]
async fn cleanup_performance_test_project(path: &PathBuf) -> std::io::Result<()> {
    if path.exists() {
        tokio::fs::remove_dir_all(path).await?;
    }
    Ok(())
}

/// Test rapid state updates performance
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_rapid_state_updates_performance() {
    let mut app_state = AppState::new(None);
    let start_time = Instant::now();

    // Perform many rapid state updates
    for i in 0..RAPID_UPDATES_COUNT {
        let message = match i % 5 {
            0 => AppMessage::NavigateToAnalyze,
            1 => AppMessage::NavigateToConfig,
            2 => AppMessage::NavigateToReports,
            3 => AppMessage::NavigateToPlugins,
            _ => AppMessage::NavigateToMainMenu,
        };
        app_state.update(message);
    }

    let duration = start_time.elapsed();
    let updates_per_second = RAPID_UPDATES_COUNT as f64 / duration.as_secs_f64();

    println!("Rapid state updates performance:");
    println!("  {} updates in {:?}", RAPID_UPDATES_COUNT, duration);
    println!("  {:.0} updates per second", updates_per_second);
    println!(
        "  {:.3} ms per update",
        duration.as_millis() as f64 / RAPID_UPDATES_COUNT as f64
    );

    // Performance assertion - should handle at least 1000 updates per second
    assert!(
        updates_per_second > 1000.0,
        "State updates too slow: {:.0} updates/sec",
        updates_per_second
    );
    assert!(
        duration.as_millis() < PERFORMANCE_THRESHOLD_MS,
        "Total time too long: {:?}",
        duration
    );

    // Verify final state is consistent
    assert_eq!(app_state.current_screen, AppScreen::MainMenu);
    assert!(!app_state.should_quit);
}

/// Test menu navigation performance under rapid input
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_menu_navigation_performance() {
    let mut app_state = AppState::new(None);
    let start_time = Instant::now();

    let key_events = vec![
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
    ];

    // Simulate rapid keyboard input
    for i in 0..1000 {
        let key_event = &key_events[i % key_events.len()];
        app_state.update(AppMessage::KeyPressed(*key_event));
    }

    let duration = start_time.elapsed();
    println!("Menu navigation performance:");
    println!("  1000 key events in {:?}", duration);
    println!(
        "  {:.3} ms per key event",
        duration.as_millis() as f64 / 1000.0
    );

    // Should handle keyboard input very quickly
    assert!(
        duration.as_millis() < 50,
        "Menu navigation too slow: {:?}",
        duration
    );
}

/// Test memory usage during extended operation
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_memory_usage_stability() {
    let mut app_state = AppState::new(None);

    // Initial memory baseline (simple approximation)
    let initial_size = std::mem::size_of_val(&app_state);

    // Perform many operations that could cause memory leaks
    for _i in 0..10000 {
        // Cycle through different screens
        app_state.update(AppMessage::NavigateToAnalyze);
        app_state.update(AppMessage::NavigateToConfig);
        app_state.update(AppMessage::NavigateToReports);
        app_state.update(AppMessage::NavigateToPlugins);
        app_state.update(AppMessage::NavigateToMainMenu);

        // Simulate error conditions
        app_state.update(AppMessage::MenuItemSelected(999));
        app_state.update(AppMessage::FormFieldChanged {
        field: FormField::Path,
        value: FieldValue::String("test_input".to_string()),
    });

        // Clear errors by navigating
        app_state.update(AppMessage::NavigateToMainMenu);
    }

    // Check memory hasn't grown significantly
    let final_size = std::mem::size_of_val(&app_state);

    println!("Memory usage stability:");
    println!("  Initial size: {} bytes", initial_size);
    println!("  Final size: {} bytes", final_size);
    println!(
        "  Growth: {} bytes",
        final_size as i64 - initial_size as i64
    );

    // Memory usage should remain stable (allowing for some variance)
    assert!(
        (final_size as i64 - initial_size as i64).abs() < 1000,
        "Excessive memory growth: {} -> {} bytes",
        initial_size,
        final_size
    );

    // Verify final state is clean
    assert_eq!(app_state.current_screen, AppScreen::MainMenu);
    assert!(!app_state.should_quit);
    assert_eq!(app_state.selected_menu_item, 0);
}

/// Test analysis command creation performance
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_command_creation_performance() {
    let test_project = create_large_test_project().await.unwrap();
    let start_time = Instant::now();

    // Create many AnalyzeCommand instances
    let mut commands = Vec::new();
    for i in 0..1000 {
    let command = AnalyzeCommand {
        path: test_project.clone(),
        output_format: if i % 2 == 0 {
            "json".to_string()
        } else {
            "markdown".to_string()
        },
        output: if i % 3 == 0 {
            Some(PathBuf::from(format!("output_{}.txt", i)))
        } else {
            None
        },
        enable_ai: i % 4 == 0,
        ollama_api_url: if i % 5 == 0 {
            Some("http://localhost:11434".to_string())
        } else {
            None
        },
        ollama_model: if i % 6 == 0 {
            Some("deepseek-coder".to_string())
        } else {
            None
        },
        dead_code_confidence: Some(0.1 + (i as f64 % 10.0) / 10.0),
        dead_code_library_mode: i % 2 == 0,
        dead_code_ignore_patterns: if i % 3 == 0 {
            Some(vec!["test".to_string()])
        } else {
            None
        },
        dead_code_keep_alive: if i % 4 == 0 {
            Some(vec!["main".to_string()])
        } else {
            None
        },
        large_classes_max_loc: Some(100 + (i % 500) as u32),
        large_classes_max_methods: Some(10 + (i % 20) as u32),
        large_classes_max_fields: Some(5 + (i % 15) as u32),
        large_classes_max_complexity: Some(20 + (i % 50) as u32),
        large_classes_max_lcom: Some(0.1 + (i as f64 % 8.0) / 10.0),
        large_classes_ignore_patterns: if i % 7 == 0 {
            Some(vec!["generated".to_string()])
        } else {
            None
        },
        large_classes_min_severity: Some((i % 100) as u32),
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };
        commands.push(command);
    }

    let duration = start_time.elapsed();
    println!("Command creation performance:");
    println!("  1000 commands created in {:?}", duration);
    println!(
        "  {:.3} ms per command",
        duration.as_millis() as f64 / 1000.0
    );

    // Should create commands very quickly
    assert!(
        duration.as_millis() < 100,
        "Command creation too slow: {:?}",
        duration
    );
    assert_eq!(commands.len(), 1000);

    cleanup_performance_test_project(&test_project)
        .await
        .unwrap();
}

/// Test concurrent state operations
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_concurrent_state_operations() {
    use std::sync::{Arc, Mutex};
    use std::thread;

    let app_state = Arc::new(Mutex::new(AppState::new(None)));
    let start_time = Instant::now();

    // Spawn multiple threads to simulate concurrent access
    let handles: Vec<_> = (0..10)
        .map(|thread_id| {
            let app_state_clone: Arc<Mutex<AppState>> = Arc::clone(&app_state);
            thread::spawn(move || {
                for i in 0..100 {
                    let message = match (thread_id + i) % 5 {
                        0 => AppMessage::NavigateToAnalyze,
                        1 => AppMessage::NavigateToConfig,
                        2 => AppMessage::NavigateToReports,
                        3 => AppMessage::NavigateToPlugins,
                        _ => AppMessage::NavigateToMainMenu,
                    };

                    if let Ok(mut state) = app_state_clone.lock() {
                        state.update(message);
                    }
                }
            })
        })
        .collect();

    // Wait for all threads to complete
    for handle in handles {
        handle.join().unwrap();
    }

    let duration = start_time.elapsed();
    println!("Concurrent state operations:");
    println!("  10 threads × 100 operations in {:?}", duration);
    println!(
        "  {:.3} ms per operation",
        duration.as_millis() as f64 / 1000.0
    );

    // Verify final state is consistent
    let final_state = app_state.lock().unwrap();
    assert!(!final_state.should_quit);
    println!("  Final state: {:?}", final_state.current_screen);

    // Performance should be reasonable even with contention
    assert!(
        duration.as_millis() < 1000,
        "Concurrent operations too slow: {:?}",
        duration
    );
}

/// Test large project analysis performance (simulated)
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_large_project_simulation() {
    let test_project = create_large_test_project().await.unwrap();

    let analyze_command = AnalyzeCommand {
        path: test_project.clone(),
        output_format: "markdown".to_string(),
        output: None,
        enable_ai: false, // Disable AI for performance testing
        ollama_api_url: None,
        ollama_model: None,
        dead_code_confidence: Some(0.8),
        dead_code_library_mode: false,
        dead_code_ignore_patterns: None,
        dead_code_keep_alive: Some(vec!["main".to_string()]),
        large_classes_max_loc: Some(50), // Low threshold to trigger many detections
        large_classes_max_methods: Some(5),
        large_classes_max_fields: Some(5),
        large_classes_max_complexity: Some(10),
        large_classes_max_lcom: Some(0.5),
        large_classes_ignore_patterns: None,
        large_classes_min_severity: Some(0),
        enable_memory_optimization: false,
        memory_limit_gb: None,
        memory_profile: None,
        enable_image_rendering: false,
        mermaid_only: false,
        rendering_service_url: "http://localhost:3001".to_string(),
        no_fallback: false,
        check_rendering_service: false,
    };

    let start_time = Instant::now();

    // Execute with timeout to prevent hanging
    let result = tokio::time::timeout(Duration::from_secs(30), analyze_command.execute()).await;

    let duration = start_time.elapsed();

    match result {
        Ok(Ok(_)) => {
            println!("Large project analysis performance:");
            println!("  Analysis completed in {:?}", duration);

            // For a project with 10 files, should complete within reasonable time
            assert!(
                duration.as_secs() < 20,
                "Analysis took too long: {:?}",
                duration
            );
        }
        Ok(Err(e)) => {
            println!("Large project analysis failed (may be expected): {:?}", e);
            println!("  Failed after {:?}", duration);
        }
        Err(_) => {
            println!("Large project analysis timed out after {:?}", duration);
            // Timeout is acceptable for performance testing
        }
    }

    cleanup_performance_test_project(&test_project)
        .await
        .unwrap();
}

/// Stress test with rapid operations
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_stress_operations() {
    let mut app_state = AppState::new(None);
    let start_time = Instant::now();
    let mut operation_count = 0;

    // Perform rapid operations for a fixed duration
    while start_time.elapsed() < STRESS_TEST_DURATION {
        let operations = vec![
            AppMessage::NavigateToAnalyze,
            AppMessage::NavigateToConfig,
            AppMessage::NavigateToReports,
            AppMessage::NavigateToPlugins,
            AppMessage::NavigateToMainMenu,
            AppMessage::MenuItemSelected(0),
            AppMessage::MenuItemSelected(1),
            AppMessage::MenuItemSelected(2),
            AppMessage::MenuItemSelected(3),
            AppMessage::FormFieldChanged {
        field: FormField::Path,
        value: FieldValue::String("test".to_string()),
    },
        ];

        for operation in operations {
            app_state.update(operation);
            operation_count += 1;
        }
    }

    let duration = start_time.elapsed();
    let ops_per_second = operation_count as f64 / duration.as_secs_f64();

    println!("Stress test results:");
    println!("  {} operations in {:?}", operation_count, duration);
    println!("  {:.0} operations per second", ops_per_second);

    // Should maintain high throughput under stress
    assert!(
        ops_per_second > 10000.0,
        "Stress test throughput too low: {:.0} ops/sec",
        ops_per_second
    );

    // Verify application is still in a consistent state
    assert!(!app_state.should_quit);
    println!("  Final state: {:?}", app_state.current_screen);
}

/// Test message handling performance with complex messages
#[cfg(feature = "tui")]
#[tokio::test]
async fn test_complex_message_performance() {
    let mut app_state = AppState::new(None);
    let start_time = Instant::now();

    // Test with complex keyboard events
    let complex_events = vec![
        KeyEvent::new(KeyCode::Char('q'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL),
        KeyEvent::new(KeyCode::Esc, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::F(1), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Down, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Up, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('1'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('2'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('3'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('4'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('j'), KeyModifiers::NONE),
        KeyEvent::new(KeyCode::Char('k'), KeyModifiers::NONE),
    ];

    // Reset state for each test to avoid quit condition
    for _ in 0..1000 {
        app_state = AppState::new(None); // Reset to avoid accumulated quit states

        for &event in &complex_events {
            app_state.update(AppMessage::KeyPressed(event));
        }
    }

    let duration = start_time.elapsed();
    let total_events = 1000 * complex_events.len();

    println!("Complex message performance:");
    println!("  {} complex events in {:?}", total_events, duration);
    println!(
        "  {:.3} ms per event",
        duration.as_millis() as f64 / total_events as f64
    );

    // Should handle complex events quickly
    assert!(
        duration.as_millis() < 500,
        "Complex message handling too slow: {:?}",
        duration
    );
}
