use std::path::Path;

fn main() {
    // Rebuild if WIT files change
    let wit_dir = Path::new("wit");
    if wit_dir.exists() {
        println!("cargo:rerun-if-changed=wit/core-analysis.wit");
    }

    // Rebuild if plugin manifest changes
    let plugin_toml = Path::new("plugin.toml");
    if plugin_toml.exists() {
        println!("cargo:rerun-if-changed=plugin.toml");
    }

    // Rebuild if source files change
    println!("cargo:rerun-if-changed=src/");
}
