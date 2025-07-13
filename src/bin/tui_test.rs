//! Simple test binary for the TUI
//!
//! Run with: cargo run --bin tui_test --features tui

use color_eyre::Result;
use uveddi::tui::events::run_tui;

fn main() -> Result<()> {
    // Initialize color_eyre for better error reporting
    color_eyre::install()?;
    
    // Initialize logging
    env_logger::init();
    
    // Run the TUI
    run_tui()?;
    
    println!("Thanks for using Uveddi TUI!");
    Ok(())
}