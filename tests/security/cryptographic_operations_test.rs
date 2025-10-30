//! Comprehensive cryptographic operations testing
//! 
//! This module tests all cryptographic operations in the Uveddi security framework
//! to ensure correctness, security, and resistance to timing attacks.

use ring::{digest, rand, signature, agreement, aead};
use rusqlite::Connection;
use std::time::{Duration, Instant};
use tempfile::TempDir;
use uveddi::security::{
    authentication::AuthManager,
    authorization::RoleManager,
    secrets::SecretManager,
};

/// Test key derivation functions for correctness and consistency
#[tokio::test]
async fn test_key_derivation_correctness() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Test PBKDF2 key derivation
    let password = b"test_password_123";
    let salt = b"test_salt_456";
    
    let key1 = secret_manager.derive_key(password, salt, 100_000)
        .expect("Key derivation failed");
    let key2 = secret_manager.derive_key(password, salt, 100_000)
        .expect("Key derivation failed");
    
    // Same inputs should produce same outputs
    assert_eq!(key1, key2, "Key derivation not deterministic");
    
    // Different salts should produce different keys
    let different_salt = b"different_salt";
    let key3 = secret_manager.derive_key(password, different_salt, 100_000)
        .expect("Key derivation failed");
    
    assert_ne!(key1, key3, "Different salts should produce different keys");
}

/// Test encryption/decryption cycle for various data sizes
#[tokio::test]
async fn test_encryption_decryption_cycle() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Test data of various sizes
    let test_cases = vec![
        b"small".to_vec(),
        b"medium_sized_data_for_testing_encryption".to_vec(),
        vec![0xAA; 1024], // 1KB of data
        vec![0xBB; 64 * 1024], // 64KB of data
    ];
    
    for (i, plaintext) in test_cases.into_iter().enumerate() {
        // Generate random key for this test
        let key = secret_manager.generate_encryption_key()
            .expect("Failed to generate encryption key");
        
        // Encrypt the data
        let encrypted = secret_manager.encrypt(&plaintext, &key)
            .expect(&format!("Encryption failed for test case {}", i));
        
        // Verify encrypted data is different from plaintext
        assert_ne!(encrypted, plaintext, "Encrypted data same as plaintext for case {}", i);
        
        // Decrypt the data
        let decrypted = secret_manager.decrypt(&encrypted, &key)
            .expect(&format!("Decryption failed for test case {}", i));
        
        // Verify decrypted data matches original
        assert_eq!(decrypted, plaintext, "Decrypted data doesn't match original for case {}", i);
    }
}

/// Test resistance to timing attacks in key operations
#[tokio::test]
async fn test_timing_attack_resistance() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Generate test keys
    let correct_key = secret_manager.generate_encryption_key()
        .expect("Failed to generate correct key");
    let wrong_key = secret_manager.generate_encryption_key()
        .expect("Failed to generate wrong key");
    
    let test_data = b"timing_attack_test_data";
    let encrypted = secret_manager.encrypt(test_data, &correct_key)
        .expect("Encryption failed");
    
    // Measure decryption times
    let mut correct_times = Vec::new();
    let mut wrong_times = Vec::new();
    
    // Perform multiple measurements
    for _ in 0..100 {
        // Time correct key decryption
        let start = Instant::now();
        let _ = secret_manager.decrypt(&encrypted, &correct_key);
        correct_times.push(start.elapsed());
        
        // Time wrong key decryption
        let start = Instant::now();
        let _ = secret_manager.decrypt(&encrypted, &wrong_key);
        wrong_times.push(start.elapsed());
    }
    
    // Calculate average times
    let avg_correct: Duration = correct_times.iter().sum::<Duration>() / correct_times.len() as u32;
    let avg_wrong: Duration = wrong_times.iter().sum::<Duration>() / wrong_times.len() as u32;
    
    // Times should be similar (within 20% difference) to prevent timing attacks
    let time_diff_ratio = if avg_correct > avg_wrong {
        avg_correct.as_nanos() as f64 / avg_wrong.as_nanos() as f64
    } else {
        avg_wrong.as_nanos() as f64 / avg_correct.as_nanos() as f64
    };
    
    assert!(time_diff_ratio < 1.2, 
           "Timing difference too large: {}ms vs {}ms (ratio: {:.2})", 
           avg_correct.as_millis(), avg_wrong.as_millis(), time_diff_ratio);
}

