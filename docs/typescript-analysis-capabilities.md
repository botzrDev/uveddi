# TypeScript Analysis Capabilities in Uveddi

This document describes the comprehensive TypeScript analysis capabilities implemented in Uveddi v0.9.0-alpha, including architectural improvements, anti-pattern detection, and advanced language-specific features.

## Overview

Uveddi now provides full TypeScript analysis support that goes beyond basic parsing to include:

- **Advanced AST Parsing**: Complete support for TypeScript-specific constructs
- **TypeScript-Specific Anti-Pattern Detection**: Tailored detection for TS patterns
- **Framework-Aware Analysis**: Recognition of Angular, NestJS, and other TS frameworks
- **Interface and Namespace Analysis**: Detection of oversized interfaces and namespaces
- **Generic Type Support**: Analysis of generic classes and functions
- **Decorator Pattern Recognition**: Understanding of TypeScript decorators

## Architectural Improvements

### Enhanced AST Parsing

The TypeScript parser has been completely redesigned to handle TypeScript-specific constructs:

```rust
// Enhanced TypeScript AST parsing now supports:
"interface_declaration" => { /* Interface analysis */ }
"type_alias_declaration" => { /* Type alias analysis */ }
"enum_declaration" => { /* Enum analysis */ }
"module_declaration" | "namespace_declaration" => { /* Namespace analysis */ }
```

### Tree-sitter Query Improvements

TypeScript analysis now uses dedicated tree-sitter queries instead of falling back to JavaScript:

```rust
// TypeScript-specific queries
const TYPESCRIPT_CLASS_QUERY: &str = r#"
(class_declaration
  name: (type_identifier) @name
  body: (class_body) @body
)
(class_declaration
  name: (type_identifier) @name
  type_parameters: (type_parameters) @type_params
  body: (class_body) @body
)
"#;

const TYPESCRIPT_INTERFACE_QUERY: &str = r#"
(interface_declaration
  name: (type_identifier) @interface_name
  body: (object_type) @interface_body
)
"#;
```

## TypeScript-Specific Anti-Pattern Detection

### God Object Detection

Enhanced God Object detection for TypeScript includes:

#### 1. Class Analysis
- **Method Threshold**: 15 methods (stricter than JavaScript due to better type system)
- **Field Threshold**: 10 fields (stricter than JavaScript due to interface support)
- **Type Parameter Analysis**: Detection of overly complex generic classes

#### 2. Interface Analysis (God Interface Pattern)
- **Property Threshold**: 8 properties (interfaces should be focused)
- **Cohesion Analysis**: Detection of interfaces with unrelated properties
- **Severity Levels**: Medium (9-12 props), High (13-20 props), Critical (20+ props)

#### 3. Namespace Analysis (God Namespace Pattern)
- **Declaration Threshold**: 20 declarations per namespace
- **Nested Analysis**: Detection of deeply nested namespace structures
- **Severity Levels**: Medium (21-30), High (31-50), Critical (50+ declarations)

### Framework-Aware Exclusions

TypeScript analysis includes sophisticated framework pattern recognition:

#### Angular Framework Patterns
```typescript
// These are excluded from God Object detection
@Component({ /* ... */ })
class DashboardComponent { /* Large but legitimate */ }

@Service()
class UserService { /* Large but legitimate */ }

@Module()
class AppModule { /* Large but legitimate */ }
```

#### NestJS Framework Patterns  
```typescript
// These are excluded from God Object detection
@Injectable()
class UserService { /* Large service is normal */ }

@Controller('users')
class UserController { /* Controllers can be large */ }
```

#### DTO Pattern Recognition
```typescript
// High field-to-method ratio with decorators indicates DTO
class CreateUserDto {
    @IsString() firstName: string;
    @IsEmail() email: string;
    @IsOptional() phone?: string;
    // No methods - excluded from God Object detection
}
```

## Supported TypeScript Constructs

