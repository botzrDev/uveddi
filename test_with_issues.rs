// Test file with obvious architectural issues for report validation

struct GodObject {
    field1: String,
    field2: String,
    field3: String,
    field4: String,
    field5: String,
    field6: String,
    field7: String,
    field8: String,
    field9: String,
    field10: String,
    field11: String,
    field12: String,
    field13: String,
    field14: String,
    field15: String,
    field16: String,
    field17: String,
    field18: String,
    field19: String,
    field20: String,
}

impl GodObject {
    // Very long method with lots of responsibilities
    pub fn massive_method(&self) -> String {
        let mut result = String::new();
        result.push_str("line 1\n");
        result.push_str("line 2\n");
        result.push_str("line 3\n");
        result.push_str("line 4\n");
        result.push_str("line 5\n");
        result.push_str("line 6\n");
        result.push_str("line 7\n");
        result.push_str("line 8\n");
        result.push_str("line 9\n");
        result.push_str("line 10\n");
        result.push_str("line 11\n");
        result.push_str("line 12\n");
        result.push_str("line 13\n");
        result.push_str("line 14\n");
        result.push_str("line 15\n");
        result.push_str("line 16\n");
        result.push_str("line 17\n");
        result.push_str("line 18\n");
        result.push_str("line 19\n");
        result.push_str("line 20\n");
        result.push_str("line 21\n");
        result.push_str("line 22\n");
        result.push_str("line 23\n");
        result.push_str("line 24\n");
        result.push_str("line 25\n");
        result.push_str("line 26\n");
        result.push_str("line 27\n");
        result.push_str("line 28\n");
        result.push_str("line 29\n");
        result.push_str("line 30\n");
        result.push_str("line 31\n");
        result.push_str("line 32\n");
        result.push_str("line 33\n");
        result.push_str("line 34\n");
        result.push_str("line 35\n");
        result.push_str("line 36\n");
        result.push_str("line 37\n");
        result.push_str("line 38\n");
        result.push_str("line 39\n");
        result.push_str("line 40\n");
        result.push_str("line 41\n");
        result.push_str("line 42\n");
        result.push_str("line 43\n");
        result.push_str("line 44\n");
        result.push_str("line 45\n");
        result.push_str("line 46\n");
        result.push_str("line 47\n");
        result.push_str("line 48\n");
        result.push_str("line 49\n");
        result.push_str("line 50\n");
        result.push_str("line 51\n");
        result.push_str("line 52\n");
        result.push_str("line 53\n");
        result.push_str("line 54\n");
        result.push_str("line 55\n");
        result.push_str("line 56\n");
        result.push_str("line 57\n");
        result.push_str("line 58\n");
        result.push_str("line 59\n");
        result.push_str("line 60\n");
        result
    }

    // Duplicate code
    pub fn duplicate_code_1(&self) -> String {
        let mut result = String::new();
        result.push_str("duplicate line 1\n");
        result.push_str("duplicate line 2\n");
        result.push_str("duplicate line 3\n");
        result.push_str("duplicate line 4\n");
        result.push_str("duplicate line 5\n");
        result
    }

    // More duplicate code
    pub fn duplicate_code_2(&self) -> String {
        let mut result = String::new();
        result.push_str("duplicate line 1\n");
        result.push_str("duplicate line 2\n");
        result.push_str("duplicate line 3\n");
        result.push_str("duplicate line 4\n");
        result.push_str("duplicate line 5\n");
        result
    }

    // Dead code that's never called
    fn unused_method(&self) -> i32 {
        42
    }

    fn another_unused_method(&self) -> bool {
        true
    }
}

// Magic numbers everywhere
const MAGIC_NUMBER_1: i32 = 42;
const MAGIC_NUMBER_2: i32 = 123;
const MAGIC_NUMBER_3: i32 = 999;