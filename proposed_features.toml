# Proposed simplified feature structure for uveddi
# This is a design artifact — not yet applied to Cargo.toml.

[features]
default = ["standard"]

# Profiles
minimal = ["dev-ultra-minimal"]                 # smallest dependency surface (rusqlite, bincode)
standard = ["minimal", "tree-sitter", "prometheus"]
full = ["standard", "security", "memory-optimization", "tui", "wasm-plugins", "web"]

# Language packs (aggregate over leaf language flags)
languages-core = ["rust-lang", "python-lang"]
languages-web = ["javascript-lang", "typescript-lang"]
languages-all = ["languages-core", "languages-web"]

# Integrations and capabilities
ai-integration = ["ai", "local-ai"]
web = ["web-full"]

# Development and testing
dev-tools = ["chaos", "regression-detection"]
testing = ["sla-monitoring"]

# Notes
# - Replace dev-* and production* usages in code/docs with profiles above
# - Prefer language packs over dev-*-only variants (remove dev-rust-only, dev-python-only, dev-js-only, dev-ts-only)
# - Collapse web-client/server/full to a single "web" capability unless a split is truly required
# - Keep leaf toggles as-is where code needs gating: security, memory-optimization, wasm-plugins, ai, local-ai, image-rendering, tui, prometheus, rust-lang, python-lang, javascript-lang, typescript-lang

