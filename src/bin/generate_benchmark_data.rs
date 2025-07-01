use std::{fs, io::Write, path::Path};

const NUM_FILES: usize = 100;

fn main() -> std::io::Result<()> {
    let small_dir = Path::new("target/benchmark-data/small");
    generate_dataset(small_dir, NUM_FILES, 5)?;

    let medium_dir = Path::new("target/benchmark-data/medium");
    generate_dataset(medium_dir, NUM_FILES * 5, 20)?;

    Ok(())
}

fn generate_dataset(dir: &Path, num_files: usize, complexity: usize) -> std::io::Result<()> {
    if dir.exists() {
        fs::remove_dir_all(dir)?;
    }
    fs::create_dir_all(dir)?;

    for i in 0..num_files {
        let file_path = dir.join(format!("module_{}.rs", i));
        let mut file = fs::File::create(&file_path)?;
        let content = generate_complex_rust_code(i, complexity, num_files);
        file.write_all(content.as_bytes())?;
    }

    println!(
        "Generated {} complex rust files in {}",
        num_files,
        dir.display()
    );
    Ok(())
}

fn generate_complex_rust_code(file_index: usize, complexity: usize, total_files: usize) -> String {
    let mut code = String::new();
    code.push_str(&format!("// Module {}\n\n", file_index));

    // Add some imports to other generated modules
    for j in 1..=3 {
        let import_index = (file_index + j) % total_files;
        code.push_str(&format!("use crate::module_{}::Struct{};\n", import_index, import_index));
    }
    code.push_str("\n");

    // Add a struct definition
    code.push_str(&format!("pub struct Struct{} {{\n", file_index));
    for i in 0..complexity {
        code.push_str(&format!("    field_{}: u32,\n", i));
    }
    code.push_str("}\n\n");

    // Add an implementation block with some functions
    code.push_str(&format!("impl Struct{} {{\n", file_index));
    for i in 0..complexity {
        code.push_str(&format!("    pub fn func_{}(&self, arg: u32) -> u32 {{\n", i));
        code.push_str(&format!("        self.field_{} + arg\n", i));
        code.push_str("    }\n");
    }
    code.push_str("}\n");

    code
}
