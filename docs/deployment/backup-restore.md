# Backup & Restore Guide

This guide explains how to back up and restore Uveddi data and configuration.

## What to Back Up

- `~/.uveddi/logs/` — Log files
- `~/.uveddi/config/` — User and system configuration
- `~/.uveddi/cache/` — Analysis cache (optional)
- Any generated reports or exported data

## Backup Procedure

1. Stop any running Uveddi processes (if needed)
2. Copy the relevant directories/files to your backup location:
   ```bash
   cp -r ~/.uveddi/logs ~/backups/uveddi-logs-$(date +%F)
   cp -r ~/.uveddi/config ~/backups/uveddi-config-$(date +%F)
   cp -r ~/.uveddi/cache ~/backups/uveddi-cache-$(date +%F)
   ```
3. Store backups securely and verify integrity

## Restore Procedure

1. Stop Uveddi if running
2. Copy backup files back to their original locations:
   ```bash
   cp -r ~/backups/uveddi-logs-YYYY-MM-DD ~/.uveddi/logs
   cp -r ~/backups/uveddi-config-YYYY-MM-DD ~/.uveddi/config
   cp -r ~/backups/uveddi-cache-YYYY-MM-DD ~/.uveddi/cache
   ```
3. Restart Uveddi

## Tips

- Automate backups with cron or your preferred scheduler
- Test restores periodically to ensure backup integrity

---

> **Note:** For enterprise deployments, integrate with your organization's backup solution.
