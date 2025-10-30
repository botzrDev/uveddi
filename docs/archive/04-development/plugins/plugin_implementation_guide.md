# Uveddi Plugin System Implementation Guide (Option 1 - Uuid-based)

## Implementation Steps

1. **Verify Existing Types**:
```rust
// In src/plugins/types.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(Uuid);  // Confirm this matches existing implementation

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Ready,
    Unloaded,
    // Add any other existing variants
}
```

2. **Update Engine Implementation**:
```rust
// In src/plugins/engine.rs
pub struct WasmPluginEngine {
    plugin_adapters: Arc<RwLock<HashMap<PluginId, WasmPluginAdapter>>>,
    // Maintain existing fields but ensure they use PluginId
}
```

3. **Update Registry Implementation**:
```rust
// In src/plugins/registry.rs
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginMetadata>,
    // Maintain existing functionality but ensure PluginId usage
}
```

4. **Add Conversion Traits**:
```rust
// In src/plugins/types.rs
impl From<Uuid> for PluginId {
    fn from(uuid: Uuid) -> Self {
        Self(uuid)
    }
}

impl From<PluginId> for Uuid {
    fn from(id: PluginId) -> Self {
        id.0
    }
}
```

5. **Add Utility Methods**:
```rust
impl PluginId {
    pub fn new() -> Self {
        Self(Uuid::new_v4())
    }
    
    pub fn as_uuid(&self) -> &Uuid {
        &self.0
    }
}
```

## Testing Requirements

1. **Unit Tests for PluginId**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_plugin_id_generation() {
        let id1 = PluginId::new();
        let id2 = PluginId::new();
        assert_ne!(id1, id2);
    }

    #[test]
    fn test_plugin_id_conversions() {
        let uuid = Uuid::new_v4();
        let id = PluginId::from(uuid);
        assert_eq!(Uuid::from(id), uuid);
    }
}
```

2. **PluginStatus Tests**:
```rust
#[test]
fn test_plugin_status_serialization() {
    let status = PluginStatus::Ready;
    let serialized = serde_json::to_string(&status).unwrap();
    let deserialized: PluginStatus = serde_json::from_str(&serialized).unwrap();
    assert_eq!(status, deserialized);
}
```

## Migration Checklist

1. Verify all plugin-related code uses `PluginId` instead of raw `Uuid`
2. Ensure serialization/deserialization tests pass
3. Update any documentation referencing plugin identifiers
4. Confirm no performance regression in plugin lookup operations

## Verification Steps

1. Run plugin tests:
```bash
cargo test -p uveddi --lib plugins
```

2. Check for compilation errors:
```bash
cargo check --all-targets
```

3. Verify integration with existing plugins