/// Test secure random number generation
#[tokio::test]
async fn test_secure_random_generation() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Generate multiple random values
    let mut random_values = Vec::new();
    for _ in 0..1000 {
        let random_bytes = secret_manager.generate_random_bytes(32)
            .expect("Failed to generate random bytes");
        assert_eq!(random_bytes.len(), 32, "Random bytes wrong length");
        random_values.push(random_bytes);
    }
    
    // Check for duplicates (should be extremely unlikely)
    for (i, val1) in random_values.iter().enumerate() {
        for (j, val2) in random_values.iter().enumerate().skip(i + 1) {
            assert_ne!(val1, val2, "Duplicate random values at indices {} and {}", i, j);
        }
    }
    
    // Test entropy (basic chi-square test)
    let all_bytes: Vec<u8> = random_values.into_iter().flatten().collect();
    let mut byte_counts = [0u32; 256];
    
    for byte in &all_bytes {
        byte_counts[*byte as usize] += 1;
    }
    
    // Expected frequency for each byte value
    let expected_freq = all_bytes.len() as f64 / 256.0;
    
    // Calculate chi-square statistic
    let chi_square: f64 = byte_counts.iter()
        .map(|&count| {
            let diff = count as f64 - expected_freq;
            (diff * diff) / expected_freq
        })
        .sum();
    
    // Critical value for chi-square test (255 degrees of freedom, p=0.001)
    let critical_value = 310.457;
    assert!(chi_square < critical_value, 
           "Random bytes failed entropy test: chi-square = {:.2}", chi_square);
}

/// Test hash function consistency and collision resistance
#[tokio::test]
async fn test_hash_function_security() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Test SHA-256 consistency
    let test_data = b"hash_test_data";
    let hash1 = secret_manager.hash_sha256(test_data)
        .expect("SHA-256 hash failed");
    let hash2 = secret_manager.hash_sha256(test_data)
        .expect("SHA-256 hash failed");
    
    assert_eq!(hash1, hash2, "Hash function not deterministic");
    assert_eq!(hash1.len(), 32, "SHA-256 hash wrong length");
    
    // Test different inputs produce different hashes
    let different_data = b"different_hash_test_data";
    let hash3 = secret_manager.hash_sha256(different_data)
        .expect("SHA-256 hash failed");
    
    assert_ne!(hash1, hash3, "Different inputs should produce different hashes");
    
    // Test HMAC functionality
    let key = secret_manager.generate_hmac_key()
        .expect("Failed to generate HMAC key");
    
    let hmac1 = secret_manager.hmac_sha256(test_data, &key)
        .expect("HMAC failed");
    let hmac2 = secret_manager.hmac_sha256(test_data, &key)
        .expect("HMAC failed");
    
    assert_eq!(hmac1, hmac2, "HMAC not deterministic");
    
    // Different keys should produce different HMACs
    let different_key = secret_manager.generate_hmac_key()
        .expect("Failed to generate HMAC key");
    let hmac3 = secret_manager.hmac_sha256(test_data, &different_key)
        .expect("HMAC failed");
    
    assert_ne!(hmac1, hmac3, "Different keys should produce different HMACs");
}

/// Test certificate validation and trust chain verification
#[tokio::test]
async fn test_certificate_validation() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Load test certificates (these would be test fixtures)
    let valid_cert = include_bytes!("../fixtures/certs/valid_cert.pem");
    let expired_cert = include_bytes!("../fixtures/certs/expired_cert.pem");
    let self_signed_cert = include_bytes!("../fixtures/certs/self_signed_cert.pem");
    
    // Test valid certificate
    let validation_result = secret_manager.validate_certificate(valid_cert)
        .expect("Certificate validation failed");
    assert!(validation_result.is_valid, "Valid certificate marked as invalid");
    
    // Test expired certificate
    let validation_result = secret_manager.validate_certificate(expired_cert)
        .expect("Certificate validation failed");
    assert!(!validation_result.is_valid, "Expired certificate marked as valid");
    assert!(validation_result.errors.iter().any(|e| e.contains("expired")), 
           "Expired certificate error not detected");
    
    // Test self-signed certificate
    let validation_result = secret_manager.validate_certificate(self_signed_cert)
        .expect("Certificate validation failed");
    assert!(!validation_result.is_valid, "Self-signed certificate marked as valid without CA");
}