### 1. Classes with Advanced Features
```typescript
// Generic classes
class Repository<T extends BaseEntity> { }

// Classes with decorators
@Injectable()
class UserService { }

// Classes with inheritance and type parameters
class AdvancedComponent<T, U> extends BaseComponent<T> implements Validator<U> { }
```

### 2. Interfaces
```typescript
// Simple interfaces
interface User {
    id: string;
    name: string;
}

// Generic interfaces  
interface Repository<T> {
    findById(id: string): Promise<T>;
}

// Large interfaces (detected as God Interfaces)
interface MassiveState {
    // 20+ properties trigger detection
}
```

### 3. Type Aliases
```typescript
// Simple type aliases
type UserRole = 'admin' | 'user';

// Complex type aliases with generics
type EventHandler<T> = (event: T) => void;

// Conditional types
type ApiResponse<T> = T extends string ? StringResponse : ObjectResponse;
```

### 4. Enums
```typescript
// Numeric enums
enum HttpStatus {
    OK = 200,
    BadRequest = 400
}

// String enums
enum Theme {
    Light = 'light',
    Dark = 'dark'
}
```

### 5. Namespaces
```typescript
// Namespaces with functions
namespace Utilities {
    export function format(): string { }
    export function parse(): object { }
    // Large namespaces trigger God Namespace detection
}

// Module declarations
declare module 'external-lib' {
    export interface Config { }
}
```

### 6. Decorators
```typescript
// Class decorators
@Component({
    selector: 'app-user'
})

// Method decorators
@Get('/users')
public getUsers() { }

// Property decorators
@IsEmail()
private email: string;

// Parameter decorators
public create(@Body() data: CreateDto) { }
```

## Analysis Configuration

### TypeScript-Specific Thresholds

```rust
// Default thresholds for TypeScript
method_thresholds.insert(SourceLanguage::TypeScript, 15);
field_thresholds.insert(SourceLanguage::TypeScript, 10);

// Interface-specific thresholds
interface_property_threshold: 8,

// Namespace-specific thresholds  
namespace_declaration_threshold: 20,
```

### Framework Detection

```rust
// Supported TypeScript frameworks
framework_modules.insert("angular".to_string());
framework_modules.insert("nest".to_string());
framework_modules.insert("nestjs".to_string());
framework_modules.insert("typescript".to_string());
framework_modules.insert("next".to_string());
framework_modules.insert("nuxt".to_string());
framework_modules.insert("svelte".to_string());
```

## Usage Examples

### Basic Analysis
```bash
# Analyze TypeScript files with enhanced detection
cargo run --features=production -- analyze ./src --output-format html

# TypeScript-specific configuration
cargo run --features=production -- analyze ./src --config typescript-react.toml
```

### Configuration File
```toml
# typescript-analysis.toml
[analysis]
languages = ["typescript"]

[thresholds.typescript]
god_object_methods = 15
god_object_fields = 10
god_interface_properties = 8
god_namespace_declarations = 20

[frameworks]
detect_angular = true
detect_nestjs = true
exclude_dto_patterns = true
```

## Analysis Output

### God Object Detection
```json
{
    "issues": [
        {
            "type": "God Object",
            "severity": "High", 
            "description": "God Object detected: 'MassiveUserManager' has 22 methods and 12 fields. (Thresholds: methods>15, fields>10)",
            "file": "src/services/user-manager.ts",
            "line": 15,
            "suggestions": [
                "Split into UserService, SessionService, and PermissionService",
                "Extract data access logic to Repository pattern",
                "Use dependency injection for service composition"
            ]
        }
    ]
}
```

### God Interface Detection
```json
{
    "issues": [
        {
            "type": "God Interface",
            "severity": "Critical",
            "description": "God Interface detected: 'MassiveApplicationState' has 25 properties. Interfaces should be focused and cohesive. (Threshold: >8)",
            "file": "src/types/app-state.ts", 
            "line": 12,
            "suggestions": [
                "Split into UserState, SystemState, and UIState interfaces",
                "Use composition instead of large interfaces",
                "Consider using discriminated unions for different states"
            ]
        }
    ]
}
```

