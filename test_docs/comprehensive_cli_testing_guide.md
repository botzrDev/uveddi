# Uveddi CLI Alpha Testing Comprehensive Guide

This document provides a systematic approach to testing the Uveddi CLI alpha release. It covers all available subcommands, their options, expected behaviors, and includes templates for reporting issues and providing feedback.

## Table of Contents

1. [Testing Environment Setup](#testing-environment-setup)
2. [Main Help and Version Commands](#main-help-and-version-commands)
3. [Analyze Command](#analyze-command)
4. [Detect Command](#detect-command)
5. [Repo Command](#repo-command)
6. [Test Command](#test-command)
7. [Benchmark Command](#benchmark-command)
8. [Jira Command](#jira-command)
9. [Config Command](#config-command)
10. [Report Command](#report-command)
11. [UI Command](#ui-command)
12. [CI Command](#ci-command)
13. [Issue Reporting Template](#issue-reporting-template)
14. [Summary Template](#summary-template)

## Testing Environment Setup

Before beginning testing, ensure:

1. Uveddi is installed and accessible in your path
2. You have test projects available for analysis
3. You have repositories with different languages (Rust, Python, JavaScript, TypeScript)
4. Create test scenarios for permissions, invalid paths, etc.

```bash
# Verify installation
uveddi --version

# Set up a test environment
mkdir -p ~/uveddi_testing/test_projects
mkdir -p ~/uveddi_testing/reports
mkdir -p ~/uveddi_testing/configs

# Create permission test directory
mkdir -p ~/uveddi_testing/test_projects/permission_test
chmod 000 ~/uveddi_testing/test_projects/permission_test

# Clone test repositories if needed
git clone https://github.com/example/rust-repo.git ~/uveddi_testing/test_projects/rust_repo
git clone https://github.com/example/python-repo.git ~/uveddi_testing/test_projects/python_repo
```

## Main Help and Version Commands

Test the basic help and version information:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Basic Help | `uveddi --help` | List all available commands with descriptions | | |
| Version Check | `uveddi --version` | Display current version | | |
| Help on Nonexistent Command | `uveddi nonexistent --help` | Clear error message | | |

## Analyze Command

Test the analyze command with various options:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi analyze --help` | Show analyze options | | |
| Basic Analysis | `uveddi analyze ./test_projects/rust_repo` | Complete analysis with results | | |
| Verbose Output | `uveddi analyze ./test_projects/rust_repo --verbose` | Detailed analysis output | | |
| HTML Output | `uveddi analyze ./test_projects/rust_repo --output-format html --output ./reports/report.html` | Generate HTML report | | |
| JSON Output | `uveddi analyze ./test_projects/rust_repo --output-format json --output ./reports/report.json` | Generate JSON report | | |
| Markdown Output | `uveddi analyze ./test_projects/rust_repo --output-format markdown --output ./reports/report.md` | Generate Markdown report | | |
| Language Filter | `uveddi analyze ./test_projects/rust_repo --languages rust,python` | Analyze only rust and python files | | |
| Timing Info | `uveddi analyze ./test_projects/rust_repo --timing` | Include timing information | | |
| With Config | `uveddi analyze ./test_projects/rust_repo -c ./configs/benchmark-config.toml` | Use custom config | | |
| Invalid Path | `uveddi analyze ./nonexistent_path` | Proper error message | | |
| Permission Error | `uveddi analyze ~/uveddi_testing/test_projects/permission_test` | Permission error with helpful message | | |

## Detect Command

Test the detect command for specific issue detection:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi detect --help` | Show detect options | | |
| Basic Detection | `uveddi detect ./test_projects/rust_repo` | Detect issues with default settings | | |
| Magic Values | `uveddi detect ./test_projects/rust_repo --magic-values` | Detect magic values | | |
| Custom Rules | `uveddi detect ./test_projects/rust_repo --rules ./configs/custom_rules.toml` | Use custom detection rules | | |
| Output JSON | `uveddi detect ./test_projects/rust_repo --output ./reports/detect.json` | Save detection results | | |
| Invalid Rules File | `uveddi detect ./test_projects/rust_repo --rules ./nonexistent.toml` | Proper error message | | |

## Repo Command

Test the repo command for repository management:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi repo --help` | Show repo subcommands | | |
| Clone | `uveddi repo clone https://github.com/example/repo.git` | Clone repository | | |
| List | `uveddi repo list` | List managed repositories | | |
| Update | `uveddi repo update` | Update all repositories | | |
| Remove | `uveddi repo remove repo_name` | Remove repository | | |
| Invalid Clone URL | `uveddi repo clone https://invalid-url.git` | Proper error message | | |
| Remove Nonexistent | `uveddi repo remove nonexistent_repo` | Proper error message | | |

## Test Command

Test the test command for running tests:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi test --help` | Show test options | | |
| Basic Test | `uveddi test ./test_projects/rust_repo` | Run tests | | |
| With Coverage | `uveddi test ./test_projects/rust_repo --coverage` | Run tests with coverage | | |
| Output | `uveddi test ./test_projects/rust_repo --output ./reports/test_report.txt` | Output test results | | |
| Failing Tests | `uveddi test ./test_projects/failing_tests` | Show failing tests correctly | | |
| Invalid Path | `uveddi test ./nonexistent_path` | Proper error message | | |

## Benchmark Command

Test the benchmark command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi benchmark --help` | Show benchmark options | | |
| Basic Benchmark | `uveddi benchmark ./test_projects/rust_repo` | Run benchmark | | |
| With Config | `uveddi benchmark ./test_projects/rust_repo --config ./configs/benchmark-config.toml` | Run with custom config | | |
| Output | `uveddi benchmark ./test_projects/rust_repo --output ./reports/benchmark_report.json` | Save benchmark results | | |
| Invalid Config | `uveddi benchmark ./test_projects/rust_repo --config ./nonexistent.toml` | Proper error message | | |

## Jira Command

Test the Jira integration command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi jira --help` | Show jira subcommands | | |
| Link | `uveddi jira link UV-123` | Link to Jira issue | | |
| Status | `uveddi jira status UV-123` | Show issue status | | |
| Sync | `uveddi jira sync` | Sync with Jira | | |
| Invalid Issue | `uveddi jira status INVALID-123` | Proper error message | | |
| Missing Credentials | `uveddi jira sync --clear-credentials` then `uveddi jira sync` | Prompt for credentials | | |

## Config Command

Test the config command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi config --help` | Show config subcommands | | |
| Show | `uveddi config show` | Display current configuration | | |
| Set | `uveddi config set analysis.level=high` | Change config value | | |
| Reset | `uveddi config reset` | Reset to default config | | |
| Validate | `uveddi config validate --file ./configs/benchmark-config.toml` | Validate config file | | |
| Invalid Key | `uveddi config set nonexistent.key=value` | Proper error message | | |
| Invalid Value | `uveddi config set analysis.level=invalid` | Proper error message | | |

## Report Command

Test the report command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi report --help` | Show report options | | |
| Analysis Report | `uveddi report analysis` | Generate analysis report | | |
| Test Report | `uveddi report test` | Generate test report | | |
| Benchmark Report | `uveddi report benchmark` | Generate benchmark report | | |
| Format | `uveddi report analysis --format html` | Generate HTML report | | |
| Output | `uveddi report analysis --output ./reports/analysis_report.html` | Save report to file | | |
| Invalid Format | `uveddi report analysis --format invalid` | Proper error message | | |

## UI Command

Test the UI command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi ui --help` | Show UI options | | |
| Launch UI | `uveddi ui` | Launch the UI interface | | |
| Port | `uveddi ui --port 3000` | Launch UI on specific port | | |

## CI Command

Test the CI command:

| Test Case | Command | Expected Result | Actual Result | Notes |
|-----------|---------|-----------------|---------------|-------|
| Command Help | `uveddi ci --help` | Show CI options | | |
| Check | `uveddi ci check ./test_projects/rust_repo --debt-threshold 50` | Run CI quality check | | |
| Critical Threshold | `uveddi ci check ./test_projects/rust_repo --critical-threshold 0` | Enforce zero critical issues | | |
| Invalid Threshold | `uveddi ci check ./test_projects/rust_repo --debt-threshold -10` | Proper error message | | |

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
- Command Structure: [Feedback on command organization]
- Error Messages: [Feedback on error message clarity]
- Documentation: [Feedback on help text and docs]
- Performance: [Feedback on command execution speed]

## Documentation Gaps
[Missing or unclear documentation]

## Feature Completeness
[Assessment of feature completeness against requirements]

## Suggestions for Improvement
1. [Suggestion 1]
2. [Suggestion 2]
3. [...]
```
