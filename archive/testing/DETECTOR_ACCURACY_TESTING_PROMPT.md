# GPT Developer Prompt: Uveddi Detector Accuracy Testing

## Mission

Perform comprehensive accuracy testing of all Uveddi detectors on real-world codebases to validate detection quality, measure false positive/negative rates, and ensure production readiness.

## Your Role

You are a senior quality assurance engineer specializing in static analysis tool validation. Your task is to:

1. Test all Uveddi detectors against real-world open-source codebases
2. Manually verify each detected issue for accuracy
3. Identify false positives and false negatives
4. Measure detector precision, recall, and F1 scores
5. Provide detailed accuracy reports with recommendations

## Prerequisites

### 1. Setup Uveddi
```bash
# Clone and build Uveddi
git clone https://github.com/botzrDev/uveddi.git
cd uveddi
cargo build --release --features minimal

# Verify installation
./target/release/uveddi --version
# Expected: uveddi 1.0.0

# Run doctor to ensure everything is configured
./target/release/uveddi doctor --fix
```

### 2. Prepare Test Repositories

Clone the following real-world codebases for testing:

```bash
mkdir ~/uveddi-accuracy-testing
cd ~/uveddi-accuracy-testing

# Rust projects (varying complexity)
git clone https://github.com/rust-lang/cargo.git           # Large, well-maintained
git clone https://github.com/tokio-rs/tokio.git            # Async runtime
git clone https://github.com/serde-rs/serde.git            # Serialization library
git clone https://github.com/clap-rs/clap.git              # CLI library

# Python projects
git clone https://github.com/django/django.git             # Web framework
git clone https://github.com/pallets/flask.git             # Micro framework
git clone https://github.com/psf/requests.git              # HTTP library
git clone https://github.com/ansible/ansible.git           # Automation tool

# JavaScript/TypeScript projects  
git clone https://github.com/facebook/react.git            # UI library
git clone https://github.com/microsoft/vscode.git          # Code editor
git clone https://github.com/expressjs/express.git         # Web framework
git clone https://github.com/nestjs/nest.git               # TypeScript framework

# Projects known to have issues (for testing)
git clone https://github.com/minimaxir/big-list-of-naughty-strings.git  # Edge cases
```

## Testing Protocol

### Phase 1: Baseline Analysis (All Detectors)

For each test repository, run complete analysis:

```bash
cd ~/uveddi-accuracy-testing/[repository-name]

# Run full analysis with all detectors
uveddi analyze . \
  --output-format json \
  --output ~/uveddi-accuracy-testing/results/[repo-name]-full.json

# Also generate human-readable report
uveddi analyze . \
  --output-format markdown \
  --output ~/uveddi-accuracy-testing/results/[repo-name]-full.md
```

**Record for each repository:**
- Total files analyzed
- Total issues found
- Issues by detector type
- Issues by severity (high/medium/low)
- Analysis duration
- Memory usage

### Phase 2: Detector-Specific Deep Dive

Test each detector individually and validate accuracy.

---

## Detector 1: God Object Detector

### What It Should Detect
- Classes/modules with too many responsibilities (>20 methods, >500 LOC)
- High coupling between unrelated functionalities
- Classes doing multiple unrelated things

### Testing Protocol

```bash
# Rust example
uveddi analyze cargo/src --output-format json > results/cargo-god-object.json

# Python example  
uveddi analyze django/django --output-format json > results/django-god-object.json

# JavaScript example
uveddi analyze react/packages --output-format json > results/react-god-object.json
```

### Manual Verification Process

For each detected God Object issue:

1. **Open the file** and review the class/module
2. **Count responsibilities** - How many distinct things does it do?
3. **Check cohesion** - Are the methods related to each other?
4. **Determine verdict**:
   - ✅ **True Positive**: Class genuinely has too many responsibilities
   - ❌ **False Positive**: Class is complex but cohesive (legitimate)
   - ⚠️ **Borderline**: Debatable, needs context

### Recording Template

