# Deployment Assets

This folder gathers the scripts and compose files used for local development and container deployments.

- `Dockerfile.development`, `Dockerfile.production`, `Dockerfile.release`: Docker build recipes for their respective targets.
- `docker-compose.dev.yml`, `docker-compose.yml`: Compose definitions for multi-service setups.
- `docker-entrypoint.sh`: Entry-point script for container images.
- `install.sh`, `uninstall.sh`: Helper scripts for provisioning local environments.

These files used to live in the repository root; relocating them keeps the root cleaner while keeping deployment resources easy to find.
