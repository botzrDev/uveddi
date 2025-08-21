# GPT Dev Prompt: Real-World Uveddi Testing

## Objective
You are an expert software testing engineer tasked with comprehensively testing Uveddi, an architectural analysis tool, on real-world codebases to validate all functionality, identify issues, and ensure production readiness before release.

## System Context
- **Tool**: Uveddi v0.9.0-alpha - Rust-based architectural analysis tool
- **Location**: `/home/austingreen/Documents/botzr/projects/uveddi`
- **Current Status**: All core functionality implemented, documentation updated, ready for real-world validation

## Testing Framework

### Phase 1: Environment Validation (5 minutes)
```bash
# Verify system state
cd /home/austingreen/Documents/botzr/projects/uveddi
cargo --version && node --version && ollama --version

# Test core builds
cargo build --features=dev-core    # Should take ~13s
cargo build --features=community   # Should take ~19s

# Verify services can start
cargo run --features=community -- serve --port 8888 --rendering-port 3333 &
sleep 5
curl http://localhost:8888/health
curl http://localhost:3333/health
pkill -f "uveddi.*serve"
```

### Phase 2: Real-World Codebase Selection (10 minutes)
Test on diverse, representative codebases. Select 3-4 from these categories:

#### **Large-Scale Production Codebases**
1. **Rust Projects**:
   - `git clone https://github.com/tokio-rs/tokio` (async runtime - ~500k LOC)
   - `git clone https://github.com/actix/actix-web` (web framework - ~100k LOC)
   - `git clone https://github.com/serde-rs/serde` (serialization - ~50k LOC)

2. **Python Projects**:
   - `git clone https://github.com/django/django` (web framework - ~800k LOC)
   - `git clone https://github.com/psf/requests` (HTTP library - ~30k LOC)
   - `git clone https://github.com/pallets/flask` (micro framework - ~50k LOC)

3. **JavaScript/TypeScript Projects**:
   - `git clone https://github.com/microsoft/vscode` (editor - ~2M LOC)
   - `git clone https://github.com/facebook/react` (library - ~200k LOC)
   - `git clone https://github.com/vercel/next.js` (framework - ~300k LOC)

4. **Mixed Language Projects**:
   - `git clone https://github.com/microsoft/TypeScript` (TypeScript compiler)
   - `git clone https://github.com/elastic/elasticsearch` (search engine)

#### **Selection Criteria**
- At least one large codebase (>500k LOC)
- At least one multi-language repository
- Include both monorepo and single-language projects
- Mix of architectural patterns (microservices, monolith, library)

### Phase 3: Comprehensive Analysis Testing (45 minutes)

#### **Test 1: Basic CLI Analysis (10 minutes per codebase)**
```bash
# For each selected codebase:
CODEBASE_PATH="/path/to/codebase"
CODEBASE_NAME="codebase-name"

# Test all output formats
time cargo run --features=community -- analyze "$CODEBASE_PATH" \
  --output-format html \
  --output "reports/${CODEBASE_NAME}-analysis.html"

time cargo run --features=community -- analyze "$CODEBASE_PATH" \
  --output-format json \
  --output "reports/${CODEBASE_NAME}-analysis.json"

time cargo run --features=community -- analyze "$CODEBASE_PATH" \
  --output-format markdown \
  --output "reports/${CODEBASE_NAME}-analysis.md"

# Verify file sizes and content
ls -la reports/${CODEBASE_NAME}*
```

#### **Test 2: AI-Enhanced Analysis (10 minutes)**
```bash
# Ensure Ollama is running
ollama serve &
ollama pull deepseek-coder:6.7b

# Test AI integration
cargo run --features=community -- analyze "$CODEBASE_PATH" \
  --enable-ai \
  --ollama-model deepseek-coder:6.7b \
  --output-format html \
  --output "reports/${CODEBASE_NAME}-ai-enhanced.html"

# Verify AI insights are present
grep -i "ai_insight" "reports/${CODEBASE_NAME}-ai-enhanced.html"
```

