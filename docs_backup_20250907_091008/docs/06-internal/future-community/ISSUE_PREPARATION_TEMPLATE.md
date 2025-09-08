# Issue Preparation Template for Beginner-Friendly Tasks

Use this template to prepare existing issues for new contributors. This ensures consistent quality and reduces barriers to entry.

## Issue Enhancement Checklist

### 📋 Basic Information
- [ ] **Clear Title**: Descriptive and action-oriented (use `[Good First Issue]` prefix)
- [ ] **Detailed Description**: What needs to be done and why it matters
- [ ] **Acceptance Criteria**: Specific, measurable outcomes with checkboxes
- [ ] **Files to Modify**: Exact file paths and line numbers where possible

### 🎯 Beginner-Friendly Enhancements
- [ ] **Background Context**: Why this task is important for the project
- [ ] **Step-by-Step Guide**: Detailed implementation steps with code examples
- [ ] **Expected Challenges**: Common pitfalls and how to avoid them
- [ ] **Learning Resources**: Links to relevant documentation and tutorials

### 🛠 Technical Details
- [ ] **Prerequisites**: Required tools, knowledge, and setup steps
- [ ] **Environment Setup**: Specific development environment requirements
- [ ] **Testing Instructions**: How to verify the solution works correctly
- [ ] **Code Examples**: Sample code patterns or similar implementations

### 🤝 Support Structure
- [ ] **Mentor Assignment**: Designated helper for questions and guidance
- [ ] **Related Issues**: Links to similar completed tasks for reference
- [ ] **Community Resources**: Where to get help (Discord, GitHub Discussions)
- [ ] **Review Process**: What to expect during code review and merge

## Template Application Example

### Before (Typical Issue)
```markdown
Title: Add CSV output
Description: We need CSV output for reports.
```

### After (Beginner-Ready Issue)
```markdown
Title: [Good First Issue] Add CSV output format for analysis reports

Description:
## 📋 What You'll Build
Add CSV export functionality to allow users to open analysis results in Excel/Google Sheets.

## 🎯 Acceptance Criteria
- [ ] Add `--format csv` option to CLI command
- [ ] Generate CSV with columns: file, issue_type, severity, line_number, description
- [ ] Include header row with column names
- [ ] Add unit tests for CSV generation
- [ ] Update help text to mention CSV option
- [ ] Update documentation with CSV examples

## 🛠 Implementation Guide

### Step 1: Add CLI Option
**File**: `src/cli/analyze_command.rs` (around line 45)
```rust
#[arg(long, value_enum, default_value_t = OutputFormat::Json)]
pub format: OutputFormat,
```

Add `Csv` variant to the `OutputFormat` enum.

### Step 2: Create CSV Formatter
**New File**: `src/report/csv_formatter.rs`
```rust
use csv::Writer;
use crate::analysis::AnalysisResult;

