#!/usr/bin/env rust-script

//! Documentation Audit Analyzer
//! 
//! This script systematically analyzes Rust source files in the Uveddi codebase 
//! to identify missing module headers and public item documentation.

use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
struct PublicItem {
    item_type: String,
    name: String,
    line_number: usize,
    has_doc: bool,
    doc_content: Option<String>,
}

#[derive(Debug, Clone)]
struct FileAnalysis {
    path: PathBuf,
    has_module_header: bool,
    module_header_content: Option<String>,
    public_items: Vec<PublicItem>,
    error: Option<String>,
}

#[derive(Debug)]
struct DocumentationAudit {
    files_analyzed: usize,
    files_with_module_headers: usize,
    files_missing_module_headers: Vec<PathBuf>,
    total_public_items: usize,
    documented_public_items: usize,
    undocumented_items: Vec<(PathBuf, PublicItem)>,
    critical_modules: Vec<PathBuf>,
}

impl DocumentationAudit {
    fn new() -> Self {
        Self {
            files_analyzed: 0,
            files_with_module_headers: 0,
            files_missing_module_headers: Vec::new(),
            total_public_items: 0,
            documented_public_items: 0,
            undocumented_items: Vec::new(),
            critical_modules: Vec::new(),
        }
    }

    fn module_coverage_percentage(&self) -> f64 {
        if self.files_analyzed == 0 {
            return 0.0;
        }
        (self.files_with_module_headers as f64 / self.files_analyzed as f64) * 100.0
    }

    fn public_item_coverage_percentage(&self) -> f64 {
        if self.total_public_items == 0 {
            return 0.0;
        }
        (self.documented_public_items as f64 / self.total_public_items as f64) * 100.0
    }
}

fn analyze_rust_file(file_path: &Path) -> FileAnalysis {
    let content = match fs::read_to_string(file_path) {
        Ok(content) => content,
        Err(e) => {
            return FileAnalysis {
                path: file_path.to_path_buf(),
                has_module_header: false,
                module_header_content: None,
                public_items: Vec::new(),
                error: Some(format!("Failed to read file: {}", e)),
            };
        }
    };

    let lines: Vec<&str> = content.lines().collect();
    let mut analysis = FileAnalysis {
        path: file_path.to_path_buf(),
        has_module_header: false,
        module_header_content: None,
        public_items: Vec::new(),
        error: None,
    };

    // Check for module header in first 60 lines
    let mut module_header_lines = Vec::new();
    let mut in_module_doc = false;
    
    for (_line_num, line) in lines.iter().enumerate().take(60) {
        let trimmed = line.trim();
        if trimmed.starts_with("//!") {
            in_module_doc = true;
            module_header_lines.push(trimmed);
        } else if in_module_doc && (trimmed.is_empty() || trimmed.starts_with("//")) {
            // Continue collecting if we're in module docs
            if trimmed.starts_with("//!") {
                module_header_lines.push(trimmed);
            } else if !trimmed.is_empty() && !trimmed.starts_with("//!") {
                break; // End of module docs
            }
        } else if in_module_doc {
            break; // End of module docs
        }
    }

    if !module_header_lines.is_empty() {
        analysis.has_module_header = true;
        analysis.module_header_content = Some(module_header_lines.join("\n"));
    }

    // Find public items and check for documentation
    for (line_num, line) in lines.iter().enumerate() {
        let trimmed = line.trim();
        
        // Skip test modules and internal items
        if trimmed.contains("#[cfg(test)]") || trimmed.contains("#[test]") {
            continue;
        }

        // Check for public declarations
        if let Some(item) = extract_public_item(trimmed, line_num + 1) {
            // Check for documentation in previous lines
            let has_doc = check_for_documentation(&lines, line_num);
            let mut doc_item = item;
            doc_item.has_doc = has_doc;
            
            if has_doc {
                doc_item.doc_content = extract_doc_content(&lines, line_num);
            }
            
            analysis.public_items.push(doc_item);
        }
    }

    analysis
}

fn extract_public_item(line: &str, line_number: usize) -> Option<PublicItem> {
    let trimmed = line.trim();
    
    // Match various public item patterns
    if let Some(captures) = regex_match_pub_item(trimmed) {
        return Some(PublicItem {
            item_type: captures.0.to_string(),
            name: captures.1.to_string(),
            line_number,
            has_doc: false,
            doc_content: None,
        });
    }
    
    None
}

