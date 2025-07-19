# UV-49 Performance Baseline Analysis Report

**Generated:** 2025-07-18T23:54:59.717197555+00:00

## Executive Summary

- **Total Measurements:** 80
- **Average Rendering Time:** 3.00ms
- **P95 Rendering Time:** 7.00ms
- **P99 Rendering Time:** 614.00ms
- **Cache Hit Rate:** 80.0%
- **Critical Bottlenecks:** 1

## Performance Target Analysis

- **<50ms Target Compliance:** 95.0%
- **Status:** ✅ TARGET MET

## Identified Bottlenecks

### Outlier Performance (Critical)
- **Description:** Maximum rendering time (1815.00ms) indicates severe outliers
- **Impact:** 1615.00ms
- **Recommendation:** Implement timeouts, circuit breakers, and consistent resource allocation

## Diagram Type Performance

### sequence Diagrams
- **Sample Count:** 20
- **Average Time:** 12.55ms
- **P95 Time:** 7.00ms
- **P99 Time:** 186.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 75

### medium Diagrams
- **Sample Count:** 20
- **Average Time:** 27.70ms
- **P95 Time:** 5.00ms
- **P99 Time:** 491.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 50

### simple Diagrams
- **Sample Count:** 20
- **Average Time:** 94.35ms
- **P95 Time:** 6.00ms
- **P99 Time:** 1815.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 10

### complex Diagrams
- **Sample Count:** 20
- **Average Time:** 34.15ms
- **P95 Time:** 5.00ms
- **P99 Time:** 614.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 200

## Next Steps for UV-47 Implementation

Based on this baseline analysis, the following optimizations are recommended:

1. **Outlier Performance:** Implement timeouts, circuit breakers, and consistent resource allocation
