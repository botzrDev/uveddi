# UV-82 Chaos Engineering Research Prompt

## Context
You are a senior DevOps engineer and chaos engineering specialist tasked with implementing comprehensive chaos engineering capabilities for the Uveddi project - a Rust-based static code analysis and architectural visualization tool. The system includes:

- **Rust backend** with microservices architecture
- **Node.js rendering service** for diagram generation
- **TypeScript frontend** with React
- **SQLite database** for data persistence
- **Resilience patterns** already implemented (circuit breakers, retry logic, fallback mechanisms)

## Current State
The Uveddi project has:
✅ Basic resilience patterns implemented (`src/resilience/`)
✅ Performance testing framework (`src/analysis/performance/testing.rs`)
✅ Comprehensive test suite with >85% coverage
✅ CI/CD pipeline with GitHub Actions
✅ Monitoring and metrics collection (`src/monitoring/`)

## Research Objectives

### Primary Research Questions
1. **What are the best practices for chaos engineering in Rust microservices architectures?**
2. **How can we implement safe failure injection without affecting production systems?**
3. **What specific chaos experiments should we design for a code analysis and rendering service?**
4. **How do we measure and validate system recovery time and data integrity during chaos experiments?**
5. **What tools and frameworks are most suitable for Rust-based chaos engineering?**

### Technical Requirements
- Must integrate with existing Rust codebase and testing infrastructure
- Should work with tokio async runtime
- Must be safe for CI/CD pipeline execution
- Should provide detailed metrics and reporting
- Must support both unit-level and system-level chaos experiments

## Specific Research Areas

### 1. Chaos Engineering Tools and Frameworks
**Research Focus**: Evaluate tools specifically for Rust ecosystems
- `chaos-monkey-rs` and similar Rust-native tools
- `tokio-test` for async failure simulation
- Integration with existing `tower` middleware stack
- Custom failure injection implementations
- Compatibility with `tracing` and `metrics` crates

### 2. Failure Injection Patterns
**Research Focus**: Specific failure scenarios for our architecture
- **Network failures**: Simulate rendering service unavailability
- **Database failures**: SQLite connection drops and corruption scenarios
- **Memory pressure**: Simulate OOM conditions during large file analysis
- **CPU exhaustion**: Simulate high-load scenarios during concurrent analysis
- **Disk I/O failures**: File system errors during code scanning
- **Timeout scenarios**: Slow response simulation for external services

### 3. Safe Chaos Testing Implementation
**Research Focus**: How to implement chaos testing without production impact
- Test environment isolation strategies
- Feature flags for chaos experiment control
- Gradual rollout and blast radius limitation
- Automated rollback mechanisms
- Safety checks and circuit breakers for chaos experiments

### 4. Metrics and Observability
**Research Focus**: What to measure during chaos experiments
- **Recovery time metrics**: Time to detect, isolate, and recover from failures
- **Data integrity validation**: Ensuring no data corruption during failures
- **User experience impact**: Response time and error rate measurements
- **Resource utilization**: CPU, memory, and I/O during failure scenarios
- **Cascade failure detection**: Identifying failure propagation patterns

### 5. CI/CD Integration
**Research Focus**: Automating chaos experiments in development workflow
- GitHub Actions integration strategies
- Automated chaos testing in PR validation
- Performance regression detection through chaos experiments
- Reporting and alerting for chaos experiment results
- Integration with existing test suites

## Expected Deliverables

### 1. Implementation Architecture
- Detailed design for chaos engineering framework integration
- Code structure and module organization
- Integration points with existing resilience patterns
- Configuration and feature flag strategies

### 2. Experiment Catalog
- Comprehensive list of chaos experiments specific to Uveddi
- Experiment severity levels and safety classifications
- Expected outcomes and success criteria for each experiment
- Rollback and recovery procedures

### 3. Tooling Recommendations
- Specific tool recommendations with pros/cons analysis
- Implementation complexity assessment
- Integration effort estimates
- Licensing and maintenance considerations

### 4. Testing Strategy
- Unit-level chaos testing approaches
- Integration-level failure simulation
- End-to-end chaos experiment design
- Automated validation and assertion strategies

### 5. Monitoring and Alerting
- Metrics collection strategy during chaos experiments
- Dashboard design for chaos experiment monitoring
- Alerting rules for experiment failures or unexpected outcomes
- Reporting templates for experiment results

## Code Examples Required
Please provide specific Rust code examples for:
- Basic failure injection using `tokio-test`
- Circuit breaker integration with chaos experiments
- Metrics collection during failure scenarios
- Automated experiment orchestration
- Recovery validation and assertion patterns

## Integration Considerations
- Must work with existing `tower` middleware stack
- Should integrate with `tracing` for observability
- Must be compatible with `tokio` async runtime
- Should leverage existing `metrics` collection infrastructure
- Must work within GitHub Actions CI/CD pipeline

## Success Criteria
The research should enable implementation of:
- [ ] Safe, automated chaos experiments in CI/CD
- [ ] Comprehensive failure scenario coverage
- [ ] Detailed recovery time and integrity measurements
- [ ] Integration with existing monitoring and alerting
- [ ] Minimal impact on development velocity
- [ ] Clear experiment results and actionable insights

## Timeline
This research should provide sufficient detail to begin implementation within 2-3 days of completion.

---

**Please provide a comprehensive research report addressing all these areas with specific, actionable recommendations and code examples for the Uveddi project.**