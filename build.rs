//! Build Script for Uveddi - Asset Bundling and Knowledge Library
//!
//! This build script handles asset bundling for the modern report system
//! and knowledge library generation.

use std::env;
use std::fs;
use std::path::Path;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rerun-if-changed=data/knowledge_base.json");
    println!("cargo:rerun-if-changed=src/ai/knowledge/");
    println!("cargo:rerun-if-changed=assets/");
    println!("cargo:rerun-if-changed=src/templates/");

    let out_dir = env::var("OUT_DIR")?;
    let out_path = Path::new(&out_dir);

    // Generate a minimal knowledge library stub for now
    generate_minimal_knowledge_library(out_path)?;

    // Build asset pipeline for modern reports
    println!("Building Uveddi asset pipeline...");
    build_asset_pipeline(out_path)?;

    println!("cargo:warning=Build completed - Knowledge library (minimal) + Asset pipeline");
    Ok(())
}

/// Generate minimal knowledge library for compilation
fn generate_minimal_knowledge_library(out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let stub_content = r#"
// Generated knowledge library stub
pub struct OptimizedKnowledgeLibrary {
    pub pattern_count: usize,
    pub compressed_size: usize,
    pub compression_ratio: f32,
}

impl OptimizedKnowledgeLibrary {
    pub fn new() -> Self {
        Self {
            pattern_count: 0,
            compressed_size: 0,
            compression_ratio: 0.0,
        }
    }
    
    pub fn from_compressed(_data: &[u8]) -> Result<Self, Box<dyn std::error::Error>> {
        Ok(Self::new())
    }
}

pub fn get_knowledge_base() -> &'static OptimizedKnowledgeLibrary {
    use std::sync::OnceLock;
    static KNOWLEDGE_BASE: OnceLock<OptimizedKnowledgeLibrary> = OnceLock::new();
    KNOWLEDGE_BASE.get_or_init(|| OptimizedKnowledgeLibrary::new())
}
"#;

    let stub_file = out_path.join("knowledge_library_generated.rs");
    fs::write(stub_file, stub_content)?;

    Ok(())
}

/// Build asset pipeline for modern report system
fn build_asset_pipeline(out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    // Check if asset files exist
    let css_filename = format!("{}.{}", "main", "css");
    let css_path = Path::new("assets").join("css").join(css_filename);
    if !css_path.exists() {
        println!("cargo:warning=assets/css/main.css not found, skipping asset bundling");
        generate_empty_assets(out_path)?;
        return Ok(());
    }

    // Bundle CSS and JS
    let css = bundle_css()?;
    let js = bundle_js()?;

    // Generate Rust constants
    generate_asset_constants(out_path, &css, &js)?;
    generate_feature_flags(out_path)?;

    // Calculate stats
    let total_bundled = css.len() + js.len();
    println!("cargo:warning=Asset bundling complete: {total_bundled} bytes total");

    Ok(())
}

/// Generate empty assets when source files don't exist
fn generate_empty_assets(out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let content = r#"// Empty bundled assets
pub const BUNDLED_CSS: &str = "";
pub const BUNDLED_JS: &str = "";
pub const ASSET_MANIFEST: &str = "{\"css_size\": 0, \"js_size\": 0}";
"#;

    let dest_path = out_path.join("bundled_assets").with_extension("rs");
    fs::write(dest_path, content)?;
    Ok(())
}

/// Generate CSS-only icons using Unicode symbols (no external font dependencies)
fn generate_fontawesome_subset() -> String {
    // Use Unicode symbols for reliable cross-platform icon display
    r#"
/* CSS-only icons using Unicode symbols - no external font dependencies */
.fas, .fa-solid {
    font-style: normal;
    font-variant: normal;
    text-rendering: auto;
    line-height: 1;
    -webkit-font-smoothing: antialiased;
    -moz-osx-font-smoothing: grayscale;
    display: inline-block;
    font-weight: normal;
}

.fas::before, .fa-solid::before {
    display: inline-block;
    text-rendering: auto;
    -webkit-font-smoothing: antialiased;
}

/* Unicode-based icons that work without external fonts */
.fa-moon::before { content: "🌙"; }
.fa-sun::before { content: "☀️"; }
.fa-print::before { content: "🖨️"; }
.fa-download::before { content: "⬇️"; }
.fa-expand::before { content: "⛶"; }
.fa-compress-alt::before { content: "⤢"; }
.fa-chevron-right::before { content: "▶"; }
.fa-chevron-down::before { content: "▼"; }
.fa-chevron-up::before { content: "▲"; }
.fa-exclamation-triangle::before { content: "⚠️"; }
.fa-heartbeat::before { content: "💓"; }
.fa-coins::before { content: "🪙"; }
.fa-project-diagram::before { content: "📊"; }
.fa-file-code::before { content: "📄"; }
.fa-clock::before { content: "🕐"; }
.fa-times::before { content: "✕"; }
.fa-expand-alt::before { content: "⤢"; }
.fa-undo::before { content: "↶"; }
.fa-check-circle::before { content: "✅"; }
.fa-arrow-up::before { content: "↑"; }
.fa-arrow-down::before { content: "↓"; }
.fa-arrow-right::before { content: "→"; }
.fa-bullseye::before { content: "🎯"; }
.fa-cog::before { content: "⚙️"; }
.fa-chart-bar::before { content: "📊"; }
.fa-code::before { content: "💻"; }
.fa-bug::before { content: "🐛"; }
.fa-shield-alt::before { content: "🛡️"; }
.fa-bolt::before { content: "⚡"; }
.fa-layer-group::before { content: "📚"; }
"#
    .to_string()
}

