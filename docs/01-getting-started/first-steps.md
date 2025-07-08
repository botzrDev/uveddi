# First Steps with Uveddi

## Running Your First Analysis

1. Navigate to your project directory:
   ```bash
   cd /path/to/your/project
   ```

2. Run basic analysis:
   ```bash
   uveddi analyze .
   ```

3. View results:
   ```bash
   cat uveddi_report.json
   ```

## Understanding the Output

Uveddi generates reports with:
- Architectural issues
- Code smells
- Dependency graphs
- AI-generated explanations

Example output structure:
```json
{
  "summary": {
    "issues_found": 12,
    "critical": 3,
    "warnings": 5,
    "suggestions": 4
  },
  "details": [
    {
      "type": "cyclic_dependency",
      "severity": "critical",
      "description": "Module A and Module B have a circular dependency",
      "locations": ["src/module_a.rs", "src/module_b.rs"],
      "suggested_fix": "Introduce an interface module..."
    }
  ]
}
```

## Common Commands

### Basic Analysis
```bash
uveddi analyze ./project --output report.json
```

### Focused Analysis
```bash
uveddi analyze ./project --focus cyclic_dependencies
```

### Interactive Mode
```bash
uveddi analyze ./project --interactive
```

### Generate Visual Report
```bash
uveddi analyze ./project --visual --output report.html
```

## Next Steps

1. Explore different analysis modes:
   ```bash
   uveddi analyze --help
   ```

2. Configure your project analysis:
   ```bash
   uveddi config set analysis.parallel_jobs 8
   ```

3. Integrate with your CI/CD pipeline:
   ```bash
   uveddi analyze ./project --fail-on critical
