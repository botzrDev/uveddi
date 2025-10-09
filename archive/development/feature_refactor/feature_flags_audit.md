# Current Feature Flag Analysis

Date: 2025-09-21

This document summarizes the current state of the feature flag system in this repository and provides data to inform consolidation.

## Statistics
- Total feature flags: 42
- Most complex feature (by direct feature deps): enterprise (8); production (7)
- Least used features: multiple with zero cfg usage (see list below)
- Features with no conditional compilation: 23

## Dependency Complexity
- Maximum dependency depth: 3
- Circular dependencies: 0 detected
- Mutual exclusions (implicit): development profiles (dev-*) vs production/prod-secure should be mutually exclusive; not encoded in Cargo.toml today

## Usage Distribution (by Rust files gated with cfg(feature))
- Features used in >50% of modules: none
- Features used in <5% of modules: chaos, graphql-api, javascript-lang, local-ai, mimalloc, python-lang, regression-detection, reqwest, rust-lang, tui, typescript-lang, yaml
- Never-used in cfg(feature) (23):
  - alpha, crypto-full, crypto-minimal, dev-core, dev-full, dev-js-only, dev-minimal, dev-python-only, dev-rust-only, dev-ts-only, dev-ultra-minimal, enterprise, full-featured, production, production-secure, sla-monitoring, streaming-iterator, tower, tower-http, web-client, web-full, web-server, zero-cost

Raw usage matrix: feature_usage_matrix.csv

## Inventory
- Full [features] block snapshot: feature_flags_current.txt
- Parsed list: features_list.txt
- Dependencies (normalized): feature_dependencies.csv

## Feature Categories
- Core Profiles: dev-ultra-minimal, dev-minimal, dev-core, dev-full, production, production-secure
- Language Support: rust-lang, python-lang, javascript-lang, typescript-lang, tree-sitter (aggregate)
- Integrations: wasm-plugins, web-client, web-server, web-full, prometheus, security, ai, local-ai, image-rendering
- Optimization: memory-optimization (mimalloc, bumpalo, memmap2, rkyv), mimalloc (internal)
- Development/Testing: chaos, regression-detection, sla-monitoring
- Convenience/Legacy: zero-cost, full-featured, enterprise, alpha
- Internal flags (empty arrays): streaming-iterator, tower, tower-http, mimalloc, graphql-api, yaml, reqwest

## Notable Dependencies (from Cargo.toml)
- dev-core: depends on dev-minimal; enables tokio-stream, async-stream, prometheus
- production: tree-sitter, security, memory-optimization, web-full, tui, wasm-plugins, prometheus (+ dep crates)
- production-secure: like production but without ‘security’ feature; retains most deps (+ dep crates)
- tree-sitter: aggregate for rust/python/javascript/typescript
- security: crypto-full, web-full, casbin, oauth2, openidconnect, jsonwebtoken, vaultrs, config
- memory-optimization: mimalloc, bumpalo(+herd), memmap2, rkyv
- wasm-plugins: wasmtime(+wasi), cap-std, wit-bindgen

## Observations
- Many profile/convenience features are not used in cfg(feature) and only wire dependencies; this inflates the feature surface without modularizing code.
- Four dev-* language-only variants duplicate the purpose of leaf language features and can be removed in favor of a small set of language pack features.
- web-client/server/full can be collapsed into a single web feature unless separate gating is required in code.
- production vs production-secure differ mainly in dependency selection and could be handled via a security sub-feature or runtime configuration.
- Internal empty flags (streaming-iterator, tower, tower-http, graphql-api, yaml, reqwest) appear unused for cfg and can be eliminated or scoped to crates that need them.

## Recommendations (High Level)
- Replace dev-* and production* profiles with minimal/standard/full profiles.
- Introduce language packs: languages-core (rust, python), languages-web (javascript, typescript), languages-all.
- Keep a small set of optional capability toggles: security, memory-optimization, wasm-plugins, ai/local-ai, image-rendering, tui, prometheus, web.
- Remove legacy/convenience aggregations: alpha, zero-cost, full-featured, enterprise (or redefine enterprise as a profile alias in documentation rather than a feature).

## Files Produced by Audit
- feature_flags_current.txt – raw features block
- features_list.txt – list of feature names
- feature_dependencies.tsv / feature_dependencies.csv – parsed deps
- conditional_compilation.txt – all #[cfg(feature=..)] matches
- feature_imports.txt – surrounding lines where feature gates are used
- feature_usage_matrix.csv – usage counts per feature
- feature_graph_summary.tsv – per-feature depth, deps, usage

