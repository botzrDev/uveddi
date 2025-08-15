# 🚨 IMMEDIATE ACTION REQUIRED: Tree-Sitter Panic Fix

## Quick Context
Uveddi analysis tool crashes with: `range end index 445 out of range for slice of length 369` in tree-sitter UTF-8 extraction.

## GPT Developer Task

### Problem
```bash
cargo run --bin uveddi -- analyze test-project
# CRASHES: tree_sitter::Node::utf8_text buffer overflow
```

### Your Job
Fix the unsafe `node.utf8_text()` calls causing buffer overflows in the CFG builder.

### Key Files
1. `src/analysis/cfg.rs` - **PRIMARY TARGET** (CfgBuilder::build_from_ast)
2. `src/analysis/detectors/anti_patterns/code_duplication.rs` - Triggers the crash
3. Any file calling `node.utf8_text(source)` without bounds checking

### The Fix Pattern
```rust
// BEFORE (crashes):
let text = node.utf8_text(source_bytes)?;

// AFTER (safe):
if node.end_byte() > source_bytes.len() {
    return Err(AnalysisError::InvalidNodeRange);
}
let text = node.utf8_text(source_bytes)?;
```

### Success Test
```bash
cd /home/austingreen/Documents/botzr/projects/uveddi
cargo run --bin uveddi -- analyze test-project
# Should complete without panic
```

### Requirements
- ✅ No more panics on tree-sitter operations
- ✅ Preserve all existing functionality  
- ✅ Add proper error handling
- ✅ Analysis pipeline completes successfully

### Time Estimate
**30-60 minutes** - This is a targeted bounds-checking fix, not a major refactor.

---
**Status:** Ready for immediate GPT developer pickup  
**Priority:** P0 - Blocking all analysis functionality
