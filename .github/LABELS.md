# GitHub Labels Guide

## Issue Difficulty Levels

### 🟢 Beginner (`difficulty/beginner`)
- **Time**: 1-4 hours
- **Context**: Minimal project knowledge needed
- **Examples**: Documentation updates, simple bug fixes, adding tests
- **Prerequisites**: Basic Git, language fundamentals

### 🟡 Intermediate (`difficulty/intermediate`) 
- **Time**: 4-8 hours
- **Context**: Some project architecture understanding
- **Examples**: New features, refactoring, integration work
- **Prerequisites**: Project setup experience, domain knowledge

### 🔴 Advanced (`difficulty/advanced`)
- **Time**: 8+ hours
- **Context**: Deep project understanding required
- **Examples**: Core architecture changes, performance optimization
- **Prerequisites**: Extensive codebase familiarity

## Skill Categories

- `skill/documentation` - Writing, updating docs
- `skill/frontend` - TypeScript, React, UI work
- `skill/backend` - Rust, core analysis engine
- `skill/testing` - Unit tests, integration tests
- `skill/devops` - CI/CD, Docker, deployment

## Special Labels

- `good-first-issue` - Perfect for new contributors
- `beginner-friendly` - Accessible to project newcomers
- `time/quick-win` - 1-2 hour tasks
- `time/weekend-project` - 4-8 hour projects

## Label Creation

To create these labels in GitHub:

1. Go to GitHub repository Settings → Labels
2. Create each label with the specified color and description:

### Primary Labels
```yaml
good-first-issue:
  color: "7057ff"
  description: "Good for newcomers - well-defined, limited scope, clear instructions"

beginner-friendly:
  color: "0e8a16"
  description: "Suitable for developers new to the project"
```

### Difficulty Labels
```yaml
difficulty/beginner:
  color: "c2e0c6"
  description: "Easy task, 1-4 hours, minimal context needed"

difficulty/intermediate:
  color: "fef2c0"
  description: "Moderate task, 4-8 hours, some project knowledge needed"

difficulty/advanced:
  color: "f9d0c4"
  description: "Complex task, 8+ hours, deep project understanding required"
```

### Skill-Based Labels
```yaml
skill/documentation:
  color: "0052cc"
  description: "Primarily documentation work"

skill/frontend:
  color: "1d76db"
  description: "Frontend/UI development"

skill/backend:
  color: "0e4b99"
  description: "Backend/core logic development"

skill/testing:
  color: "5319e7"
  description: "Testing and quality assurance"

skill/devops:
  color: "0366d6"
  description: "CI/CD, deployment, infrastructure"
```

### Time Investment Labels
```yaml
time/quick-win:
  color: "bfd4f2"
  description: "Can be completed in 1-2 hours"

time/weekend-project:
  color: "d4edda"
  description: "Perfect for a weekend project (4-8 hours)"
```