```markdown
## God Object Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Class/Module:** `ClassName`
- **Metrics:**
  - Methods: [count]
  - LOC: [count]
  - Responsibilities identified: [list]
- **Manual Review:**
  - Cohesion: [High/Medium/Low]
  - Single Responsibility Principle violated: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Confidence:** [High/Medium/Low]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- Borderline: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >80% (at most 20% false positives)
- **Recall:** Cannot easily measure without manual codebase audit
- **Severity accuracy:** Issues marked as high severity should be clearly problematic

---

## Detector 2: Code Duplication Detector

### What It Should Detect
- Identical or near-identical code blocks (>6 lines)
- Copy-pasted functions with minor variations
- Repeated patterns that should be abstracted

### Testing Protocol

```bash
# Look for duplication in large codebases
uveddi analyze tokio/tokio --output-format json > results/tokio-duplication.json
uveddi analyze vscode/src --output-format json > results/vscode-duplication.json
uveddi analyze ansible/lib --output-format json > results/ansible-duplication.json
```

### Manual Verification Process

For each duplication issue:

1. **Compare the code blocks** side by side
2. **Calculate similarity** - What percentage is identical?
3. **Check context** - Are they legitimately separate concerns?
4. **Consider language idioms** - Some patterns are common/necessary
5. **Determine verdict**:
   - ✅ **True Positive**: Genuine duplication that should be refactored
   - ❌ **False Positive**: Similar but legitimately separate (e.g., boilerplate, test fixtures)
   - ⚠️ **Borderline**: Could go either way

### Recording Template

```markdown
## Code Duplication Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File1:Line1] ↔ [File2:Line2]
- **Block size:** [N lines]
- **Similarity:** [percentage]%
- **Manual Review:**
  - Actual similarity: [percentage]%
  - Same functionality: [Yes/No]
  - Can be abstracted: [Yes/No/Maybe]
  - Language idiom/pattern: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]

#### Common False Positive Patterns
1. [Pattern description]
2. [Pattern description]
```

### Acceptance Criteria
- **Precision target:** >75% (duplication detection is inherently noisy)
- **Minimum block size:** Should not flag blocks <6 lines
- **Similarity threshold:** Should be configurable

---

## Detector 3: Dead Code Detector

### What It Should Detect
- Unused functions, methods, classes
- Unreachable code paths
- Unused imports/variables (with caution for public APIs)

### Testing Protocol

```bash
# Test on libraries (challenging - public APIs may appear unused)
uveddi analyze serde/serde --output-format json > results/serde-deadcode.json

# Test on applications (easier - clear entry points)
uveddi analyze flask/src/flask --output-format json > results/flask-deadcode.json
```

### Manual Verification Process

For each dead code issue:

1. **Search for usages** - Use `grep -r "function_name"` across codebase
2. **Check if public API** - Is this exported for external use?
3. **Check test coverage** - Is it only used in tests?
4. **Check build configuration** - Is it used in specific build configs?
5. **Determine verdict**:
   - ✅ **True Positive**: Genuinely unused, can be removed
   - ❌ **False Positive**: Used but not detected (reflection, public API, macros, etc.)
   - ⚠️ **Borderline**: Questionable (e.g., debug code, planned features)

### Recording Template

```markdown
## Dead Code Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Item:** `function_name` / `class_name`
- **Type:** [Function/Method/Class/Import]
- **Manual Review:**
  - Found usages: [count]
  - Public API: [Yes/No]
  - Used in tests: [Yes/No]
  - Build-specific: [Yes/No]
  - Dynamic usage (reflection/macros): [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]

#### Known Limitations
- Cannot detect: [list patterns that cause false positives]
```

### Acceptance Criteria
- **Precision target:** >85% for applications, >70% for libraries
- **Should handle:** Public APIs, dynamic invocation, macros
- **Configuration:** Should have flags for library vs application mode

---

## Detector 4: Large Class Detector

### What It Should Detect
- Classes exceeding LOC thresholds (>500 LOC default)
- Classes with too many methods (>20)
- Classes with too many fields (>15)

### Testing Protocol

```bash
uveddi analyze cargo/src --output-format json > results/cargo-large-class.json
uveddi analyze django/django --output-format json > results/django-large-class.json
uveddi analyze nest/packages --output-format json > results/nest-large-class.json
```

### Manual Verification Process

For each large class issue:

1. **Count actual LOC** - Verify the reported size
2. **Assess complexity** - Is size justified by complexity?
3. **Check if cohesive** - Do all parts belong together?
4. **Domain appropriateness** - Some domains require large classes (e.g., state machines)
5. **Determine verdict**:
   - ✅ **True Positive**: Class is too large and should be split
   - ❌ **False Positive**: Size is justified (e.g., generated code, domain model)
   - ⚠️ **Borderline**: Large but acceptable in context

### Recording Template

```markdown
## Large Class Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Class:** `ClassName`
- **Metrics:**
  - LOC: [count] (threshold: [threshold])
  - Methods: [count]
  - Fields: [count]
