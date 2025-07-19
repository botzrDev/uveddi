# UV-49 Performance Baseline Analysis Report

**Generated:** 2025-07-19T18:53:27.710089184+00:00

## Executive Summary

- **Total Measurements:** 80
- **Average Rendering Time:** 4.00ms
- **P95 Rendering Time:** 9.00ms
- **P99 Rendering Time:** 970.00ms
- **Cache Hit Rate:** 80.0%
- **Critical Bottlenecks:** 1

## Performance Target Analysis

- **<50ms Target Compliance:** 95.0%
- **Status:** ✅ TARGET MET

## Identified Bottlenecks

### Outlier Performance (Critical)
- **Description:** Maximum rendering time (1006.00ms) indicates severe outliers
- **Impact:** 806.00ms
- **Recommendation:** Implement timeouts, circuit breakers, and consistent resource allocation

## Diagram Type Performance

### medium Diagrams
- **Sample Count:** 20
- **Average Time:** 53.90ms
- **P95 Time:** 6.00ms
- **P99 Time:** 1006.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 50

### complex Diagrams
- **Sample Count:** 20
- **Average Time:** 48.05ms
- **P95 Time:** 9.00ms
- **P99 Time:** 882.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 200

### sequence Diagrams
- **Sample Count:** 20
- **Average Time:** 10.85ms
- **P95 Time:** 5.00ms
- **P99 Time:** 149.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 75

### simple Diagrams
- **Sample Count:** 20
- **Average Time:** 52.00ms
- **P95 Time:** 5.00ms
- **P99 Time:** 970.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 10

## Next Steps for UV-47 Implementation

Based on this baseline analysis, the following optimizations are recommended:

1. **Outlier Performance:** Implement timeouts, circuit breakers, and consistent resource allocation
