//! tests/plugin/plugin_trait.rs

use codeatlas::plugin::CodeAtlasPlugin;

struct TestPlugin;

impl CodeAtlasPlugin for TestPlugin {
    fn name(&self) -> &'static str {
        "Test Plugin"
    }

    fn on_load(&self) {
        // For testing, we can just print something
        println!("TestPlugin loaded");
    }
}

#[test]
fn test_plugin_trait() {
    let plugin = TestPlugin;
    assert_eq!(plugin.name(), "Test Plugin");
    plugin.on_load(); // Just call it to make sure it doesn't panic
}