- **Manual Review:**
  - Actual LOC (excluding comments): [count]
  - Cohesion level: [High/Medium/Low]
  - Domain complexity justifies size: [Yes/No]
  - Generated code: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >80%
- **Thresholds should be:** Configurable per language
- **Should exclude:** Generated code, auto-generated parsers

---

## Detector 5: Long Methods Detector

### What It Should Detect
- Methods exceeding LOC threshold (>50 LOC default)
- Methods with high cyclomatic complexity
- Methods doing too many things

### Testing Protocol

```bash
uveddi analyze tokio/tokio/src --output-format json > results/tokio-long-methods.json
uveddi analyze requests/requests --output-format json > results/requests-long-methods.json
uveddi analyze express/lib --output-format json > results/express-long-methods.json
```

### Manual Verification Process

For each long method issue:

1. **Count actual LOC** - Verify length
2. **Assess complexity** - Is it actually complex or just long?
3. **Check for single responsibility** - Does it do one thing?
4. **Consider error handling** - Is length due to proper error handling?
5. **Determine verdict**:
   - ✅ **True Positive**: Method is too long and should be refactored
   - ❌ **False Positive**: Length is justified (e.g., error handling, state machine)
   - ⚠️ **Borderline**: Long but acceptable

### Recording Template

```markdown
## Long Methods Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Method:** `method_name`
- **Metrics:**
  - LOC: [count] (threshold: [threshold])
  - Cyclomatic complexity: [count if available]
- **Manual Review:**
  - Actual LOC (excluding comments): [count]
  - Number of responsibilities: [count]
  - Primarily error handling: [Yes/No]
  - Can be split logically: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >75%
- **Should account for:** Error handling, guard clauses
- **Should flag:** Methods with multiple distinct responsibilities

---

## Detector 6: Tight Coupling Detector

### What It Should Detect
- High number of dependencies between modules
- Classes that depend on too many other classes
- Circular dependencies (detected separately)

### Testing Protocol

```bash
uveddi analyze cargo/crates --output-format json > results/cargo-coupling.json
uveddi analyze flask/src --output-format json > results/flask-coupling.json
uveddi analyze react/packages --output-format json > results/react-coupling.json
```

### Manual Verification Process

For each tight coupling issue:

1. **Review dependencies** - Are they all necessary?
2. **Check abstraction level** - Is coupling to interfaces or implementations?
3. **Assess impact** - Would changes propagate widely?
4. **Consider architecture** - Is this expected in this layer?
5. **Determine verdict**:
   - ✅ **True Positive**: Genuinely too tightly coupled
   - ❌ **False Positive**: Normal for the architectural layer
   - ⚠️ **Borderline**: High coupling but justified

### Recording Template

```markdown
## Tight Coupling Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Module:** `module_name`
- **Dependencies:** [count]
- **Manual Review:**
  - Unique dependencies: [count]
  - Dependencies on abstractions: [count]
  - Dependencies on implementations: [count]
  - Architectural layer: [Presentation/Business/Data/etc.]
  - Coupling is appropriate for layer: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >70% (coupling is context-dependent)
- **Should consider:** Architectural patterns (DDD, hexagonal, etc.)
- **Configurable thresholds:** By project type

---

## Detector 7: Cyclic Dependencies Detector

### What It Should Detect
- Module A imports B, B imports A (direct cycles)
- Longer cycles: A → B → C → A
- Package-level cycles

### Testing Protocol

```bash
# Test on modular codebases
uveddi analyze serde/serde_derive --output-format json > results/serde-cycles.json
uveddi analyze django/django --output-format json > results/django-cycles.json
uveddi analyze vscode/src --output-format json > results/vscode-cycles.json
```

### Manual Verification Process

For each cycle detected:

1. **Trace the cycle** - Follow the import chain
2. **Verify existence** - Confirm all links in the cycle exist
3. **Check if problematic** - Some cycles are benign (e.g., type-only)
4. **Consider language features** - Some languages handle cycles better
5. **Determine verdict**:
   - ✅ **True Positive**: Real cycle that causes problems
   - ❌ **False Positive**: Not actually a cycle or benign
   - ⚠️ **Borderline**: Cycle exists but may not be problematic

### Recording Template