### God Namespace Detection
```json
{
    "issues": [
        {
            "type": "God Namespace", 
            "severity": "Medium",
            "description": "God Namespace detected: 'UtilityFunctions' has 25 declarations. Consider splitting into multiple namespaces. (Threshold: >20)",
            "file": "src/utils/utilities.ts",
            "line": 8,
            "suggestions": [
                "Split into DateUtils, ValidationUtils, CryptoUtils namespaces",
                "Consider using separate modules instead of large namespaces",
                "Group related functions by domain"
            ]
        }
    ]
}
```

## Framework-Specific Analysis

### Angular Applications
- **Component Analysis**: Large Angular components are analyzed but framework components are excluded from God Object detection
- **Service Detection**: Injectable services are recognized and given higher thresholds
- **Module Analysis**: Angular modules are excluded from standard class analysis

### NestJS Applications  
- **Controller Analysis**: NestJS controllers are recognized and given appropriate thresholds
- **Service Analysis**: Injectable services are properly categorized
- **DTO Recognition**: Classes with validation decorators are identified as DTOs

### React with TypeScript
- **Component Props**: Large prop interfaces are detected as potential God Interfaces
- **Hook Analysis**: Custom hooks with many dependencies are flagged
- **Context Analysis**: Large context objects are identified

## Performance Considerations

### Optimizations
- **Query Caching**: TypeScript tree-sitter queries are compiled once and reused
- **Parallel Analysis**: Classes, interfaces, and namespaces are analyzed in parallel
- **Memory Efficiency**: Large ASTs are processed in chunks to manage memory usage

### Scalability
- **Large Codebases**: Tested with TypeScript codebases up to 100k+ lines
- **Complex Type Systems**: Handles deeply nested generic types and complex inheritance
- **Framework Scale**: Works with large Angular and NestJS applications

## Limitations and Future Improvements

### Current Limitations
1. **Cross-file Analysis**: Current analysis is file-scoped, doesn't track dependencies across files
2. **Type System Analysis**: Limited semantic analysis of TypeScript type system
3. **Dynamic Analysis**: No runtime behavior analysis, only static structure
4. **Third-party Types**: Limited analysis of external library type definitions

### Planned Improvements
1. **Project-wide Analysis**: Cross-file dependency tracking and analysis
2. **Type System Integration**: Integration with TypeScript compiler API for semantic analysis  
3. **Performance Metrics**: Integration with TypeScript performance tracing
4. **Advanced Patterns**: Detection of more complex TypeScript patterns (mapped types, template literals)

## Testing and Validation

### Test Coverage
- **Unit Tests**: Comprehensive test suite for all TypeScript constructs
- **Integration Tests**: End-to-end testing with real TypeScript projects
- **Framework Tests**: Specific test cases for Angular, NestJS, and React applications
- **Edge Cases**: Testing of complex generic types, deeply nested structures, and edge cases

### Validation Approach
- **Real-world Projects**: Tested against popular open-source TypeScript projects
- **Framework Applications**: Validated with actual Angular and NestJS applications
- **Performance Testing**: Benchmarked against large TypeScript codebases
- **Accuracy Validation**: Manual review of detection accuracy and false positive rates

## Conclusion

The enhanced TypeScript analysis in Uveddi provides comprehensive, framework-aware analysis that goes far beyond basic parsing. With sophisticated anti-pattern detection, TypeScript-specific construct support, and intelligent framework recognition, Uveddi now offers professional-grade TypeScript code analysis capabilities.

The implementation successfully addresses the limitations identified in ISSUES_TO_FIX.md and provides a foundation for continued improvement and expansion of TypeScript analysis features.