/// Test key rotation functionality
#[tokio::test]
async fn test_key_rotation() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Generate initial key
    let initial_key = secret_manager.generate_encryption_key()
        .expect("Failed to generate initial key");
    
    // Test data
    let test_data = b"key_rotation_test_data";
    
    // Encrypt with initial key
    let encrypted = secret_manager.encrypt(test_data, &initial_key)
        .expect("Encryption failed");
    
    // Rotate to new key
    let new_key = secret_manager.rotate_key(&initial_key)
        .expect("Key rotation failed");
    
    // Verify new key is different
    assert_ne!(initial_key, new_key, "Key rotation produced same key");
    
    // Should still be able to decrypt with old key
    let decrypted = secret_manager.decrypt(&encrypted, &initial_key)
        .expect("Decryption with old key failed");
    assert_eq!(decrypted, test_data, "Decryption with old key incorrect");
    
    // New encryptions should use new key
    let new_encrypted = secret_manager.encrypt(test_data, &new_key)
        .expect("Encryption with new key failed");
    
    // Verify old key can't decrypt new data
    assert!(secret_manager.decrypt(&new_encrypted, &initial_key).is_err(), 
           "Old key should not decrypt new data");
    
    // Verify new key can decrypt new data
    let new_decrypted = secret_manager.decrypt(&new_encrypted, &new_key)
        .expect("Decryption with new key failed");
    assert_eq!(new_decrypted, test_data, "Decryption with new key incorrect");
}

/// Test secure memory handling and cleanup
#[tokio::test]
async fn test_secure_memory_handling() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Test that sensitive data is properly zeroized
    let sensitive_data = b"very_sensitive_password_data";
    let mut secure_buffer = secret_manager.create_secure_buffer(sensitive_data.len())
        .expect("Failed to create secure buffer");
    
    secure_buffer.copy_from_slice(sensitive_data);
    
    // Verify data is there
    assert_eq!(&secure_buffer[..], sensitive_data);
    
    // Clear the buffer
    secret_manager.secure_zero(&mut secure_buffer);
    
    // Verify data is cleared (all zeros)
    assert!(secure_buffer.iter().all(|&b| b == 0), "Secure buffer not properly cleared");
}

/// Test cryptographic algorithm consistency across platforms
#[tokio::test]
async fn test_algorithm_consistency() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Known test vectors for various algorithms
    let test_vectors = vec![
        // SHA-256 test vectors
        (b"abc".as_slice(), "ba7816bf8f01cfea414140de5dae2223b00361a396177a9cb410ff61f20015ad"),
        (b"".as_slice(), "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855"),
        (b"message digest".as_slice(), "f7846f55cf23e14eebeab5b4e1550cad5b509e3348fbc4efa3a1413d393cb650"),
    ];
    
    for (input, expected_hex) in test_vectors {
        let hash = secret_manager.hash_sha256(input)
            .expect("SHA-256 hash failed");
        let hash_hex = hex::encode(hash);
        assert_eq!(hash_hex, expected_hex, 
                  "SHA-256 hash mismatch for input: {:?}", std::str::from_utf8(input));
    }
}

/// Test error handling in cryptographic operations
#[tokio::test]
async fn test_cryptographic_error_handling() {
    let secret_manager = SecretManager::new().expect("Failed to create SecretManager");
    
    // Test decryption with wrong key
    let key1 = secret_manager.generate_encryption_key()
        .expect("Failed to generate key1");
    let key2 = secret_manager.generate_encryption_key()
        .expect("Failed to generate key2");
    
    let plaintext = b"test_data";
    let encrypted = secret_manager.encrypt(plaintext, &key1)
        .expect("Encryption failed");
    
    // Should fail with wrong key
    assert!(secret_manager.decrypt(&encrypted, &key2).is_err(), 
           "Decryption should fail with wrong key");
    
    // Test with corrupted data
    let mut corrupted = encrypted.clone();
    corrupted[0] ^= 0xFF; // Flip bits in first byte
    
    assert!(secret_manager.decrypt(&corrupted, &key1).is_err(), 
           "Decryption should fail with corrupted data");
    
    // Test with empty/invalid inputs
    assert!(secret_manager.decrypt(&[], &key1).is_err(), 
           "Decryption should fail with empty data");
    
    assert!(secret_manager.encrypt(plaintext, &[]).is_err(), 
           "Encryption should fail with empty key");
}

#[cfg(test)]
mod helpers {
    use super::*;
    
    /// Helper to create test certificates for validation tests
    pub fn create_test_certificates() {
        // This would generate test certificates for the test fixtures
        // Implementation would go here
    }
    
    /// Helper to measure timing with high precision
    pub fn measure_timing<F, R>(mut operation: F) -> (R, Duration)
    where
        F: FnMut() -> R,
    {
        let start = Instant::now();
        let result = operation();
        let duration = start.elapsed();
        (result, duration)
    }
}