pub fn format_csv(results: &AnalysisResult) -> Result<String, Box<dyn Error>> {
    let mut wtr = Writer::from_writer(vec![]);
    
    // Write header
    wtr.write_record(&["file", "issue_type", "severity", "line_number", "description"])?;
    
    // Write data rows
    for issue in &results.issues {
        wtr.write_record(&[
            &issue.file_path,
            &issue.issue_type.to_string(),
            &issue.severity.to_string(),
            &issue.line_number.to_string(),
            &issue.description,
        ])?;
    }
    
    Ok(String::from_utf8(wtr.into_inner()?)?)
}
```

### Step 3: Integrate Formatter
**File**: `src/report/mod.rs`
Add the CSV formatter to the main report generation logic.

### Step 4: Add Tests
**New File**: `tests/report/csv_tests.rs`
Test the CSV output format with sample data.

## 📚 Resources
- [CSV crate documentation](https://docs.rs/csv/)
- [Similar JSON implementation](src/report/json_formatter.rs)
- [CLI argument examples](src/cli/analyze_command.rs)
- [Rust enums guide](https://doc.rust-lang.org/book/ch06-01-defining-an-enum.html)

## 🤝 Getting Help
**Mentor**: @username - Available for questions (responds within 24 hours)
**Estimated Time**: 2-3 hours
**Skills**: Basic Rust, CSV format understanding
**Complexity Score**: 2.0/5 (Beginner-friendly)

## 💡 Expected Challenges
1. **CSV Library Usage**: The `csv` crate has specific patterns for writing data
   - **Solution**: Follow the examples in the documentation and existing JSON formatter
2. **Error Handling**: Proper error propagation from CSV operations
   - **Solution**: Use `?` operator and `Box<dyn Error>` for simple error handling
3. **Testing**: Creating appropriate test cases for CSV output
   - **Solution**: Look at existing JSON tests for patterns

## 🔗 Related Work
- See #123: JSON output implementation (similar pattern)
- See #456: XML output discussion (future enhancement)
- See docs/08-examples/basic-example.md for output format examples

## ✅ Definition of Done
- [ ] Code compiles without warnings
- [ ] All tests pass (including new CSV tests)
- [ ] Documentation updated with CSV examples
- [ ] CLI help text includes CSV option
- [ ] Mentor approval on implementation approach
- [ ] Code review completed and approved
```

## Quick Application Workflow

### For Documentation Tasks
1. **Add context** about why the documentation matters
2. **Provide examples** of good documentation from similar projects
3. **Specify formatting standards** (Markdown, style guides)
4. **Include review checklist** for grammar, clarity, and completeness

### For Testing Tasks
1. **Explain testing philosophy** and why tests matter
2. **Provide test examples** and patterns from existing codebase
3. **Specify coverage expectations** and edge cases to consider
4. **Include debugging tips** for when tests fail

### For Frontend Tasks
1. **Provide design mockups** or describe expected behavior
2. **Specify accessibility requirements** (WCAG guidelines)
3. **Include cross-browser testing** requirements
4. **Link to component library** and design system documentation

### For Backend Tasks
1. **Explain business logic** and system impact
2. **Provide API specifications** or interface requirements
3. **Include performance considerations** and constraints
4. **Specify error handling** and edge case requirements

### For DevOps Tasks
1. **Explain infrastructure context** and deployment pipeline
2. **Provide security requirements** and best practices
3. **Include monitoring and logging** considerations
4. **Specify rollback procedures** and safety measures

## Quality Gates

Before marking an issue as ready for new contributors:

### Technical Completeness
- [ ] **Implementation path is clear** with specific files and functions identified
- [ ] **All dependencies are documented** (libraries, tools, configurations)
- [ ] **Testing strategy is defined** with clear pass/fail criteria
- [ ] **Review criteria are specified** to set clear expectations

### Support Infrastructure
- [ ] **Mentor is assigned** with confirmed availability
- [ ] **Resources are validated** (all links work, documentation is current)
- [ ] **Communication channels are clear** (where to ask questions)
- [ ] **Success metrics are defined** (what constitutes completion)

### Community Readiness
- [ ] **Issue is properly labeled** with difficulty, skills, and time estimates
- [ ] **Prerequisites are minimal** or clearly documented
- [ ] **Onboarding is streamlined** with minimal setup friction
- [ ] **Celebration is planned** for successful completion

## Continuous Improvement

### Gather Feedback
- **Post-completion surveys** to understand contributor experience
- **Mentor feedback** on common questions and blockers
- **Community discussions** about improving the process

### Iterate on Templates
- **Update based on learnings** from completed tasks
- **Add new sections** as patterns emerge
- **Remove friction** where contributors get stuck

### Scale Successfully
- **Create specialized templates** for different task types
- **Automate template application** where possible
- **Train mentors** on effective issue preparation

---

*Use this template to transform any task into a beginner-friendly contribution opportunity. The goal is to eliminate barriers and provide clear pathways to success for new contributors.*