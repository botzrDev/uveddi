//! GraphQL subscriptions for real-time updates

use async_graphql::{Context, Result, Subscription, ID};
use futures_util::{Stream, StreamExt};
use std::sync::Arc;
use tokio::sync::broadcast;

use super::context::GraphQLContext;
use super::types::*;

/// Events that can be subscribed to
#[derive(Debug, Clone)]
pub enum SubscriptionEvent {
    AnalysisProgress {
        analysis_run_id: i64,
        progress: AnalysisProgressUpdate,
    },
    NewIssue {
        project_id: i64,
        issue: ArchitecturalIssue,
    },
    PerformanceMetrics {
        analysis_run_id: i64,
        metrics: ComponentPerformanceMetrics,
    },
    CacheStats {
        stats: CacheStats,
    },
}

/// Event broadcaster for managing subscriptions
pub struct EventBroadcaster {
    sender: broadcast::Sender<SubscriptionEvent>,
}

impl EventBroadcaster {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    /// Publish an analysis progress update
    pub fn publish_analysis_progress(
        &self,
        analysis_run_id: i64,
        progress: AnalysisProgressUpdate,
    ) {
        let _ = self.sender.send(SubscriptionEvent::AnalysisProgress {
            analysis_run_id,
            progress,
        });
    }

    /// Publish a new issue detection
    pub fn publish_new_issue(&self, project_id: i64, issue: ArchitecturalIssue) {
        let _ = self
            .sender
            .send(SubscriptionEvent::NewIssue { project_id, issue });
    }

    /// Publish performance metrics update
    pub fn publish_performance_metrics(
        &self,
        analysis_run_id: i64,
        metrics: ComponentPerformanceMetrics,
    ) {
        let _ = self.sender.send(SubscriptionEvent::PerformanceMetrics {
            analysis_run_id,
            metrics,
        });
    }

    /// Publish cache statistics update
    pub fn publish_cache_stats(&self, stats: CacheStats) {
        let _ = self.sender.send(SubscriptionEvent::CacheStats { stats });
    }

    /// Create a subscription stream
    pub fn subscribe(&self) -> broadcast::Receiver<SubscriptionEvent> {
        self.sender.subscribe()
    }
}

impl Default for EventBroadcaster {
    fn default() -> Self {
        Self::new()
    }
}

pub struct SubscriptionRoot;

#[Subscription]
impl SubscriptionRoot {
    /// Subscribe to analysis progress updates for a specific analysis run
    async fn analysis_progress(
        &self,
        ctx: &Context<'_>,
        analysis_run_id: ID,
    ) -> Result<impl Stream<Item = AnalysisProgressUpdate>> {
        let context = ctx.data::<GraphQLContext>()?;
        let broadcaster = ctx.data::<Arc<EventBroadcaster>>()?;

        let run_id: i64 = analysis_run_id
            .parse()
            .map_err(|_| async_graphql::Error::new("Invalid analysis run ID"))?;

        let receiver = broadcaster.subscribe();

        Ok(
            tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(
                move |event| async move {
                    match event {
                        Ok(SubscriptionEvent::AnalysisProgress {
                            analysis_run_id,
                            progress,
                        }) if analysis_run_id == run_id => Some(progress),
                        _ => None,
                    }
                },
            ),
        )
    }

    /// Subscribe to new issues detected for a specific project
    async fn new_issues(
        &self,
        ctx: &Context<'_>,
        project_id: ID,
    ) -> Result<impl Stream<Item = ArchitecturalIssue>> {
        let context = ctx.data::<GraphQLContext>()?;
        let broadcaster = ctx.data::<Arc<EventBroadcaster>>()?;

        let proj_id: i64 = project_id
            .parse()
            .map_err(|_| async_graphql::Error::new("Invalid project ID"))?;

        let receiver = broadcaster.subscribe();

        Ok(
            tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(
                move |event| async move {
                    match event {
                        Ok(SubscriptionEvent::NewIssue { project_id, issue })
                            if project_id == proj_id =>
                        {
                            Some(issue)
                        }
                        _ => None,
                    }
                },
            ),
        )
    }

    /// Subscribe to performance metrics updates for a specific analysis run
    async fn performance_metrics(
        &self,
        ctx: &Context<'_>,
        analysis_run_id: ID,
    ) -> Result<impl Stream<Item = ComponentPerformanceMetrics>> {
        let context = ctx.data::<GraphQLContext>()?;
        let broadcaster = ctx.data::<Arc<EventBroadcaster>>()?;

        let run_id: i64 = analysis_run_id
            .parse()
            .map_err(|_| async_graphql::Error::new("Invalid analysis run ID"))?;

        let receiver = broadcaster.subscribe();

        Ok(
            tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(
                move |event| async move {
                    match event {
                        Ok(SubscriptionEvent::PerformanceMetrics {
                            analysis_run_id,
                            metrics,
                        }) if analysis_run_id == run_id => Some(metrics),
                        _ => None,
                    }
                },
            ),
        )
    }

    /// Subscribe to cache statistics updates
    async fn cache_stats(&self, ctx: &Context<'_>) -> Result<impl Stream<Item = CacheStats>> {
        let context = ctx.data::<GraphQLContext>()?;
        let broadcaster = ctx.data::<Arc<EventBroadcaster>>()?;

        let receiver = broadcaster.subscribe();

        Ok(
            tokio_stream::wrappers::BroadcastStream::new(receiver).filter_map(|event| async move {
                match event {
                    Ok(SubscriptionEvent::CacheStats { stats }) => Some(stats),
                    _ => None,
                }
            }),
        )
    }
}

/// Helper trait for integration with the analysis engine
pub trait SubscriptionIntegration {
    /// Register event broadcaster with the analysis engine
    fn register_broadcaster(&mut self, broadcaster: Arc<EventBroadcaster>);
}

// TODO: Implement SubscriptionIntegration for AnalysisEngine
// This would allow the analysis engine to publish events as analysis progresses