```markdown
## Cyclic Dependencies Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: Cycle of length [N]
- **Cycle path:** ModuleA → ModuleB → ModuleC → ModuleA
- **Manual Review:**
  - Verified all links: [Yes/No]
  - Type of imports: [Runtime/Type-only/Mixed]
  - Causes compilation issues: [Yes/No]
  - Causes runtime issues: [Yes/No]
  - Can be broken easily: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >90% (should be very accurate)
- **Should detect:** All cycle lengths (2, 3, 4+)
- **Should differentiate:** Type-only vs runtime dependencies

---

## Detector 8: Magic Values Detector

### What It Should Detect
- Hardcoded numbers (except 0, 1, -1, common constants)
- Hardcoded strings used as identifiers/keys
- Repeated literal values that should be constants

### Testing Protocol

```bash
uveddi analyze clap/src --output-format json > results/clap-magic-values.json
uveddi analyze ansible/lib --output-format json > results/ansible-magic-values.json
uveddi analyze express/lib --output-format json > results/express-magic-values.json
```

### Manual Verification Process

For each magic value issue:

1. **Examine the value** - Is it truly "magic" or self-documenting?
2. **Check usage frequency** - Is it repeated? Used once?
3. **Consider context** - Is it a domain-specific constant?
4. **Check if documented** - Is there a comment explaining it?
5. **Determine verdict**:
   - ✅ **True Positive**: Should be a named constant
   - ❌ **False Positive**: Self-documenting or acceptable (e.g., `array[0]`, test values)
   - ⚠️ **Borderline**: Could be improved but not critical

### Recording Template

```markdown
## Magic Values Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Value:** `[value]`
- **Type:** [Number/String]
- **Manual Review:**
  - Self-documenting: [Yes/No]
  - Repeated in codebase: [count times]
  - Has nearby comment: [Yes/No]
  - Common constant (0,1,-1,etc.): [Yes/No]
  - Test fixture value: [Yes/No]
  - Should be named constant: [Yes/No]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]

#### Noise Analysis
- Common false positives: [list patterns]
- Recommended exclusions: [list values to ignore]
```

### Acceptance Criteria
- **Precision target:** >60% (very noisy detector by nature)
- **Should exclude:** 0, 1, -1, 2 (powers of 2), 100 (percentages)
- **Should flag:** Repeated values, complex numbers without context

---

## Security Detectors Testing

### Security Detector 1: SQL Injection Detection

### What It Should Detect
- String concatenation in SQL queries
- Unsanitized user input in queries
- Missing parameterized queries
- Dynamic query construction without proper escaping

### Testing Protocol

```bash
# Test on web frameworks and DB libraries
uveddi analyze django/django/db --output-format json > results/django-sql-injection.json
uveddi analyze flask/src/flask --output-format json > results/flask-sql-injection.json

# Create test cases with known vulnerabilities
cat > test-sql-injection.py << 'EOF'
import sqlite3

# VULNERABLE: String concatenation
def get_user_by_id_bad(user_id):
    conn = sqlite3.connect('db.sqlite')
    cursor = conn.cursor()
    query = "SELECT * FROM users WHERE id = " + user_id  # Should detect
    cursor.execute(query)
    return cursor.fetchone()

# VULNERABLE: F-string formatting
def get_user_by_name_bad(name):
    conn = sqlite3.connect('db.sqlite')
    cursor = conn.cursor()
    query = f"SELECT * FROM users WHERE name = '{name}'"  # Should detect
    cursor.execute(query)
    return cursor.fetchone()

# SAFE: Parameterized query
def get_user_by_id_good(user_id):
    conn = sqlite3.connect('db.sqlite')
    cursor = conn.cursor()
    query = "SELECT * FROM users WHERE id = ?"
    cursor.execute(query, (user_id,))  # Should NOT detect
    return cursor.fetchone()

# SAFE: ORM usage
from django.contrib.auth.models import User
def get_user_orm(user_id):
    return User.objects.get(id=user_id)  # Should NOT detect
EOF

