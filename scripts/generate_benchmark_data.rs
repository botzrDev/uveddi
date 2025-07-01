use std::{fs, io::Write};

fn main() {
    let dir = "target/benchmark-data/small";
    fs::create_dir_all(dir).unwrap();
    for i in 0..100 {
        let file_path = format!("{}/file_{}.txt", dir, i);
        let mut file = fs::File::create(&file_path).unwrap();
        let content = vec![b'a'; 1024 * 100]; // 100 KB
        file.write_all(&content).unwrap();
    }
    println!("Generated 100 files of 100KB each in {}", dir);
}
