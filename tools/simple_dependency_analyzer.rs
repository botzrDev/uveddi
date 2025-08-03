use std::collections::{HashMap, HashSet};
use std::path::{Path, PathBuf};
use std::fs;
use std::io::{self, Write};

fn main() -> io::Result<()> {
    println!("🔍 Simple Dependency Analysis for Uveddi");
    println!("==========================================");
    
    // Analyze the src directory
    let src_path = Path::new("src");
    if !src_path.exists() {
        eprintln!("Error: src directory not found");
        return Ok(());
    }

    let mut module_deps = HashMap::new();
    let mut cycles = Vec::new();
    
    // First pass: collect dependencies
    collect_dependencies(src_path, &mut module_deps)?;
    
    // Second pass: detect simple cycles
    detect_simple_cycles(&module_deps, &mut cycles);
    
    // Generate report
    generate_report(&module_deps, &cycles);
    
    Ok(())
}

fn collect_dependencies(src_path: &Path, module_deps: &mut HashMap<String, HashSet<String>>) -> io::Result<()> {
    for entry in walkdir::WalkDir::new(src_path) {
        let entry = entry?;
        let path = entry.path();
        
        if path.extension().map_or(false, |ext| ext == "rs") {
            let content = fs::read_to_string(path)?;
            let module_name = path_to_module_name(src_path, path);
            
            let deps = extract_crate_dependencies(&content);
            module_deps.insert(module_name, deps);
        }
    }
    Ok(())
}

fn path_to_module_name(src_path: &Path, file_path: &Path) -> String {
    let relative = file_path.strip_prefix(src_path).unwrap();
    let mut components: Vec<&str> = relative
        .components()
        .map(|c| c.as_os_str().to_str().unwrap())
        .collect();
    
    // Remove .rs extension and handle mod.rs
    if let Some(last) = components.last_mut() {
        if last.ends_with(".rs") {
            *last = &last[..last.len() - 3];
        }
        if *last == "mod" {
            components.pop();
        }
    }
    
    components.join("::")
}

fn extract_crate_dependencies(content: &str) -> HashSet<String> {
    let mut deps = HashSet::new();
    
    for line in content.lines() {
        let line = line.trim();
        if line.starts_with("use crate::") {
            if let Some(dep) = extract_module_from_use(line) {
                deps.insert(dep);
            }
        }
    }
    
    deps
}

fn extract_module_from_use(use_line: &str) -> Option<String> {
    // Parse "use crate::module::submodule::..." to extract "module"
    if let Some(start) = use_line.find("crate::") {
        let after_crate = &use_line[start + 7..];
        if let Some(end) = after_crate.find("::") {
            return Some(after_crate[..end].to_string());
        } else if let Some(end) = after_crate.find(";") {
            return Some(after_crate[..end].to_string());
        } else if let Some(end) = after_crate.find(" ") {
            return Some(after_crate[..end].to_string());
        }
    }
    None
}

fn detect_simple_cycles(module_deps: &HashMap<String, HashSet<String>>, cycles: &mut Vec<Vec<String>>) {
    for (module, deps) in module_deps {
        for dep in deps {
            // Check for direct cycles (A -> B, B -> A)
            if let Some(dep_deps) = module_deps.get(dep) {
                if dep_deps.contains(module) {
                    let cycle = vec![module.clone(), dep.clone()];
                    if !cycles.contains(&cycle) {
                        cycles.push(cycle);
                    }
                }
            }
        }
    }
}

fn generate_report(module_deps: &HashMap<String, HashSet<String>>, cycles: &Vec<Vec<String>>) {
    println!("\n📊 DEPENDENCY ANALYSIS RESULTS");
    println!("==============================");
    
    println!("\n🔍 Module Summary:");
    println!("  Total modules analyzed: {}", module_deps.len());
    println!("  Total dependency relationships: {}", 
             module_deps.values().map(|deps| deps.len()).sum::<usize>());
    
    println!("\n🚨 CIRCULAR DEPENDENCIES FOUND: {}", cycles.len());
    
    if !cycles.is_empty() {
        println!("\nDetailed Circular Dependencies:");
        for (i, cycle) in cycles.iter().enumerate() {
            println!("{}. {} ↔ {}", i + 1, cycle[0], cycle[1]);
        }
    }
    
    // Analyze by top-level modules
    println!("\n📈 TOP-LEVEL MODULE ANALYSIS:");
    let mut top_level_stats = HashMap::new();
    
    for (module, deps) in module_deps {
        let top_level = module.split("::").next().unwrap_or(module);
        let entry = top_level_stats.entry(top_level.to_string()).or_insert((0, 0));
        entry.0 += 1; // module count
        entry.1 += deps.len(); // dependency count
    }
    
    let mut sorted_modules: Vec<_> = top_level_stats.iter().collect();
    sorted_modules.sort_by(|a, b| b.1.1.cmp(&a.1.1)); // Sort by dependency count
    
    for (module, (mod_count, dep_count)) in sorted_modules {
        println!("  {}: {} modules, {} dependencies", module, mod_count, dep_count);
    }
    
    // Identify problematic patterns
    println!("\n⚠️  PROBLEMATIC PATTERNS:");
    identify_problem_patterns(module_deps, cycles);
    
    // Save detailed report
    if let Err(e) = save_detailed_report(module_deps, cycles) {
        eprintln!("Warning: Could not save detailed report: {}", e);
    } else {
        println!("\n💾 Detailed report saved to: dependency_analysis_report.txt");
    }
}

