# Uveddi Analysis Configuration

This project has been set up with Uveddi for automated code analysis.

## Project Information
- **Type**: Custom
- **Languages**: unknown
- **Estimated Size**: Small
- **Has Tests**: false
- **Has Documentation**: false

## Quick Start

```bash
# Run analysis
uveddi analyze .

# Run with AI explanations
uveddi analyze . --enable-ai

# Generate HTML report
uveddi analyze . --output-format html --output reports/analysis.html

# Check system health
uveddi doctor

# View configuration
uveddi config show
```

## Configuration

The project configuration is stored in `uveddi.toml`. You can:

- Edit the file directly
- Use `uveddi config set key value` to update settings
- Regenerate with `uveddi init --force`

## Git Integration

Git hooks not installed. Run `uveddi init --git-hooks` to enable automated analysis.

## Troubleshooting

If you encounter issues:

1. Run health check: `uveddi doctor`
2. Check configuration: `uveddi config validate`
3. View detailed help: `uveddi help troubleshooting`

For more information, see the [Uveddi documentation](https://github.com/botzrDev/uveddi).
