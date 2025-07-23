# Pull Request Template

## Summary
Brief description of the changes in this PR.

## Type of Change
- [ ] Bug fix (non-breaking change which fixes an issue)
- [ ] New feature (non-breaking change which adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Documentation update
- [ ] Refactoring (no functional changes)
- [ ] Performance improvement
- [ ] Test addition or update

## Related Issues
Closes #[issue_number]

## Changes Made
- 
- 
- 

## Testing
- [ ] Tests added/updated for new functionality
- [ ] All existing tests pass
- [ ] Manual testing completed

## Ubiquitous Language & Terminology Review

### Naming Conventions ✅
- [ ] All new types use PascalCase matching [glossary terms](docs/09-community/glossary.md)
- [ ] Function names follow snake_case with domain-appropriate verbs
- [ ] Module names use snake_case reflecting domain boundaries
- [ ] Constants use SCREAMING_SNAKE_CASE with domain prefixes

### Anti-Pattern Terminology 📋
- [ ] Code structures use `anti_pattern` (snake_case)
- [ ] Type names use `AntiPattern` prefix (PascalCase) 
- [ ] Documentation uses "anti-pattern" (hyphenated)
- [ ] No inconsistent variants (`antipattern`, `anti.pattern`) used

### Component Architecture 🏗️
- [ ] "Engine" reserved for core orchestrators only
- [ ] "Service" used for internal business logic
- [ ] "Provider" used for external integrations  
- [ ] "Manager" used for resource management
- [ ] "Builder" used for object construction
- [ ] "Detector" used for issue identification components

### Spelling & Language Consistency 🔤
- [ ] US spelling used consistently (`analyze`/`analysis`/`analyzer`)
- [ ] No UK spellings (`analyse`/`analyser`) present
- [ ] "Configuration" spelled out in documentation, `Config` suffix in code
- [ ] Technical abbreviations match [glossary standards](docs/09-community/glossary.md#common-abbreviations)

### Documentation Standards 📚
- [ ] All public APIs documented with standardized terminology
- [ ] Code comments align with ubiquitous language definitions
- [ ] Error messages use consistent terminology
- [ ] Examples demonstrate proper terminology usage
- [ ] Links to glossary terms where appropriate

### Domain Alignment 🎯
- [ ] New code aligns with existing domain boundaries
- [ ] No terminology overloading or concept conflicts introduced
- [ ] Cross-domain interactions use consistent interface terminology
- [ ] Plugin/extension terminology follows established patterns

## Architecture Review

### Design Principles
- [ ] Follows Single Responsibility Principle
- [ ] Maintains separation of concerns
- [ ] Adheres to existing architectural patterns
- [ ] No architectural anti-patterns introduced

### Integration Points
- [ ] Maintains clean interfaces between domains
- [ ] No tight coupling introduced
- [ ] Follows dependency injection patterns where applicable
- [ ] Plugin architecture respected if applicable

## Security Considerations
- [ ] No security vulnerabilities introduced
- [ ] Authentication/authorization patterns followed
- [ ] Input validation implemented where necessary
- [ ] No secrets or sensitive data exposed

## Performance Impact
- [ ] No significant performance regressions
- [ ] Memory usage considerations addressed
- [ ] Caching strategies implemented if beneficial
- [ ] Async patterns used appropriately

## Breaking Changes
If this PR introduces breaking changes, please describe:
- What breaks
- Migration path for users
- Documentation updates needed

## Additional Notes
Any additional information that reviewers should know.

---

### For Reviewers

#### Terminology Review Checklist
Please verify the following during review:

**Code Structure Review:**
- [ ] All new structs/enums follow PascalCase domain naming
- [ ] Function signatures use appropriate domain vocabulary
- [ ] Variable names reflect business concepts clearly
- [ ] No ambiguous or overloaded terminology

**Documentation Review:**
- [ ] Public APIs documented with glossary-consistent terms
- [ ] Code comments use standardized vocabulary
- [ ] Examples follow terminology guidelines
- [ ] Error messages provide clear, consistent language

**Architectural Consistency:**
- [ ] Component responsibilities clearly defined using domain language
- [ ] Interface boundaries respect terminology standards  
- [ ] No violation of established naming patterns
- [ ] Domain concepts properly encapsulated

#### Quick Terminology Reference
- **Anti-Pattern**: `anti_pattern` (code), `AntiPattern` (types), "anti-pattern" (docs)
- **Analysis**: `analyze` (verb), `analysis` (noun), `analyzer` (agent)
- **Components**: Engine (orchestrators), Service (business logic), Provider (external), Manager (resources), Builder (construction)
- **Configuration**: `Config` (code suffix), "configuration" (documentation)

#### Common Issues to Check
- [ ] No `antipattern` (single word) usage
- [ ] No UK spellings (`analyse`, `analyser`) 
- [ ] No `WasmPluginEngine` (should be `WasmPluginRuntime`)
- [ ] No abbreviated "config" in user-facing documentation
- [ ] No overuse of "Engine" suffix for non-orchestrators

#### Documentation Links
- [Ubiquitous Language Glossary](docs/09-community/glossary.md)
- [Terminology Mapping](docs/05-development/terminology-mapping.md) 
- [Enforcement Process](docs/05-development/ubiquitous-language-enforcement.md)