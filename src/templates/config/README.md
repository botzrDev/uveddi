# Uveddi Configuration Templates

This directory contains pre-configured templates for common project types and use cases. These templates provide optimized settings for different development scenarios.

## Available Templates

### Language-Specific Templates

#### `rust-library.toml`
- **Use Case**: Rust library projects (crates)
- **Features**: 
  - High confidence thresholds for public APIs
  - Library-specific ignore patterns (examples, benches, docs)
  - Strict code quality standards
  - Optimized for crate publishing

#### `python-web-app.toml`  
- **Use Case**: Python web applications (Django, Flask, FastAPI)
- **Features**:
  - Lower confidence for Python's dynamic nature
  - Web framework patterns (migrations, static files, etc.)
  - Django/Flask specific keep-alive patterns
  - Balanced quality thresholds

#### `typescript-react.toml`
- **Use Case**: React applications with TypeScript
- **Features**:
  - React component optimization
  - TypeScript static typing benefits  
  - Next.js and build tool support
  - Small component size enforcement

### Project Scale Templates

#### `monorepo.toml`
- **Use Case**: Large monorepo projects with multiple languages
- **Features**:
  - Multi-language support with overrides
  - Comprehensive ignore patterns for all languages
  - Relaxed thresholds for complex projects
  - Package/workspace specific patterns

### CI/CD Templates

#### `minimal-ci.toml` 
- **Use Case**: Fast CI/CD pipelines
- **Features**:
  - Lightweight model for speed
  - High confidence to reduce false positives
  - Minimal analysis scope
  - Optimized for quick feedback

#### `strict-quality.toml`
- **Use Case**: High-quality code with stringent requirements  
- **Features**:
  - Very strict thresholds
  - Comprehensive analysis
  - Best AI model for detailed insights
  - Enterprise-grade quality standards

## Using Templates

### With `uveddi init`
```bash
# Interactive selection
uveddi init

# Specify template directly
uveddi init --template rust-library
uveddi init --template python-web-app
uveddi init --template typescript-react
uveddi init --template monorepo
```

### Manual Usage
```bash
# Copy template to your project
cp /path/to/uveddi/templates/config/rust-library.toml ./uveddi.toml

# Customize as needed
uveddi config validate --suggestions
```

### From CI/CD
```bash
# Use minimal template for fast CI
cp templates/config/minimal-ci.toml ./uveddi.toml
uveddi analyze ./src --timeout 300

# Use strict template for release gates
cp templates/config/strict-quality.toml ./uveddi.toml  
uveddi analyze ./src --fail-on critical
```

## Template Customization

All templates can be customized for your specific needs:

1. **Copy the closest template**:
   ```bash
   cp rust-library.toml my-project.toml
   ```

2. **Adjust thresholds**:
   ```toml
   [large_classes]
   max_logical_loc = 120  # Adjust for your team standards
   ```

3. **Add custom ignore patterns**:
   ```toml
   [dead_code]
   ignore_patterns = [
       "target/**",
       "my-custom-generated/**"
   ]
   ```

4. **Validate changes**:
   ```bash
   uveddi config validate --suggestions
   ```

## Template Comparison

| Template | AI Model | LOC Threshold | Methods | Confidence | Use Case |
|----------|----------|---------------|---------|------------|----------|
| rust-library | deepseek-coder:6.7b | 150 | 15 | 0.9 | Library development |
| python-web-app | deepseek-coder:6.7b | 300 | 30 | 0.7 | Web applications |
| typescript-react | deepseek-coder:6.7b | 200 | 20 | 0.8 | React apps |
| monorepo | codellama:7b | 400 | 40 | 0.75 | Large projects |
| minimal-ci | llama2:7b | 500 | 50 | 0.85 | CI/CD speed |
| strict-quality | deepseek-coder:6.7b | 100 | 10 | 0.95 | High quality |

## Best Practices

1. **Start with the closest template** to your project type
2. **Adjust gradually** - don't make all thresholds strict at once
3. **Validate regularly** with `uveddi config validate --suggestions`
4. **Test changes** with `uveddi analyze ./src` before committing
5. **Document customizations** in your project README

## Template Updates

Templates are updated with each Uveddi release to incorporate:
- New language patterns
- Improved threshold recommendations  
- Latest AI model optimizations
- Community feedback

To update your templates:
```bash
uveddi init --template <your-template> --force
# Review changes and merge with your customizations
```

## Contributing

Found improvements for existing templates or have ideas for new ones?
- Open an issue at https://github.com/botzrDev/uveddi/issues
- Submit a PR with your template additions
- Share your custom configurations with the community

## Template Validation

All templates are automatically validated for:
- Syntax correctness
- Threshold reasonableness
- Pattern effectiveness
- Performance impact

You can validate any template:
```bash
uveddi config validate --file template-name.toml --suggestions
```