# Python Project Analysis Guide

## Overview

This guide covers Uveddi analysis for Python projects, including Django, Flask, FastAPI, and library projects.

## Key Analysis Areas

### 1. Dead Code Detection

Python's dynamic nature makes dead code detection challenging, but Uveddi handles:

- **Unused functions and classes**
- **Unreachable code** after returns/raises
- **Imported but unused modules**
- **Dynamic code** (eval, exec) analysis
- **Decorator-related complexity**

**Configuration Tips:**
```toml
[dead_code]
confidence_threshold = 0.7  # Lower due to Python's dynamic nature
library_mode = false        # Usually false for applications
ignore_patterns = [
    "tests/**",
    "test_*.py",
    "migrations/**",        # Django migrations
    "manage.py",            # Django management
    "setup.py",             # Package setup
    "conftest.py"           # pytest configuration
]
keep_alive_patterns = [
    "__main__",
    "__init__",
    "main",
    "app",                  # Flask/FastAPI apps
    "wsgi_application"      # Django WSGI
]
```

### 2. Large Classes Detection

Python-specific considerations:
- **God classes** with too many methods
- **Data classes** with too many fields
- **Inheritance hierarchies** that are too deep
- **Mixin overuse**

**Python-Specific Thresholds:**
- Max logical LOC: 300 (Python is more verbose)
- Max methods: 30 (including magic methods)
- Max fields: 20 (including properties)

### 3. Architectural Anti-Patterns

**Common Python Anti-Patterns:**
- **Circular imports** (very common in Python)
- **God modules** (everything in __init__.py)
- **Import spaghetti** (complex import dependencies)
- **Magic values** in configuration

### 4. Python-Specific Analysis

**Import Analysis:**
- Relative vs absolute imports
- Wildcard imports (`from module import *`)
- Missing __init__.py files
- Circular import detection

**Code Style:**
- PEP 8 compliance
- Docstring coverage
- Type hint usage
- Exception handling patterns

## Framework-Specific Guidelines

### Django Projects

```toml
[large_classes]
ignore_patterns = [
    "migrations/**",
    "settings.py",
    "urls.py"
]

[dead_code]
keep_alive_patterns = [
    "Meta",                 # Model meta classes
    "save",                 # Model methods called by Django
    "clean",                # Form validation methods
    "get_absolute_url",     # Model methods
    "Admin"                 # Admin classes
]
```

**Django-Specific Issues:**
- **Fat models** - Move business logic to services
- **Fat views** - Use class-based views and mixins
- **Settings bloat** - Split settings files
- **Migration conflicts** - Regular migration cleanup

### Flask Projects

```toml
[dead_code]
keep_alive_patterns = [
    "app",
    "create_app",
    "before_request",
    "after_request",
    "teardown_appcontext"
]
```

**Flask-Specific Issues:**
- **Blueprint organization** - Logical separation of concerns
- **Application factory** - Proper app initialization
- **Extension management** - Avoid circular imports with extensions

### FastAPI Projects

```toml
[dead_code]
keep_alive_patterns = [
    "app",
    "router",
    "dependency",
    "Depends",
    "get_db"
]
```

**FastAPI-Specific Issues:**
- **Dependency injection** complexity
- **Router organization** - Group related endpoints
- **Model validation** - Proper Pydantic usage

## Best Practices

### Module Organization
```python
# Good: Clear module structure
myproject/
├── models/
│   ├── __init__.py
│   ├── user.py
│   └── order.py
├── services/
│   ├── __init__.py
│   ├── user_service.py
│   └── order_service.py
└── views/
    ├── __init__.py
    ├── user_views.py
    └── order_views.py

# Bad: Everything in one file
myproject/
└── app.py  # 2000+ lines
```

### Import Management
```python
# Good: Explicit imports
from myproject.models.user import User
from myproject.services.auth import authenticate

# Bad: Wildcard imports
from myproject.models import *
from myproject.services import *
```

### Class Design
```python
# Before: God class
class UserManager:
    def create_user(self): ...
    def update_user(self): ...
    def delete_user(self): ...
    def send_email(self): ...
    def generate_report(self): ...
    def process_payment(self): ...
    def log_activity(self): ...

# After: Separated responsibilities
class UserRepository:
    def create(self): ...
    def update(self): ...
    def delete(self): ...

class UserService:
    def __init__(self, repo, email_service, payment_service):
        self.repo = repo
        self.email_service = email_service
        self.payment_service = payment_service
```

