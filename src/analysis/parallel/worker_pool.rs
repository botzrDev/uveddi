//! Worker pool implementation for parallel task execution
//!
//! This module manages a pool of worker threads that process diagram generation
//! tasks in parallel, with support for work stealing and resource-aware scaling.

use std::sync::Arc;
use std::collections::VecDeque;
use std::sync::atomic::{AtomicUsize, Ordering};
use tokio::sync::Mutex;
use crate::monitoring::parallel_metrics::ParallelMetricsCollector;
use super::engine::DiagramGenerationTask;

/// Manages a pool of worker threads for parallel processing
pub struct WorkerPool {
    workers: Vec<Worker>,
    global_queue: Arc<Mutex<VecDeque<DiagramGenerationTask>>>,
    metrics: Arc<ParallelMetricsCollector>,
    active_workers: AtomicUsize,
}

impl WorkerPool {
    /// Creates a new worker pool with default configuration
    pub fn new() -> Self {
        // Determine optimal thread count based on available cores
        let num_workers = std::thread::available_parallelism()
            .map(|n| n.get())
            .unwrap_or(4)
            .min(32); // Cap at 32 workers
        
        let global_queue = Arc::new(Mutex::new(VecDeque::new()));
        let metrics = Arc::new(ParallelMetricsCollector::new());
        
        let workers = (0..num_workers)
            .map(|id| Worker::new(id, global_queue.clone(), metrics.clone()))
            .collect();
        
        WorkerPool {
            workers,
            global_queue,
            metrics,
            active_workers: AtomicUsize::new(0),
        }
    }
    
    /// Submit tasks to the worker pool
    pub async fn submit_tasks(&self, tasks: Vec<DiagramGenerationTask>) {
        let mut queue = self.global_queue.lock().await;
        for task in tasks {
            queue.push_back(task);
        }
        self.metrics.track_queue_size(queue.len());
    }
    
    /// Get current active worker count
    pub fn active_workers(&self) -> usize {
        self.active_workers.load(Ordering::SeqCst)
    }
    
    /// Scale the worker pool based on current load
    pub async fn scale_workers(&mut self, target_count: usize) {
        // Implementation will be added in subsequent steps
        // Will add or remove workers based on target_count
    }
}

/// Individual worker thread implementation
struct Worker {
    id: usize,
    local_queue: VecDeque<DiagramGenerationTask>,
    global_queue: Arc<Mutex<VecDeque<DiagramGenerationTask>>>,
    metrics: Arc<ParallelMetricsCollector>,
}

impl Worker {
    fn new(
        id: usize,
        global_queue: Arc<Mutex<VecDeque<DiagramGenerationTask>>>,
        metrics: Arc<ParallelMetricsCollector>,
    ) -> Self {
        Worker {
            id,
            local_queue: VecDeque::new(),
            global_queue,
            metrics,
        }
    }
    
    /// Main worker processing loop
    async fn run(&mut self) {
        loop {
            // Try to get a task from local queue
            if let Some(task) = self.local_queue.pop_front() {
                self.process_task(task).await;
                continue;
            }
            
            // Try to steal work from global queue
            if let Some(task) = self.steal_work().await {
                self.process_task(task).await;
                continue;
            }
            
            // No work available, sleep briefly
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
        }
    }
    
    /// Process a single diagram generation task
    async fn process_task(&self, task: DiagramGenerationTask) {
        // Implementation will be added in subsequent steps
        // Will generate the diagram based on task specification
        self.metrics.track_task_started();
    }
    
    /// Attempt to steal work from the global queue
    async fn steal_work(&mut self) -> Option<DiagramGenerationTask> {
        let mut queue = self.global_queue.lock().await;
        queue.pop_front()
    }
}
