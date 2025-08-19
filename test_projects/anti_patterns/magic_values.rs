// Test file for Magic Values detection

fn calculate_area(radius: f64) -> f64 {
    // Magic number 3.14159 should be detected
    radius * radius * 3.14159
}

fn calculate_tax(amount: f64) -> f64 {
    // Multiple magic numbers
    if amount > 1000.0 {
        amount * 0.15 + 50.0 - 25.5
    } else {
        amount * 0.08 + 10.0
    }
}

fn process_status(status: i32) -> String {
    match status {
        200 => "OK".to_string(),        // HTTP status codes are magic numbers
        404 => "Not Found".to_string(),
        500 => "Internal Server Error".to_string(),
        _ => "Unknown".to_string(),
    }
}

fn array_operations() {
    let mut data = vec![0; 100];  // Magic number for array size
    for i in 0..42 {              // Magic number for loop limit
        if i % 7 == 0 {           // Magic number for modulo
            data[i] = i * 13;     // Magic number for multiplier  
        }
    }
}