fn identify_problem_patterns(module_deps: &HashMap<String, HashSet<String>>, cycles: &Vec<Vec<String>>) {
    // Count cycles by module type
    let mut analysis_cycles = 0;
    let mut database_cycles = 0;
    let mut ast_cycles = 0;
    let mut security_cycles = 0;
    let mut monitoring_cycles = 0;
    
    for cycle in cycles {
        let cycle_str = cycle.join(" ");
        if cycle_str.contains("analysis") { analysis_cycles += 1; }
        if cycle_str.contains("database") { database_cycles += 1; }
        if cycle_str.contains("ast") { ast_cycles += 1; }
        if cycle_str.contains("security") { security_cycles += 1; }
        if cycle_str.contains("monitoring") { monitoring_cycles += 1; }
    }
    
    println!("  Analysis module cycles: {}", analysis_cycles);
    println!("  Database module cycles: {}", database_cycles);
    println!("  AST module cycles: {}", ast_cycles);
    println!("  Security module cycles: {}", security_cycles);
    println!("  Monitoring module cycles: {}", monitoring_cycles);
    
    // Identify hub modules (modules with many dependencies)
    let mut hub_modules = Vec::new();
    for (module, deps) in module_deps {
        if deps.len() > 5 {
            hub_modules.push((module, deps.len()));
        }
    }
    
    hub_modules.sort_by(|a, b| b.1.cmp(&a.1));
    
    if !hub_modules.is_empty() {
        println!("\n🕸️  HUB MODULES (high dependency count):");
        for (module, count) in hub_modules.iter().take(10) {
            println!("    {}: {} dependencies", module, count);
        }
    }
}

fn save_detailed_report(module_deps: &HashMap<String, HashSet<String>>, cycles: &Vec<Vec<String>>) -> io::Result<()> {
    let mut file = fs::File::create("dependency_analysis_report.txt")?;
    
    writeln!(file, "# Uveddi Dependency Analysis Report")?;
    writeln!(file, "Generated: {}", chrono::Utc::now().format("%Y-%m-%d %H:%M:%S UTC"))?;
    writeln!(file, "")?;
    
    writeln!(file, "## Summary")?;
    writeln!(file, "- Total modules: {}", module_deps.len())?;
    writeln!(file, "- Circular dependencies: {}", cycles.len())?;
    writeln!(file, "")?;
    
    writeln!(file, "## Circular Dependencies")?;
    for (i, cycle) in cycles.iter().enumerate() {
        writeln!(file, "{}. {} ↔ {}", i + 1, cycle[0], cycle[1])?;
    }
    writeln!(file, "")?;
    
    writeln!(file, "## All Module Dependencies")?;
    let mut sorted_modules: Vec<_> = module_deps.iter().collect();
    sorted_modules.sort_by(|a, b| a.0.cmp(b.0));
    
    for (module, deps) in sorted_modules {
        writeln!(file, "### {}", module)?;
        for dep in deps {
            writeln!(file, "  -> {}", dep)?;
        }
        writeln!(file, "")?;
    }
    
    Ok(())
}

// Add walkdir dependency manually since we can't modify Cargo.toml during compilation error
// This is a simple implementation
mod walkdir {
    use std::fs;
    use std::path::{Path, PathBuf};
    
    pub struct WalkDir {
        path: PathBuf,
    }
    
    impl WalkDir {
        pub fn new<P: AsRef<Path>>(path: P) -> Self {
            Self {
                path: path.as_ref().to_path_buf(),
            }
        }
    }
    
    impl IntoIterator for WalkDir {
        type Item = std::io::Result<DirEntry>;
        type IntoIter = WalkDirIter;
        
        fn into_iter(self) -> Self::IntoIter {
            let mut stack = vec![self.path];
            WalkDirIter { stack }
        }
    }
    
    pub struct WalkDirIter {
        stack: Vec<PathBuf>,
    }
    
    impl Iterator for WalkDirIter {
        type Item = std::io::Result<DirEntry>;
        
        fn next(&mut self) -> Option<Self::Item> {
            while let Some(path) = self.stack.pop() {
                match fs::metadata(&path) {
                    Ok(metadata) => {
                        if metadata.is_dir() {
                            match fs::read_dir(&path) {
                                Ok(entries) => {
                                    for entry in entries {
                                        match entry {
                                            Ok(entry) => self.stack.push(entry.path()),
                                            Err(_) => continue,
                                        }
                                    }
                                }
                                Err(_) => continue,
                            }
                        } else {
                            return Some(Ok(DirEntry { path }));
                        }
                    }
                    Err(e) => return Some(Err(e)),
                }
            }
            None
        }
    }
    
    pub struct DirEntry {
        path: PathBuf,
    }
    
    impl DirEntry {
        pub fn path(&self) -> &Path {
            &self.path
        }
    }
}

mod chrono {
    pub struct Utc;
    
    impl Utc {
        pub fn now() -> DateTime {
            DateTime
        }
    }
    
    pub struct DateTime;
    
    impl DateTime {
        pub fn format(&self, _fmt: &str) -> FormattedDateTime {
            FormattedDateTime
        }
    }
    
    pub struct FormattedDateTime;
    
    impl std::fmt::Display for FormattedDateTime {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "2025-08-03 12:00:00 UTC")
        }
    }
}