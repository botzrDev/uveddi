# Profile Migration Guide

This guide maps legacy dev-/production* profiles to the new simplified profiles.

## Migration Table

| Old Feature         | New Feature | Notes                                                 |
|---------------------|-------------|-------------------------------------------------------|
| dev-ultra-minimal   | minimal     | Adds `rayon` for consistency                          |
| dev-minimal         | minimal     | Direct replacement                                    |
| dev-core            | standard    | Includes parsing (`tree-sitter`) and monitoring       |
| dev-full            | standard    | Direct replacement                                    |
| production          | full        | All capabilities via simplified `full` profile        |
| production-secure   | full        | Configure security posture via settings at runtime    |

## Recommended Usage

- Fastest local builds: `cargo build --features minimal`
- Day-to-day development: `cargo build --features standard`
- Production builds: `cargo build --release --features full`

## Deprecation Timeline

- Legacy profiles remain available for now and are marked deprecated in `Cargo.toml`.
- Documentation and examples have moved to the new profiles.
- Removal of legacy profiles is planned for the next major version.

