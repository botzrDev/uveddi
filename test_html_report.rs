use std::collections::HashMap;
use std::path::Path;
use uveddi::database::crud::Database;
use uveddi::database::models::{AnalysisRun, AntiPatternType, ArchitecturalIssue};
use uveddi::report::ReportGenerator;
use chrono::Utc;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    env_logger::init();

    // Create mock analysis data
    println!("Creating mock analysis data for HTML report test...");

    let analysis_run = AnalysisRun {
        run_id: Some(1),
        project_id: 1,
        start_time: Utc::now(),
        end_time: Some(Utc::now()),
        status: "completed".to_string(),
        total_files_analyzed: Some(150),
        total_issues_found: Some(8),
        analysis_config: "{}".to_string(),
    };

    // Create anti-pattern types
    let mut anti_pattern_types = HashMap::new();
    anti_pattern_types.insert(
        1,
        AntiPatternType {
            anti_pattern_type_id: Some(1),
            name: "God Object".to_string(),
            description: "A class that has too many responsibilities and is difficult to maintain".to_string(),
            category: "Design".to_string(),
        },
    );
    anti_pattern_types.insert(
        2,
        AntiPatternType {
            anti_pattern_type_id: Some(2),
            name: "Dead Code".to_string(),
            description: "Code that is never executed or used".to_string(),
            category: "Maintenance".to_string(),
        },
    );
    anti_pattern_types.insert(
        3,
        AntiPatternType {
            anti_pattern_type_id: Some(3),
            name: "Magic Values".to_string(),
            description: "Hard-coded values that should be constants".to_string(),
            category: "Readability".to_string(),
        },
    );

    // Create architectural issues
    let issues = vec![
        ArchitecturalIssue {
            issue_id: Some(1),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/services/user_service.rs".to_string(),
            start_line: Some(45),
            end_line: Some(150),
            severity: "high".to_string(),
            description: "God Object detected: 'UserService' has 25 methods and 15 fields, violating the Single Responsibility Principle".to_string(),
            code_snippet: Some(r#"pub struct UserService {
    database: Database,
    email_service: EmailService,
    auth_service: AuthService,
    logging_service: LoggingService,
    cache_service: CacheService,
    // ... 10 more fields
}

impl UserService {
    pub fn create_user(&self) { /* ... */ }
    pub fn update_user(&self) { /* ... */ }
    pub fn delete_user(&self) { /* ... */ }
    pub fn send_email(&self) { /* ... */ }
    pub fn authenticate(&self) { /* ... */ }
    // ... 20 more methods
}"#.to_string()),
            ai_explanation: Some("This class has grown too large and handles too many different responsibilities including user management, email sending, authentication, and caching. Consider breaking this into smaller, focused classes: UserRepository for data access, UserEmailService for email operations, and UserAuthService for authentication logic.".to_string()),
        },
        ArchitecturalIssue {
            issue_id: Some(2),
            analysis_run_id: 1,
            anti_pattern_type_id: 2,
            file_path: "src/utils/old_helpers.rs".to_string(),
            start_line: Some(12),
            end_line: Some(25),
            severity: "medium".to_string(),
            description: "Dead Code detected: Function 'legacy_format_date' is never called".to_string(),
            code_snippet: Some(r#"fn legacy_format_date(date: &DateTime<Utc>) -> String {
    // This function was replaced by new_format_date but never removed
    format!("{}", date.format("%Y-%m-%d %H:%M:%S"))
}"#.to_string()),
            ai_explanation: Some("This function appears to be legacy code that was replaced but never removed. Dead code increases maintenance burden and can confuse developers. Consider removing this function if it's truly unused, or documenting why it's kept if there's a specific reason.".to_string()),
        },
        ArchitecturalIssue {
            issue_id: Some(3),
            analysis_run_id: 1,
            anti_pattern_type_id: 3,
            file_path: "src/config/constants.rs".to_string(),
            start_line: Some(8),
            end_line: Some(8),
            severity: "low".to_string(),
            description: "Magic Value detected: Hard-coded value '86400' should be a named constant".to_string(),
            code_snippet: Some(r#"let cache_timeout = 86400; // seconds"#.to_string()),
            ai_explanation: Some("Magic numbers make code harder to understand and maintain. The value 86400 represents seconds in a day, but this isn't immediately obvious. Consider defining: const SECONDS_PER_DAY: u64 = 86400; This makes the code more readable and easier to modify.".to_string()),
        },
        ArchitecturalIssue {
            issue_id: Some(4),
            analysis_run_id: 1,
            anti_pattern_type_id: 1,
            file_path: "src/models/report_generator.rs".to_string(),
            start_line: Some(100),
            end_line: Some(300),
            severity: "critical".to_string(),
            description: "God Object detected: 'ReportGenerator' has 45 methods handling multiple concerns including data processing, formatting, file I/O, and validation".to_string(),
            code_snippet: Some(r#"pub struct ReportGenerator {
    data_processor: DataProcessor,
    file_writer: FileWriter,
    validator: DataValidator,
    formatter: HtmlFormatter,
    email_sender: EmailSender,
    // ... many more fields
}

impl ReportGenerator {
    // Data processing methods
    pub fn process_raw_data(&self) { /* ... */ }
    pub fn clean_data(&self) { /* ... */ }
    pub fn aggregate_data(&self) { /* ... */ }
    
    // Formatting methods  
    pub fn format_html(&self) { /* ... */ }
    pub fn format_pdf(&self) { /* ... */ }
    pub fn format_csv(&self) { /* ... */ }
    
    // File I/O methods
    pub fn save_to_file(&self) { /* ... */ }
    pub fn read_template(&self) { /* ... */ }
    
    // Email methods
    pub fn send_report_email(&self) { /* ... */ }
    
    // ... 30+ more methods
}"#.to_string()),
            ai_explanation: Some("This is a severe God Object anti-pattern. The ReportGenerator class is handling data processing, multiple output formats, file operations, and email sending. This violates the Single Responsibility Principle and makes the code difficult to test and maintain. Recommend splitting into: DataProcessor, ReportFormatter (with strategy pattern for different formats), FileManager, and EmailService.".to_string()),
        },
    ];

    // Create report generator
    let generator = ReportGenerator::new()
        .with_ai_explanations(true)
        .with_code_snippets(true)
        .with_diagrams(true);

    // Generate HTML report
    println!("Generating HTML report...");
    let html_output_path = Path::new("test_analysis_report.html");
    
    match generator.generate_html_report(&analysis_run, &issues, &anti_pattern_types, Some(html_output_path)) {
        Ok(_) => {
            println!("✅ HTML report successfully generated at: {}", html_output_path.display());
            println!("📁 File size: {} bytes", std::fs::metadata(html_output_path)?.len());
            println!("\n🌐 You can open the HTML file in your browser to view the interactive report!");
            
            // Also generate markdown for comparison
            println!("\nGenerating Markdown report for comparison...");
            let md_output_path = Path::new("test_analysis_report.md");
            match generator.generate_markdown_report(&analysis_run, &issues, &anti_pattern_types, Some(md_output_path)) {
                Ok(_) => println!("✅ Markdown report also generated at: {}", md_output_path.display()),
                Err(e) => println!("⚠️  Failed to generate markdown report: {}", e),
            }
        }
        Err(e) => {
            println!("❌ Failed to generate HTML report: {}", e);
            return Err(e.into());
        }
    }

    println!("\n🎉 HTML report generation test completed successfully!");
    Ok(())
}