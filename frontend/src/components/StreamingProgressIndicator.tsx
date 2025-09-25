/**
 * Streaming Progress Indicator
 *
 * Real-time progress visualization with WebSocket streaming,
 * showcasing sub-second updates and cache performance metrics.
 */

import React, { useState, useEffect, useCallback } from 'react';
import {
  Card, CardContent, CardHeader, CardTitle,
  Progress, Badge, Alert, AlertCircle,
  CheckCircle, Clock, Zap, Activity, Database
} from '@/components/ui';
import { useWebSocket } from '@/hooks/useWebSocket';
import { formatDuration } from '@/utils/formatters';

interface StreamingProgress {
  analysis_id: string;
  phase: 'initializing' | 'parsing' | 'analyzing' | 'caching' | 'finalizing' | 'complete';
  progress: number; // 0-100
  current_file?: string;
  files_processed: number;
  total_files: number;
  estimated_remaining?: string;
  cache_hits: number;
  cache_misses: number;
  speedup_factor: number;
  timestamp: string;
}

interface PerformanceMetrics {
  avg_response_time_ms: number;
  current_throughput_fps: number; // files per second
  cache_efficiency: number;
  memory_usage_mb: number;
}

export interface StreamingProgressIndicatorProps {
  analysisId?: string;
  showDetailedMetrics?: boolean;
  onComplete?: (results: any) => void;
  className?: string;
}