fn regex_match_pub_item(line: &str) -> Option<(String, String)> {
    // Simple pattern matching for public items
    if line.starts_with("pub struct ") {
        if let Some(name) = extract_name_after("pub struct ", line) {
            return Some(("struct".to_string(), name));
        }
    } else if line.starts_with("pub enum ") {
        if let Some(name) = extract_name_after("pub enum ", line) {
            return Some(("enum".to_string(), name));
        }
    } else if line.starts_with("pub trait ") {
        if let Some(name) = extract_name_after("pub trait ", line) {
            return Some(("trait".to_string(), name));
        }
    } else if line.starts_with("pub fn ") {
        if let Some(name) = extract_name_after("pub fn ", line) {
            return Some(("function".to_string(), name));
        }
    } else if line.starts_with("pub mod ") {
        if let Some(name) = extract_name_after("pub mod ", line) {
            return Some(("module".to_string(), name));
        }
    } else if line.starts_with("pub type ") {
        if let Some(name) = extract_name_after("pub type ", line) {
            return Some(("type".to_string(), name));
        }
    } else if line.starts_with("pub const ") {
        if let Some(name) = extract_name_after("pub const ", line) {
            return Some(("const".to_string(), name));
        }
    } else if line.starts_with("pub static ") {
        if let Some(name) = extract_name_after("pub static ", line) {
            return Some(("static".to_string(), name));
        }
    } else if line.starts_with("pub use ") {
        if let Some(name) = extract_name_after("pub use ", line) {
            return Some(("use".to_string(), name));
        }
    }
    
    None
}

fn extract_name_after(prefix: &str, line: &str) -> Option<String> {
    if let Some(after_prefix) = line.strip_prefix(prefix) {
        // Extract the identifier name
        let name_part = after_prefix
            .split_whitespace()
            .next()?
            .split('(')
            .next()?
            .split('<')
            .next()?
            .split(':')
            .next()?
            .trim();
        
        if !name_part.is_empty() {
            return Some(name_part.to_string());
        }
    }
    None
}

fn check_for_documentation(lines: &[&str], current_line: usize) -> bool {
    // Check 1-5 lines above for documentation comments
    let start = if current_line >= 5 { current_line - 5 } else { 0 };
    
    for i in start..current_line {
        let line = lines.get(i).unwrap_or(&"").trim();
        if line.starts_with("///") || line.starts_with("/**") {
            return true;
        }
    }
    false
}

fn extract_doc_content(lines: &[&str], current_line: usize) -> Option<String> {
    let start = if current_line >= 10 { current_line - 10 } else { 0 };
    let mut doc_lines = Vec::new();
    let mut collecting = false;
    
    for i in start..current_line {
        let line = lines.get(i).unwrap_or(&"").trim();
        if line.starts_with("///") || line.starts_with("/**") {
            collecting = true;
            doc_lines.push(line);
        } else if collecting && (line.is_empty() || line.starts_with("//")) {
            doc_lines.push(line);
        } else if collecting && !line.is_empty() {
            break;
        }
    }
    
    if !doc_lines.is_empty() {
        Some(doc_lines.join("\n"))
    } else {
        None
    }
}

fn is_critical_module(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    
    // Define critical modules that should have comprehensive documentation
    let critical_patterns = [
        "src/lib.rs",
        "src/main.rs",
        "src/analysis/mod.rs",
        "src/analysis/engine.rs",
        "src/api/mod.rs",
        "src/ai/mod.rs",
        "src/plugins/mod.rs",
        "src/security/mod.rs",
        "src/tui/mod.rs",
        "src/cli/mod.rs",
    ];
    
    for pattern in &critical_patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }
    false
}

fn audit_rust_documentation(src_dir: &Path) -> DocumentationAudit {
    let mut audit = DocumentationAudit::new();
    
    // Find all .rs files
    let rust_files = find_rust_files(src_dir);
    
    for file_path in rust_files {
        let analysis = analyze_rust_file(&file_path);
        
        if analysis.error.is_some() {
            continue;
        }
        
        audit.files_analyzed += 1;
        
        if analysis.has_module_header {
            audit.files_with_module_headers += 1;
        } else {
            audit.files_missing_module_headers.push(file_path.clone());
        }
        
        if is_critical_module(&file_path) {
            audit.critical_modules.push(file_path.clone());
        }
        
        // Process public items
        for item in analysis.public_items {
            audit.total_public_items += 1;
            
            if item.has_doc {
                audit.documented_public_items += 1;
            } else {
                audit.undocumented_items.push((file_path.clone(), item));
            }
        }
    }
    
    audit
}

