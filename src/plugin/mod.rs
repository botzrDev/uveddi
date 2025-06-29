/// Plugin trait for CodeAtlas (Sprint 3 foundation)
pub trait CodeAtlasPlugin {
    /// Name of the plugin
    fn name(&self) -> &'static str;
    /// Called when the plugin is loaded
    fn on_load(&self);
}

/// Simple plugin discovery (stub)
pub fn discover_plugins() -> Vec<Box<dyn CodeAtlasPlugin>> {
    // In Sprint 3, this is a stub. Sprint 4 will support WASM plugins.
    vec![]
}
