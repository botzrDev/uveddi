use std::env;
use std::fs;
use std::path::Path;

fn main() {
    let out_dir = env::var("OUT_DIR").unwrap();
    
    // Generate WIT bindings if WIT files exist
    let wit_dir = Path::new("wit");
    if wit_dir.exists() {
        println!("cargo:rerun-if-changed=wit");
        
        // Use wit-bindgen to generate bindings
        // This would typically be done by the wit-bindgen crate
        // For now, we'll just ensure the directory exists
    }
    
    // Copy plugin.toml to output directory for packaging
    let plugin_toml = Path::new("plugin.toml");
    if plugin_toml.exists() {
        let dest_path = Path::new(&out_dir).join("plugin.toml");
        fs::copy(plugin_toml, dest_path).unwrap();
        println!("cargo:rerun-if-changed=plugin.toml");
    }
    
    println!("cargo:rerun-if-changed=src/");
}