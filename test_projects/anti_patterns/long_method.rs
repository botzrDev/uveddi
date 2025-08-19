// Test file for Long Method detection

fn very_long_method(input: Vec<i32>) -> Vec<i32> {
    let mut result = Vec::new();
    
    // Start of a very long method (50+ lines to trigger detection)
    println!("Starting processing of {} items", input.len());
    
    for item in &input {
        if *item > 0 {
            println!("Processing positive item: {}", item);
            let processed = item * 2;
            
            if processed > 100 {
                println!("Large value detected: {}", processed);
                let adjusted = processed - 50;
                result.push(adjusted);
            } else if processed > 50 {
                println!("Medium value: {}", processed);
                let adjusted = processed - 25;
                result.push(adjusted);
            } else {
                println!("Small value: {}", processed);
                result.push(processed);
            }
        } else if *item == 0 {
            println!("Zero value found");
            result.push(1);
        } else {
            println!("Processing negative item: {}", item);
            let processed = item.abs() * 3;
            
            if processed > 150 {
                println!("Large negative converted: {}", processed);
                let adjusted = processed - 75;
                result.push(-adjusted);
            } else if processed > 75 {
                println!("Medium negative: {}", processed);  
                let adjusted = processed - 50;
                result.push(-adjusted);
            } else {
                println!("Small negative: {}", processed);
                result.push(-processed);
            }
        }
        
        // Add some more complexity to increase line count
        let temp = result.len();
        if temp % 10 == 0 {
            println!("Processed {} items so far", temp);
            
            // Some additional processing
            let average: f64 = result.iter().sum::<i32>() as f64 / result.len() as f64;
            println!("Current average: {:.2}", average);
            
            if average > 50.0 {
                println!("High average detected");
                for val in result.iter_mut() {
                    *val = (*val as f64 * 0.9) as i32;
                }
            } else if average < -50.0 {
                println!("Low average detected");
                for val in result.iter_mut() {
                    *val = (*val as f64 * 1.1) as i32;
                }
            }
        }
    }
    
    // Final processing and validation
    println!("Final result length: {}", result.len());
    
    // Sort the results
    result.sort();
    println!("Results sorted");
    
    // Remove duplicates
    result.dedup();
    println!("Duplicates removed, final length: {}", result.len());
    
    // Final validation
    let mut valid_count = 0;
    for val in &result {
        if val.abs() < 1000 {
            valid_count += 1;
        }
    }
    println!("Valid values: {}/{}", valid_count, result.len());
    
    result
}

fn short_method() -> i32 {
    42
}
