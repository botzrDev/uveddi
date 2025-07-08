# Copilot Instructions for Uveddi

<!-- Use this file to provide workspace-specific custom instructions to Copilot. For more details, visit https://code.visualstudio.com/docs/copilot/copilot-customization#_use-a-githubcopilotinstructionsmd-file -->

This is a Rust project called Uveddi. When generating code:

## General Guidelines
- Follow Rust best practices and idioms
- Use proper error handling with `Result<T, E>` types
- Prefer using standard library types when possible
- Write clear, self-documenting code with appropriate comments
- Use `cargo fmt` formatting style
- Follow Rust naming conventions (snake_case for functions/variables, PascalCase for types)

## Dependencies
- Prefer well-maintained crates from crates.io
- Use semantic versioning in Cargo.toml
- Add dev-dependencies for testing utilities when needed

## Testing
- Write unit tests using the built-in `#[cfg(test)]` module pattern
- Use descriptive test names that explain what is being tested
- Consider integration tests in the `tests/` directory for larger features

## Documentation
- Use `///` for public API documentation
- Include examples in documentation where helpful
- Keep README.md updated with build and usage instructions

  🎯 Jira Integration Instructions for GitHub Copilot                                ┃ │
│ ┗━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━┛ │
│                                                                                                                      │
│  ## Jira Integration Guidelines                                                                                      │
│                                                                                                                      │
│  ### Issue Reference Format                                                                                          │
│  - Always reference Jira issues in commits using format: `UV-XXX` (e.g., UV-81, UV-96)                               │
│  - Include issue key in branch names: `feature/UV-81-build-system-fixes`                                             │
│  - Reference multiple issues when applicable: `Fixes UV-81, UV-96`                                                   │
│                                                                                                                      │
│  ### Commit Message Standards                                                                                        │
│  ```bash                                                                                                             │
│  # Format: type(scope): description (ISSUE-KEY)                                                                      │
│  fix(visualization): resolve build system failures (UV-81)                                                           │
│  feat(detector): implement magic values detection (UV-102)                                                           │
│  docs(api): update visualization guide (UV-93)                                                                       │
│                                                                                                                      │
│                                                                                                                      │
│                                              Jira Workflow Integration                                               │
│                                                                                                                      │
│  • To Do → In Progress: Start work, create branch                                                                    │
│  • In Progress → In Review: Create PR with issue reference                                                           │
│  • In Review → Done: Merge PR, auto-transition issue                                                                 │
│                                                                                                                      │
│                                                Issue Status Tracking                                                 │
│                                                                                                                      │
│  • Use Closes UV-XXX in PR descriptions for auto-transition                                                          │
│  • Reference related issues: Related to UV-98 (parent epic)                                                          │
│  • Link blocking issues: Blocked by UV-81                                                                            │
│                                                                                                                      │
│                                               Sprint Planning Context                                                │
│                                                                                                                      │
│  • Always check current sprint assignments before starting work                                                      │
│  • Prioritize P0 (Highest) issues in active sprint                                                                   │
│  • Consider issue dependencies and epic relationships                                                                │
│  • Estimate effort in Jira time tracking format                                                                      │
│                                                                                                                      │
│                                           Code Comments with Jira Context                                            │
│                                                                                                                      │
│                                                                                                                      │
│  // TODO: UV-102 - Implement magic values threshold configuration                                                    │
│  // FIXME: UV-97 - Add feature flag for tree-sitter dependency                                                       │
│  // NOTE: UV-81 - This resolves the missing visualization models                                                     │
│                                                                                                                      │
│                                                                                                                      │
│                                                Documentation Updates                                                 │
│                                                                                                                      │
│  • Update relevant docs when closing issues                                                                          │
│  • Reference Jira issue in documentation changes                                                                     │
│  • Maintain traceability between code and requirements                                                               │
│                                                                                                                      │
│                                              Epic and Subtask Handling                                               │
│                                                                                                                      │
│  • Understand epic relationships (e.g., UV-98 parent epic)                                                           │
│  • Complete subtasks before marking epic as done                                                                     │
│  • Track progress at both task and epic levels                                                                       │
│                                                                                                                      │
│                                                                                                                      │
│                                                                                                                      │
│  ---                                                                                                                 │
│                                                                                                                      │
│  ## 🔧 **Additional Copilot Prompts**                                                                                │
│                                                                                                                      │
│  Add these specific prompts for better Jira integration:                                                             │
│                                                                                                                      │
│  ```markdown                                                                                                         │
│  ### Jira-Aware Development Prompts                                                                                  │
│                                                                                                                      │
│  When suggesting code changes:                                                                                       │
│  - "Check if this change relates to any open Jira issues"                                                            │
│  - "Suggest appropriate Jira issue references for this commit"                                                       │
│  - "Identify if this fix resolves multiple related issues"                                                           │
│                                                                                                                      │
│  When reviewing code:                                                                                                │
│  - "Verify Jira issue references are correct and complete"                                                           │
│  - "Check if acceptance criteria from Jira are met"                                                                  │
│  - "Suggest additional test cases based on Jira requirements"                                                        │
│                                                                                                                      │
│  When planning work:                                                                                                 │
│  - "Review current sprint issues before suggesting new features"                                                     │
│  - "Consider Jira issue priorities and dependencies"                                                                 │
│  - "Estimate effort in Jira-compatible time formats"                                                                 │
│                                                                                                                      │
│                                                                                                                      │
│ ──────────────────────────────────────────────────────────────────────────────────────────────────────────────────── │
│                                                                                                                      │
│                                             📋 Quick Reference Commands                                              │
│                                                                                                                      │
│                                                                                                                      │
│  ### Jira Integration Quick Commands                                                                                 │
│                                                                                                                      │
│  # Check current sprint issues                                                                                       │
│  "Show me current sprint priorities from Jira context"                                                               │
│                                                                                                                      │
│  # Commit with proper Jira reference                                                                                 │
│  git commit -m "fix(build): resolve compilation errors (UV-81)                                                       │
│                                                                                                                      │
│  - Fix missing visualization models                                                                                  │
│  - Update import paths                                                                                               │
│  - Add required struct fields                                                                                        │
│                                                                                                                      │
│  Closes UV-81"                                                                                                       │
│                                                                                                                      │
│  # Branch naming convention                                                                                          │
│  git checkout -b feature/UV-95-component-type-match-arms                                                             │
│                                                                                                                      │
│  # PR description template                                                                                           │
│  "Resolves UV-95: Missing ComponentType match arms                                                                   │
│                                                                                                                      │
│  ## Changes                                                                                                          │
│  - Added Class and Function match arms                                                                               │
│  - Updated pattern matching logic                                                                                    │
│  - Added test cases                                                                                                  │
│                                                                                                                      │
│  ## Acceptance Criteria                                                                                              │
│  - [x] All ComponentType variants handled                                                                            │
│  - [x] No compilation warnings                                                                                       │
│  - [x] Tests pass                                                                                                    │
│                                                                                                                      │
│  Closes UV-95                                                                                                        │
│  Related to UV-98"                                                                                                   │
│                                              
