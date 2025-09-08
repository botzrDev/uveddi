# Documentation Style Guide

> **Purpose**: Establish consistent voice, tone, and formatting standards for all Uveddi documentation to build trust, ensure clarity, and strengthen our privacy-focused developer community.

## Voice & Brand Identity

### Core Voice Characteristics

**Technical-friendly**: We respect our audience's expertise while remaining accessible to newcomers.
- Use precise technical language when necessary
- Define technical terms on first use
- Prefer concrete examples over abstract concepts
- Balance depth with readability

**Privacy-conscious**: We lead with privacy as a core value, not an afterthought.
- Emphasize local-first architecture in features
- Highlight data ownership and control
- Address privacy concerns proactively
- Use "privacy-preserving" over "secure" when applicable

**Developer-empathetic**: We understand real-world development challenges and constraints.
- Acknowledge time pressures and technical debt
- Provide practical, actionable guidance
- Show respect for existing codebases and workflows
- Focus on solving actual problems, not theoretical ones

### Voice Examples

✅ **Good**: "Uveddi analyzes your code locally, ensuring your intellectual property never leaves your machine."

❌ **Avoid**: "Our secure platform protects your data."

✅ **Good**: "Running `uveddi analyze` on a large codebase? Use `--parallel 4` to speed up analysis while keeping memory usage reasonable."

❌ **Avoid**: "Configure parallel processing for optimal performance."

## Tone Variations by Content Type

### Getting Started / Tutorials
**Tone**: Encouraging, step-by-step, confidence-building
- Use "you" to directly address the reader
- Break complex tasks into small, achievable steps
- Include expected outcomes and troubleshooting
- Celebrate progress ("Great! You've successfully...")

### Technical Reference
**Tone**: Precise, authoritative, comprehensive
- Use imperative voice for actions
- Include all necessary parameters and options
- Provide complete examples
- Link to related concepts

### Architecture / Concepts
**Tone**: Educational, thoughtful, big-picture focused
- Use "we" when discussing design decisions
- Explain the "why" behind technical choices
- Connect concepts to real-world scenarios
- Include tradeoffs and alternatives

### Troubleshooting / FAQ
**Tone**: Solution-oriented, empathetic, practical
- Acknowledge frustration without dwelling on it
- Provide specific, testable solutions
- Include common variations of problems
- Link to prevention strategies

## Grammar & Language Guidelines

### Writing Style

**Active Voice**: Prefer active over passive construction.
- ✅ "Uveddi detects architectural anti-patterns"
- ❌ "Architectural anti-patterns are detected by Uveddi"

**Present Tense**: Use present tense for current capabilities.
- ✅ "The analysis engine processes your code"
- ❌ "The analysis engine will process your code"

**Sentence Length**: Aim for 15-20 words per sentence. Break complex ideas into multiple sentences.

**Paragraph Structure**: One main idea per paragraph. Start with the most important information.

### Technical Terminology

**Consistency**: Use the same term throughout a document for the same concept.
- Code analysis (not "code scanning" or "code review")
- Anti-pattern (not "antipattern" or "anti pattern")
- Privacy-first (not "privacy-focused" or "privacy-centric")

**Capitalization**: 
- Uveddi (always capitalized)
- CLI tool (not "cli tool")
- AI-powered (hyphenated when used as adjective)

**Abbreviations**: Spell out on first use, then use abbreviation.
- "Command Line Interface (CLI)" then "CLI"
- "Application Programming Interface (API)" then "API"

## Inclusive Language

### Recommended Terms

| Instead of | Use |
|------------|-----|
| guys, folks | everyone, team, developers |
| master/slave | primary/secondary, leader/follower |
| whitelist/blacklist | allowlist/denylist |
| sanity check | consistency check, validation |
| dummy data | sample data, placeholder data |

### Accessibility Considerations

**Alt Text**: All images require descriptive alt text.
**Link Text**: Use descriptive link text, not "click here" or "read more".
**Code Examples**: Include explanatory text before and after code blocks.

## Formatting Standards

### Headers

Use sentence case for headers:
- ✅ "Getting started with Uveddi"
- ❌ "Getting Started With Uveddi"

### Code Formatting

**Inline Code**: Use backticks for commands, file names, and short code snippets.
- Run `uveddi --help` to see all options
- Edit the `uveddi.toml` configuration file

**Code Blocks**: Include language identifier for syntax highlighting.

```rust
// Use descriptive comments
let config = Config::load("uveddi.toml")?;
```

**Command Examples**: Show both command and expected output.

```bash
$ uveddi analyze src/
✓ Found 12 files to analyze
✓ Detected 3 architectural issues
Analysis complete in 2.1s
```

### Lists

**Parallel Structure**: Keep list items grammatically consistent.
- ✅ All items start with verbs: "Install Uveddi", "Configure settings", "Run analysis"
- ❌ Mixed structure: "Install Uveddi", "Configuration of settings", "Running the analysis"

**Ordering**: Use logical order (chronological, importance, alphabetical).

### Links and References

**External Links**: Open in same tab unless specifically noted.
**Internal Links**: Use relative paths from document root.
**Reference Style**: Use descriptive text, not bare URLs.

## Content Organization

### Page Structure

1. **Purpose Statement**: One sentence explaining the page's goal
2. **Prerequisites**: What readers need before starting
3. **Main Content**: Organized with clear headers
4. **Next Steps**: Where to go after completing this page

### Documentation Types

**Tutorials**: Step-by-step learning experiences
- Focus on successful completion
- Include verification steps
- Provide context for each action

**How-to Guides**: Task-oriented solutions
- Assume basic knowledge
- Focus on specific problems
- Include troubleshooting

**Reference**: Comprehensive information
- Organized for lookup, not reading
- Include all options and parameters
- Cross-reference related concepts

**Explanations**: Understanding-oriented background
- Provide context and rationale
- Connect to bigger picture
- Include examples and analogies

## Quality Checklist

Before publishing documentation:

- [ ] Voice aligns with brand characteristics
- [ ] Tone appropriate for content type
- [ ] Grammar follows style guidelines
- [ ] Inclusive language used throughout
- [ ] Code examples tested and working
- [ ] Links functional and descriptive
- [ ] Headers use sentence case
- [ ] One main idea per paragraph
- [ ] Technical terms defined on first use
- [ ] Privacy-first messaging reinforced

## Examples in Practice

### Feature Announcement

**Template Structure**:
1. What's new (headline benefit)
2. Why it matters (developer pain point)
3. How it works (technical overview)
4. Privacy implications (always included)
5. Getting started (actionable next steps)

### Error Message Documentation

**Template Structure**:
1. Error description (what happened)
2. Common causes (why it happens)
3. Solution steps (how to fix)
4. Prevention (how to avoid)

### API Documentation

**Template Structure**:
1. Purpose (what this endpoint does)
2. Parameters (what inputs it accepts)
3. Response (what outputs it returns)
4. Example (working code sample)
5. Error handling (what can go wrong)

---

**Remember**: This style guide is a living document. Update it as we learn what works best for our community. When in doubt, prioritize clarity and respect for our users' time and expertise.