## Common Issues and Solutions

### Issue: Circular Imports
**Problem:** Module A imports B, B imports A
**Solution:** Move shared code to separate module or use late imports

```python
# Before: Circular import
# user.py imports order.py
# order.py imports user.py

# After: Extract shared types
# shared/types.py contains UserId, OrderId
# user.py and order.py both import shared/types.py

# Or use late imports
def get_user_orders(user_id):
    from .order import Order  # Late import
    return Order.objects.filter(user_id=user_id)
```

### Issue: Import Spaghetti
**Problem:** Complex web of imports
**Solution:** Dependency inversion and clear layers

```python
# Before: Direct database access everywhere
class UserController:
    def get_user(self, user_id):
        import sqlite3
        conn = sqlite3.connect('db.sqlite3')
        # Direct database access

# After: Layered architecture
class UserController:
    def __init__(self, user_service):
        self.user_service = user_service
    
    def get_user(self, user_id):
        return self.user_service.get_user(user_id)
```

### Issue: Magic Values
**Problem:** Hardcoded values throughout code
**Solution:** Configuration management

```python
# Before: Magic values
if user.age > 18 and user.score > 75:
    send_email(user.email, "Congratulations!")

# After: Configuration
class BusinessRules:
    MIN_AGE = 18
    MIN_SCORE = 75
    CONGRATULATIONS_EMAIL = "Congratulations!"

if user.age > BusinessRules.MIN_AGE and user.score > BusinessRules.MIN_SCORE:
    send_email(user.email, BusinessRules.CONGRATULATIONS_EMAIL)
```

## Analysis Commands

```bash
# Basic Python analysis
uveddi analyze src/

# Include test files in analysis
uveddi analyze . --include-patterns "test_*.py,tests/**"

# Focus on import analysis
uveddi analyze src/ --detectors imports,circular-dependencies

# Django project analysis
uveddi analyze . --framework django --ignore-patterns "migrations/**"

# Generate detailed report with AI insights
uveddi analyze src/ --output-format html --enable-ai

# Performance-focused analysis
uveddi analyze src/ --detectors performance,memory --strict
```

## Virtual Environment Integration

```bash
# Activate virtual environment first
source venv/bin/activate  # or `venv\Scripts\activate` on Windows

# Then run analysis
uveddi analyze src/

# Or specify Python path
PYTHON_PATH=/path/to/venv/lib/python3.9/site-packages uveddi analyze src/
```

## CI/CD Integration

```yaml
# .github/workflows/analysis.yml
name: Python Code Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Set up Python
        uses: actions/setup-python@v4
        with:
          python-version: '3.9'
      - name: Install dependencies
        run: |
          python -m pip install --upgrade pip
          pip install -r requirements.txt
      - name: Install Uveddi
        run: pip install uveddi
      - name: Run Analysis
        run: uveddi analyze src/ --output-format json --output analysis.json
      - name: Upload Results
        uses: actions/upload-artifact@v3
        with:
          name: analysis-results
          path: analysis.json
```

## Integration with Python Tools

### Black (Code Formatter)
```bash
# Run before analysis for consistent formatting
black src/
uveddi analyze src/
```

### isort (Import Sorting)
```bash
# Sort imports before analysis
isort src/
uveddi analyze src/ --detectors imports
```

### mypy (Type Checking)
```bash
# Type check before analysis
mypy src/
uveddi analyze src/ --include-type-info
```

## Performance Considerations

- **Large Django projects:** Use `--exclude-migrations` for faster analysis
- **Virtual environments:** Ensure proper PATH setup
- **Import complexity:** Use `--max-import-depth` to limit analysis depth
- **Memory usage:** Configure `--memory-limit` for large codebases

## Troubleshooting

### Import Resolution Issues
```bash
# Check Python path
echo $PYTHONPATH

# Verify Uveddi can import your modules
uveddi doctor --parsers --python-path /path/to/your/project

# Run with verbose import logging
uveddi analyze src/ --verbose --log-imports
```

### Performance Issues
```bash
# Profile analysis performance
uveddi analyze src/ --profile

# Reduce scope for large projects
uveddi analyze src/main_app/ --exclude-patterns "tests/**"

# Use faster but less thorough analysis
uveddi analyze src/ --fast-mode
```

For more Python-specific best practices, see [PEP 8](https://www.python.org/dev/peps/pep-0008/) and the [Google Python Style Guide](https://google.github.io/styleguide/pyguide.html).