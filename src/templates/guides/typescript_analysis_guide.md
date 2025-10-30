# TypeScript Project Analysis Guide

## Overview

This guide covers Uveddi analysis for TypeScript projects, including Node.js with TypeScript, React with TypeScript, Angular, and library projects.

## Key Analysis Areas

### 1. Dead Code Detection

TypeScript's static typing helps with dead code detection:

- **Unused functions, classes, and interfaces**
- **Unreachable code** after type guards
- **Unused type definitions** and generics
- **Dead imports** and re-exports
- **Conditional compilation** based on types

**Configuration Tips:**
```toml
[dead_code]
confidence_threshold = 0.8  # Higher due to static typing
library_mode = true         # Often true for TypeScript libraries
ignore_patterns = [
    "node_modules/**",
    "dist/**",
    "build/**",
    "coverage/**",
    "*.test.ts",
    "*.spec.ts",
    "*.d.ts",               # Type declaration files
    "__tests__/**",
    ".next/**"
]
keep_alive_patterns = [
    "export",
    "default",
    "interface",            # Exported interfaces
    "type",                 # Type aliases
    "enum",                 # Exported enums
    "namespace",            # Namespaces
    "declare"               # Ambient declarations
]
```

### 2. Large Classes Detection

TypeScript-specific considerations:
- **Classes with too many methods**
- **Interfaces with too many properties**
- **Complex generic types**
- **Inheritance hierarchies**

**TypeScript-Specific Thresholds:**
- Max logical LOC: 250
- Max methods: 25
- Max properties: 20 (including interface members)
- Max type parameters: 5 (for generics)

### 3. Architectural Anti-Patterns

**Common TypeScript Anti-Patterns:**
- **Any type overuse** (`any` everywhere)
- **Type assertion abuse** (excessive `as` casting)
- **Complex generic constraints**
- **Deep inheritance hierarchies**
- **Circular type dependencies**
- **Magic values** in type definitions

### 4. TypeScript-Specific Analysis

**Type System Analysis:**
- Type coverage percentage
- `any` usage detection
- Strict mode compliance
- Generic complexity
- Union type complexity

**Module System Analysis:**
- ES6 modules vs CommonJS
- Type-only imports/exports
- Re-export patterns
- Declaration merging issues

## Framework-Specific Guidelines

### React with TypeScript

```toml
[large_classes]
max_logical_loc = 200      # React components should be smaller
max_methods = 20

[dead_code]
keep_alive_patterns = [
    "FC",                   # React.FC
    "Component",
    "Props",                # Component props interfaces
    "State",                # Component state interfaces
    "useEffect",
    "useState",
    "forwardRef",
    "memo"
]

ignore_patterns = [
    "*.stories.tsx",        # Storybook files
    "*.test.tsx",
    "__mocks__/**"
]
```

**React + TypeScript Issues:**
- **Props interface bloat** - Keep props focused
- **Generic component overuse** - Balance flexibility and simplicity
- **Hook type complexity** - Simplify custom hook types
- **Context type safety** - Proper context typing

### Angular Projects

```toml
[dead_code]
keep_alive_patterns = [
    "@Component",
    "@Injectable",
    "@NgModule",
    "@Directive",
    "@Pipe",
    "OnInit",
    "OnDestroy",
    "constructor"
]

ignore_patterns = [
    "src/environments/**",
    "*.spec.ts",
    "e2e/**"
]
```

**Angular-Specific Issues:**
- **Service injection complexity** - Proper dependency injection
- **Component template size** - Move logic to services
- **Module organization** - Feature modules vs shared modules

### Node.js with TypeScript

```toml
[dead_code]
keep_alive_patterns = [
    "main",
    "default",
    "express",
    "app.listen",
    "middleware",
    "router"
]
```

**Node.js + TypeScript Issues:**
- **Type definition management** - Proper @types usage
- **Module resolution** - Path mapping configuration
- **Build configuration** - Proper TypeScript compilation

## Best Practices

