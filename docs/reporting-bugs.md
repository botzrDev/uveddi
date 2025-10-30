# Reporting Bugs in Uveddi Alpha

Thank you for helping us improve Uveddi! This guide explains how to effectively report bugs during our alpha testing phase.

## Before Reporting
1. Check the [Known Issues](./known-issues.md) document to see if your issue is already documented
2. Ensure you're running the latest version: `uveddi --version`
3. Try to reproduce the issue at least twice to confirm it's consistent

## How to Report
### Preferred Method: GitHub Issues
1. Go to our [Issues page](https://github.com/botzrDev/uveddi/issues)
2. Click "New Issue"
3. Select "Bug Report" template
4. Fill in all required sections

### Alternative Methods
- **Email**: bugs@uveddi.com (include "Alpha Bug" in subject)
- **Community Forum**: Post in #alpha-bugs on our [Discord server](https://discord.gg/uveddi)

## Required Information
1. **Description**:
   - Clear explanation of the problem
   - Expected vs actual behavior

2. **Reproduction Steps**:
   - Step-by-step instructions to reproduce the issue
   - Example:
     1. Run `uveddi analyze --lang rust ./src`
     2. Select "Architecture Analysis" option
     3. Observe error message

3. **Environment**:
   - OS version: `uname -a`
   - Shell: `echo $SHELL`
   - Uveddi version: `uveddi --version`
   - Relevant dependencies: `rustc --version`, `node --version`

4. **Error Messages**:
   - Copy/paste exact error messages
   - Include stack traces if available

5. **Log Files**:
   - Located in `~/.uveddi/logs/`
   - Include relevant log snippets (sanitize sensitive info)

## Optional but Helpful
- Screenshots or screen recordings of the issue
- Minimal code sample that triggers the issue
- Related Jira issue key if applicable (e.g., UV-XXX)

## Response Timeline
- We'll acknowledge your report within 48 hours
- Critical bugs will be prioritized for the next patch release
- You'll receive updates as we work on the issue
- Fixed issues will be noted in the [CHANGELOG.md](../CHANGELOG.md)

## Bug Triage Process
1. Initial review within 48 hours
2. Validation and reproduction by our team
3. Prioritization based on severity and impact
4. Development and testing of fix
5. Fix included in next release

Thank you for helping us build a better Uveddi!
