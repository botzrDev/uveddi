# Feature Flags: Full Audit and Consolidation Plan

Date: 2025-09-21

This report provides a complete audit of the current feature flag system and a concrete consolidation plan, including a proposed simplified feature structure and migration steps.

---

## Executive Summary
- Total feature flags: 42
- Maximum dependency depth: 3
- Most complex features (by direct feature deps): enterprise (8), production (7)
- Circular dependencies: 0 detected
- Features never used in cfg(feature) gates: 23
- Heavily gated modules by files: tree-sitter (37), security (12), wasm-plugins (12), ai (10), memory-optimization (8)

Key recommendations:
- Replace dev-* and production* profiles with: minimal, standard, full
- Use language packs: languages-core, languages-web, languages-all
- Collapse web-client/server/full into a single web gate
- Keep essential toggles: security, memory-optimization, wasm-plugins, ai/local-ai, image-rendering, tui, prometheus, rust-lang, python-lang, javascript-lang, typescript-lang
- Remove legacy/convenience and empty internal flags (alpha, zero-cost, full-featured, enterprise, dev-*-only variants, streaming-iterator, tower, tower-http, graphql-api, yaml, reqwest)

---

## Inventory and Data Artifacts
- Raw features block snapshot: feature_flags_current.txt
- Feature list: features_list.txt (count: 42)
- Conditional compilation hits: conditional_compilation.txt (lines: 481)
- Surrounding imports/usages: feature_imports.txt (lines: 774)
- Usage matrix (CSV): feature_usage_matrix.csv
- Parsed dependencies: feature_dependencies.{tsv,csv}
- Graph summary: feature_graph_summary.tsv
- Never-used gates list: features_never_used_in_cfg.txt (count: 23)

---

## Current Feature Structure Highlights (Cargo.toml)
- Profiles:
  - dev-ultra-minimal → rusqlite, bincode
  - dev-minimal → dev-ultra-minimal, rayon
  - dev-core → dev-minimal, tokio-stream, async-stream, prometheus
  - dev-full → dev-core, tree-sitter
  - production / production-secure → full capability aggregates (tree-sitter, security, memory-optimization, web-full, tui, wasm-plugins, prometheus + several deps)
- Language features: rust-lang, python-lang, javascript-lang, typescript-lang, tree-sitter (aggregate)
- Capabilities: security, wasm-plugins, memory-optimization, ai, local-ai, image-rendering, prometheus, tui, web-*
- Convenience: zero-cost, full-featured, enterprise, alpha
- Internal (mostly empty): streaming-iterator, tower, tower-http, mimalloc, graphql-api, yaml, reqwest

---

## Dependency Analysis
- Maximum dependency depth: 3
- Most complex feature (direct feature deps):
  - enterprise: 8 (local-ai, tree-sitter, memory-optimization, image-rendering, tui, wasm-plugins, chaos, sla-monitoring)
  - production: 7 (tree-sitter, security, memory-optimization, web-full, tui, wasm-plugins, prometheus; plus several direct dep crates)
- Circular dependencies: not detected
- Mutual exclusions: Not encoded; logically dev-* and production/prod-secure should be mutually exclusive

---

## Usage Patterns
- Features used in >50% of modules: none
- Features used in <5% of modules (nonzero):
  - chaos, graphql-api, javascript-lang, local-ai, mimalloc, python-lang, regression-detection, reqwest, rust-lang, tui, typescript-lang, yaml
- Never used in cfg(feature) gates (23):
  - alpha, crypto-full, crypto-minimal, dev-core, dev-full, dev-js-only, dev-minimal, dev-python-only, dev-rust-only, dev-ts-only, dev-ultra-minimal, enterprise, full-featured, production, production-secure, sla-monitoring, streaming-iterator, tower, tower-http, web-client, web-full, web-server, zero-cost
- Most referenced gates by files:
  - tree-sitter (37), security (12), wasm-plugins (12), ai (10), memory-optimization (8), image-rendering (6), prometheus (6)

---

