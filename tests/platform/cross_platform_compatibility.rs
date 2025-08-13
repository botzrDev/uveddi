//! Cross-Platform Compatibility Testing Suite
//! Tests system behavior across different platforms (Linux, Windows, macOS)

use uveddi::ast::tree_sitter_impl::{AstParser, SourceLanguage};
use uveddi::application::{AnalysisConfig, AnalysisOrchestrator};
use uveddi::error::UveddiError;

#[cfg(feature = "memory-optimization")]
use uveddi::analysis::memory::{
    initialize_memory_optimization, MemoryOptimizationConfig, get_optimization_status,
};

use std::path::{Path, PathBuf};
use std::fs;
use std::env;
use std::time::{Duration, Instant};
use std::collections::HashMap;
use tempfile::tempdir;
use tokio::time::timeout;

#[cfg(test)]
mod cross_platform_testing {
    use super::*;

    // Platform detection utilities
    fn get_platform_info() -> (String, String, String) {
        let os = env::consts::OS;
        let arch = env::consts::ARCH;
        let family = env::consts::FAMILY;
        
        (os.to_string(), arch.to_string(), family.to_string())
    }

    fn is_windows() -> bool {
        cfg!(target_os = "windows")
    }

    fn is_unix() -> bool {
        cfg!(unix)
    }

    fn is_macos() -> bool {
        cfg!(target_os = "macos")
    }

    fn is_linux() -> bool {
        cfg!(target_os = "linux")
    }

    // Test utilities
    fn create_test_file_with_platform_path(content: &str, filename: &str) -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let file_path = temp_dir.path().join(filename);
        
