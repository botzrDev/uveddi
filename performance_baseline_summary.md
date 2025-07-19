# UV-49 Performance Baseline Analysis Report

**Generated:** 2025-07-19T13:20:33.217152532+00:00

## Executive Summary

- **Total Measurements:** 80
- **Average Rendering Time:** 4.00ms
- **P95 Rendering Time:** 5.00ms
- **P99 Rendering Time:** 143.00ms
- **Cache Hit Rate:** 80.0%
- **Critical Bottlenecks:** 1

## Performance Target Analysis

- **<50ms Target Compliance:** 95.0%
- **Status:** ✅ TARGET MET

## Identified Bottlenecks

### Outlier Performance (Critical)
- **Description:** Maximum rendering time (222.00ms) indicates severe outliers
- **Impact:** 22.00ms
- **Recommendation:** Implement timeouts, circuit breakers, and consistent resource allocation

## Diagram Type Performance

### complex Diagrams
- **Sample Count:** 20
- **Average Time:** 14.70ms
- **P95 Time:** 5.00ms
- **P99 Time:** 222.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 200

### simple Diagrams
- **Sample Count:** 20
- **Average Time:** 9.80ms
- **P95 Time:** 4.00ms
- **P99 Time:** 127.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 10

### medium Diagrams
- **Sample Count:** 20
- **Average Time:** 10.00ms
- **P95 Time:** 4.00ms
- **P99 Time:** 132.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 50

### sequence Diagrams
- **Sample Count:** 20
- **Average Time:** 10.85ms
- **P95 Time:** 5.00ms
- **P99 Time:** 143.00ms
- **Success Rate:** 100.0%
- **Complexity Score:** 75

## Next Steps for UV-47 Implementation

Based on this baseline analysis, the following optimizations are recommended:

1. **Outlier Performance:** Implement timeouts, circuit breakers, and consistent resource allocation