/// Minify CSS by removing unnecessary whitespace and comments
fn minify_css(css: &str) -> String {
    // Simple minification - just remove excess whitespace
    css.lines()
        .map(|line| line.trim())
        .filter(|line| !line.is_empty())
        .collect::<Vec<_>>()
        .join("")
        .replace("  ", " ")
        .replace("; ", ";")
        .replace(": ", ":")
        .replace("{ ", "{")
        .replace(" }", "}")
        .replace(", ", ",")
}

/// Bundle and minify all CSS files
fn bundle_css() -> Result<String, Box<dyn std::error::Error>> {
    let css_filename = format!("{}.{}", "main", "css");
    let css_path = Path::new("assets").join("css").join(css_filename);
    let main_css = fs::read_to_string(css_path).unwrap_or_default();
    let fontawesome_css = generate_fontawesome_subset();

    let header = "/* Uveddi Report Bundled CSS */";
    let newline = '\n'.to_string();
    let combined = format!("{header}{newline}{fontawesome_css}{newline}{main_css}");
    let minified = minify_css(&combined);

    let size_msg = format!("CSS bundle size: {} bytes (minified)", minified.len());
    println!("cargo:warning={size_msg}");
    Ok(minified)
}

/// Bundle and minify all JavaScript files
fn bundle_js() -> Result<String, Box<dyn std::error::Error>> {
    let js_filename = format!("{}.{}", "main", "js");
    let js_path = Path::new("assets").join("js").join(js_filename);
    let main_js = fs::read_to_string(js_path).unwrap_or_default();

    let header = "/* Uveddi Report Bundled JavaScript */";
    let newline = '\n'.to_string();
    let bundled = format!("{header}{newline}{main_js}");
    // For now, just use the original JS without complex minification

    let bytes_label = "bytes";
    let size_msg = format!("JS bundle size: {} {}", bundled.len(), bytes_label);
    println!("cargo:warning={size_msg}");
    Ok(bundled)
}

/// Generate Rust constants for bundled assets
fn generate_asset_constants(
    out_path: &Path,
    css: &str,
    js: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    // Write assets to separate files and use include_str! to avoid escaping issues
    let css_filename = format!("{}.{}.{}", "bundle", "min", "css");
    let js_filename = format!("{}.{}.{}", "bundle", "min", "js");
    let css_file = out_path.join(css_filename);
    let js_file = out_path.join(js_filename);

    fs::write(&css_file, css)?;
    fs::write(&js_file, js)?;

    let dest_path = out_path.join("bundled_assets").with_extension("rs");

    let content = format!(
        r#"// Auto-generated bundled assets
// Do not edit this file directly

/// Bundled and minified CSS for reports
pub const BUNDLED_CSS: &str = include_str!("bundle.min.css");

/// Bundled and minified JavaScript for reports  
pub const BUNDLED_JS: &str = include_str!("bundle.min.js");

/// Asset manifest with metadata
pub const ASSET_MANIFEST: &str = "{{
    \"css_size\": {},
    \"js_size\": {},
    \"fontawesome_icons\": \"placeholder\"
}}";
"#,
        css.len(),
        js.len()
    );

    fs::write(dest_path, content)?;
    Ok(())
}

/// Generate feature flag constants
fn generate_feature_flags(out_path: &Path) -> Result<(), Box<dyn std::error::Error>> {
    let dest_path = out_path.join("feature_flags").with_extension("rs");

    let mut content = String::new();
    content.push_str("// Auto-generated feature flags\n");
    content.push_str("// Do not edit this file directly\n");
    content.push('\n');
    content.push_str("/// Feature flag for modern template system\n");
    content.push_str("pub const USE_MODERN_TEMPLATES: bool = true;\n");
    content.push('\n');
    content.push_str("/// Feature flag for native diagram rendering\n");
    content.push_str("pub const USE_NATIVE_DIAGRAMS: bool = false; // Will be implemented later\n");
    content.push('\n');
    content.push_str("/// Feature flag for asset bundling\n");
    content.push_str("pub const USE_BUNDLED_ASSETS: bool = true;\n");

    fs::write(dest_path, content)?;
    Ok(())
}