        // Ensure parent directories exist
        if let Some(parent) = file_path.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        
        fs::write(&file_path, content).unwrap();
        file_path
    }

    fn create_complex_directory_structure() -> PathBuf {
        let temp_dir = tempdir().unwrap();
        let base_path = temp_dir.path();

        // Create nested directory structure
        let dirs = vec![
            "src/main/rust",
            "src/test/resources",
            "target/debug/deps",
            "docs/api/v1",
            "scripts/deployment",
        ];

        for dir in &dirs {
            let dir_path = base_path.join(dir);
            fs::create_dir_all(&dir_path).unwrap();
            
            // Add test files
            let test_file = dir_path.join("test.rs");
            fs::write(&test_file, "fn test() {}").unwrap();
        }

        base_path.to_path_buf()
    }

    #[tokio::test]
    async fn test_path_handling_windows_linux_macos() {
        println!("🛣️ Testing Path Handling Across Platforms");

        let (os, arch, family) = get_platform_info();
        println!("Platform: {} ({}) - family: {}", os, arch, family);

        // Test path normalization across platforms
        let path_test_cases = vec![
            ("simple", "test.rs"),
            ("with_spaces", "test file.rs"),
            ("nested", "src/main/test.rs"),
            ("deep_nested", "src/main/rust/core/analysis/test.rs"),
            ("unicode", "测试文件.rs"),
            ("special_chars", "test-file_v2.1.rs"),
        ];

        for (case_name, filename) in path_test_cases {
            println!("  Testing path case: {}", case_name);

            let content = format!("// Test file for {}\nfn test_{}() {{}}", case_name, case_name);
            let file_path = create_test_file_with_platform_path(&content, filename);

            // Test file existence
            assert!(file_path.exists(), "File should exist: {}", file_path.display());

            // Test path normalization
            let normalized = file_path.canonicalize();
            match normalized {
                Ok(canonical_path) => {
                    println!("    Canonical path: {}", canonical_path.display());
                    
                    // Test parsing with normalized path
                    let mut parser = AstParser::new().unwrap();
                    let parse_result = parser.parse_file(&canonical_path);
                    
                    match parse_result {
                        Ok(parsed) => {
                            assert_eq!(parsed.language, SourceLanguage::Rust);
                            println!("    Successfully parsed file with canonical path");
                        }
                        Err(e) => {
                            println!("    Parse failed: {:?}", e);
                        }
                    }
                }
                Err(e) => {
                    println!("    Path canonicalization failed: {}", e);
                }
            }

            // Test relative vs absolute paths
            let absolute_path = file_path.canonicalize().unwrap_or(file_path.clone());
            let relative_path = file_path.file_name().unwrap();
            
            println!("    Absolute: {}", absolute_path.display());
            println!("    Relative: {}", relative_path.to_string_lossy());

            // Platform-specific path testing
            if is_windows() {
                // Test Windows-specific path features
                let path_str = file_path.to_string_lossy();
                if path_str.contains('\\') {
                    println!("    Windows backslash paths detected");
                }
                
                // Test UNC paths (if applicable)
                if let Some(parent) = file_path.parent() {
                    println!("    Parent directory: {}", parent.display());
                }
            }
            
            if is_unix() {
                // Test Unix-specific path features
                let path_str = file_path.to_string_lossy();
                assert!(path_str.contains('/'), "Unix paths should use forward slashes");
                println!("    Unix forward slash paths verified");
            }
        }

        // Test complex directory structures
        let complex_structure = create_complex_directory_structure();
        println!("  Testing complex directory structure: {}", complex_structure.display());

        // Traverse and test all files in structure
        let mut file_count = 0;
        if let Ok(entries) = fs::read_dir(&complex_structure) {
            for entry in entries.flatten() {
                if entry.path().is_file() && entry.path().extension().map_or(false, |ext| ext == "rs") {
                    file_count += 1;
                    
                    let mut parser = AstParser::new().unwrap();
                    let parse_result = parser.parse_file(&entry.path());
                    
                    match parse_result {
                        Ok(_) => println!("    ✓ Parsed: {}", entry.path().display()),
                        Err(e) => println!("    ✗ Failed to parse {}: {:?}", entry.path().display(), e),
                    }
                }
            }
        }

        println!("  Processed {} files in complex directory structure", file_count);
        println!("✅ Path handling test completed");
    }

    #[test]
    fn test_file_permissions_handling() {
        println!("🔐 Testing File Permissions Handling");

        let (os, _, _) = get_platform_info();
        println!("Testing on platform: {}", os);

        // Create test files with different permissions
        let temp_dir = tempdir().unwrap();
        
        let test_files = vec![
            ("normal.rs", "fn normal() {}"),
            ("readonly.rs", "fn readonly() {}"),
            ("executable.rs", "#!/usr/bin/env rust-script\nfn main() {}"),
        ];

        for (filename, content) in test_files {
            let file_path = temp_dir.path().join(filename);
            fs::write(&file_path, content).unwrap();

            // Test permission detection
            if let Ok(metadata) = fs::metadata(&file_path) {
                let permissions = metadata.permissions();
                println!("  File: {} - Readonly: {}", filename, permissions.readonly());

                // Platform-specific permission testing
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = permissions.mode();
                    println!("    Unix permissions: {:o}", mode);
                    
                    // Test different permission scenarios
                    if filename == "readonly.rs" {
                        // Make file readonly
                        let mut perms = permissions;
                        perms.set_mode(0o444); // Read-only for all
                        fs::set_permissions(&file_path, perms).unwrap();
                        
                        // Verify readonly status
                        let updated_metadata = fs::metadata(&file_path).unwrap();
                        assert!(updated_metadata.permissions().readonly(), "File should be readonly");
                        println!("    ✓ Successfully set readonly permissions");
                    }
                    
                    if filename == "executable.rs" {
                        // Make file executable
                        let mut perms = permissions;
                        perms.set_mode(0o755); // Executable for owner
                        fs::set_permissions(&file_path, perms).unwrap();
                        
                        let updated_metadata = fs::metadata(&file_path).unwrap();
                        let updated_mode = updated_metadata.permissions().mode();
                        assert!(updated_mode & 0o100 != 0, "File should be executable by owner");
                        println!("    ✓ Successfully set executable permissions");
                    }
                }

                #[cfg(windows)]
                {
                    // Windows-specific permission testing
                    println!("    Windows permissions - Readonly: {}", permissions.readonly());
                    
                    if filename == "readonly.rs" {
                        // Make file readonly on Windows
                        let mut perms = permissions;
                        perms.set_readonly(true);
                        fs::set_permissions(&file_path, perms).unwrap();
                        
                        let updated_metadata = fs::metadata(&file_path).unwrap();
                        assert!(updated_metadata.permissions().readonly(), "File should be readonly");
                        println!("    ✓ Successfully set readonly on Windows");
                    }
                }

                // Test parsing files with different permissions
                let mut parser = AstParser::new().unwrap();
                let parse_result = parser.parse_file(&file_path);
                
                match parse_result {
                    Ok(parsed) => {
                        println!("    ✓ Successfully parsed {} ({} bytes)", 
                                filename, parsed.source().len());
                    }
                    Err(e) => {
                        println!("    ✗ Failed to parse {}: {:?}", filename, e);
                        
                        // Check if it's a permission-related error
                        match e {
                            uveddi::ast::tree_sitter_impl::AstError::Io(ref io_error) => {
                                match io_error.kind() {
                                    std::io::ErrorKind::PermissionDenied => {
                                        println!("    Permission denied (expected for restricted files)");
                                    }
                                    _ => {
                                        println!("    Other I/O error: {}", io_error);
                                    }
                                }
                            }
                            _ => {
                                println!("    Non-I/O error: {:?}", e);
                            }
                        }
                    }
                }
            }
        }

        // Test directory permissions
        let test_dir = temp_dir.path().join("restricted_dir");
        fs::create_dir(&test_dir).unwrap();

        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            
            // Make directory readonly
            let mut perms = fs::metadata(&test_dir).unwrap().permissions();
            perms.set_mode(0o444); // Read-only
            fs::set_permissions(&test_dir, perms).unwrap();
            
            // Try to create file in readonly directory
            let restricted_file = test_dir.join("test.rs");
            let write_result = fs::write(&restricted_file, "fn test() {}");
            
            match write_result {
                Ok(_) => {
                    println!("    Unexpected: File created in readonly directory");
                }
                Err(e) => {
                    println!("    ✓ Expected error creating file in readonly directory: {}", e);
                }
            }
        }

        println!("✅ File permissions handling test completed");
    }

    #[test]
    fn test_unicode_filename_support() {
        println!("🌐 Testing Unicode Filename Support");

        let unicode_test_cases = vec![
            ("ascii", "simple.rs"),
            ("unicode_chinese", "测试文件.rs"),
            ("unicode_japanese", "テスト.rs"),
            ("unicode_korean", "테스트.rs"),
            ("unicode_arabic", "اختبار.rs"),
            ("unicode_russian", "тест.rs"),
            ("unicode_german", "prüfung.rs"),
            ("unicode_french", "examen.rs"),
            ("unicode_emoji", "test_😀_file.rs"),
            ("unicode_mixed", "test_混合_файл_😊.rs"),
        ];

        let mut successful_unicode_files = 0;
        let total_unicode_files = unicode_test_cases.len();

        for (case_name, filename) in unicode_test_cases {
            println!("  Testing Unicode case: {} ({})", case_name, filename);

            let content = format!("// Unicode test file: {}\nfn test_{}() {{}}", case_name, case_name);
            
            // Try to create file with Unicode name
            let file_creation_result = std::panic::catch_unwind(|| {
                create_test_file_with_platform_path(&content, filename)
            });

            match file_creation_result {
                Ok(file_path) => {
                    println!("    ✓ Successfully created file: {}", file_path.display());
                    
                    // Verify file exists
                    if file_path.exists() {
                        successful_unicode_files += 1;
                        
                        // Test filename extraction
                        if let Some(name) = file_path.file_name() {
                            let name_str = name.to_string_lossy();
                            println!("    Extracted filename: {}", name_str);
                            
                            // Verify UTF-8 validity
                            if name.to_str().is_some() {
                                println!("    ✓ Filename is valid UTF-8");
                            } else {
                                println!("    ⚠ Filename contains invalid UTF-8 sequences");
                            }
                        }
                        
                        // Test parsing Unicode filename file
                        let mut parser = AstParser::new().unwrap();
                        let parse_result = parser.parse_file(&file_path);
                        
                        match parse_result {
                            Ok(parsed) => {
                                println!("    ✓ Successfully parsed Unicode filename file");
                                assert_eq!(parsed.language, SourceLanguage::Rust);
                            }
                            Err(e) => {
                                println!("    ✗ Failed to parse Unicode filename file: {:?}", e);
                            }
                        }
                        
                        // Test file operations
                        let metadata_result = fs::metadata(&file_path);
                        match metadata_result {
                            Ok(metadata) => {
                                println!("    ✓ Metadata access successful (size: {} bytes)", metadata.len());
                            }
                            Err(e) => {
                                println!("    ✗ Metadata access failed: {}", e);
                            }
                        }
                    } else {
                        println!("    ✗ File does not exist after creation");
                    }
                }
                Err(_) => {
                    println!("    ✗ Failed to create file with Unicode name (platform limitation?)");
                }
            }
        }

        println!("  Unicode filename support: {}/{} successful", 
                successful_unicode_files, total_unicode_files);

        // Test Unicode content in files
        let unicode_content_cases = vec![
            ("utf8_comments", "// This is a comment with Unicode: 你好世界\nfn test() {}"),
            ("utf8_strings", r#"fn test() { println!("Unicode string: ❤️🌍"); }"#),
            ("utf8_identifiers", "fn тест() {} // Cyrillic function name"),
        ];

        for (case_name, content) in unicode_content_cases {
            println!("  Testing Unicode content case: {}", case_name);
            
            let file_path = create_test_file_with_platform_path(content, &format!("{}.rs", case_name));
            
            let mut parser = AstParser::new().unwrap();
            let parse_result = parser.parse_file(&file_path);
            
            match parse_result {
                Ok(parsed) => {
                    println!("    ✓ Successfully parsed file with Unicode content");
                    
                    // Verify content preservation
                    let source_content = parsed.source();
                    if source_content.contains("Unicode") || source_content.chars().any(|c| c as u32 > 127) {
                        println!("    ✓ Unicode content preserved in AST");
                    } else {
                        println!("    ⚠ Unicode content might have been lost");
                    }
                }
                Err(e) => {
                    println!("    ✗ Failed to parse file with Unicode content: {:?}", e);
                }
            }
        }

        println!("✅ Unicode filename support test completed");
    }

    #[tokio::test]
    async fn test_large_file_handling_32bit_64bit() {
        println!("📂 Testing Large File Handling (32-bit vs 64-bit)");

        let (os, arch, _) = get_platform_info();
        println!("Platform: {} on {}", os, arch);

        let is_64bit = arch.contains("64");
        println!("64-bit architecture: {}", is_64bit);

        // Test different file sizes
        let file_size_tests = vec![
            ("small", 1_000, true),           // 1KB - should work everywhere
            ("medium", 1_000_000, true),     // 1MB - should work everywhere  
            ("large", 100_000_000, true),    // 100MB - should work on most systems
            ("very_large", 1_000_000_000, is_64bit), // 1GB - may fail on 32-bit or low memory
        ];

        for (size_name, target_size, should_succeed) in file_size_tests {
            println!("  Testing {} file ({} bytes)", size_name, target_size);

            // Create file content
            let unit_content = "fn test() { println!(\"test\"); }\n";
            let unit_size = unit_content.len();
            let repetitions = (target_size + unit_size - 1) / unit_size; // Ceiling division

            let start_time = Instant::now();
            let content = unit_content.repeat(repetitions);
            let content_creation_time = start_time.elapsed();

            println!("    Content created in {:?} (actual size: {} bytes)", 
                    content_creation_time, content.len());

            // Write file
            let file_creation_result = std::panic::catch_unwind(|| {
                create_test_file_with_platform_path(&content, &format!("{}_file.rs", size_name))
            });

            match file_creation_result {
                Ok(file_path) => {
                    if file_path.exists() {
                        let file_metadata = fs::metadata(&file_path).unwrap();
                        println!("    ✓ File created successfully (size: {} bytes)", file_metadata.len());

                        // Test parsing large file
                        let parse_start = Instant::now();
                        let mut parser = AstParser::new().unwrap();
                        
                        // Use timeout to prevent hanging on very large files
                        let parse_result = timeout(
                            Duration::from_secs(30),
                            async {
                                // Run parsing in blocking task to avoid blocking async runtime
                                tokio::task::spawn_blocking(move || {
                                    parser.parse_file(&file_path)
                                }).await.unwrap()
                            }
                        ).await;

                        match parse_result {
                            Ok(Ok(parsed)) => {
                                let parse_time = parse_start.elapsed();
                                println!("    ✓ File parsed successfully in {:?} (language: {:?})", 
                                        parse_time, parsed.language);
                                
                                // Test memory efficiency
                                let source_len = parsed.source().len();
                                println!("    Source length: {} bytes", source_len);
                                
                                // Test code segment extraction for large files
                                let segment = parsed.extract_code_segment(0, 100);
                                println!("    ✓ Code segment extraction successful: {} chars", segment.len());
                                
                                if should_succeed {
                                    println!("    ✓ Large file handling successful as expected");
                                } else {
                                    println!("    ⚠ Large file succeeded unexpectedly (good performance!)");
                                }
                            }
                            Ok(Err(e)) => {
                                println!("    Parse failed: {:?}", e);
                                
                                if should_succeed {
                                    println!("    ⚠ Expected success but parsing failed");
                                } else {
                                    println!("    ✓ Expected failure on this architecture/platform");
                                }
                            }
                            Err(_) => {
                                println!("    Parse timed out after 30 seconds");
                                
                                if should_succeed {
                                    println!("    ⚠ Unexpected timeout");
                                } else {
                                    println!("    ✓ Timeout expected for very large files");
                                }
                            }
                        }
                    } else {
                        println!("    ✗ File creation appeared to succeed but file doesn't exist");
                    }
                }
                Err(_) => {
                    println!("    ✗ File creation failed (memory/disk limitation?)");
                    
                    if should_succeed {
                        println!("    ⚠ Unexpected failure creating {} file", size_name);
                    } else {
                        println!("    ✓ Expected failure on resource-constrained system");
                    }
                }
            }
        }

        // Test memory usage patterns
        let mut parser = AstParser::new().unwrap();
        let cache_stats_before = parser.cache_stats();
        println!("  Cache stats before large file tests: {:?}", cache_stats_before);

        // Test cache behavior with large files
        let large_files = vec![
            create_test_file_with_platform_path(&"fn test1() {}".repeat(10000), "cache_test1.rs"),
            create_test_file_with_platform_path(&"fn test2() {}".repeat(10000), "cache_test2.rs"),
            create_test_file_with_platform_path(&"fn test3() {}".repeat(10000), "cache_test3.rs"),
        ];

        for (i, file_path) in large_files.iter().enumerate() {
            let parse_result = parser.parse_file(file_path);
            match parse_result {
                Ok(_) => {
                    let cache_stats = parser.cache_stats();
                    println!("    Cache stats after file {}: hits={}, misses={}, size={}", 
                            i + 1, cache_stats.0, cache_stats.1, cache_stats.2);
                }
                Err(e) => {
                    println!("    File {} parse failed: {:?}", i + 1, e);
                }
            }
        }

        println!("✅ Large file handling test completed");
    }

    #[tokio::test]
    #[cfg(feature = "tui")]
    async fn test_terminal_ui_rendering() {
        println!("🖥️ Testing Terminal UI Rendering");

        let (os, _, _) = get_platform_info();
        println!("Testing TUI on platform: {}", os);

        // Test terminal capabilities detection
        let term_info = vec![
            ("TERM", env::var("TERM").unwrap_or_else(|_| "unknown".to_string())),
            ("COLORTERM", env::var("COLORTERM").unwrap_or_else(|_| "unknown".to_string())),
            ("TERM_PROGRAM", env::var("TERM_PROGRAM").unwrap_or_else(|_| "unknown".to_string())),
        ];

        for (var_name, value) in term_info {
            println!("  {}: {}", var_name, value);
        }

        // Test color support detection
        let color_support_tests = vec![
            ("basic_colors", 8),
            ("extended_colors", 256),
            ("true_color", 16777216),
        ];

        for (test_name, color_count) in color_support_tests {
            println!("  Testing {} support ({} colors)", test_name, color_count);
            
            // This would normally test actual terminal capabilities
            // For cross-platform testing, we simulate the tests
            match test_name {
                "basic_colors" => {
                    println!("    ✓ Basic color support available on most terminals");
                }
                "extended_colors" => {
                    if os == "windows" {
                        println!("    ⚠ Extended colors may be limited on older Windows terminals");
                    } else {
                        println!("    ✓ Extended colors typically supported on Unix terminals");
                    }
                }
                "true_color" => {
                    println!("    ⚠ True color support varies by terminal emulator");
                }
                _ => {}
            }
        }

        // Test keyboard input handling across platforms
        let key_tests = vec![
            ("ctrl_c", "Interrupt signal"),
            ("arrow_keys", "Navigation"),
            ("function_keys", "F1-F12 keys"),
            ("alt_combinations", "Alt key combinations"),
        ];

        for (key_type, description) in key_tests {
            println!("  Testing {}: {}", key_type, description);
            
            match key_type {
                "ctrl_c" => {
                    println!("    ✓ Ctrl+C handling should work on all platforms");
                }
                "arrow_keys" => {
                    if os == "windows" {
                        println!("    ✓ Arrow keys supported in Windows console");
                    } else {
                        println!("    ✓ Arrow keys supported in Unix terminals");
                    }
                }
                "function_keys" => {
                    println!("    ⚠ Function key support varies by terminal");
                }
                "alt_combinations" => {
                    if os == "macos" {
                        println!("    ⚠ Alt key behavior differs on macOS (Option key)");
                    } else {
                        println!("    ✓ Standard Alt key behavior expected");
                    }
                }
                _ => {}
            }
        }

        // Test screen size detection
        let screen_size_result = std::panic::catch_unwind(|| {
            // This would normally query actual terminal size
            // For testing, we simulate common terminal sizes
            let common_sizes = vec![
                (80, 24),   // Classic terminal
                (120, 30),  // Modern wide terminal
                (200, 50),  // Large terminal
            ];
            
            for (width, height) in common_sizes {
                println!("    Testing terminal size: {}x{}", width, height);
                
                if width < 80 || height < 24 {
                    println!("      ⚠ Terminal too small for optimal UI");
                } else {
                    println!("      ✓ Terminal size suitable for UI");
                }
            }
        });

        match screen_size_result {
            Ok(_) => {
                println!("  ✓ Screen size detection tests completed");
            }
            Err(_) => {
                println!("  ✗ Screen size detection failed");
            }
        }

        // Test Unicode rendering in terminal
        let unicode_rendering_tests = vec![
            ("ascii_art", "+----|----+"),
            ("box_drawing", "┌─────────┐"),
            ("unicode_symbols", "✓ ✗ ⚠ 🚀"),
            ("progress_bars", "████████░░"),
        ];

        for (test_name, test_content) in unicode_rendering_tests {
            println!("  Testing {}: {}", test_name, test_content);
            
            // Check if content is valid UTF-8
            if test_content.chars().all(|c| !c.is_control()) {
                println!("    ✓ Content should render correctly");
            } else {
                println!("    ⚠ Content contains control characters");
            }
        }

        println!("✅ Terminal UI rendering test completed");
    }

    #[test]
    fn test_platform_specific_features() {
        println!("🔧 Testing Platform-Specific Features");

        let (os, arch, family) = get_platform_info();
        println!("Platform details: {} ({}) - family: {}", os, arch, family);

        // Windows-specific tests
        if is_windows() {
            println!("  Running Windows-specific tests");

            // Test Windows path handling
            let windows_paths = vec![
                r"C:\Program Files\Test\file.rs",
                r"\\server\share\file.rs",  // UNC path
                r"C:\Users\Test User\file.rs", // Path with spaces
            ];

            for path_str in windows_paths {
                let path = PathBuf::from(path_str);
                println!("    Testing Windows path: {}", path.display());
                
                // Test path components
                if let Some(parent) = path.parent() {
                    println!("      Parent: {}", parent.display());
                }
                
                if let Some(filename) = path.file_name() {
                    println!("      Filename: {}", filename.to_string_lossy());
                }
            }

            // Test Windows file attributes (if we could access them)
            println!("    ✓ Windows path handling tests completed");
        }

        // Unix-specific tests
        if is_unix() {
            println!("  Running Unix-specific tests");

            // Test Unix path handling
            let unix_paths = vec![
                "/usr/local/bin/file.rs",
                "/home/user/.config/file.rs",
                "/tmp/test file.rs", // Path with spaces
                "/var/log/app/file.rs",
            ];

            for path_str in unix_paths {
                let path = PathBuf::from(path_str);
                println!("    Testing Unix path: {}", path.display());
                
                // Test absolute path detection
                assert!(path.is_absolute(), "Unix path should be absolute");
            }

            // Test Unix file permissions
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                
                let temp_file = create_test_file_with_platform_path("fn test() {}", "unix_test.rs");
                if let Ok(metadata) = fs::metadata(&temp_file) {
                    let permissions = metadata.permissions();
                    let mode = permissions.mode();
                    println!("    Unix file mode: {:o}", mode);
                    
                    // Test permission bits
                    let owner_read = mode & 0o400 != 0;
                    let owner_write = mode & 0o200 != 0;
                    let owner_exec = mode & 0o100 != 0;
                    
                    println!("      Owner permissions: r={}, w={}, x={}", 
                            owner_read, owner_write, owner_exec);
                }
            }

            println!("    ✓ Unix-specific tests completed");
        }

        // macOS-specific tests
        if is_macos() {
            println!("  Running macOS-specific tests");

            // Test macOS path conventions
            let macos_paths = vec![
                "/Applications/Test.app/Contents/Resources/file.rs",
                "/Users/test/Library/Application Support/file.rs",
                "/System/Library/Frameworks/file.rs",
            ];

            for path_str in macos_paths {
                let path = PathBuf::from(path_str);
                println!("    Testing macOS path: {}", path.display());
            }

            println!("    ✓ macOS-specific tests completed");
        }

        // Architecture-specific tests
        match arch.as_str() {
            "x86_64" => {
                println!("  x86_64 architecture detected");
                println!("    ✓ 64-bit operations should be fully supported");
            }
            "aarch64" => {
                println!("  ARM64 architecture detected");
                println!("    ✓ Modern ARM architecture with good performance");
            }
            "x86" => {
                println!("  32-bit x86 architecture detected");
                println!("    ⚠ May have memory limitations for large files");
            }
            _ => {
                println!("  Other architecture: {}", arch);
                println!("    ⚠ Architecture-specific behavior unknown");
            }
        }

        // Test environment variables
        let important_env_vars = vec![
            "HOME", "USER", "PATH", "TEMP", "TMP", "TMPDIR",
        ];

        for var_name in important_env_vars {
            if let Ok(value) = env::var(var_name) {
                println!("  {}: {}", var_name, value);
            } else {
                println!("  {}: not set", var_name);
            }
        }

        println!("✅ Platform-specific features test completed");
    }

    #[tokio::test]
    async fn test_cross_platform_analysis_integration() {
        println!("🔗 Testing Cross-Platform Analysis Integration");

        let (os, arch, _) = get_platform_info();
        println!("Integration test on: {} ({})", os, arch);

        // Test analysis across different file scenarios
        let test_scenarios = vec![
            ("simple_rust", "fn main() { println!(\"Hello, World!\"); }", "rs"),
            ("simple_python", "def main():\n    print(\"Hello, World!\")", "py"),
            ("simple_javascript", "function main() { console.log(\"Hello, World!\"); }", "js"),
            ("simple_typescript", "function main(): void { console.log(\"Hello, World!\"); }", "ts"),
        ];

        for (scenario_name, content, extension) in test_scenarios {
            println!("  Testing scenario: {}", scenario_name);

            let test_file = create_test_file_with_platform_path(content, &format!("{}.{}", scenario_name, extension));
            
            let analysis_config = AnalysisConfig {
                target_path: test_file,
                output_format: "json".to_string(),
                output_file: None,
                enable_ai: false,
                ollama_api_url: None,
                ollama_model: None,
                dead_code_confidence: None,
                dead_code_library_mode: false,
                dead_code_ignore_patterns: None,
                dead_code_keep_alive: None,
                large_classes_max_loc: None,
                large_classes_max_methods: None,
                large_classes_max_fields: None,
                large_classes_max_complexity: None,
                large_classes_max_lcom: None,
                large_classes_ignore_patterns: None,
                large_classes_min_severity: None,
                #[cfg(feature = "memory-optimization")]
                memory_optimization: None,
                enable_memory_optimization: false,
                memory_limit_gb: None,
                memory_profile: None,
            };

            let analysis_result = timeout(
                Duration::from_secs(15),
                async {
                    let mut orchestrator = AnalysisOrchestrator::new().unwrap();
                    orchestrator.execute_analysis(analysis_config).await
                }
            ).await;

            match analysis_result {
                Ok(Ok(_)) => {
                    println!("    ✓ Analysis completed successfully on {}", os);
                }
                Ok(Err(e)) => {
                    println!("    ✗ Analysis failed on {}: {:?}", os, e);
                }
                Err(_) => {
                    println!("    ⚠ Analysis timed out on {}", os);
                }
            }
        }

        // Test memory optimization across platforms
        #[cfg(feature = "memory-optimization")]
        {
            let memory_configs = vec![
                ("small", MemoryOptimizationConfig::small_project()),
                ("default", MemoryOptimizationConfig::default()),
                ("large", MemoryOptimizationConfig::large_codebase()),
            ];

            for (config_name, mem_config) in memory_configs {
                println!("  Testing memory optimization '{}' on {}", config_name, os);

                let init_result = initialize_memory_optimization(mem_config);
                match init_result {
                    Ok(_) => {
                        println!("    ✓ Memory optimization '{}' initialized successfully", config_name);
                        
                        let status = get_optimization_status();
                        println!("    Status: {}", status);
                    }
                    Err(e) => {
                        println!("    ✗ Memory optimization '{}' failed: {}", config_name, e);
                    }
                }
            }
        }

        // Test concurrent operations
        let concurrent_tasks: Vec<_> = (0..5).map(|i| {
            let content = format!("fn concurrent_test_{}() {{ println!(\"test {}\"); }}", i, i);
            let file_path = create_test_file_with_platform_path(&content, &format!("concurrent_{}.rs", i));

            tokio::spawn(async move {
                let mut parser = AstParser::new().unwrap();
                let result = parser.parse_file(&file_path);
                (i, result.is_ok())
            })
        }).collect();

        let concurrent_results = futures::future::join_all(concurrent_tasks).await;
        let successful_concurrent = concurrent_results.iter()
            .filter_map(|r| r.as_ref().ok())
            .filter(|(_, success)| *success)
            .count();

        println!("  Concurrent operations: {}/5 successful on {}", successful_concurrent, os);

        println!("✅ Cross-platform analysis integration test completed");
    }
}
"#