fn find_rust_files(dir: &Path) -> Vec<PathBuf> {
    let mut rust_files = Vec::new();
    
    if dir.is_file() && dir.extension().map_or(false, |ext| ext == "rs") {
        rust_files.push(dir.to_path_buf());
        return rust_files;
    }
    
    if let Ok(entries) = fs::read_dir(dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && !should_skip_directory(&path) {
                rust_files.extend(find_rust_files(&path));
            } else if path.extension().map_or(false, |ext| ext == "rs") {
                rust_files.push(path);
            }
        }
    }
    
    rust_files
}

fn should_skip_directory(path: &Path) -> bool {
    let dir_name = path.file_name().unwrap_or_default().to_string_lossy();
    
    // Skip test directories, build artifacts, and other non-source directories
    matches!(dir_name.as_ref(), "tests" | "target" | "true" | "bin")
}

fn main() {
    let src_dir = Path::new("src");
    
    println!("🔍 Starting Rust Documentation Audit for Uveddi...");
    println!("📁 Analyzing directory: {}", src_dir.display());
    println!();
    
    let audit = audit_rust_documentation(src_dir);
    
    // Print summary
    println!("📊 RUST DOCUMENTATION AUDIT RESULTS");
    println!("=====================================");
    println!();
    
    println!("📈 COVERAGE SUMMARY:");
    println!("  • Files analyzed: {}", audit.files_analyzed);
    println!("  • Module header coverage: {:.1}% ({}/{})", 
             audit.module_coverage_percentage(), 
             audit.files_with_module_headers, 
             audit.files_analyzed);
    println!("  • Public item documentation coverage: {:.1}% ({}/{})", 
             audit.public_item_coverage_percentage(), 
             audit.documented_public_items, 
             audit.total_public_items);
    println!();
    
    // Files missing module headers
    if !audit.files_missing_module_headers.is_empty() {
        println!("❌ FILES MISSING MODULE HEADERS ({}):", audit.files_missing_module_headers.len());
        for path in &audit.files_missing_module_headers {
            let priority = if is_critical_module(path) { "🔴 CRITICAL" } else { "🟡 NORMAL" };
            println!("  {} {}", priority, path.display());
        }
        println!();
    }
    
    // Undocumented public items (grouped by file)
    if !audit.undocumented_items.is_empty() {
        println!("❌ UNDOCUMENTED PUBLIC ITEMS ({}):", audit.undocumented_items.len());
        
        let mut grouped: HashMap<PathBuf, Vec<PublicItem>> = HashMap::new();
        for (path, item) in &audit.undocumented_items {
            grouped.entry(path.clone()).or_default().push(item.clone());
        }
        
        // Sort by priority (critical modules first)
        let mut sorted_files: Vec<_> = grouped.keys().collect();
        sorted_files.sort_by_key(|path| (!is_critical_module(path), *path));
        
        for path in sorted_files {
            let items = grouped.get(path).unwrap();
            let priority = if is_critical_module(path) { "🔴 CRITICAL" } else { "🟡 NORMAL" };
            println!("  {} {} ({} items):", priority, path.display(), items.len());
            
            for item in items {
                println!("    • Line {}: pub {} {}", item.line_number, item.item_type, item.name);
            }
        }
        println!();
    }
    
    // Recommendations
    println!("💡 RECOMMENDATIONS:");
    println!("  1. Focus on critical modules first (marked with 🔴)");
    println!("  2. Add module headers (//!) to files missing them");
    println!("  3. Document public APIs with /// comments");
    println!("  4. Consider using #[doc(hidden)] for internal public items");
    println!();
    
    if audit.module_coverage_percentage() < 80.0 {
        println!("⚠️  Module header coverage is below 80% - consider prioritizing this");
    }
    if audit.public_item_coverage_percentage() < 70.0 {
        println!("⚠️  Public API documentation coverage is below 70% - this impacts usability");
    }
}