# 🔧 Uveddi Plugin System Type Fixes

## Current Errors
The project has compilation errors due to missing `PluginId` and `PluginStatus` types in the plugins module. These types are referenced throughout the plugin engine and registry but are not defined.

## Required Fixes

### 1. Define PluginId Type
```rust
// In src/plugins/types.rs
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PluginId(String);

impl PluginId {
    pub fn new(id: impl Into<String>) -> Self {
        Self(id.into())
    }
    
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl Display for PluginId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}
```

### 2. Define PluginStatus Type
```rust
// In src/plugins/types.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PluginStatus {
    Active,
    Inactive,
    Error(String),
    Loading,
    Unloading,
}
```

### 3. Update PluginEngine
```rust
// In src/plugins/engine.rs
pub struct WasmPluginEngine {
    plugin_adapters: Arc<RwLock<HashMap<PluginId, WasmPluginAdapter>>>,
    // ... rest of implementation
}
```

### 4. Update PluginRegistry
```rust
// In src/plugins/registry.rs
pub struct PluginRegistry {
    plugins: HashMap<PluginId, PluginMetadata>,
    // ... rest of implementation
}
```

### 5. Add Imports
Ensure all files have proper imports for these types:
```rust
use crate::plugins::types::{PluginId, PluginStatus};
```

## Implementation Steps

1. Create/modify `src/plugins/types.rs` with the type definitions
2. Update all plugin engine and registry files to use these types
3. Add proper trait derives (Clone, Debug, etc.) as needed
4. Update any serialization/deserialization code
5. Add conversion traits if needed (From/Into for String)

## Testing Requirements

1. Unit tests for PluginId creation and display
2. Tests for PluginStatus serialization
3. Integration tests for plugin lifecycle with these types
4. Benchmark any performance impact

## Verification

After implementing:
1. Run `cargo check --all-targets`
2. Run plugin-specific tests: `cargo test -p uveddi --lib plugins`
3. Check serialization round-trips
4. Verify no performance regression in plugin operations

## Additional Considerations

- Thread safety for PluginId usage
- Serialization compatibility with existing plugin manifests
- Proper error handling for invalid PluginId formats
- Documentation for public types
