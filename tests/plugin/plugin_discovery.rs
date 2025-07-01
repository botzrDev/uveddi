//! tests/plugin/plugin_discovery.rs

use uveddi::plugin::discover_plugins;

#[test]
fn test_plugin_discovery() {
    let plugins = discover_plugins();
    assert!(plugins.is_empty());
}