## Categorization
- Core Profiles: dev-ultra-minimal, dev-minimal, dev-core, dev-full, production, production-secure
- Language Support: rust-lang, python-lang, javascript-lang, typescript-lang, tree-sitter (aggregate)
- Integrations: wasm-plugins, web-client/server/full, prometheus, security, ai, local-ai, image-rendering
- Optimization: memory-optimization (mimalloc, bumpalo, memmap2, rkyv), mimalloc (leaf)
- Development/Testing: chaos, regression-detection, sla-monitoring
- Convenience/Legacy: zero-cost, full-featured, enterprise, alpha
- Internal/Empty: streaming-iterator, tower, tower-http, mimalloc, graphql-api, yaml, reqwest

---

## Redundancy and Issues Identified
- dev-*-only language variants duplicate language leaves and increase surface area
- production vs production-secure largely differ by dependency selection; policy better encoded via a security sub-feature or runtime config
- web-client/server/full split unused in code; better as a single web capability
- Multiple internal/empty flags unused in cfg and can be removed
- Several convenience aggregations (alpha, zero-cost, full-featured, enterprise) add names without increasing modularity

---

## Consolidation Opportunities
- Merger:
  - web-client, web-server, web-full → web
  - tree-sitter aggregate → languages-all pack
- Elimination:
  - alpha, zero-cost, full-featured, enterprise
  - dev-rust-only, dev-python-only, dev-js-only, dev-ts-only
  - streaming-iterator, tower, tower-http, graphql-api, yaml, reqwest (internal/unused)
- Simplification:
  - Replace dev-*, production* with minimal, standard, full profiles
  - Encode security posture via security toggle, not separate production profiles
- Default inclusion:
  - standard profile includes tree-sitter and prometheus for a useful default dev experience

---

## Proposed Simplified Structure (Design)
```toml
# proposed_features.toml
[features]
default = ["standard"]

# Profiles
minimal = ["dev-ultra-minimal"]                 # smallest dependency surface (rusqlite, bincode)
standard = ["minimal", "tree-sitter", "prometheus"]
full = ["standard", "security", "memory-optimization", "tui", "wasm-plugins", "web"]

# Language packs
languages-core = ["rust-lang", "python-lang"]
languages-web  = ["javascript-lang", "typescript-lang"]
languages-all  = ["languages-core", "languages-web"]

# Integrations and capabilities
ai-integration = ["ai", "local-ai"]
web            = ["web-full"]

# Dev/Testing
dev-tools = ["chaos", "regression-detection"]
testing   = ["sla-monitoring"]
```

Notes:
- Replace dev-* and production* usages in code/docs with the profiles above
- Prefer language packs over dev-*-only variants
- Keep leaf toggles where code needs gating: security, memory-optimization, wasm-plugins, ai, local-ai, image-rendering, tui, prometheus, rust-lang, python-lang, javascript-lang, typescript-lang

---

## Migration Strategy
1) Phase 1 – Eliminate unused/convenience
   - Remove: alpha, zero-cost, full-featured, enterprise
   - Remove dev-*-only variants and internal empty flags
   - Provide deprecation mapping in docs for one release cycle
2) Phase 2 – Introduce profiles and packs
   - Add minimal, standard, full; add languages-core/web/all; add ai-integration; single web gate
   - Document mapping from old → new (e.g., dev-core ≈ standard; dev-full ≈ full; production ≈ full + security + web)
3) Phase 3 – Update code gating
   - Replace #[cfg(feature = "dev-*")] and convenience gates with profiles and leaf toggles
   - Collapse web gating to #[cfg(feature = "web")] unless split is needed
4) Phase 4 – CI/CD + Docs
   - Reduce test matrix to profiles + language packs + a handful of toggles
   - Update README/examples; publish migration guide

---

## Impact Assessment
- Breaking changes: feature names deprecated/removed; provide mapping
- CI/CD: fewer combinations; faster builds; clearer profiles
- Documentation: update configuration reference and examples
- Testing: significant matrix reduction; clearer coverage boundaries

---

## Verification
Commands:
```
wc -l feature_flags_current.txt
grep -c "^[a-zA-Z]" Cargo.toml

cargo check --features minimal
cargo check --features production
cargo check --no-default-features
```

---

## Completion Notes
- Total features found: 42
- Unused features (no cfg gating): 23
- Consolidation opportunities: dev-*-only, convenience profiles, web-* split, internal empty flags
- Proposed feature reduction: 42 → ~22
- Migration complexity (1–10): 6

