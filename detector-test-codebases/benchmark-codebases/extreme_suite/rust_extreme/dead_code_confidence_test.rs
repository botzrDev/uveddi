/*
 * DEAD CODE CONFIDENCE STRESS TEST
 * Tests various confidence levels for dead code detection
 * Expected Detection: DeadCodeDetector with confidence scores
 */

#[allow(dead_code)]
// 99% Confidence: Private function never called
fn private_never_called() -> i32 {
    42
}

// 90% Confidence: Public function not called in codebase  
pub fn exported_but_unused() -> String {
    "unused".to_string()
}

// 70% Confidence: Exported function with no obvious usage
#[no_mangle]
pub extern "C" fn c_export_unused() -> i32 {
    0
}

// 50% Confidence: Interface implementation that might be used dynamically
pub trait DynamicInterface {
    fn dynamic_method(&self) -> bool;
}

pub struct UnusedImplementation;

impl DynamicInterface for UnusedImplementation {
    fn dynamic_method(&self) -> bool {
        true
    }
}

// 30% Confidence: Public API that could be used by external code
pub fn public_api_maybe_used() -> Vec<i32> {
    vec![1, 2, 3]
}

// 10% Confidence: Framework hook that might be called
#[no_mangle]
pub extern "C" fn plugin_hook() {
    // Framework might call this
}

// False dead code: Used via macro
macro_rules! use_hidden_function {
    () => {
        hidden_function_used_by_macro()
    };
}

fn hidden_function_used_by_macro() -> bool {
    true
}

// Test the macro (this makes the function live)
pub fn test_macro() -> bool {
    use_hidden_function!()
}

// Commented out "zombie" code
/*
fn zombie_code() -> String {
    "I might be reactivated".to_string()
}
*/

// Conditionally compiled code
#[cfg(feature = "experimental")]
pub fn feature_gated_function() -> i32 {
    experimental_logic()
}

#[cfg(feature = "experimental")]
fn experimental_logic() -> i32 {
    100
}
