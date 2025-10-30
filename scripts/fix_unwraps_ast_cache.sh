#!/bin/bash
# UV-276: Fix unwraps in AST cache file

file="src/analysis/cache/ast.rs"

echo "🔧 Fixing unwraps in $file"

# Replace cache read unwraps
sed -i 's/self\.cache\.read()\.unwrap()/self.safe_cache_read().map_err(|_| { error!("Cache read lock failed"); return None; }).unwrap_or_else(|_| return None)/g' "$file"

# Replace cache write unwraps
sed -i 's/self\.cache\.write()\.unwrap()/self.safe_cache_write().map_err(|_| { error!("Cache write lock failed"); return None; }).unwrap_or_else(|_| return None)/g' "$file"

# Replace metrics lock unwraps
sed -i 's/self\.metrics\.lock()\.unwrap()/match self.safe_metrics_lock() { Ok(guard) => guard, Err(_) => { error!("Metrics lock failed"); return; } }/g' "$file"

# Replace memory usage lock unwraps
sed -i 's/self\.memory_usage\.lock()\.unwrap()/match self.safe_memory_usage_lock() { Ok(guard) => guard, Err(_) => { error!("Memory usage lock failed"); return; } }/g' "$file"

# Replace LRU order lock unwraps
sed -i 's/self\.lru_order\.lock()\.unwrap()/match self.safe_lru_order_lock() { Ok(guard) => guard, Err(_) => { error!("LRU order lock failed"); return; } }/g' "$file"

echo "✅ Completed unwrap fixes in $file"