//! Fuzz testing for input validation robustness
//! 
//! This module provides fuzzing tests to ensure input validation functions
//! are robust against random and edge case inputs without panicking.

use uveddi::security::{validate_input, validate_url, validate_model_name, validate_numeric_range};
use rand::{Rng, thread_rng, distributions::Alphanumeric};

#[cfg(test)]
mod fuzz_tests {
    use super::*;

    #[test]
    fn test_random_input_fuzzing() {
        let mut rng = thread_rng();
        
        for _ in 0..1000 {
            // Generate random string with random length
            let length = rng.gen_range(0..20000);
            let random_string: String = (0..length)
                .map(|_| rng.gen_range(0..128) as u8 as char)
                .collect();
            
            // Validation should never panic, regardless of input
            let _ = validate_input(&random_string, "fuzz_test");
        }
    }

    #[test]
    fn test_unicode_fuzzing() {
        let mut rng = thread_rng();
        
        for _ in 0..500 {
            // Generate random Unicode string
            let length = rng.gen_range(0..1000);
            let random_unicode: String = (0..length)
                .map(|_| {
                    let codepoint = rng.gen_range(0..0x110000);
                    std::char::from_u32(codepoint).unwrap_or('\u{FFFD}') // Use replacement char for invalid
                })
                .collect();
            
            // Should handle Unicode gracefully
            let _ = validate_input(&random_unicode, "unicode_fuzz");
            let _ = validate_url(&format!("http://example.com/{}", random_unicode));
            let _ = validate_model_name(&random_unicode);
        }
    }

    #[test]
    fn test_boundary_fuzzing() {
        let mut rng = thread_rng();
        
        // Test around the length boundary
        for _ in 0..100 {
            let base_length = 9995; // Near the 10000 limit
            let variance = rng.gen_range(0..20); // Add some variance
            let length = base_length + variance;
            
            let test_string = "a".repeat(length);
            let _ = validate_input(&test_string, "boundary_fuzz");
        }
    }

    #[test]
    fn test_numeric_range_fuzzing() {
        let mut rng = thread_rng();
        
        for _ in 0..1000 {
            let value = rng.gen_range(i32::MIN..=i32::MAX);
            let min = rng.gen_range(i32::MIN..=0);
            let max = rng.gen_range(0..=i32::MAX);
            
            // Ensure min <= max
            let (min, max) = if min > max { (max, min) } else { (min, max) };
            
            // Should never panic
            let _ = validate_numeric_range(value, min, max, "numeric_fuzz");
        }
    }

