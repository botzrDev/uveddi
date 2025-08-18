import { act, waitFor } from '@testing-library/react';

export interface PerformanceMetrics {
  renderTime: number;
  updateTime: number;
  memoryUsage: number;
  componentCount: number;
}

export const measureRenderPerformance = async <T extends Function>(
  renderFunction: T,
  iterations: number = 10
): Promise<PerformanceMetrics> => {
  const metrics: PerformanceMetrics[] = [];
  
  for (let i = 0; i < iterations; i++) {
    const startTime = performance.now();
    const startMemory = (performance as any).memory?.usedJSHeapSize || 0;
    
    await act(async () => {
      renderFunction();
    });
    
    const endTime = performance.now();
    const endMemory = (performance as any).memory?.usedJSHeapSize || 0;
    
    metrics.push({
      renderTime: endTime - startTime,
      updateTime: 0, // Will be measured separately
      memoryUsage: endMemory - startMemory,
      componentCount: document.querySelectorAll('[data-testid]').length
    });
  }
  
  // Calculate averages
  return {
    renderTime: metrics.reduce((sum, m) => sum + m.renderTime, 0) / iterations,
    updateTime: metrics.reduce((sum, m) => sum + m.updateTime, 0) / iterations,
    memoryUsage: metrics.reduce((sum, m) => sum + m.memoryUsage, 0) / iterations,
    componentCount: metrics[metrics.length - 1].componentCount
  };
};

export const measureUpdatePerformance = async (
  updateFunction: Function,
  iterations: number = 5
): Promise<number> => {
  const times: number[] = [];
  
  for (let i = 0; i < iterations; i++) {
    const startTime = performance.now();
    
    await act(async () => {
      await updateFunction();
    });
    
    const endTime = performance.now();
    times.push(endTime - startTime);
  }
  
  return times.reduce((sum, time) => sum + time, 0) / iterations;
};

export const createPerformanceObserver = (): Promise<PerformanceEntry[]> => {
  return new Promise((resolve) => {
    const entries: PerformanceEntry[] = [];
    
    if ('PerformanceObserver' in window) {
      const observer = new PerformanceObserver((list) => {
        entries.push(...list.getEntries());
      });
      
      observer.observe({ entryTypes: ['measure', 'navigation', 'paint'] });
      
      setTimeout(() => {
        observer.disconnect();
        resolve(entries);
      }, 5000);
    } else {
      resolve([]);
    }
  });
};

export const waitForComponentStability = async (
  selector: string,
  timeout: number = 5000
): Promise<void> => {
  let lastCount = 0;
  let stableCount = 0;
  
  return waitFor(() => {
    const currentCount = document.querySelectorAll(selector).length;
    
    if (currentCount === lastCount) {
      stableCount++;
    } else {
      stableCount = 0;
      lastCount = currentCount;
    }
    
    // Consider stable after 3 consecutive same counts
    if (stableCount < 3) {
      throw new Error('Component not yet stable');
    }
  }, { timeout });
};

export const measureScrollPerformance = async (
  scrollContainer: Element,
  scrollDistance: number,
  steps: number = 10
): Promise<number[]> => {
  const times: number[] = [];
  const stepSize = scrollDistance / steps;
  
  for (let i = 0; i < steps; i++) {
    const startTime = performance.now();
    
    await act(async () => {
      scrollContainer.scrollTo({
        top: (i + 1) * stepSize,
        behavior: 'smooth'
      });
    });
    
    // Wait for scroll to complete
    await new Promise(resolve => setTimeout(resolve, 100));
    
    const endTime = performance.now();
    times.push(endTime - startTime);
  }
  
  return times;
};

export const measureComponentMountTime = async (
  componentRender: Function
): Promise<number> => {
  const startTime = performance.now();
  
  await act(async () => {
    componentRender();
  });
  
  // Wait for component to fully mount
  await waitFor(() => {
    const elements = document.querySelectorAll('[data-testid]');
    if (elements.length === 0) {
      throw new Error('No components mounted yet');
    }
  });
  
  return performance.now() - startTime;
};