uveddi analyze test-sql-injection.py --output-format json > results/sql-injection-test.json
```

### Manual Verification Process

For each SQL injection issue:

1. **Identify the query construction** - How is the SQL built?
2. **Trace user input** - Is the data from untrusted sources?
3. **Check sanitization** - Is there proper escaping/validation?
4. **Verify exploitability** - Can this actually be exploited?
5. **Determine verdict**:
   - ✅ **True Positive**: Genuine SQL injection vulnerability
   - ❌ **False Positive**: Safe (parameterized, ORM, static query)
   - ⚠️ **Borderline**: Depends on input source

### Recording Template

```markdown
## SQL Injection Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Query construction:** [String concat / f-string / template / parameterized]
- **Manual Review:**
  - User input used: [Yes/No/Unknown]
  - Sanitization present: [Yes/No]
  - Parameterized query: [Yes/No]
  - ORM usage: [Yes/No]
  - Exploitable: [Yes/No/Maybe]
- **Severity:** [Critical/High/Medium/Low]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]

#### Test Case Results
- Detected vulnerable patterns: [count]/[expected]
- Missed vulnerabilities (false negatives): [count]
- Incorrectly flagged safe code: [count]
```

### Acceptance Criteria
- **Precision target:** >85%
- **Must detect:** String concatenation, f-strings with user input
- **Must NOT flag:** Parameterized queries, ORM usage, static queries

---

### Security Detector 2: XSS (Cross-Site Scripting) Detection

### What It Should Detect
- Unescaped user input in HTML output
- Dangerous DOM manipulation
- `innerHTML` usage with user data
- Template injection vulnerabilities

### Testing Protocol

```bash
# Test on frontend frameworks
uveddi analyze react/packages --output-format json > results/react-xss.json
uveddi analyze express/lib --output-format json > results/express-xss.json

# Create test cases
cat > test-xss.js << 'EOF'
// VULNERABLE: Direct innerHTML
function displayUserComment(comment) {
    document.getElementById('comment').innerHTML = comment;  // Should detect
}

// VULNERABLE: Unescaped template
function renderUser(name) {
    return `<div>Welcome, ${name}!</div>`;  // Should detect if name is from user
}

// SAFE: React escapes by default
function UserGreeting({ name }) {
    return <div>Welcome, {name}!</div>;  // Should NOT detect
}

// SAFE: Proper escaping
function displayUserCommentSafe(comment) {
    const escaped = escapeHtml(comment);
    document.getElementById('comment').innerHTML = escaped;  // Should NOT detect
}

// SAFE: textContent instead of innerHTML
function displayText(text) {
    document.getElementById('comment').textContent = text;  // Should NOT detect
}
EOF

uveddi analyze test-xss.js --output-format json > results/xss-test.json
```

### Manual Verification Process

For each XSS issue:

1. **Identify the sink** - Where is data output? (innerHTML, template, etc.)
2. **Trace data source** - Is it user-controlled?
3. **Check escaping** - Is there proper sanitization?
4. **Assess context** - HTML, JS, URL, CSS context?
5. **Determine verdict**:
   - ✅ **True Positive**: Real XSS vulnerability
   - ❌ **False Positive**: Properly escaped or not user-controlled
   - ⚠️ **Borderline**: Unclear data flow

### Recording Template

```markdown
## XSS Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Sink:** [innerHTML / template / DOM manipulation]
- **Manual Review:**
  - User-controlled data: [Yes/No/Unknown]
  - Escaping function used: [Yes/No] - [function name]
  - Framework auto-escaping: [Yes/No]
  - Context: [HTML/JavaScript/URL/CSS]
  - Exploitable: [Yes/No/Maybe]
- **Severity:** [Critical/High/Medium/Low]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >80%
- **Must detect:** innerHTML with user data, unsafe template usage
- **Must NOT flag:** Framework-escaped output, textContent usage

---

### Security Detector 3: Command Injection Detection

### What It Should Detect
- Unsanitized user input in system commands
- Shell command construction with string concatenation
- Use of `eval()` or similar with user data
- Unsafe subprocess/exec usage

### Testing Protocol

```bash
# Test on system interaction code
uveddi analyze ansible/lib --output-format json > results/ansible-cmd-injection.json

# Create test cases
cat > test-cmd-injection.py << 'EOF'
import subprocess
import os

# VULNERABLE: Shell=True with user input
def ping_host_bad(hostname):
    subprocess.call(f"ping -c 1 {hostname}", shell=True)  # Should detect

# VULNERABLE: String concatenation
def list_files_bad(directory):
    os.system("ls -la " + directory)  # Should detect

# SAFE: Argument list without shell
def ping_host_good(hostname):
    subprocess.call(["ping", "-c", "1", hostname])  # Should NOT detect

# SAFE: Validated input
def ping_host_validated(hostname):
    import re
    if re.match(r'^[a-zA-Z0-9.-]+$', hostname):
        subprocess.call(["ping", "-c", "1", hostname])  # Should NOT detect
EOF

uveddi analyze test-cmd-injection.py --output-format json > results/cmd-injection-test.json
```