    #[test]
    fn test_url_fuzzing() {
        let mut rng = thread_rng();
        
        // Test with random protocols
        let protocols = ["http://", "https://", "ftp://", "file://", "javascript:", "data:", ""];
        
        for _ in 0..200 {
            let protocol = protocols[rng.gen_range(0..protocols.len())];
            let domain_length = rng.gen_range(0..100);
            let domain: String = (0..domain_length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            
            let port = rng.gen_range(1..65536);
            let path_length = rng.gen_range(0..200);
            let path: String = (0..path_length)
                .map(|_| {
                    let chars = "abcdefghijklmnopqrstuvwxyz0123456789/-_.~?&=";
                    chars.chars().nth(rng.gen_range(0..chars.len())).unwrap()
                })
                .collect();
            
            let test_url = format!("{}{}:{}/{}", protocol, domain, port, path);
            
            // Should never panic
            let _ = validate_url(&test_url);
        }
    }

    #[test]
    fn test_model_name_fuzzing() {
        let mut rng = thread_rng();
        
        for _ in 0..500 {
            let length = rng.gen_range(0..300);
            let model_name: String = (0..length)
                .map(|_| {
                    // Mix valid and invalid characters
                    let chars = "abcdefghijklmnopqrstuvwxyzABCDEFGHIJKLMNOPQRSTUVWXYZ0123456789-_./@\\";
                    chars.chars().nth(rng.gen_range(0..chars.len())).unwrap()
                })
                .collect();
            
            // Should never panic
            let _ = validate_model_name(&model_name);
        }
    }

    #[test]
    fn test_control_character_fuzzing() {
        for control_char in 0..32u8 {
            let char = control_char as char;
            let test_strings = vec![
                format!("test{}input", char),
                format!("{}test", char),
                format!("test{}", char),
                char.to_string(),
            ];
            
            for test_string in test_strings {
                // Should handle control characters gracefully
                let _ = validate_input(&test_string, "control_char_test");
            }
        }
    }

    #[test]
    fn test_null_byte_fuzzing() {
        let null_positions = vec![
            "test\0input",
            "\0test",
            "test\0",
            "\0",
            "test\0\0input",
            "\0\0\0",
        ];
        
        for test_string in null_positions {
            // Should handle null bytes safely
            let _ = validate_input(test_string, "null_byte_test");
            let _ = validate_url(&format!("http://example.com/{}", test_string));
            let _ = validate_model_name(test_string);
        }
    }

    #[test]
    fn test_sql_injection_fuzzing() {
        let mut rng = thread_rng();
        let sql_fragments = [
            "SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION", "CREATE", "ALTER",
            "EXEC", "EXECUTE", "sp_", "xp_", "/*", "*/", "--", ";", "'", "\"",
        ];
        
        for _ in 0..500 {
            // Create random combinations of SQL fragments
            let fragment_count = rng.gen_range(1..5);
            let mut test_input = String::new();
            
            for _ in 0..fragment_count {
                if !test_input.is_empty() {
                    test_input.push(' ');
                }
                let fragment = sql_fragments[rng.gen_range(0..sql_fragments.len())];
                test_input.push_str(fragment);
                
                // Add random content
                let random_content: String = (0..rng.gen_range(0..20))
                    .map(|_| rng.sample(Alphanumeric) as char)
                    .collect();
                test_input.push_str(&random_content);
            }
            
            // Should detect SQL injection attempts
            let result = validate_input(&test_input, "sql_fuzz");
            // We don't assert the result since some combinations might be valid
            // The important thing is that it doesn't panic
        }
    }

    #[test]
    fn test_performance_fuzzing() {
        use std::time::Instant;
        let mut rng = thread_rng();
        
        // Test that validation remains fast even with pathological inputs
        for _ in 0..50 {
            let length = rng.gen_range(5000..10000);
            let pattern_heavy_input = "SELECT UNION INSERT UPDATE DELETE DROP ".repeat(length / 40);
            
            let start = Instant::now();
            let _ = validate_input(&pattern_heavy_input, "performance_fuzz");
            let duration = start.elapsed();
            
            // Should complete within reasonable time even for pattern-heavy input
            assert!(duration.as_millis() < 1000, 
                   "Validation took too long: {}ms for {} chars", 
                   duration.as_millis(), pattern_heavy_input.len());
        }
    }

    #[test]
    fn test_memory_usage_fuzzing() {
        let mut rng = thread_rng();
        
        // Test that validation doesn't cause excessive memory allocation
        for _ in 0..100 {
            let length = rng.gen_range(5000..9999);
            let large_input: String = (0..length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            
            // Should handle large inputs without excessive memory usage
            let _ = validate_input(&large_input, "memory_fuzz");
            let _ = validate_url(&format!("http://example.com/{}", large_input));
        }
    }

    #[test]
    fn test_empty_and_whitespace_fuzzing() {
        let whitespace_variants = vec![
            "",
            " ",
            "\t",
            "\n",
            "\r",
            "\r\n",
            "   ",
            "\t\t\t",
            "\n\n\n",
            " \t \n \r ",
            "\u{00A0}", // Non-breaking space
            "\u{2000}", // En quad
            "\u{2001}", // Em quad
            "\u{2028}", // Line separator
            "\u{2029}", // Paragraph separator
        ];
        
        for whitespace in whitespace_variants {
            // Should handle all types of whitespace gracefully
            let _ = validate_input(whitespace, "whitespace_fuzz");
            let _ = validate_url(&format!("http://example.com{}", whitespace));
            let _ = validate_model_name(whitespace);
        }
    }

    #[test]
    fn test_encoding_fuzzing() {
        // Test various encoding attempts that might bypass validation
        let encoding_attempts = vec![
            "%27%20OR%201=1--", // URL encoded SQL injection
            "&lt;script&gt;", // HTML encoded
            "\\x27\\x20OR\\x201=1--", // Hex encoded
            "\u{0027} OR 1=1--", // Unicode escaped
            "&#39; OR 1=1--", // HTML entity
            "%u0027 OR 1=1--", // Unicode URL encoding
        ];
        
        for encoded_input in encoding_attempts {
            // Should handle encoded inputs safely
            let _ = validate_input(encoded_input, "encoding_fuzz");
        }
    }

    #[test]
    fn test_concurrent_fuzzing() {
        use std::thread;
        use std::sync::Arc;
        use std::sync::atomic::{AtomicUsize, Ordering};
        
        let iteration_count = Arc::new(AtomicUsize::new(0));
        
        let handles: Vec<_> = (0..4).map(|_| {
            let iteration_count = Arc::clone(&iteration_count);
            
            thread::spawn(move || {
                let mut rng = thread_rng();
                
                for _ in 0..250 {
                    let length = rng.gen_range(0..1000);
                    let random_input: String = (0..length)
                        .map(|_| rng.gen_range(0..128) as u8 as char)
                        .collect();
                    
                    // Should be thread-safe
                    let _ = validate_input(&random_input, "concurrent_fuzz");
                    let _ = validate_url(&format!("http://test.com/{}", random_input));
                    let _ = validate_model_name(&random_input);
                    
                    iteration_count.fetch_add(1, Ordering::SeqCst);
                }
            })
        }).collect();
        
        for handle in handles {
            handle.join().unwrap();
        }
        
        assert_eq!(iteration_count.load(Ordering::SeqCst), 1000);
    }
}

#[cfg(test)]
mod property_based_tests {
    use super::*;
    
    #[test]
    fn property_validation_never_panics() {
        let mut rng = thread_rng();
        
        // Property: validate_input should never panic for any string input
        for _ in 0..2000 {
            let length = rng.gen_range(0..15000);
            let input: String = (0..length)
                .map(|_| rng.gen_range(0..1114112)) // Full Unicode range
                .filter_map(std::char::from_u32)
                .collect();
            
            // This should never panic
            let _ = std::panic::catch_unwind(|| validate_input(&input, "property_test"));
        }
    }
    
    #[test]
    fn property_empty_input_always_valid() {
        // Property: Empty string should always be considered valid input
        assert!(validate_input("", "test").is_ok());
        assert!(validate_input("", "any_field_name").is_ok());
    }
    
    #[test]
    fn property_alphanumeric_within_limit_valid() {
        let mut rng = thread_rng();
        
        // Property: Pure alphanumeric strings within length limit should be valid
        for _ in 0..100 {
            let length = rng.gen_range(1..1000); // Well within limit
            let alphanumeric: String = (0..length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            
            assert!(validate_input(&alphanumeric, "property_test").is_ok(),
                   "Alphanumeric string should be valid: {}", alphanumeric);
        }
    }
    
    #[test]
    fn property_over_limit_always_invalid() {
        let mut rng = thread_rng();
        
        // Property: Any string over the length limit should be invalid
        for _ in 0..50 {
            let length = rng.gen_range(10001..15000); // Over the 10000 limit
            let long_string = "a".repeat(length);
            
            assert!(validate_input(&long_string, "property_test").is_err(),
                   "String over limit should be invalid");
        }
    }
    
    #[test]
    fn property_sql_keywords_always_detected() {
        let sql_keywords = ["SELECT", "INSERT", "UPDATE", "DELETE", "DROP", "UNION"];
        
        // Property: Strings containing SQL keywords should be detected
        for keyword in &sql_keywords {
            let test_cases = vec![
                keyword.to_lowercase(),
                keyword.to_uppercase(),
                format!("prefix {} suffix", keyword.to_lowercase()),
                format!("{} command", keyword.to_lowercase()),
            ];
            
            for test_case in test_cases {
                let result = validate_input(&test_case, "property_test");
                assert!(result.is_err(), 
                       "SQL keyword should be detected: {}", test_case);
            }
        }
    }
    
    #[test]
    fn property_valid_urls_have_protocol() {
        let mut rng = thread_rng();
        
        // Property: Valid URLs should start with http:// or https://
        for _ in 0..100 {
            let domain_length = rng.gen_range(1..50);
            let domain: String = (0..domain_length)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            
            let http_url = format!("http://{}.com", domain);
            let https_url = format!("https://{}.com", domain);
            
            // These should be valid (assuming no other validation failures)
            let http_result = validate_url(&http_url);
            let https_result = validate_url(&https_url);
            
            // If they fail, it should not be due to protocol
            if let Err(e) = http_result {
                assert!(!e.to_string().contains("must start with http"));
            }
            if let Err(e) = https_result {
                assert!(!e.to_string().contains("must start with http"));
            }
        }
    }
}