export const createMemoryLeakDetector = () => {
  const initialHeapSize = (performance as any).memory?.usedJSHeapSize || 0;
  
  return {
    check: () => {
      if (!(performance as any).memory) {
        return { hasLeak: false, growth: 0, message: 'Memory API not available' };
      }
      
      const currentHeapSize = (performance as any).memory.usedJSHeapSize;
      const growth = currentHeapSize - initialHeapSize;
      const growthMB = growth / (1024 * 1024);
      
      // Consider a leak if memory grew by more than 10MB
      const hasLeak = growthMB > 10;
      
      return {
        hasLeak,
        growth: growthMB,
        message: hasLeak 
          ? `Potential memory leak detected: ${growthMB.toFixed(2)}MB increase`
          : `Memory usage normal: ${growthMB.toFixed(2)}MB increase`
      };
    }
  };
};

export const measureVirtualScrollPerformance = async (
  container: Element,
  itemCount: number,
  viewportSize: number
): Promise<{
  renderTime: number;
  scrollTime: number;
  memoryEfficiency: number;
}> => {
  const startRender = performance.now();
  
  // Simulate virtual scroll rendering
  await act(async () => {
    // This would trigger virtual scroll component rendering
    container.dispatchEvent(new Event('scroll'));
  });
  
  const renderTime = performance.now() - startRender;
  
  // Measure scroll performance
  const startScroll = performance.now();
  
  await act(async () => {
    container.scrollTo({ top: itemCount * 50 }); // Assuming 50px per item
  });
  
  const scrollTime = performance.now() - startScroll;
  
  // Calculate memory efficiency (lower is better)
  const renderedElements = container.querySelectorAll('[data-virtual-item]').length;
  const memoryEfficiency = renderedElements / itemCount;
  
  return {
    renderTime,
    scrollTime,
    memoryEfficiency
  };
};

export const createPerformanceBenchmark = (name: string) => ({
  start: () => performance.mark(`${name}-start`),
  end: () => {
    performance.mark(`${name}-end`);
    performance.measure(name, `${name}-start`, `${name}-end`);
    
    const measure = performance.getEntriesByName(name, 'measure')[0];
    return measure?.duration || 0;
  },
  clear: () => {
    performance.clearMarks(`${name}-start`);
    performance.clearMarks(`${name}-end`);
    performance.clearMeasures(name);
  }
});

export const assertPerformanceThreshold = (
  actualTime: number,
  expectedTime: number,
  testName: string
) => {
  if (actualTime > expectedTime) {
    throw new Error(
      `Performance threshold exceeded for ${testName}: ` +
      `${actualTime.toFixed(2)}ms > ${expectedTime}ms`
    );
  }
};

export const createStressTestRunner = () => ({
  async runStressTest(
    testFunction: Function,
    iterations: number,
    concurrency: number = 1
  ): Promise<{
    averageTime: number;
    maxTime: number;
    minTime: number;
    failureCount: number;
    successCount: number;
  }> {
    const results: number[] = [];
    let failureCount = 0;
    let successCount = 0;
    
    const batches = Math.ceil(iterations / concurrency);
    
    for (let batch = 0; batch < batches; batch++) {
      const batchPromises = [];
      
      for (let i = 0; i < concurrency && (batch * concurrency + i) < iterations; i++) {
        batchPromises.push(
          (async () => {
            try {
              const startTime = performance.now();
              await testFunction();
              const endTime = performance.now();
              results.push(endTime - startTime);
              successCount++;
            } catch (error) {
              failureCount++;
              console.error(`Stress test iteration failed:`, error);
            }
          })()
        );
      }
      
      await Promise.all(batchPromises);
    }
    
    return {
      averageTime: results.reduce((sum, time) => sum + time, 0) / results.length,
      maxTime: Math.max(...results),
      minTime: Math.min(...results),
      failureCount,
      successCount
    };
  }
});