### Manual Verification Process

For each command injection issue:

1. **Identify command execution** - What function is used?
2. **Check for shell=True** - Is shell interpretation enabled?
3. **Trace user input** - Is the command from untrusted source?
4. **Verify sanitization** - Is there input validation?
5. **Determine verdict**:
   - ✅ **True Positive**: Real command injection vulnerability
   - ❌ **False Positive**: Safe (array args, validated input, static command)
   - ⚠️ **Borderline**: Depends on input validation

### Recording Template

```markdown
## Command Injection Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Function:** [subprocess.call / os.system / eval / exec]
- **Manual Review:**
  - Uses shell=True: [Yes/No]
  - User input in command: [Yes/No/Unknown]
  - Input validation: [Yes/No] - [method]
  - Uses argument array: [Yes/No]
  - Exploitable: [Yes/No/Maybe]
- **Severity:** [Critical/High/Medium/Low]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >85%
- **Must detect:** shell=True with user input, string concatenation in commands
- **Must NOT flag:** Argument arrays, validated input, static commands

---

### Security Detector 4: Path Traversal Detection

### What It Should Detect
- User input in file paths without validation
- Missing path sanitization before file operations
- Unsafe use of `../` in paths
- Directory traversal vulnerabilities

### Testing Protocol

```bash
# Create test cases
cat > test-path-traversal.py << 'EOF'
import os

# VULNERABLE: Direct user input in path
def read_user_file_bad(filename):
    with open(f"/var/data/{filename}", 'r') as f:  # Should detect
        return f.read()

# VULNERABLE: os.path.join with user input
def read_file_bad2(filename):
    path = os.path.join("/var/data", filename)  # Should detect
    with open(path, 'r') as f:
        return f.read()

# SAFE: Path validation
def read_user_file_good(filename):
    # Resolve and validate path
    base = os.path.abspath("/var/data")
    requested = os.path.abspath(os.path.join(base, filename))
    if not requested.startswith(base):  # Should NOT detect
        raise ValueError("Invalid path")
    with open(requested, 'r') as f:
        return f.read()

# SAFE: Whitelist approach
def read_allowed_file(filename):
    allowed = ["config.txt", "data.json"]
    if filename not in allowed:  # Should NOT detect
        raise ValueError("File not allowed")
    with open(f"/var/data/{filename}", 'r') as f:
        return f.read()
EOF

uveddi analyze test-path-traversal.py --output-format json > results/path-traversal-test.json
```

### Manual Verification Process

For each path traversal issue:

1. **Identify file operation** - What function accesses the filesystem?
2. **Trace path construction** - How is the path built?
3. **Check validation** - Is there path sanitization?
4. **Test exploitability** - Can `../` escape the intended directory?
5. **Determine verdict**:
   - ✅ **True Positive**: Real path traversal vulnerability
   - ❌ **False Positive**: Properly validated or not user-controlled
   - ⚠️ **Borderline**: Unclear if validation is sufficient

### Recording Template

```markdown
## Path Traversal Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **File operation:** [open / read / write / os.path.join]
- **Manual Review:**
  - User-controlled path: [Yes/No/Unknown]
  - Path validation: [Yes/No] - [method]
  - Whitelist approach: [Yes/No]
  - Realpath check: [Yes/No]
  - Exploitable with ../ : [Yes/No/Maybe]
- **Severity:** [Critical/High/Medium/Low]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]
```

### Acceptance Criteria
- **Precision target:** >80%
- **Must detect:** User input in paths without validation
- **Must NOT flag:** Validated paths, whitelisted files, realpath checks

---

### Security Detector 5: Hardcoded Secrets Detection

### What It Should Detect
- API keys in source code
- Passwords in plain text
- Private keys and certificates
- Database credentials
- Authentication tokens

### Testing Protocol

```bash
# Test on real codebases (should find few/no secrets in good projects)
uveddi analyze requests/requests --output-format json > results/requests-secrets.json

# Create test cases
cat > test-secrets.py << 'EOF'
# VULNERABLE: Hardcoded API key
API_KEY = "sk_live_51H1234567890abcdef"  # Should detect

