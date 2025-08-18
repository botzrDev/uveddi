import React from 'react';
import { render } from '../../test-utils/test-utils';
import {
  measureRenderPerformance,
  measureUpdatePerformance,
  waitForComponentStability,
  measureScrollPerformance,
  createMemoryLeakDetector,
  assertPerformanceThreshold,
  createPerformanceBenchmark,
  createStressTestRunner
} from '../../test-utils/performanceHelpers';
import {
  generateLargeDataset,
  generateStressTestData,
  generateMockInteractiveReport
} from '../../test-utils/dataGenerators';

// Import dashboard components
import { PriorityMatrix } from '../../components/dashboard/PriorityMatrix';
import { QualityScoreCard } from '../../components/dashboard/QualityScoreCard';
import { InteractiveDependencyGraph } from '../../components/dashboard/InteractiveDependencyGraph';
import { TechnicalDebtTracker } from '../../components/dashboard/TechnicalDebtTracker';
import { SmartSearchPanel } from '../../components/dashboard/SmartSearchPanel';

describe('Dashboard Performance Tests', () => {
  const largeDataset = generateLargeDataset();
  const stressTestData = generateStressTestData();
  const memoryLeakDetector = createMemoryLeakDetector();

  describe('Component Render Performance', () => {
    it('should render PriorityMatrix with large dataset within time threshold', async () => {
      const benchmark = createPerformanceBenchmark('priority-matrix-render');
      
      const renderFunction = () => {
        render(
          <PriorityMatrix
            issues={largeDataset.issues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
        );
      };

      benchmark.start();
      const metrics = await measureRenderPerformance(renderFunction, 5);
      const duration = benchmark.end();

      // Should render within 2 seconds even with 10k issues
      assertPerformanceThreshold(metrics.renderTime, 2000, 'PriorityMatrix large dataset render');
      
      console.log(`PriorityMatrix render time: ${metrics.renderTime.toFixed(2)}ms`);
      console.log(`Memory usage: ${(metrics.memoryUsage / 1024 / 1024).toFixed(2)}MB`);
      
      benchmark.clear();
    });

    it('should render QualityScoreCard efficiently', async () => {
      const complexMetrics = stressTestData.complexMetrics;
      
      const renderFunction = () => {
        render(
          <QualityScoreCard
            qualityMetrics={complexMetrics}
            historicalData={largeDataset.historicalData}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => {}}
          />
        );
      };

      const metrics = await measureRenderPerformance(renderFunction, 10);
      
      // Should render within 500ms
      assertPerformanceThreshold(metrics.renderTime, 500, 'QualityScoreCard render');
      
      console.log(`QualityScoreCard render time: ${metrics.renderTime.toFixed(2)}ms`);
    });

    it('should handle InteractiveDependencyGraph with many nodes', async () => {
      const renderFunction = () => {
        render(
          <InteractiveDependencyGraph
            dependencies={largeDataset.dependencies}
            onNodeSelect={() => {}}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={true}
          />
        );
      };

      const metrics = await measureRenderPerformance(renderFunction, 3);
      
      // Dependency graphs are more complex, allow 3 seconds
      assertPerformanceThreshold(metrics.renderTime, 3000, 'InteractiveDependencyGraph render');
      
      console.log(`InteractiveDependencyGraph render time: ${metrics.renderTime.toFixed(2)}ms`);
      console.log(`Component count: ${metrics.componentCount}`);
    });

    it('should render SmartSearchPanel with large dataset efficiently', async () => {
      const renderFunction = () => {
        render(
          <SmartSearchPanel
            issues={largeDataset.issues}
            onSearchResults={() => {}}
            onSaveSearch={() => {}}
            onLoadSavedSearch={() => {}}
            savedSearches={[]}
          />
        );
      };

      const metrics = await measureRenderPerformance(renderFunction, 5);
      
      // Search panel should render quickly
      assertPerformanceThreshold(metrics.renderTime, 1000, 'SmartSearchPanel render');
      
      console.log(`SmartSearchPanel render time: ${metrics.renderTime.toFixed(2)}ms`);
    });
  });

  describe('Update Performance', () => {
    it('should handle priority matrix filtering efficiently', async () => {
      let component: any;
      
      const { rerender } = render(
        <PriorityMatrix
          issues={largeDataset.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
          ref={(ref) => { component = ref; }}
        />
      );

      const updateFunction = async () => {
        const filteredIssues = largeDataset.issues.filter(issue => issue.severity === 'high');
        rerender(
          <PriorityMatrix
            issues={filteredIssues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
        );
      };

      const updateTime = await measureUpdatePerformance(updateFunction, 5);
      
      // Filtering should be fast
      assertPerformanceThreshold(updateTime, 300, 'PriorityMatrix filtering');
      
      console.log(`PriorityMatrix filter update time: ${updateTime.toFixed(2)}ms`);
    });

    it('should handle search updates efficiently', async () => {
      const { rerender } = render(
        <SmartSearchPanel
          issues={largeDataset.issues}
          onSearchResults={() => {}}
          onSaveSearch={() => {}}
          onLoadSavedSearch={() => {}}
          savedSearches={[]}
        />
      );

      const updateFunction = async () => {
        const searchResults = largeDataset.issues.filter(issue => 
          issue.title.toLowerCase().includes('method')
        );
        
        // Simulate search result update
        rerender(
          <SmartSearchPanel
            issues={searchResults}
            onSearchResults={() => {}}
            onSaveSearch={() => {}}
            onLoadSavedSearch={() => {}}
            savedSearches={[]}
          />
        );
      };

      const updateTime = await measureUpdatePerformance(updateFunction, 10);
      
      // Search updates should be very fast
      assertPerformanceThreshold(updateTime, 100, 'SmartSearchPanel search update');
      
      console.log(`SmartSearchPanel search update time: ${updateTime.toFixed(2)}ms`);
    });
  });

  describe('Memory Performance', () => {
    it('should not have memory leaks in component mounting/unmounting', async () => {
      const iterations = 50;
      
      for (let i = 0; i < iterations; i++) {
        const { unmount } = render(
          <div>
            <PriorityMatrix
              issues={largeDataset.issues.slice(0, 100)}
              onIssueSelect={() => {}}
              onFilterChange={() => {}}
              selectedIssues={[]}
            />
            <QualityScoreCard
              qualityMetrics={generateMockInteractiveReport().quality_metrics}
              historicalData={largeDataset.historicalData.slice(0, 30)}
              onDrillDown={() => {}}
              onBenchmarkCompare={() => {}}
            />
          </div>
        );
        
        unmount();
        
        // Force garbage collection if available
        if (global.gc) {
          global.gc();
        }
      }
      
      const leakCheck = memoryLeakDetector.check();
      console.log(leakCheck.message);
      
      // Should not have significant memory growth
      expect(leakCheck.hasLeak).toBe(false);
    });

    it('should handle large datasets without excessive memory usage', async () => {
      const initialMemory = (performance as any).memory?.usedJSHeapSize || 0;
      
      render(
        <div>
          <PriorityMatrix
            issues={stressTestData.issues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
          <InteractiveDependencyGraph
            dependencies={stressTestData.dependencies}
            onNodeSelect={() => {}}
            onClusterAnalysis={() => {}}
            selectedNodes={[]}
            highlightCycles={false} // Disable for performance
          />
        </div>
      );

      await waitForComponentStability('[data-testid]');
      
      const finalMemory = (performance as any).memory?.usedJSHeapSize || 0;
      const memoryGrowthMB = (finalMemory - initialMemory) / (1024 * 1024);
      
      console.log(`Memory growth with stress test data: ${memoryGrowthMB.toFixed(2)}MB`);
      
      // Should not use more than 100MB for stress test data
      expect(memoryGrowthMB).toBeLessThan(100);
    });
  });

  describe('Scroll Performance', () => {
    it('should handle virtual scrolling efficiently', async () => {
      const { container } = render(
        <div style={{ height: '400px', overflow: 'auto' }}>
          <TechnicalDebtTracker
            technicalDebt={{
              total_debt_hours: 1000,
              debt_by_category: {
                'code-smell': 400,
                'architecture': 300,
                'security': 200,
                'performance': 100
              },
              debt_trend: Array.from({ length: 1000 }, (_, i) => 1000 - i),
              priority_items: Array.from({ length: 1000 }, (_, i) => ({
                issue_id: `issue-${i}`,
                debt_hours: Math.floor(Math.random() * 10) + 1,
                roi_score: Math.random()
              }))
            }}
            onDebtItemClick={() => {}}
            onCategoryFilter={() => {}}
            onTimeRangeChange={() => {}}
          />
        </div>
      );

      const scrollContainer = container.firstChild as Element;
      const scrollTimes = await measureScrollPerformance(scrollContainer, 5000, 20);
      
      const averageScrollTime = scrollTimes.reduce((a, b) => a + b, 0) / scrollTimes.length;
      
      // Each scroll step should complete within 50ms
      assertPerformanceThreshold(averageScrollTime, 50, 'Virtual scroll performance');
      
      console.log(`Average scroll time: ${averageScrollTime.toFixed(2)}ms`);
    });
  });

  describe('Stress Testing', () => {
    it('should handle concurrent component interactions', async () => {
      const stressRunner = createStressTestRunner();
      
      const { rerender } = render(
        <PriorityMatrix
          issues={largeDataset.issues}
          onIssueSelect={() => {}}
          onFilterChange={() => {}}
          selectedIssues={[]}
        />
      );

      const concurrentUpdateTest = async () => {
        const randomIssues = largeDataset.issues
          .sort(() => Math.random() - 0.5)
          .slice(0, Math.floor(Math.random() * 1000) + 100);
        
        rerender(
          <PriorityMatrix
            issues={randomIssues}
            onIssueSelect={() => {}}
            onFilterChange={() => {}}
            selectedIssues={[]}
          />
        );
      };

      const results = await stressRunner.runStressTest(
        concurrentUpdateTest,
        100, // 100 iterations
        5    // 5 concurrent operations
      );

      console.log('Stress test results:', {
        averageTime: `${results.averageTime.toFixed(2)}ms`,
        maxTime: `${results.maxTime.toFixed(2)}ms`,
        successRate: `${(results.successCount / (results.successCount + results.failureCount) * 100).toFixed(1)}%`
      });

      // Should have at least 90% success rate
      expect(results.successCount / (results.successCount + results.failureCount)).toBeGreaterThan(0.9);
      
      // Average time should be reasonable
      expect(results.averageTime).toBeLessThan(1000);
    });

    it('should maintain performance under sustained load', async () => {
      const iterations = 200;
      const renderTimes: number[] = [];
      
      for (let i = 0; i < iterations; i++) {
        const startTime = performance.now();
        
        const { unmount } = render(
          <QualityScoreCard
            qualityMetrics={generateMockInteractiveReport().quality_metrics}
            historicalData={largeDataset.historicalData.slice(0, 50)}
            onDrillDown={() => {}}
            onBenchmarkCompare={() => {}}
          />
        );
        
        const endTime = performance.now();
        renderTimes.push(endTime - startTime);
        
        unmount();
        
        // Brief pause to simulate real usage
        await new Promise(resolve => setTimeout(resolve, 10));
      }

      const averageTime = renderTimes.reduce((a, b) => a + b, 0) / renderTimes.length;
      const maxTime = Math.max(...renderTimes);
      const minTime = Math.min(...renderTimes);
      
      console.log('Sustained load test results:', {
        averageTime: `${averageTime.toFixed(2)}ms`,
        maxTime: `${maxTime.toFixed(2)}ms`,
        minTime: `${minTime.toFixed(2)}ms`,
        iterations
      });

      // Performance should not degrade significantly over time
      const lastTenAverage = renderTimes.slice(-10).reduce((a, b) => a + b, 0) / 10;
      const firstTenAverage = renderTimes.slice(0, 10).reduce((a, b) => a + b, 0) / 10;
      const degradationRatio = lastTenAverage / firstTenAverage;
      
      // Performance degradation should be less than 50%
      expect(degradationRatio).toBeLessThan(1.5);
    });
  });

  describe('Network Performance Simulation', () => {
    it('should handle slow network conditions gracefully', async () => {
      // Simulate slow network by adding artificial delays
      const slowNetworkDelay = 2000; // 2 seconds
      
      const startTime = performance.now();
      
      // Simulate loading state
      const { rerender } = render(
        <QualityScoreCard
          qualityMetrics={generateMockInteractiveReport().quality_metrics}
          historicalData={[]}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      // Simulate network delay
      await new Promise(resolve => setTimeout(resolve, slowNetworkDelay));
      
      // Update with actual data
      rerender(
        <QualityScoreCard
          qualityMetrics={generateMockInteractiveReport().quality_metrics}
          historicalData={largeDataset.historicalData}
          onDrillDown={() => {}}
          onBenchmarkCompare={() => {}}
        />
      );

      const totalTime = performance.now() - startTime;
      
      // Should handle the transition smoothly
      expect(totalTime).toBeGreaterThan(slowNetworkDelay);
      expect(totalTime).toBeLessThan(slowNetworkDelay + 500); // Within 500ms of expected time
      
      console.log(`Slow network simulation completed in: ${totalTime.toFixed(2)}ms`);
    });
  });
});

// Performance regression detection
describe('Performance Regression Tests', () => {
  const performanceBaselines = {
    priorityMatrixRender: 2000, // ms
    qualityScoreCardRender: 500, // ms
    dependencyGraphRender: 3000, // ms
    searchPanelUpdate: 100, // ms
    memoryUsageLimit: 100 // MB
  };

  it('should not regress from performance baselines', async () => {
    const results = {
      priorityMatrix: 0,
      qualityScoreCard: 0,
      dependencyGraph: 0,
      searchPanel: 0,
      memoryUsage: 0
    };

    // Test each component
    const priorityRender = () => render(
      <PriorityMatrix
        issues={largeDataset.issues}
        onIssueSelect={() => {}}
        onFilterChange={() => {}}
        selectedIssues={[]}
      />
    );
    const priorityMetrics = await measureRenderPerformance(priorityRender, 3);
    results.priorityMatrix = priorityMetrics.renderTime;

    const qualityRender = () => render(
      <QualityScoreCard
        qualityMetrics={generateMockInteractiveReport().quality_metrics}
        historicalData={largeDataset.historicalData}
        onDrillDown={() => {}}
        onBenchmarkCompare={() => {}}
      />
    );
    const qualityMetrics = await measureRenderPerformance(qualityRender, 5);
    results.qualityScoreCard = qualityMetrics.renderTime;

    const dependencyRender = () => render(
      <InteractiveDependencyGraph
        dependencies={largeDataset.dependencies}
        onNodeSelect={() => {}}
        onClusterAnalysis={() => {}}
        selectedNodes={[]}
        highlightCycles={false}
      />
    );
    const dependencyMetrics = await measureRenderPerformance(dependencyRender, 2);
    results.dependencyGraph = dependencyMetrics.renderTime;

    // Log all results
    console.log('Performance Regression Test Results:');
    Object.entries(results).forEach(([key, value]) => {
      console.log(`${key}: ${typeof value === 'number' ? value.toFixed(2) + 'ms' : value}`);
    });

    // Assert against baselines
    expect(results.priorityMatrix).toBeLessThan(performanceBaselines.priorityMatrixRender);
    expect(results.qualityScoreCard).toBeLessThan(performanceBaselines.qualityScoreCardRender);
    expect(results.dependencyGraph).toBeLessThan(performanceBaselines.dependencyGraphRender);
  });
});