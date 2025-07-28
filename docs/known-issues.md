# Known Issues - Alpha Release

This document lists known issues and limitations in the Uveddi alpha release. We're actively working to resolve these in upcoming updates.

## Critical Issues

1. **Plugin Initialization Failure** (UV-501)
   - Description: Plugins occasionally fail to load during cold start
   - Workaround: Restart the application
   - Status: High priority fix scheduled for 0.9.1

2. **Memory Leak in Analysis Engine** (UV-512)
   - Description: Memory usage grows during extended analysis sessions
   - Workaround: Limit analysis sessions to <30 minutes
   - Status: Under investigation
