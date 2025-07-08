# UV-97 Research Prompts Index

## Overview
This directory contains detailed GPT research prompts designed to accelerate the completion of UV-97 (tree-sitter feature gating) by systematically analyzing different aspects of the implementation challenge.

## Research Prompt Directory

### 🔴 **Critical Path Research** (Do First)

#### [01. API Compatibility Mapping](./01-api-compatibility-mapping.md)
**Priority:** CRITICAL - Blocking all stub implementation work
**Objective:** Create definitive API compatibility matrix for stub implementations
**Impact:** Eliminates guesswork and prevents rework due to API mismatches
**Time:** 1-2 hours research, saves 4-6 hours implementation

#### [03. Compilation Error Analysis](./03-compilation-error-analysis.md)  
**Priority:** HIGH - Provides clear roadmap for remaining work
**Objective:** Systematic analysis and prioritization of all compilation errors
**Impact:** Transforms ad-hoc error fixing into efficient batch processing
**Time:** 1 hour research, saves 3-4 hours debugging

### ⚠️ **High-Impact Research** (Do Next)

#### [04. Tree-sitter Type Usage Inventory](./04-type-usage-inventory.md)
**Priority:** HIGH - Enables systematic stub type creation
**Objective:** Complete inventory of tree-sitter type usage patterns
**Impact:** Creates minimal, targeted stub type system vs recreating all functionality
**Time:** 1-2 hours research, saves 2-3 hours implementation

#### [02. Detector Dependency Analysis](./02-detector-dependency-analysis.md)
**Priority:** MEDIUM - Enables batch detector processing
**Objective:** Classify detectors by tree-sitter dependency for optimal gating strategy
**Impact:** Batch-process similar detectors instead of individual analysis
**Time:** 1-2 hours research, saves 3-4 hours detector work

### 🔄 **Optimization Research** (Optional but Valuable)

#### [06. Build Optimization Research](./06-build-optimization-research.md)
**Priority:** MEDIUM - Speeds up development workflow
**Objective:** Optimize edit-compile-test cycle during implementation
**Impact:** Reduces cycle time from 3-5 minutes to <1 minute
**Time:** 30 minutes research, saves ongoing time throughout implementation

#### [05. Test Strategy Research](./05-test-strategy-research.md)
**Priority:** LOW - Needed for completion but not blocking
**Objective:** Systematic test gating strategy for both feature configurations
**Impact:** Efficient test adaptation vs ad-hoc test fixing
**Time:** 1 hour research, saves 2-3 hours test work

## Research Execution Strategy

### Immediate Action Plan (4-6 hours total research)
```
Phase 1: Critical Blockers (2-3 hours)
├── 01-api-compatibility-mapping.md
└── 03-compilation-error-analysis.md

Phase 2: Systematic Implementation (2-3 hours)  
├── 04-type-usage-inventory.md
└── 02-detector-dependency-analysis.md

Phase 3: Optimization (1 hour)
└── 06-build-optimization-research.md
```

### Parallel Execution Opportunities
- **Research Items 01 & 03** can be done by same person (both analyze current codebase state)
- **Research Items 02 & 04** can be done in parallel by different people
- **Research Item 06** can be done by someone while others implement based on findings from 01-04

## Expected ROI
- **Total Research Time:** 4-6 hours
- **Implementation Time Saved:** 15-20 hours  
- **Quality Improvement:** Systematic approach vs trial-and-error
- **Risk Reduction:** Comprehensive analysis prevents missed edge cases

## How to Use These Prompts

### For GPT/Claude Analysis:
1. Copy the entire research prompt
2. Provide the Uveddi codebase as context
3. Request the structured analysis as specified in the prompt
4. Use the output to systematically implement UV-97

### For Human Analysis:
1. Use the research methodology sections as step-by-step guides
2. Execute the provided commands and searches
3. Document findings in the expected output format
4. Use the analysis to plan implementation work

## Success Criteria
Completing this research should enable:
- [ ] UV-97 implementation in systematic phases vs ad-hoc fixes
- [ ] Confident API compatibility without multiple iterations
- [ ] Efficient batch processing of similar problems
- [ ] Optimal development workflow with fast feedback cycles
- [ ] Comprehensive test coverage in both feature configurations

## Dependencies
- **01** & **03** have no dependencies - can start immediately
- **02** & **04** can reference findings from **01** but can largely proceed independently  
- **05** should wait for completion of **02** (detector analysis informs test strategy)
- **06** is independent and can be done anytime

## Next Steps
1. **Choose 1-2 critical research items** based on current blocking issues
2. **Execute research prompts** with GPT/Claude or manual analysis
3. **Document findings** in structured format provided
4. **Use research outputs** to systematically implement UV-97 phases
5. **Iterate** - update research as new issues discovered

---

*This research-driven approach transforms UV-97 from a complex, ad-hoc implementation challenge into a systematic, well-understood engineering task.*
