/// Plugin trait for Uveddi (Sprint 3 foundation)
pub trait UveddiPlugin {
    /// Name of the plugin
    fn name(&self) -> &'static str;
    /// Called when the plugin is loaded
    fn on_load(&self);
}

/// Simple plugin discovery (stub)
pub fn discover_plugins() -> Vec<Box<dyn UveddiPlugin>> {
    // In Sprint 3, this is a stub. Sprint 4 will support WASM plugins.
    vec![]
}
