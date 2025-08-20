# Uveddi CLI Alpha Testing Guide

This document provides a comprehensive testing plan for the Uveddi CLI in its alpha release. It's designed to systematically evaluate all CLI subcommands, identify bugs, UX issues, and provide structured feedback.

## Table of Contents

1. [Testing Setup](#testing-setup)
2. [Main Help and Version Commands](#main-help-and-version-commands)
3. [Analyze Command](#analyze-command)
4. [Config Command](#config-command)
5. [UI Command](#ui-command)
6. [CI Command](#ci-command)
7. [Issue Reporting Template](#issue-reporting-template)
8. [Summary Template](#summary-template)

## Testing Setup

Before beginning testing, ensure:

1. Uveddi is installed and accessible in your path
2. You have a test project or repository to analyze
3. You have Rust, Python, and JavaScript files available for testing
4. Create a test folder with permissions issues to test error handling

```bash
# Set up a test environment
mkdir -p ~/uveddi_testing/test_projects
mkdir -p ~/uveddi_testing/test_projects/permission_test
chmod 000 ~/uveddi_testing/test_projects/permission_test

# Clone a test repository if needed
git clone https://github.com/example/repo.git ~/uveddi_testing/test_projects/sample_repo
```

## Main Help and Version Commands

Test the basic help and version information:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Basic Help | `uveddi --help` | List all available commands with descriptions | | |
| Version Check | `uveddi --version` | Display current version | | |

## Analyze Command

Test the analyze command with various options:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Basic Analysis | `uveddi analyze ./test_projects/sample_repo` | Complete analysis with results | | |
| Verbose Output | `uveddi analyze ./test_projects/sample_repo --verbose` | Detailed analysis output | | |
| HTML Output | `uveddi analyze ./test_projects/sample_repo --output-format html --output report.html` | Generate HTML report | | |
| JSON Output | `uveddi analyze ./test_projects/sample_repo --output-format json --output report.json` | Generate JSON report | | |
| Language Filter | `uveddi analyze ./test_projects/sample_repo --languages rust,python` | Analyze only rust and python files | | |
| Timing Info | `uveddi analyze ./test_projects/sample_repo --timing` | Include timing information | | |
| Invalid Path | `uveddi analyze ./nonexistent_path` | Proper error message | | |
| Permission Error | `uveddi analyze ~/uveddi_testing/test_projects/permission_test` | Permission error with helpful message | | |

## Config Command

Test the config command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Help | `uveddi config --help` | Show config subcommands | | |
| Show Config | `uveddi config show` | Display current configuration | | |
| Validate Config | `uveddi config validate --file ./config/benchmark-config.toml` | Validate config file | | |
| Set Config Value | `uveddi config set analysis.level=high` | Change config value | | |
| Invalid Config Key | `uveddi config set nonexistent.key=value` | Proper error message | | |
| Reset Config | `uveddi config reset` | Reset to default config | | |

## UI Command

Test the UI command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Help | `uveddi ui --help` | Show UI subcommands | | |
| Launch UI | `uveddi ui` | Launch the UI interface | | |

## CI Command

Test the CI command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Help | `uveddi ci --help` | Show CI subcommands | | |
| Check | `uveddi ci check ./test_projects/sample_repo --debt-threshold 50` | CI quality check | | |
| Invalid Threshold | `uveddi ci check ./test_projects/sample_repo --debt-threshold -10` | Proper error message | | |

## Issue Reporting Template

For each issue found during testing, document:

```
### Issue: [Short Title]

- **Command**: [Command that produced the issue]
- **Expected Behavior**: [What should have happened]
- **Actual Behavior**: [What actually happened]
- **Error Message**: [If applicable]
- **Reproduction Steps**:
  1. [Step 1]
  2. [Step 2]
  3. [...]
- **Screenshots**: [If applicable]
- **Severity**: [Low/Medium/High/Critical]
- **Impact**: [How this affects users]
- **Suggested Fix**: [Optional]
```

## Summary Template

After all testing is complete:

```
# Uveddi CLI Testing Summary

## Overall Assessment
[General assessment of the CLI's state]

## Major Issues
1. [Issue 1]
2. [Issue 2]
3. [...]

## Usability Feedback
[Feedback on general usability and user experience]

## Documentation Gaps
[Missing or unclear documentation]

## Suggestions for Improvement
1. [Suggestion 1]
2. [Suggestion 2]
3. [...]
```
