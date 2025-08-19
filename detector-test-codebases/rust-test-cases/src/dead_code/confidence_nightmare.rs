// Dead code confidence tests

#[allow(dead_code)]
fn private_never_called() {}

pub fn exported_but_unused() {}

// Functions that might be used dynamically by macros or tests
pub fn maybe_used_by_macro() {}