# VULNERABLE: Hardcoded password
DATABASE_PASSWORD = "MySecretP@ssw0rd123"  # Should detect

# VULNERABLE: AWS credentials
AWS_ACCESS_KEY = "AKIAIOSFODNN7EXAMPLE"  # Should detect
AWS_SECRET = "wJalrXUtnFEMI/K7MDENG/bPxRfiCYEXAMPLEKEY"  # Should detect

# SAFE: Environment variable
API_KEY = os.getenv('API_KEY')  # Should NOT detect

# SAFE: Configuration file
config = load_config('/etc/app/config.yml')  # Should NOT detect
API_KEY = config['api_key']

# SAFE: Placeholder/example
EXAMPLE_KEY = "your-api-key-here"  # Should NOT detect (or low severity)
EOF

uveddi analyze test-secrets.py --output-format json > results/secrets-test.json
```

### Manual Verification Process

For each hardcoded secret:

1. **Identify the secret type** - API key, password, token, etc.
2. **Check if real** - Is it an actual secret or a placeholder?
3. **Assess entropy** - High entropy suggests real secret
4. **Check context** - Test code, documentation, or production?
5. **Determine verdict**:
   - ✅ **True Positive**: Real hardcoded secret
   - ❌ **False Positive**: Placeholder, example, or not a secret
   - ⚠️ **Borderline**: Could be a secret, needs verification

### Recording Template

```markdown
## Hardcoded Secrets Detector - [Repository Name]

### Issues Found: [N]

#### Issue #1: [File:Line]
- **Secret type:** [API key / Password / Token / Private key / Credentials]
- **Value pattern:** [Redacted pattern]
- **Manual Review:**
  - Entropy: [High/Medium/Low]
  - Looks like placeholder: [Yes/No]
  - In test/example code: [Yes/No]
  - Matches known pattern: [Yes/No] - [which pattern]
  - Real secret: [Yes/No/Unknown]
- **Severity:** [Critical/High/Medium/Low]
- **Verdict:** [True Positive / False Positive / Borderline]
- **Notes:** [Explanation]

#### Summary Statistics
- True Positives: [count] ([percentage]%)
- False Positives: [count] ([percentage]%)
- **Precision:** [TP / (TP + FP)]

#### Pattern Analysis
- Patterns correctly detected: [list]
- Patterns missed: [list]
- False positive patterns: [list]
```

### Acceptance Criteria
- **Precision target:** >90% (should be very accurate)
- **Must detect:** Common secret patterns (API keys, passwords, tokens)
- **Must NOT flag:** Placeholders like "your-api-key-here", env var usage

---

## Comprehensive Accuracy Report Format

After testing all detectors, compile a master report:

```markdown
# Uveddi Detector Accuracy Report

## Executive Summary

**Test Date:** [Date]
**Repositories Tested:** [Count] ([list])
**Total Issues Analyzed:** [Count]
**Overall Precision:** [Percentage]%

### Quick Results

| Detector | Issues Found | True Positives | False Positives | Precision | Status |
|----------|--------------|----------------|-----------------|-----------|--------|
| God Object | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Code Duplication | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Dead Code | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Large Class | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Long Methods | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Tight Coupling | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Cyclic Dependencies | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Magic Values | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| SQL Injection | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| XSS | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Command Injection | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Path Traversal | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |
| Hardcoded Secrets | [N] | [N] | [N] | [%] | [✅/⚠️/❌] |

**Status Legend:**
- ✅ Meets criteria (>target precision)
- ⚠️ Below target but usable
- ❌ Needs improvement

## Detailed Findings

[Include all detector-specific reports from above]

## Cross-Detector Analysis

### Correlation Analysis
- Repositories with most issues: [list top 5]
- Detectors with highest agreement: [analysis]
- Detectors with lowest agreement: [analysis]

### Language-Specific Performance

#### Rust Projects
- Best performing detectors: [list]
- Worst performing detectors: [list]
- Common false positive patterns: [list]

#### Python Projects
- Best performing detectors: [list]
- Worst performing detectors: [list]
- Common false positive patterns: [list]

#### JavaScript/TypeScript Projects
- Best performing detectors: [list]
- Worst performing detectors: [list]
- Common false positive patterns: [list]

## Performance Metrics

### Analysis Speed
- Average files/second: [N]
- Slowest detector: [name] ([time])
- Fastest detector: [name] ([time])

### Resource Usage
- Peak memory: [MB]
- Average memory: [MB]
- CPU usage: [%]