### Type Definitions
```typescript
// Good: Clear, focused interfaces
interface User {
    id: string;
    name: string;
    email: string;
}

interface UserService {
    findById(id: string): Promise<User | null>;
    create(userData: Omit<User, 'id'>): Promise<User>;
    update(id: string, updates: Partial<User>): Promise<User>;
}

// Bad: Overly complex generic interface
interface Repository<T, K extends keyof T, U extends Record<string, any>, V = unknown> {
    find<W extends K>(query: Pick<T, W> & U): Promise<Array<T & V>>;
    // ... overly complex signature
}
```

### Class Organization
```typescript
// Before: God class
class UserManager {
    // 50+ properties
    // 30+ methods handling everything
    async createUser(): Promise<User> { /* 100+ lines */ }
    async validateUser(): Promise<boolean> { /* 50+ lines */ }
    async sendEmail(): Promise<void> { /* 75+ lines */ }
    async generateReport(): Promise<Report> { /* 200+ lines */ }
}

// After: Separated responsibilities
interface UserRepository {
    create(user: CreateUserDto): Promise<User>;
    findById(id: string): Promise<User | null>;
    update(id: string, updates: UpdateUserDto): Promise<User>;
}

interface EmailService {
    sendWelcomeEmail(user: User): Promise<void>;
    sendPasswordReset(email: string): Promise<void>;
}

class UserService {
    constructor(
        private userRepo: UserRepository,
        private emailService: EmailService,
        private validator: UserValidator
    ) {}
    
    async createUser(userData: CreateUserDto): Promise<User> {
        await this.validator.validate(userData);
        const user = await this.userRepo.create(userData);
        await this.emailService.sendWelcomeEmail(user);
        return user;
    }
}
```

### Generic Design
```typescript
// Before: Over-engineered generics
interface SuperRepository<
    T extends Record<string, any>,
    K extends keyof T,
    U extends T[K],
    V extends Partial<T>
> {
    findByField<W extends K>(
        field: W,
        value: T[W],
        options?: FindOptions<T, U, V>
    ): Promise<Array<Pick<T, W> & Partial<V>>>;
}

// After: Simple, focused generics
interface Repository<T> {
    findById(id: string): Promise<T | null>;
    create(entity: Omit<T, 'id'>): Promise<T>;
    update(id: string, updates: Partial<T>): Promise<T>;
    delete(id: string): Promise<void>;
}
```

## Common Issues and Solutions

### Issue: Any Type Overuse
**Problem:** Using `any` everywhere defeats TypeScript's purpose
**Solution:** Proper typing with gradual migration

```typescript
// Before: Any abuse
function processData(data: any): any {
    return data.map((item: any) => ({
        ...item,
        processed: true
    }));
}

// After: Proper typing
interface DataItem {
    id: string;
    name: string;
    value: number;
}

interface ProcessedDataItem extends DataItem {
    processed: boolean;
}

function processData(data: DataItem[]): ProcessedDataItem[] {
    return data.map(item => ({
        ...item,
        processed: true
    }));
}
```

### Issue: Complex Generic Constraints
**Problem:** Overly complex generic type definitions
**Solution:** Simplify and break down complex types

```typescript
// Before: Complex generic mess
type ComplexMapper<
    T extends Record<string, unknown>,
    K extends keyof T,
    U extends T[K],
    V extends Record<string, U>
> = {
    [P in keyof T]: T[P] extends U ? V[string] : T[P];
};

// After: Simpler, more focused types
type MapToString<T> = {
    [K in keyof T]: string;
};

type OptionalProps<T, K extends keyof T> = Omit<T, K> & Partial<Pick<T, K>>;
```

### Issue: Props Interface Bloat
**Problem:** React component props with too many properties
**Solution:** Composition and focused interfaces

```typescript
// Before: Bloated props interface
interface UserCardProps {
    user: User;
    showAvatar: boolean;
    showEmail: boolean;
    showPhone: boolean;
    showAddress: boolean;
    avatarSize: 'small' | 'medium' | 'large';
    onEdit: (user: User) => void;
    onDelete: (user: User) => void;
    onEmailClick: (email: string) => void;
    onPhoneClick: (phone: string) => void;
    theme: 'light' | 'dark';
    className?: string;
    style?: React.CSSProperties;
    // ... 20+ more props
}

// After: Composed interfaces
interface UserDisplayOptions {
    showAvatar: boolean;
    showContact: boolean;
    avatarSize: AvatarSize;
}

interface UserCardActions {
    onEdit: (user: User) => void;
    onDelete: (user: User) => void;
}

interface UserCardProps {
    user: User;
    display: UserDisplayOptions;
    actions?: UserCardActions;
    className?: string;
}
```