export const StreamingProgressIndicator: React.FC<StreamingProgressIndicatorProps> = ({
  analysisId,
  showDetailedMetrics = true,
  onComplete,
  className = ''
}) => {
  const [progress, setProgress] = useState<StreamingProgress | null>(null);
  const [performance, setPerformance] = useState<PerformanceMetrics | null>(null);
  const [startTime, setStartTime] = useState<Date | null>(null);
  const [isActive, setIsActive] = useState(false);

  // WebSocket connection for real-time progress updates
  const { sendMessage, lastMessage, connectionState, isConnected } = useWebSocket(
    `ws://${window.location.host}/api/v1/stream/analysis/${analysisId || 'live'}`,
    {
      onMessage: handleProgressMessage,
      shouldReconnect: () => isActive,
      reconnectInterval: 2000,
      reconnectAttempts: 10
    }
  );

  function handleProgressMessage(message: MessageEvent) {
    try {
      const data = JSON.parse(message.data);

      switch (data.type) {
        case 'analysis_progress':
          if (!startTime && data.phase !== 'complete') {
            setStartTime(new Date());
            setIsActive(true);
          }

          setProgress({
            analysis_id: data.analysis_id,
            phase: data.phase,
            progress: data.percentage || 0,
            current_file: data.current_file,
            files_processed: data.files_processed || 0,
            total_files: data.total_files || 0,
            estimated_remaining: data.estimated_remaining,
            cache_hits: data.cache_hits || 0,
            cache_misses: data.cache_misses || 0,
            speedup_factor: data.speedup_factor || 1,
            timestamp: data.timestamp
          });

          if (data.phase === 'complete') {
            setIsActive(false);
            onComplete?.(data);
          }
          break;

        case 'performance_metrics':
          setPerformance({
            avg_response_time_ms: data.avg_response_time_ms || 0,
            current_throughput_fps: data.throughput_fps || 0,
            cache_efficiency: data.cache_efficiency || 0,
            memory_usage_mb: data.memory_usage_mb || 0
          });
          break;

        case 'error':
          console.error('Streaming progress error:', data.message);
          setIsActive(false);
          break;
      }
    } catch (error) {
      console.error('Failed to parse progress message:', error);
    }
  }

  // Subscribe to progress updates when component mounts
  useEffect(() => {
    if (isConnected && analysisId) {
      sendMessage(JSON.stringify({
        action: 'subscribe',
        analysis_id: analysisId,
        events: ['progress', 'performance', 'completion']
      }));
    }
  }, [isConnected, analysisId, sendMessage]);

  // Calculate derived metrics
  const elapsedTime = startTime ? Date.now() - startTime.getTime() : 0;
  const cacheHitRate = progress ?
    (progress.cache_hits / (progress.cache_hits + progress.cache_misses)) * 100 : 0;

  // Phase descriptions and colors
  const phaseInfo = {
    initializing: { color: 'bg-gray-500', description: 'Setting up analysis environment' },
    parsing: { color: 'bg-blue-500', description: 'Parsing source files and building ASTs' },
    analyzing: { color: 'bg-purple-500', description: 'Running detectors and analysis rules' },
    caching: { color: 'bg-green-500', description: 'Optimizing with intelligent caching' },
    finalizing: { color: 'bg-orange-500', description: 'Generating reports and cleanup' },
    complete: { color: 'bg-green-600', description: 'Analysis completed successfully' }
  };

  const currentPhase = progress?.phase || 'initializing';
  const phaseStyle = phaseInfo[currentPhase];

  return (
    <div className={`space-y-4 ${className}`}>
      {/* Connection Status */}
      <div className="flex items-center justify-between">
        <h3 className="text-lg font-semibold text-gray-900">Real-time Analysis Progress</h3>
        <Badge
          variant={isConnected ? 'success' : 'destructive'}
          className="flex items-center gap-1"
        >
          <Activity className="h-3 w-3" />
          {isConnected ? 'Streaming Live' : 'Reconnecting...'}
        </Badge>
      </div>

      {/* Main Progress Card */}
      <Card className="border-l-4 border-l-blue-500">
        <CardHeader className="pb-3">
          <CardTitle className="flex items-center justify-between">
            <div className="flex items-center gap-2">
              <Clock className="h-5 w-5 text-blue-500" />
              <span>Analysis Progress</span>
              {progress?.analysis_id && (
                <Badge variant="outline" className="text-xs">
                  {progress.analysis_id.slice(-8)}
                </Badge>
              )}
            </div>

            {progress && (
              <div className="text-right">
                <div className="text-2xl font-bold text-blue-600">
                  {progress.progress.toFixed(1)}%
                </div>
                {elapsedTime > 0 && (
                  <div className="text-sm text-gray-600">
                    {formatDuration(elapsedTime)}
                  </div>
                )}
              </div>
            )}
          </CardTitle>
        </CardHeader>

        <CardContent className="space-y-4">
          {/* Phase Indicator */}
          <div className="flex items-center gap-3">
            <div className={`w-3 h-3 rounded-full ${phaseStyle.color} ${isActive ? 'animate-pulse' : ''}`} />
            <div>
              <p className="font-medium capitalize">{currentPhase}</p>
              <p className="text-sm text-gray-600">{phaseStyle.description}</p>
            </div>
          </div>

          {/* Progress Bar */}
          <Progress
            value={progress?.progress || 0}
            className="h-3"
          />

          {/* File Progress */}
          {progress && (
            <div className="grid grid-cols-2 gap-4 text-sm">
              <div>
                <p className="font-medium">Files Processed</p>
                <p className="text-gray-600">
                  {progress.files_processed.toLocaleString()} of {progress.total_files.toLocaleString()}
                </p>
              </div>

              {progress.estimated_remaining && (
                <div>
                  <p className="font-medium">Time Remaining</p>
                  <p className="text-gray-600">{progress.estimated_remaining}</p>
                </div>
              )}

              {progress.current_file && (
                <div className="col-span-2">
                  <p className="font-medium">Current File</p>
                  <p className="text-xs text-gray-600 font-mono truncate">
                    {progress.current_file}
                  </p>
                </div>
              )}
            </div>
          )}

          {/* Performance Highlight */}
          {progress && progress.speedup_factor > 1 && (
            <Alert className="border-green-200 bg-green-50">
              <Zap className="h-4 w-4 text-green-600" />
              <div>
                <strong className="text-green-800">Performance Boost Active!</strong>
                <p className="text-sm text-green-700 mt-1">
                  Cache system is accelerating analysis by{' '}
                  <strong>{progress.speedup_factor.toFixed(1)}x</strong>
                  {' '}with a{' '}
                  <strong>{cacheHitRate.toFixed(1)}%</strong>
                  {' '}hit rate.
                </p>
              </div>
            </Alert>
          )}
        </CardContent>
      </Card>

      {/* Detailed Metrics */}
      {showDetailedMetrics && (progress || performance) && (
        <div className="grid grid-cols-2 md:grid-cols-4 gap-4">
          <Card className="text-center">
            <CardContent className="pt-4">
              <div className="text-xl font-bold text-green-600">
                {progress?.speedup_factor.toFixed(1) || '—'}x
              </div>
              <p className="text-sm text-gray-600">Performance Gain</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-4">
              <div className="text-xl font-bold text-blue-600">
                {cacheHitRate.toFixed(1) || '—'}%
              </div>
              <p className="text-sm text-gray-600">Cache Hit Rate</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-4">
              <div className="text-xl font-bold text-purple-600">
                {performance?.current_throughput_fps.toFixed(1) || '—'}
              </div>
              <p className="text-sm text-gray-600">Files/Second</p>
            </CardContent>
          </Card>

          <Card className="text-center">
            <CardContent className="pt-4">
              <div className="text-xl font-bold text-orange-600">
                {performance?.avg_response_time_ms.toFixed(0) || '—'}ms
              </div>
              <p className="text-sm text-gray-600">Avg Response</p>
            </CardContent>
          </Card>
        </div>
      )}

      {/* Cache Performance Details */}
      {progress && (progress.cache_hits > 0 || progress.cache_misses > 0) && (
        <Card>
          <CardHeader>
            <CardTitle className="flex items-center gap-2 text-sm">
              <Database className="h-4 w-4 text-blue-500" />
              Cache Performance
            </CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-3 gap-4 text-center text-sm">
              <div className="p-3 bg-green-50 rounded-lg">
                <div className="text-lg font-bold text-green-600">
                  {progress.cache_hits.toLocaleString()}
                </div>
                <p className="text-green-800">Cache Hits</p>
              </div>

              <div className="p-3 bg-red-50 rounded-lg">
                <div className="text-lg font-bold text-red-600">
                  {progress.cache_misses.toLocaleString()}
                </div>
                <p className="text-red-800">Cache Misses</p>
              </div>

              <div className="p-3 bg-blue-50 rounded-lg">
                <div className="text-lg font-bold text-blue-600">
                  {cacheHitRate.toFixed(1)}%
                </div>
                <p className="text-blue-800">Efficiency</p>
              </div>
            </div>

            {cacheHitRate > 80 && (
              <div className="mt-3 p-2 bg-green-100 rounded text-center">
                <p className="text-sm font-medium text-green-800">
                  🚀 Excellent cache performance! Analysis running at optimal speed.
                </p>
              </div>
            )}
          </CardContent>
        </Card>
      )}

      {/* Completion Status */}
      {progress?.phase === 'complete' && (
        <Card className="border-green-500 bg-green-50">
          <CardContent className="pt-6">
            <div className="flex items-center gap-3">
              <CheckCircle className="h-6 w-6 text-green-600" />
              <div>
                <h3 className="font-semibold text-green-800">Analysis Complete!</h3>
                <p className="text-sm text-green-700 mt-1">
                  Processed {progress.total_files.toLocaleString()} files
                  {elapsedTime > 0 && ` in ${formatDuration(elapsedTime)}`}
                  {' '}with {progress.speedup_factor.toFixed(1)}x performance acceleration.
                </p>
              </div>
            </div>
          </CardContent>
        </Card>
      )}

      {/* Error State */}
      {!isConnected && isActive && (
        <Alert variant="destructive">
          <AlertCircle className="h-4 w-4" />
          <div>
            <strong>Connection Lost</strong>
            <p className="text-sm mt-1">
              Attempting to reconnect to live progress stream...
            </p>
          </div>
        </Alert>
      )}
    </div>
  );
};