#### **Test 3: Web Services and Dashboard (15 minutes)**
```bash
# Start all services
cargo run --features=community -- serve --port 8888 --rendering-port 3333 --development &
SERVER_PID=$!
sleep 10

# Test API endpoints
echo "=== Testing API Endpoints ==="
curl -s http://localhost:8888/health | jq .
curl -s http://localhost:3333/health | jq .

# Test analysis via API (if available)
# curl -X POST http://localhost:8888/api/v1/analyze \
#   -H "Content-Type: application/json" \
#   -d '{"path": "'$CODEBASE_PATH'", "format": "json"}'

# Test dashboard access
echo "=== Dashboard URLs ==="
echo "Main Dashboard: http://localhost:8888"
echo "Health Check: http://localhost:8888/health"
echo "Rendering Service: http://localhost:3333/health"

# Manual verification: Open browser to dashboard URLs
# Verify: Can navigate dashboard, view reports, interact with visualizations

# Cleanup
kill $SERVER_PID
```

#### **Test 4: Performance and Scale Testing (10 minutes)**
```bash
# Test on largest codebase
LARGE_CODEBASE="/path/to/largest/codebase"

# Measure performance
echo "=== Performance Testing ==="
time cargo run --features=community -- analyze "$LARGE_CODEBASE" \
  --output-format json \
  --output "reports/large-codebase-perf.json" 2>&1 | tee perf.log

# Check memory usage during analysis
# (Run in separate terminal: htop -p $(pgrep uveddi))

# Verify output quality
echo "=== Output Quality Check ==="
jq '.summary' reports/large-codebase-perf.json
```

### Phase 4: Issue Detection and Edge Cases (15 minutes)

#### **Test 5: Edge Case Handling**
```bash
# Test empty directory
mkdir -p test-cases/empty-dir
cargo run --features=community -- analyze test-cases/empty-dir

# Test single file
echo "fn main() { println!(\"Hello\"); }" > test-cases/single.rs
cargo run --features=community -- analyze test-cases/single.rs

# Test very large single file
python3 -c "
with open('test-cases/large-single.py', 'w') as f:
    for i in range(10000):
        f.write(f'def function_{i}():\n    pass\n\n')
"
cargo run --features=community -- analyze test-cases/large-single.py

# Test non-existent path
cargo run --features=community -- analyze /non/existent/path
```

#### **Test 6: Error Handling and Recovery**
```bash
# Test with corrupted files
echo "invalid rust syntax {{{" > test-cases/corrupted.rs
cargo run --features=community -- analyze test-cases/corrupted.rs

# Test permission issues (if applicable)
# chmod 000 test-cases/no-permission
# cargo run --features=community -- analyze test-cases/no-permission

# Test interrupt handling
timeout 5s cargo run --features=community -- analyze "$LARGE_CODEBASE"
```

### Phase 5: Report Quality Assessment (10 minutes)

#### **Manual Review Checklist**
For each generated report, verify:

**HTML Reports**:
- [ ] Loads without errors in browser
- [ ] All sections render correctly
- [ ] Interactive elements work (if any)
- [ ] Mermaid diagrams render properly
- [ ] CSS styling applied correctly
- [ ] Responsive design works

**JSON Reports**:
- [ ] Valid JSON structure
- [ ] All expected fields present
- [ ] Data makes sense for analyzed codebase
- [ ] Issue counts reasonable
- [ ] Metadata accurate

**Analysis Quality**:
- [ ] Anti-patterns detected make sense
- [ ] File paths are correct
- [ ] Issue severity levels appropriate
- [ ] AI insights relevant (if enabled)
- [ ] Dependency analysis accurate

### Phase 6: Critical Issue Documentation (5 minutes)

#### **Issue Categories to Track**
1. **Functionality Bugs**
   - Analysis failures
   - Incorrect results
   - Service crashes

2. **Performance Issues**
   - Excessive memory usage
   - Slow analysis times
   - Service timeouts

3. **Usability Problems**
   - Confusing error messages
   - Missing functionality
   - Poor report quality

4. **Integration Issues**
   - AI service failures
   - Rendering problems
   - Dashboard errors

