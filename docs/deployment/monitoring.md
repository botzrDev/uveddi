# Monitoring Guide

This guide describes how to monitor Uveddi in production and development environments.

## Built-in Monitoring

- Uveddi logs key events and errors to `~/.uveddi/logs/` by default.
- Use the `--log-level` flag to adjust verbosity (e.g., `info`, `debug`, `trace`).
- For performance metrics, enable the `--metrics` flag (if available).

## External Monitoring

- Integrate with system monitoring tools (e.g., Prometheus, Grafana) by exporting logs or metrics.
- Use process managers (e.g., systemd, supervisord) to monitor uptime and restarts.

## Health Checks

- Run `uveddi --health` (planned) for a quick status check.
- Monitor resource usage (CPU, memory) using standard OS tools (`top`, `htop`).

## Alerts

- Set up alerts for log errors, high memory usage, or process crashes using your preferred monitoring stack.

---

> **Note:** Monitoring features are evolving. Please report your monitoring needs or issues via [GitHub Issues](https://github.com/botzrDev/uveddi/issues).