## Analysis Commands

```bash
# Basic TypeScript analysis
uveddi analyze src/

# Include type checking results
uveddi analyze src/ --typescript --type-check

# React + TypeScript analysis
uveddi analyze src/ --typescript --jsx --react

# Node.js + TypeScript analysis
uveddi analyze src/ --typescript --node

# Library analysis with type exports
uveddi analyze src/ --typescript --library-mode

# Generate detailed type coverage report
uveddi analyze src/ --typescript --type-coverage --output-format html

# Check for TypeScript anti-patterns
uveddi analyze src/ --typescript --detectors type-safety,generics
```

## TypeScript Configuration Integration

### tsconfig.json Integration
```json
{
  "compilerOptions": {
    "strict": true,
    "noUnusedLocals": true,
    "noUnusedParameters": true,
    "exactOptionalPropertyTypes": true
  },
  "include": ["src/**/*"],
  "exclude": ["node_modules", "dist", "**/*.test.ts"]
}
```

### Uveddi Configuration
```toml
[typescript]
strict_mode = true
check_unused_types = true
max_generic_params = 4
max_union_members = 8

[large_classes]
# TypeScript-specific thresholds
max_interface_properties = 15
max_generic_constraints = 3
```

## CI/CD Integration

```yaml
# .github/workflows/typescript-analysis.yml
name: TypeScript Code Analysis

on: [push, pull_request]

jobs:
  analyze:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
          cache: 'npm'
      - name: Install dependencies
        run: npm ci
      - name: TypeScript compilation check
        run: npx tsc --noEmit
      - name: Install Uveddi
        run: npm install -g uveddi
      - name: Run TypeScript Analysis
        run: uveddi analyze src/ --typescript --output-format json --output analysis.json
      - name: Type Coverage Check
        run: uveddi analyze src/ --typescript --type-coverage --min-coverage 80
```

## Integration with TypeScript Tools

### ESLint with TypeScript
```bash
# Run ESLint with TypeScript rules
npx eslint src/ --ext .ts,.tsx --fix
uveddi analyze src/ --typescript
```

### Prettier with TypeScript
```bash
# Format TypeScript files
npx prettier --write "src/**/*.{ts,tsx}"
uveddi analyze src/ --typescript
```

### Type Coverage Tools
```bash
# Check type coverage before analysis
npx type-coverage --detail
uveddi analyze src/ --typescript --include-type-coverage
```

## Performance Considerations

- **Large projects:** Use `--incremental` for faster subsequent analyses
- **Type checking:** Enable `--type-check` only when needed (slower)
- **Generic analysis:** Complex generics slow down analysis
- **Declaration files:** Exclude `.d.ts` files from analysis for speed

## Troubleshooting

### Type Resolution Issues
```bash
# Check TypeScript compiler version
npx tsc --version

# Verify tsconfig.json is valid
npx tsc --showConfig

# Run Uveddi with TypeScript debug info
uveddi analyze src/ --typescript --verbose --debug-types
```

### Generic Analysis Issues
```bash
# Limit generic analysis complexity
uveddi analyze src/ --typescript --max-generic-depth 5

# Skip complex generic analysis
uveddi analyze src/ --typescript --skip-complex-generics
```

### Memory Issues with Large Projects
```bash
# Increase memory limit
uveddi analyze src/ --typescript --memory-limit 8GB

# Analyze in chunks
uveddi analyze src/components/ --typescript
uveddi analyze src/services/ --typescript
uveddi analyze src/utils/ --typescript
```

For more TypeScript best practices, see the [TypeScript Handbook](https://www.typescriptlang.org/docs/) and [TypeScript Do's and Don'ts](https://www.typescriptlang.org/docs/handbook/declaration-files/do-s-and-don-ts.html).