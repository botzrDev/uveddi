# Feature Consolidation Plan

Goal: Reduce exponential build/configuration complexity by consolidating 42 features into a smaller, coherent set of profiles, language packs, and capability toggles. Preserve flexibility while eliminating redundant/unused flags.

## Summary of Changes
- Consolidate dev-* and production* into profile features: minimal, standard, full
- Replace language-specific dev-* variants with language packs: languages-core, languages-web, languages-all
- Collapse web-client/server/full into a single web capability
- Keep essential capability toggles: security, memory-optimization, wasm-plugins, ai/local-ai, image-rendering, prometheus, tui
- Remove legacy/convenience features: alpha, zero-cost, full-featured, enterprise
- Remove internal/empty/unused cfg flags: streaming-iterator, tower, tower-http, graphql-api, yaml, reqwest

Estimated reduction: 42 -> ~22 effective features (profiles + packs + core capabilities + leaf language toggles)

## Migration Strategy (Phased)
1) Phase 1 – Eliminate Unused/Redundant
   - Remove convenience aliases: alpha, zero-cost, full-featured, enterprise
   - Remove dev-* language-only profiles: dev-rust-only, dev-python-only, dev-js-only, dev-ts-only
   - Remove internal empty flags not used in cfg: streaming-iterator, tower, tower-http, graphql-api, yaml, reqwest
   - Keep a deprecation alias map in docs for 1 release cycle

2) Phase 2 – Introduce Profiles and Packs
   - Add profiles: minimal, standard, full
   - Define: languages-core (rust, python), languages-web (js, ts), languages-all
   - Create ai-integration (ai, local-ai), web (web-full)
   - Document mappings from old -> new (e.g., dev-core ≈ standard; dev-full ≈ full; production ≈ full + security + web)

3) Phase 3 – Update Code Gating
   - Replace uses of #[cfg(feature = "dev-*")] and convenience flags with profile names
   - Ensure code references leaf features (security, tui, wasm-plugins, memory-optimization, ai, image-rendering) as needed
   - Collapse web gating to #[cfg(feature = "web")] unless client/server split is truly required

4) Phase 4 – CI/CD + Docs
   - Reduce test matrix to: minimal, standard, full (+ languages-all), plus toggles [security, wasm-plugins, memory-optimization, ai-integration, web, tui]
   - Update docs and examples to use profiles and language packs
   - Provide migration guide and quick mapping table

## Old → New Mapping (Guidance)
- dev-ultra-minimal → minimal
- dev-minimal → minimal
- dev-core → standard
- dev-full → full
- production, production-secure → full (+ security) [handle RSA-sensitive choices at dependency config level]
- dev-*-only → profiles + individual language features or language packs
- web-client/server/full → web
- tree-sitter (aggregate) → languages-all
- zero-cost / full-featured / enterprise / alpha → remove; document as profile aliases in docs if needed

## Build and Test Matrix After Consolidation
- Profiles: minimal, standard, full
- Language packs: languages-core, languages-web, languages-all
- Toggles: security, memory-optimization, wasm-plugins, ai-integration, web, tui, prometheus, image-rendering
- Example CI combinations:
  - minimal
  - standard + languages-core
  - full + languages-all
  - full + languages-all + security + web + wasm-plugins
  - standard + ai-integration

## Impact Assessment
- Breaking changes: feature names deprecated/removed; require mapping
- CI/CD: reduced matrix; faster builds; clearer profiles
- Documentation: update README, release notes, and configuration reference
- Testing: matrix reduction and clearer coverage boundaries

## Next Actions
- Land the proposed structure (see proposed_features.toml)
- Replace old feature references in code/tests
- Update docs + CI, announce deprecations