## Known Limitations Discovered

### God Object Detector
1. [Limitation 1]
2. [Limitation 2]

### Code Duplication Detector
1. [Limitation 1]
2. [Limitation 2]

[Continue for all detectors...]

## Recommendations

### High Priority Fixes
1. [Detector name]: [Issue] - [Recommended fix]
2. [Detector name]: [Issue] - [Recommended fix]

### Medium Priority Improvements
1. [Detector name]: [Issue] - [Recommended fix]
2. [Detector name]: [Issue] - [Recommended fix]

### Low Priority Enhancements
1. [Detector name]: [Issue] - [Recommended fix]
2. [Detector name]: [Issue] - [Recommended fix]

### Configuration Recommendations
- Suggested default thresholds: [list]
- Recommended exclusion patterns: [list]
- Language-specific configurations: [list]

## Conclusion

### Production Readiness Assessment

**Code Quality Detectors:**
- Ready for production: [list]
- Need tuning: [list]
- Not recommended: [list]

**Security Detectors:**
- Ready for production: [list]
- Need tuning: [list]
- Not recommended: [list]

### Overall Recommendation
[Final assessment and go/no-go decision]

## Appendix

### Test Methodology
[Detailed explanation of testing approach]

### Repositories Tested
[Full list with versions/commits]

### Test Cases
[Link to test case files]

### Raw Data
[Link to JSON results files]
```

---

## Success Criteria

### For Each Detector

**Minimum Acceptable Precision:**
- God Object: >80%
- Code Duplication: >75%
- Dead Code: >85% (apps), >70% (libs)
- Large Class: >80%
- Long Methods: >75%
- Tight Coupling: >70%
- Cyclic Dependencies: >90%
- Magic Values: >60%
- SQL Injection: >85%
- XSS: >80%
- Command Injection: >85%
- Path Traversal: >80%
- Hardcoded Secrets: >90%

**Performance Requirements:**
- Analysis speed: >10 files/second
- Memory usage: <500MB for large codebases
- No crashes or hangs

**Usability Requirements:**
- Clear issue descriptions
- Actionable recommendations
- Minimal configuration needed

### Overall Project Success

✅ **PASS Criteria:**
- All security detectors meet precision targets
- At least 80% of code quality detectors meet targets
- No critical bugs or crashes
- Performance within acceptable bounds

⚠️ **CONDITIONAL PASS:**
- 70-80% of detectors meet targets
- Known limitations documented
- Improvement plan in place

❌ **FAIL:**
- <70% of detectors meet targets
- Security detectors below target
- Frequent crashes or poor performance

---

## Deliverables

Submit the following:

1. **Master Accuracy Report** (comprehensive document)
2. **Per-Detector Analysis** (detailed findings)
3. **Test Case Suite** (reusable tests for future validation)
4. **Raw Data** (JSON files with all results)
5. **Recommendations Document** (prioritized fixes)
6. **Configuration Guide** (optimal settings discovered)

---

## Timeline Estimate

- **Setup:** 2-4 hours
- **Baseline testing:** 4-8 hours
- **Detector deep-dive:** 20-30 hours (2-3 hours per detector)
- **Analysis and reporting:** 8-12 hours
- **Total:** 34-54 hours (1-1.5 weeks full-time)

---

## Notes and Tips

1. **Be thorough but pragmatic** - Sample large result sets, don't verify every single issue
2. **Document patterns** - Note common false positives for future filtering
3. **Test edge cases** - Include test files with known issues
4. **Consider context** - Different project types have different norms
5. **Track time** - Note which detectors take longest to validate
6. **Ask questions** - If unsure, mark as borderline and explain why
7. **Be objective** - Base verdicts on technical merit, not personal preferences
8. **Suggest improvements** - If you see ways to reduce false positives, document them

---

## Final Checklist

Before submitting your report:

- [ ] Tested all 13 detectors (8 code quality + 5 security)
- [ ] Analyzed at least 10 real-world repositories
- [ ] Created and tested custom test cases for security detectors
- [ ] Calculated precision for each detector
- [ ] Documented all false positive patterns
- [ ] Identified detector limitations
- [ ] Provided prioritized recommendations
- [ ] Included performance metrics
- [ ] Compiled master accuracy report
- [ ] Delivered raw data files

---

**Good luck! Your thorough testing will ensure Uveddi delivers accurate, actionable results to users. 🔍**
