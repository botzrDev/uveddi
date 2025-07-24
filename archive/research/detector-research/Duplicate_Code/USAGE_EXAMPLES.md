# Duplicate Code Detector - Usage Examples

## Table of Contents
1. [Basic Usage](#basic-usage)
2. [Configuration Examples](#configuration-examples)
3. [Multi-Language Detection](#multi-language-detection)
4. [Advanced Scenarios](#advanced-scenarios)
5. [Integration Examples](#integration-examples)
6. [Performance Tuning](#performance-tuning)
7. [Error Handling](#error-handling)
8. [Real-World Use Cases](#real-world-use-cases)

## Basic Usage

### Simple Detection

```rust
use uveddi::analysis::detectors::{CodeDuplicationDetector, CodeDuplicationConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create detector with default configuration
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Analyze files for duplicates
    let files = vec![
        "src/user_service.rs".to_string(),
        "src/admin_service.rs".to_string(),
        "src/guest_service.rs".to_string(),
    ];
    
    let clones = detector.detect_duplicates(&files)?;
    
    // Display results
    println!("Found {} clone pairs", clones.len());
    for (i, clone) in clones.iter().enumerate() {
        println!("Clone {} ({}% similar):", i + 1, (clone.similarity_score * 100.0) as u32);
        println!("  File 1: {} ({})", clone.block1.file_path, clone.block1.display_name());
        println!("  File 2: {} ({})", clone.block2.file_path, clone.block2.display_name());
        println!("  Type: {}", clone.clone_type.description());
        println!();
    }
    
    Ok(())
}
```

### Directory Analysis

```rust
use std::fs;
use std::path::Path;

fn analyze_directory(dir_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Recursively find source files
    let source_files = find_source_files(dir_path)?;
    
    println!("Analyzing {} files in {}", source_files.len(), dir_path);
    
    let clones = detector.detect_duplicates(&source_files)?;
    
    // Generate summary report
    generate_summary_report(&clones);
    
    Ok(())
}

fn find_source_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    let extensions = vec!["rs", "py", "js", "ts"];
    
    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        
        if path.is_file() {
            if let Some(ext) = path.extension() {
                if extensions.contains(&ext.to_str().unwrap_or("")) {
                    files.push(path.to_string_lossy().to_string());
                }
            }
        } else if path.is_dir() {
            files.extend(find_source_files(&path.to_string_lossy())?);
        }
    }
    
    Ok(files)
}

fn generate_summary_report(clones: &[ClonePair]) {
    use std::collections::HashMap;
    
    let mut file_counts = HashMap::new();
    let mut type_counts = HashMap::new();
    
    for clone in clones {
        *file_counts.entry(&clone.block1.file_path).or_insert(0) += 1;
        *file_counts.entry(&clone.block2.file_path).or_insert(0) += 1;
        *type_counts.entry(&clone.clone_type).or_insert(0) += 1;
    }
    
    println!("=== Clone Detection Summary ===");
    println!("Total clone pairs: {}", clones.len());
    println!();
    
    println!("By clone type:");
    for (clone_type, count) in type_counts {
        println!("  {}: {}", clone_type.description(), count);
    }
    println!();
    
    println!("Files with most clones:");
    let mut file_vec: Vec<_> = file_counts.into_iter().collect();
    file_vec.sort_by(|a, b| b.1.cmp(&a.1));
    for (file, count) in file_vec.iter().take(10) {
        println!("  {}: {} clones", file, count);
    }
}
```

## Configuration Examples

### High Precision Configuration

```rust
fn detect_with_high_precision() -> Result<(), Box<dyn std::error::Error>> {
    let config = CodeDuplicationConfig {
        min_tokens: 30,                  // Larger minimum size
        similarity_threshold: 0.9,       // Very high similarity required
        hash_window_size: 6,             // Larger hash windows
        min_shared_hashes: 5,            // More shared hashes required
        normalize_identifiers: true,     // Still normalize for Type-2 detection
        normalize_literals: true,
    };
    
    let detector = CodeDuplicationDetector::new(config);
    
    let files = vec!["src/critical_module.rs".to_string()];
    let clones = detector.detect_duplicates(&files)?;
    
    println!("High precision mode found {} high-confidence clones", clones.len());
    
    // All results should be very similar
    for clone in clones {
        assert!(clone.similarity_score >= 0.9);
        println!("High confidence clone: {:.1}% similar", clone.similarity_score * 100.0);
    }
    
    Ok(())
}
```

### High Recall Configuration

```rust
fn detect_with_high_recall() -> Result<(), Box<dyn std::error::Error>> {
    let config = CodeDuplicationConfig {
        min_tokens: 15,                  // Smaller minimum size
        similarity_threshold: 0.6,       // Lower similarity threshold
        hash_window_size: 4,             // Smaller hash windows
        min_shared_hashes: 2,            // Fewer shared hashes required
        normalize_identifiers: true,
        normalize_literals: true,
    };
    
    let detector = CodeDuplicationDetector::new(config);
    
    let files = vec![
        "src/module1.rs".to_string(),
        "src/module2.rs".to_string(),
    ];
    let clones = detector.detect_duplicates(&files)?;
    
    println!("High recall mode found {} potential clones", clones.len());
    
    // Filter and prioritize results
    let high_priority: Vec<_> = clones.iter()
        .filter(|c| c.similarity_score >= 0.8)
        .collect();
    
    let medium_priority: Vec<_> = clones.iter()
        .filter(|c| c.similarity_score >= 0.7 && c.similarity_score < 0.8)
        .collect();
    
    let low_priority: Vec<_> = clones.iter()
        .filter(|c| c.similarity_score < 0.7)
        .collect();
    
    println!("High priority: {}", high_priority.len());
    println!("Medium priority: {}", medium_priority.len());
    println!("Low priority: {}", low_priority.len());
    
    Ok(())
}
```

### Performance Optimized Configuration

```rust
fn detect_with_performance_focus() -> Result<(), Box<dyn std::error::Error>> {
    let config = CodeDuplicationConfig {
        min_tokens: 50,                  // Skip small functions
        similarity_threshold: 0.8,       // Reasonable threshold
        hash_window_size: 5,             // Standard window size
        min_shared_hashes: 4,            // Reasonable filtering
        normalize_identifiers: false,    // Skip normalization for speed
        normalize_literals: false,       // Only exact matches
    };
    
    let detector = CodeDuplicationDetector::new(config);
    
    // Process large codebase efficiently
    let all_files = find_all_rust_files("large_project/")?;
    
    println!("Processing {} files with performance optimization", all_files.len());
    
    let start = std::time::Instant::now();
    let clones = detector.detect_duplicates(&all_files)?;
    let duration = start.elapsed();
    
    println!("Completed in {:?}", duration);
    println!("Found {} exact clones", clones.len());
    println!("Rate: {:.2} files/second", all_files.len() as f64 / duration.as_secs_f64());
    
    Ok(())
}

fn find_all_rust_files(dir: &str) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut files = Vec::new();
    for entry in walkdir::WalkDir::new(dir) {
        let entry = entry?;
        if entry.path().extension() == Some(std::ffi::OsStr::new("rs")) {
            files.push(entry.path().to_string_lossy().to_string());
        }
    }
    Ok(files)
}
```

## Multi-Language Detection

### Mixed Language Codebase

```rust
fn analyze_mixed_language_project() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Analyze different language files
    let files = vec![
        // Rust files
        "src/auth.rs".to_string(),
        "src/validation.rs".to_string(),
        
        // Python files
        "scripts/auth.py".to_string(),
        "scripts/validation.py".to_string(),
        
        // JavaScript files
        "frontend/auth.js".to_string(),
        "frontend/validation.js".to_string(),
    ];
    
    let clones = detector.detect_duplicates(&files)?;
    
    // Analyze cross-language patterns
    let cross_language_clones: Vec<_> = clones.iter()
        .filter(|c| {
            let lang1 = get_language_from_file(&c.block1.file_path);
            let lang2 = get_language_from_file(&c.block2.file_path);
            lang1 != lang2
        })
        .collect();
    
    println!("Cross-language clones found: {}", cross_language_clones.len());
    
    for clone in cross_language_clones {
        println!("Cross-language clone detected:");
        println!("  {} ({})", clone.block1.file_path, clone.block1.language);
        println!("  {} ({})", clone.block2.file_path, clone.block2.language);
        println!("  Similarity: {:.1}%", clone.similarity_score * 100.0);
        println!();
    }
    
    Ok(())
}

fn get_language_from_file(file_path: &str) -> &str {
    if file_path.ends_with(".rs") { "rust" }
    else if file_path.ends_with(".py") { "python" }
    else if file_path.ends_with(".js") { "javascript" }
    else { "unknown" }
}
```

### Language-Specific Analysis

```rust
fn analyze_python_specific() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    let python_files = vec![
        "api/models.py".to_string(),
        "api/views.py".to_string(),
        "api/serializers.py".to_string(),
        "tests/test_models.py".to_string(),
    ];
    
    let clones = detector.detect_duplicates(&python_files)?;
    
    // Look for common Python patterns
    let decorator_clones: Vec<_> = clones.iter()
        .filter(|c| {
            c.block1.content.contains("@") || c.block2.content.contains("@")
        })
        .collect();
    
    let class_method_clones: Vec<_> = clones.iter()
        .filter(|c| {
            c.block1.content.contains("def ") && c.block2.content.contains("def ")
        })
        .collect();
    
    println!("Python-specific analysis:");
    println!("  Decorator-related clones: {}", decorator_clones.len());
    println!("  Class method clones: {}", class_method_clones.len());
    
    Ok(())
}
```

## Advanced Scenarios

### Refactoring Opportunity Detection

```rust
fn detect_refactoring_opportunities() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    let files = vec![
        "src/user_controller.rs".to_string(),
        "src/admin_controller.rs".to_string(),
        "src/guest_controller.rs".to_string(),
    ];
    
    let clones = detector.detect_duplicates(&files)?;
    
    // Find clones that suggest extract method refactoring
    let extract_method_candidates: Vec<_> = clones.iter()
        .filter(|c| {
            // Look for clones with high similarity but different contexts
            c.similarity_score >= 0.8 && 
            c.is_cross_file() &&
            c.min_lines() >= 10  // Substantial size
        })
        .collect();
    
    println!("Extract Method Opportunities:");
    for candidate in extract_method_candidates {
        println!("  Candidate for extraction:");
        println!("    Functions: {} <-> {}", 
                 candidate.block1.display_name(), 
                 candidate.block2.display_name());
        println!("    Similarity: {:.1}%", candidate.similarity_score * 100.0);
        println!("    Size: {} lines", candidate.min_lines());
        
        // Suggest refactoring
        suggest_refactoring(candidate);
        println!();
    }
    
    Ok(())
}

fn suggest_refactoring(clone: &ClonePair) {
    println!("    Suggested refactoring:");
    println!("      1. Extract common logic to shared function");
    println!("      2. Place in common module or trait");
    println!("      3. Update both locations to use shared implementation");
    
    // Could generate more specific suggestions based on code analysis
    if clone.block1.content.contains("validation") {
        println!("      4. Consider validation utility module");
    }
    if clone.block1.content.contains("error") {
        println!("      4. Consider error handling utility");
    }
}
```

### Technical Debt Analysis

```rust
fn analyze_technical_debt() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    let files = find_all_source_files("src/")?;
    let clones = detector.detect_duplicates(&files)?;
    
    // Calculate technical debt metrics
    let total_duplicated_lines: usize = clones.iter()
        .map(|c| c.min_lines())
        .sum();
    
    let high_impact_clones: Vec<_> = clones.iter()
        .filter(|c| c.similarity_score >= 0.9 && c.min_lines() >= 20)
        .collect();
    
    let maintenance_burden: f64 = clones.iter()
        .map(|c| c.similarity_score * c.min_lines() as f64)
        .sum();
    
    println!("Technical Debt Analysis:");
    println!("  Total clone pairs: {}", clones.len());
    println!("  Duplicated lines: {}", total_duplicated_lines);
    println!("  High impact clones: {}", high_impact_clones.len());
    println!("  Maintenance burden score: {:.1}", maintenance_burden);
    
    // Prioritize fixes
    let mut prioritized_clones = clones;
    prioritized_clones.sort_by(|a, b| {
        let score_a = a.similarity_score * a.min_lines() as f64;
        let score_b = b.similarity_score * b.min_lines() as f64;
        score_b.partial_cmp(&score_a).unwrap()
    });
    
    println!("\nTop 5 clones to fix:");
    for (i, clone) in prioritized_clones.iter().take(5).enumerate() {
        println!("  {}. {} <-> {} ({:.1}% similar, {} lines)",
                 i + 1,
                 clone.block1.display_name(),
                 clone.block2.display_name(),
                 clone.similarity_score * 100.0,
                 clone.min_lines());
    }
    
    Ok(())
}
```

## Integration Examples

### CI/CD Pipeline Integration

```rust
use std::process;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args: Vec<String> = std::env::args().collect();
    
    if args.len() < 2 {
        eprintln!("Usage: {} <source_directory> [--fail-on-clones]", args[0]);
        process::exit(1);
    }
    
    let source_dir = &args[1];
    let fail_on_clones = args.contains(&"--fail-on-clones".to_string());
    
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    let files = find_source_files(source_dir)?;
    
    println!("Analyzing {} files for code duplication...", files.len());
    
    let clones = detector.detect_duplicates(&files)?;
    
    if clones.is_empty() {
        println!("✅ No code duplication detected");
        return Ok(());
    }
    
    println!("⚠️  Found {} clone pairs", clones.len());
    
    // Generate report for CI
    generate_ci_report(&clones);
    
    if fail_on_clones {
        println!("❌ Failing build due to code duplication");
        process::exit(1);
    }
    
    Ok(())
}

fn generate_ci_report(clones: &[ClonePair]) {
    println!("\n=== Code Duplication Report ===");
    
    for (i, clone) in clones.iter().enumerate() {
        println!("Clone {}: {:.1}% similar", i + 1, clone.similarity_score * 100.0);
        println!("  File 1: {}:{}", clone.block1.file_path, clone.block1.start_line);
        println!("  File 2: {}:{}", clone.block2.file_path, clone.block2.start_line);
        
        // Generate GitHub Actions annotations
        println!("::warning file={},line={}::Duplicate code detected ({}% similar to {}:{})",
                 clone.block1.file_path,
                 clone.block1.start_line,
                 (clone.similarity_score * 100.0) as u32,
                 clone.block2.file_path,
                 clone.block2.start_line);
    }
}
```

### IDE Integration

```rust
// Mock IDE integration example
struct IDEIntegration {
    detector: CodeDuplicationDetector,
}

impl IDEIntegration {
    fn new() -> Self {
        Self {
            detector: CodeDuplicationDetector::new(CodeDuplicationConfig::default()),
        }
    }
    
    fn analyze_current_file(&self, file_path: &str) -> Result<Vec<ClonePair>, Box<dyn std::error::Error>> {
        // Analyze current file against project
        let project_files = self.get_project_files()?;
        let files = vec![file_path.to_string()];
        files.extend(project_files);
        
        let clones = self.detector.detect_duplicates(&files)?;
        
        // Filter to only clones involving the current file
        let relevant_clones: Vec<_> = clones.into_iter()
            .filter(|c| c.block1.file_path == file_path || c.block2.file_path == file_path)
            .collect();
        
        Ok(relevant_clones)
    }
    
    fn get_project_files(&self) -> Result<Vec<String>, Box<dyn std::error::Error>> {
        // Mock implementation - would integrate with IDE's project management
        Ok(vec![
            "src/main.rs".to_string(),
            "src/lib.rs".to_string(),
            "src/utils.rs".to_string(),
        ])
    }
    
    fn show_clone_highlights(&self, clones: &[ClonePair]) {
        for clone in clones {
            println!("IDE: Highlighting clone at {}:{}-{}", 
                     clone.block1.file_path, 
                     clone.block1.start_line, 
                     clone.block1.end_line);
            
            // Mock IDE highlighting
            self.add_highlight(
                &clone.block1.file_path,
                clone.block1.start_line,
                clone.block1.end_line,
                "Clone detected"
            );
        }
    }
    
    fn add_highlight(&self, file: &str, start: usize, end: usize, message: &str) {
        println!("IDE: Adding highlight to {}:{}-{} ({})", file, start, end, message);
    }
}

fn ide_integration_example() -> Result<(), Box<dyn std::error::Error>> {
    let ide = IDEIntegration::new();
    
    // Analyze current file
    let clones = ide.analyze_current_file("src/current_file.rs")?;
    
    if !clones.is_empty() {
        println!("Found {} clones in current file", clones.len());
        ide.show_clone_highlights(&clones);
    }
    
    Ok(())
}
```

## Performance Tuning

### Batch Processing for Large Codebases

```rust
fn process_large_codebase() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Process in batches to manage memory
    let all_files = find_all_source_files("large_project/")?;
    let batch_size = 100;
    
    println!("Processing {} files in batches of {}", all_files.len(), batch_size);
    
    let mut all_clones = Vec::new();
    let mut processed = 0;
    
    for batch in all_files.chunks(batch_size) {
        let batch_files: Vec<String> = batch.iter().map(|f| f.clone()).collect();
        
        println!("Processing batch {}/{}", processed / batch_size + 1, 
                 (all_files.len() + batch_size - 1) / batch_size);
        
        let batch_clones = detector.detect_duplicates(&batch_files)?;
        all_clones.extend(batch_clones);
        
        processed += batch.len();
        
        // Optional: Save intermediate results
        if processed % 500 == 0 {
            save_intermediate_results(&all_clones, processed)?;
        }
    }
    
    println!("Completed processing {} files", all_files.len());
    println!("Found {} total clones", all_clones.len());
    
    Ok(())
}

fn save_intermediate_results(clones: &[ClonePair], processed: usize) -> Result<(), Box<dyn std::error::Error>> {
    let filename = format!("clones_checkpoint_{}.json", processed);
    let json = serde_json::to_string_pretty(clones)?;
    std::fs::write(filename, json)?;
    Ok(())
}
```

### Memory-Efficient Processing

```rust
fn memory_efficient_analysis() -> Result<(), Box<dyn std::error::Error>> {
    // Configure for memory efficiency
    let config = CodeDuplicationConfig {
        min_tokens: 40,         // Skip small functions
        similarity_threshold: 0.8,
        hash_window_size: 5,
        min_shared_hashes: 4,
        normalize_identifiers: true,
        normalize_literals: false,  // Reduce memory usage
    };
    
    let detector = CodeDuplicationDetector::new(config);
    
    // Process files one by one to minimize memory usage
    let files = find_all_source_files("project/")?;
    let mut all_clones = Vec::new();
    
    for file in &files {
        // Process each file against all others
        let file_clones = detector.detect_duplicates(&files)?;
        
        // Filter to only clones involving this file to avoid duplicates
        let relevant_clones: Vec<_> = file_clones.into_iter()
            .filter(|c| c.block1.file_path == *file)
            .collect();
        
        all_clones.extend(relevant_clones);
        
        // Optional: Force garbage collection
        // This is a mock - Rust handles memory automatically
        println!("Processed {}, found {} clones so far", file, all_clones.len());
    }
    
    Ok(())
}
```

## Error Handling

### Robust Error Handling

```rust
use uveddi::analysis::detectors::DuplicationError;

fn robust_analysis() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    let files = vec![
        "src/valid_file.rs".to_string(),
        "src/invalid_syntax.rs".to_string(),  // File with syntax errors
        "src/nonexistent.rs".to_string(),     // File that doesn't exist
        "src/binary_file.exe".to_string(),    // Binary file
    ];
    
    match detector.detect_duplicates(&files) {
        Ok(clones) => {
            println!("Analysis completed successfully");
            println!("Found {} clones", clones.len());
        }
        Err(DuplicationError::ParseError(msg)) => {
            eprintln!("Parse error encountered: {}", msg);
            
            // Retry with only valid files
            let valid_files = filter_valid_files(&files)?;
            let clones = detector.detect_duplicates(&valid_files)?;
            println!("Retried with {} valid files, found {} clones", 
                     valid_files.len(), clones.len());
        }
        Err(DuplicationError::UnsupportedLanguage(lang)) => {
            eprintln!("Unsupported language: {}", lang);
            
            // Filter to supported languages
            let supported_files = filter_supported_files(&files, &detector)?;
            let clones = detector.detect_duplicates(&supported_files)?;
            println!("Analyzed {} supported files", supported_files.len());
        }
        Err(DuplicationError::IoError(io_err)) => {
            eprintln!("IO error: {}", io_err);
            
            // Handle file access issues
            let accessible_files = filter_accessible_files(&files)?;
            if !accessible_files.is_empty() {
                let clones = detector.detect_duplicates(&accessible_files)?;
                println!("Analyzed {} accessible files", accessible_files.len());
            }
        }
        Err(e) => {
            eprintln!("Unexpected error: {}", e);
            return Err(e.into());
        }
    }
    
    Ok(())
}

fn filter_valid_files(files: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut valid_files = Vec::new();
    
    for file in files {
        if is_valid_source_file(file)? {
            valid_files.push(file.clone());
        }
    }
    
    Ok(valid_files)
}

fn filter_supported_files(files: &[String], detector: &CodeDuplicationDetector) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut supported_files = Vec::new();
    
    for file in files {
        let lang = detect_language(file);
        if detector.supports_language(&lang) {
            supported_files.push(file.clone());
        }
    }
    
    Ok(supported_files)
}

fn filter_accessible_files(files: &[String]) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let mut accessible_files = Vec::new();
    
    for file in files {
        if std::path::Path::new(file).exists() {
            accessible_files.push(file.clone());
        }
    }
    
    Ok(accessible_files)
}

fn is_valid_source_file(file: &str) -> Result<bool, Box<dyn std::error::Error>> {
    // Mock validation - would check file syntax
    Ok(std::path::Path::new(file).exists() && 
       (file.ends_with(".rs") || file.ends_with(".py") || file.ends_with(".js")))
}

fn detect_language(file: &str) -> String {
    if file.ends_with(".rs") { "rust".to_string() }
    else if file.ends_with(".py") { "python".to_string() }
    else if file.ends_with(".js") { "javascript".to_string() }
    else { "unknown".to_string() }
}
```

## Real-World Use Cases

### Code Review Automation

```rust
fn automated_code_review() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::high_precision());
    
    // Get files from PR/MR
    let changed_files = get_changed_files_from_git()?;
    
    if changed_files.is_empty() {
        println!("No source files changed in this PR");
        return Ok(());
    }
    
    // Analyze changed files against entire codebase
    let all_files = find_all_source_files("src/")?;
    let mut files_to_analyze = changed_files.clone();
    files_to_analyze.extend(all_files);
    
    let clones = detector.detect_duplicates(&files_to_analyze)?;
    
    // Filter to clones involving changed files
    let relevant_clones: Vec<_> = clones.into_iter()
        .filter(|c| {
            changed_files.contains(&c.block1.file_path) || 
            changed_files.contains(&c.block2.file_path)
        })
        .collect();
    
    if relevant_clones.is_empty() {
        println!("✅ No code duplication introduced in this PR");
        return Ok(());
    }
    
    println!("⚠️  Code duplication detected in PR:");
    for clone in relevant_clones {
        println!("  - {:.1}% similar: {} <-> {}", 
                 clone.similarity_score * 100.0,
                 clone.block1.display_name(),
                 clone.block2.display_name());
        
        // Generate review comment
        if changed_files.contains(&clone.block1.file_path) {
            generate_review_comment(&clone.block1, &clone.block2, clone.similarity_score);
        }
    }
    
    Ok(())
}

fn get_changed_files_from_git() -> Result<Vec<String>, Box<dyn std::error::Error>> {
    // Mock implementation - would use git commands
    Ok(vec![
        "src/new_feature.rs".to_string(),
        "src/updated_module.rs".to_string(),
    ])
}

fn generate_review_comment(block1: &CodeBlock, block2: &CodeBlock, similarity: f64) {
    println!("Review comment for {}:{}", block1.file_path, block1.start_line);
    println!("  This code is {:.1}% similar to {} ({})", 
             similarity * 100.0, 
             block2.file_path, 
             block2.display_name());
    println!("  Consider refactoring to reduce duplication.");
}
```

### Maintenance Planning

```rust
fn maintenance_planning() -> Result<(), Box<dyn std::error::Error>> {
    let detector = CodeDuplicationDetector::new(CodeDuplicationConfig::default());
    
    // Analyze entire codebase
    let all_files = find_all_source_files("src/")?;
    let clones = detector.detect_duplicates(&all_files)?;
    
    // Generate maintenance plan
    let maintenance_plan = generate_maintenance_plan(&clones);
    
    println!("=== Maintenance Plan ===");
    println!("Sprint 1 - Critical Issues ({} items):", maintenance_plan.sprint1.len());
    for item in &maintenance_plan.sprint1 {
        println!("  - {}", item);
    }
    
    println!("\nSprint 2 - High Priority ({} items):", maintenance_plan.sprint2.len());
    for item in &maintenance_plan.sprint2 {
        println!("  - {}", item);
    }
    
    println!("\nSprint 3 - Medium Priority ({} items):", maintenance_plan.sprint3.len());
    for item in &maintenance_plan.sprint3 {
        println!("  - {}", item);
    }
    
    Ok(())
}

struct MaintenancePlan {
    sprint1: Vec<String>,  // Critical
    sprint2: Vec<String>,  // High
    sprint3: Vec<String>,  // Medium
}

fn generate_maintenance_plan(clones: &[ClonePair]) -> MaintenancePlan {
    let mut plan = MaintenancePlan {
        sprint1: Vec::new(),
        sprint2: Vec::new(),
        sprint3: Vec::new(),
    };
    
    for clone in clones {
        let priority_score = clone.similarity_score * clone.min_lines() as f64;
        
        let task = format!("Refactor {} and {} ({:.1}% similar, {} lines)",
                          clone.block1.display_name(),
                          clone.block2.display_name(),
                          clone.similarity_score * 100.0,
                          clone.min_lines());
        
        if priority_score >= 100.0 {
            plan.sprint1.push(task);
        } else if priority_score >= 50.0 {
            plan.sprint2.push(task);
        } else {
            plan.sprint3.push(task);
        }
    }
    
    plan
}
```

These examples demonstrate various practical applications of the Duplicate Code Detector, from basic usage to complex integration scenarios. Each example can be adapted to specific project needs and requirements.