## Success Criteria

### **Must Pass (Blockers)**
- [ ] All selected codebases analyze without crashes
- [ ] All output formats generate valid files
- [ ] Web services start and respond to health checks
- [ ] Dashboard loads and displays basic functionality
- [ ] AI integration works when Ollama is available

### **Should Pass (Important)**
- [ ] Analysis completes in reasonable time (<5 min for 100k LOC)
- [ ] Reports contain meaningful, accurate insights
- [ ] Error handling is graceful with helpful messages
- [ ] Memory usage stays reasonable (<8GB for large codebases)
- [ ] All documented features work as described

### **Nice to Have**
- [ ] Performance meets or exceeds documented benchmarks
- [ ] Reports are publication-ready quality
- [ ] Dashboard provides excellent user experience
- [ ] AI insights add significant value

## Output Requirements

### **Immediate Actions**
1. **Critical Issues**: Stop testing and report immediately
2. **Performance Issues**: Document with specific metrics
3. **Quality Issues**: Capture screenshots/examples

### **Final Report Structure**
```markdown
# Uveddi Real-World Testing Report

## Executive Summary
- Codebases tested: [list]
- Critical issues found: [count]
- Overall readiness: [Ready/Needs Work/Not Ready]

## Test Results by Codebase
### [Codebase Name]
- Size: [LOC]
- Languages: [list]
- Analysis time: [duration]
- Issues found: [count]
- Quality assessment: [rating]
- Notable findings: [list]

## Issues Found
### Critical (Release Blockers)
### Important (Should Fix)
### Minor (Nice to Fix)

## Performance Analysis
### Build Times
### Analysis Performance
### Memory Usage
### Service Response Times

## Recommendations
### Before Release
### Post-Release
### Future Improvements
```

## Emergency Procedures

### **If Critical Issues Found**
1. **Document immediately** with:
   - Exact command that caused issue
   - Error message/stack trace
   - System state before issue
   - Steps to reproduce

2. **Isolate the issue**:
   - Test with minimal reproduction case
   - Verify it's not environment-specific
   - Check if workaround exists

3. **Stop testing** that component and move to others

### **If Performance Unacceptable**
1. **Measure precisely**:
   - Use `time` command for duration
   - Monitor memory with `htop`/`top`
   - Check disk I/O with `iotop`

2. **Test with different features**:
   - Try `dev-core` vs `community`
   - Test without AI integration
   - Test with smaller subset

### **If Service Issues**
1. **Check logs**: Look for error messages in console output
2. **Verify ports**: Ensure no conflicts with other services
3. **Test individually**: Start API and rendering services separately

## Tools and Commands Reference

### **Quick Commands**
```bash
# Fast status check
curl -s http://localhost:8888/health | jq .status
curl -s http://localhost:3333/health | jq .status

# Monitor resources
htop -p $(pgrep uveddi)

# Quick analysis
cargo run --features=dev-core -- analyze /path/to/code --output-format json

# Service restart
pkill -f "uveddi.*serve"
cargo run --features=community -- serve --port 8888 --rendering-port 3333 &
```

### **Debug Commands**
```bash
# Verbose analysis
RUST_LOG=debug cargo run --features=community -- analyze /path/to/code

# Check feature flags
cargo build --features=community --verbose

# Verify dependencies
cargo tree --features=community
```

## Expected Timeline
- **Setup**: 5 minutes
- **Codebase selection**: 10 minutes  
- **Core testing**: 45 minutes
- **Edge cases**: 15 minutes
- **Quality review**: 10 minutes
- **Documentation**: 5 minutes
- **Total**: ~90 minutes

## Final Notes
- **Be thorough but efficient** - focus on finding real issues
- **Document everything** - even small issues may indicate larger problems  
- **Test like a user** - try workflows that actual users would follow
- **Think about production** - consider real-world usage scenarios
- **Stay objective** - report both positive and negative findings

Focus on finding issues that would impact real users in production environments. The goal is to ensure Uveddi is truly ready for release with high confidence in its stability, performance, and usability.