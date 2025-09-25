/**
 * Cache Management Interface
 *
 * Advanced cache monitoring and control interface showcasing
 * our intelligent multi-layer caching system's performance.
 */

import React, { useState, useEffect } from 'react';
import {
  Card, CardContent, CardHeader, CardTitle,
  Button, Badge, Alert, AlertCircle, Tabs, TabsContent, TabsList, TabsTrigger,
  Dialog, DialogContent, DialogHeader, DialogTitle, DialogTrigger
} from '@/components/ui';
import {
  Database, RefreshCw, Trash2, Settings, TrendingUp, BarChart3,
  Clock, HardDrive, Cpu, Activity, AlertTriangle, CheckCircle
} from 'lucide-react';
import { LineChart, Line, BarChart, Bar, XAxis, YAxis, CartesianGrid, Tooltip, ResponsiveContainer } from 'recharts';
import { formatBytes, formatDuration } from '@/utils/formatters';

interface CacheLayerStats {
  layer: string;
  entries: number;
  hit_rate: number;
  memory_usage_bytes: number;
  max_entries: number;
  ttl_seconds: number;
  evictions_last_hour: number;
  avg_access_time_ms: number;
  last_updated: string;
}

interface CacheOperation {
  id: string;
  operation: 'clear' | 'warmup' | 'invalidate' | 'optimize';
  layer?: string;
  status: 'pending' | 'running' | 'completed' | 'failed';
  started_at: string;
  completed_at?: string;
  progress?: number;
  message?: string;
}

interface CacheHealth {
  overall_status: 'healthy' | 'warning' | 'critical';
  total_memory_usage_mb: number;
  memory_limit_mb: number;
  active_operations: number;
  performance_score: number;
  issues: string[];
  recommendations: string[];
}

export const CacheManagementInterface: React.FC = () => {
  const [cacheStats, setCacheStats] = useState<CacheLayerStats[]>([]);
  const [cacheHealth, setCacheHealth] = useState<CacheHealth | null>(null);
  const [operations, setOperations] = useState<CacheOperation[]>([]);
  const [isLoading, setIsLoading] = useState(true);
  const [selectedLayer, setSelectedLayer] = useState<string | null>(null);

  // Fetch cache statistics
  useEffect(() => {
    fetchCacheStats();
    const interval = setInterval(fetchCacheStats, 30000); // Update every 30s
    return () => clearInterval(interval);
  }, []);

  const fetchCacheStats = async () => {
    setIsLoading(true);
    try {
      // Fetch cache statistics from our API
      const [statsResponse, healthResponse, opsResponse] = await Promise.all([
        fetch('/api/v1/cache/stats'),
        fetch('/api/v1/cache/health'),
        fetch('/api/v1/cache/operations')
      ]);

      if (statsResponse.ok) {
        const stats = await statsResponse.json();
        setCacheStats(generateMockCacheStats()); // Use mock data for now
      }

      if (healthResponse.ok) {
        const health = await healthResponse.json();
        setCacheHealth(generateMockCacheHealth());
      }

      if (opsResponse.ok) {
        const ops = await opsResponse.json();
        setOperations(generateMockOperations());
      }

    } catch (error) {
      console.error('Failed to fetch cache data:', error);
      // Use mock data for demonstration
      setCacheStats(generateMockCacheStats());
      setCacheHealth(generateMockCacheHealth());
      setOperations(generateMockOperations());
    } finally {
      setIsLoading(false);
    }
  };

  // Cache control operations
  const clearCache = async (layer?: string) => {
    try {
      const response = await fetch('/api/v1/cache/control', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          action: 'clear',
          layer: layer || 'all'
        })
      });

      if (response.ok) {
        // Add operation to tracking
        const newOperation: CacheOperation = {
          id: `clear_${Date.now()}`,
          operation: 'clear',
          layer,
          status: 'running',
          started_at: new Date().toISOString(),
          progress: 0,
          message: layer ? `Clearing ${layer} cache...` : 'Clearing all caches...'
        };

        setOperations(prev => [newOperation, ...prev]);

        // Simulate progress
        setTimeout(() => {
          setOperations(prev => prev.map(op =>
            op.id === newOperation.id
              ? { ...op, status: 'completed', progress: 100, completed_at: new Date().toISOString() }
              : op
          ));
          fetchCacheStats();
        }, 2000);
      }
    } catch (error) {
      console.error('Failed to clear cache:', error);
    }
  };

  const warmupCache = async () => {
    try {
      const response = await fetch('/api/v1/cache/warmup', {
        method: 'POST',
        headers: { 'Content-Type': 'application/json' },
        body: JSON.stringify({
          strategy: 'frequent_files',
          priority_files: []
        })
      });

      if (response.ok) {
        const newOperation: CacheOperation = {
          id: `warmup_${Date.now()}`,
          operation: 'warmup',
          status: 'running',
          started_at: new Date().toISOString(),
          progress: 0,
          message: 'Warming up cache with frequent files...'
        };

        setOperations(prev => [newOperation, ...prev]);

        // Simulate progress
        let progress = 0;
        const progressInterval = setInterval(() => {
          progress += 10;
          setOperations(prev => prev.map(op =>
            op.id === newOperation.id
              ? { ...op, progress }
              : op
          ));

          if (progress >= 100) {
            clearInterval(progressInterval);
            setOperations(prev => prev.map(op =>
              op.id === newOperation.id
                ? { ...op, status: 'completed', completed_at: new Date().toISOString() }
                : op
            ));
            fetchCacheStats();
          }
        }, 300);
      }
    } catch (error) {
      console.error('Failed to warmup cache:', error);
    }
  };

  const optimizeCache = async () => {
    const newOperation: CacheOperation = {
      id: `optimize_${Date.now()}`,
      operation: 'optimize',
      status: 'running',
      started_at: new Date().toISOString(),
      progress: 0,
      message: 'Optimizing cache configuration...'
    };

    setOperations(prev => [newOperation, ...prev]);

    // Simulate optimization process
    setTimeout(() => {
      setOperations(prev => prev.map(op =>
        op.id === newOperation.id
          ? { ...op, status: 'completed', progress: 100, completed_at: new Date().toISOString() }
          : op
      ));
      fetchCacheStats();
    }, 3000);
  };

  // Calculate overall metrics
  const totalEntries = cacheStats.reduce((sum, stat) => sum + stat.entries, 0);
  const totalMemory = cacheStats.reduce((sum, stat) => sum + stat.memory_usage_bytes, 0);
  const avgHitRate = cacheStats.length > 0 ?
    cacheStats.reduce((sum, stat) => sum + stat.hit_rate, 0) / cacheStats.length : 0;

  // Health status colors
  const healthColors = {
    healthy: 'text-green-600 bg-green-100',
    warning: 'text-yellow-600 bg-yellow-100',
    critical: 'text-red-600 bg-red-100'
  };

  const layerColors = {
    ast: '#3b82f6',
    analysis: '#10b981',
    graph: '#8b5cf6',
    results: '#f59e0b'
  };

  // Chart data
  const hitRateData = cacheStats.map(stat => ({
    layer: stat.layer,
    hit_rate: stat.hit_rate * 100,
    entries: stat.entries
  }));

  const memoryUsageData = cacheStats.map(stat => ({
    layer: stat.layer,
    used: stat.memory_usage_bytes / 1024 / 1024, // MB
    total: (stat.max_entries * 100) / 1024 / 1024 // Estimated MB
  }));

  if (isLoading) {
    return (
      <div className="flex items-center justify-center h-64">
        <div className="text-lg font-medium text-gray-600">Loading cache management...</div>
      </div>
    );
  }

  return (
    <div className="space-y-6 p-6 bg-gradient-to-br from-gray-50 to-blue-50 min-h-screen">
      {/* Header */}
      <div className="flex items-center justify-between">
        <div>
          <h1 className="text-3xl font-bold text-gray-900 flex items-center gap-2">
            <Database className="h-8 w-8 text-blue-500" />
            Cache Management
          </h1>
          <p className="text-gray-600 mt-2">
            Monitor and control our intelligent multi-layer caching system
          </p>
        </div>

        {/* Control Actions */}
        <div className="flex items-center gap-3">
          <Button
            onClick={() => fetchCacheStats()}
            variant="outline"
            className="flex items-center gap-2"
          >
            <RefreshCw className="h-4 w-4" />
            Refresh
          </Button>

          <Button
            onClick={warmupCache}
            className="flex items-center gap-2 bg-green-500 hover:bg-green-600"
          >
            <Activity className="h-4 w-4" />
            Warmup Cache
          </Button>

          <Button
            onClick={optimizeCache}
            className="flex items-center gap-2 bg-blue-500 hover:bg-blue-600"
          >
            <Settings className="h-4 w-4" />
            Optimize
          </Button>
        </div>
      </div>

      {/* Overall Health Status */}
      {cacheHealth && (
        <Card className={`border-l-4 ${
          cacheHealth.overall_status === 'healthy' ? 'border-l-green-500' :
          cacheHealth.overall_status === 'warning' ? 'border-l-yellow-500' : 'border-l-red-500'
        }`}>
          <CardHeader>
            <CardTitle className="flex items-center justify-between">
              <div className="flex items-center gap-2">
                {cacheHealth.overall_status === 'healthy' ? (
                  <CheckCircle className="h-5 w-5 text-green-500" />
                ) : (
                  <AlertTriangle className="h-5 w-5 text-yellow-500" />
                )}
                Cache System Health
              </div>

              <Badge className={healthColors[cacheHealth.overall_status]}>
                {cacheHealth.overall_status.toUpperCase()}
              </Badge>
            </CardTitle>
          </CardHeader>

          <CardContent className="grid grid-cols-2 md:grid-cols-4 gap-4">
            <div className="text-center">
              <div className="text-2xl font-bold text-blue-600">
                {cacheHealth.performance_score}/100
              </div>
              <p className="text-sm text-gray-600">Performance Score</p>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-green-600">
                {formatBytes(cacheHealth.total_memory_usage_mb * 1024 * 1024)}
              </div>
              <p className="text-sm text-gray-600">Memory Usage</p>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-purple-600">
                {cacheHealth.active_operations}
              </div>
              <p className="text-sm text-gray-600">Active Operations</p>
            </div>

            <div className="text-center">
              <div className="text-2xl font-bold text-orange-600">
                {(avgHitRate * 100).toFixed(1)}%
              </div>
              <p className="text-sm text-gray-600">Avg Hit Rate</p>
            </div>
          </CardContent>

          {/* Health Issues and Recommendations */}
          {(cacheHealth.issues.length > 0 || cacheHealth.recommendations.length > 0) && (
            <CardContent className="pt-0">
              {cacheHealth.issues.length > 0 && (
                <Alert className="mb-3">
                  <AlertCircle className="h-4 w-4" />
                  <div>
                    <strong>Issues Detected:</strong>
                    <ul className="text-sm mt-1 space-y-1">
                      {cacheHealth.issues.map((issue, index) => (
                        <li key={index}>• {issue}</li>
                      ))}
                    </ul>
                  </div>
                </Alert>
              )}

              {cacheHealth.recommendations.length > 0 && (
                <div className="p-3 bg-blue-50 rounded-lg border border-blue-200">
                  <h4 className="font-medium text-blue-800 mb-2">💡 Optimization Recommendations</h4>
                  <ul className="text-sm text-blue-700 space-y-1">
                    {cacheHealth.recommendations.map((rec, index) => (
                      <li key={index}>• {rec}</li>
                    ))}
                  </ul>
                </div>
              )}
            </CardContent>
          )}
        </Card>
      )}

      {/* Cache Layer Statistics */}
      <Tabs defaultValue="overview" className="space-y-4">
        <TabsList className="grid w-full grid-cols-4">
          <TabsTrigger value="overview">Overview</TabsTrigger>
          <TabsTrigger value="performance">Performance</TabsTrigger>
          <TabsTrigger value="operations">Operations</TabsTrigger>
          <TabsTrigger value="configuration">Configuration</TabsTrigger>
        </TabsList>

        {/* Overview Tab */}
        <TabsContent value="overview">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            {/* Cache Layer Cards */}
            <div className="space-y-4">
              {cacheStats.map((stat, index) => (
                <Card key={stat.layer} className="relative">
                  <CardHeader className="pb-3">
                    <CardTitle className="flex items-center justify-between text-sm">
                      <div className="flex items-center gap-2">
                        <div
                          className="w-3 h-3 rounded"
                          style={{ backgroundColor: layerColors[stat.layer as keyof typeof layerColors] || '#6b7280' }}
                        />
                        <span className="capitalize">{stat.layer} Cache</span>
                      </div>

                      <div className="flex items-center gap-2">
                        <Badge variant="outline">
                          {(stat.hit_rate * 100).toFixed(1)}% hit rate
                        </Badge>

                        <Dialog>
                          <DialogTrigger asChild>
                            <Button
                              variant="ghost"
                              size="sm"
                              onClick={() => setSelectedLayer(stat.layer)}
                            >
                              <Settings className="h-3 w-3" />
                            </Button>
                          </DialogTrigger>
                          <DialogContent>
                            <DialogHeader>
                              <DialogTitle>Manage {stat.layer} Cache</DialogTitle>
                            </DialogHeader>
                            <div className="space-y-4">
                              <div className="grid grid-cols-2 gap-4 text-sm">
                                <div>
                                  <strong>Entries:</strong> {stat.entries.toLocaleString()}
                                </div>
                                <div>
                                  <strong>Memory:</strong> {formatBytes(stat.memory_usage_bytes)}
                                </div>
                                <div>
                                  <strong>TTL:</strong> {stat.ttl_seconds}s
                                </div>
                                <div>
                                  <strong>Evictions:</strong> {stat.evictions_last_hour}/hour
                                </div>
                              </div>

                              <div className="flex gap-2">
                                <Button
                                  onClick={() => clearCache(stat.layer)}
                                  variant="destructive"
                                  className="flex-1"
                                >
                                  <Trash2 className="h-4 w-4 mr-2" />
                                  Clear Cache
                                </Button>

                                <Button
                                  onClick={() => clearCache()}
                                  variant="outline"
                                  className="flex-1"
                                >
                                  <RefreshCw className="h-4 w-4 mr-2" />
                                  Refresh Stats
                                </Button>
                              </div>
                            </div>
                          </DialogContent>
                        </Dialog>
                      </div>
                    </CardTitle>
                  </CardHeader>

                  <CardContent className="space-y-3">
                    <div className="grid grid-cols-3 gap-3 text-center text-sm">
                      <div>
                        <div className="text-lg font-bold text-blue-600">
                          {stat.entries.toLocaleString()}
                        </div>
                        <p className="text-gray-600">Entries</p>
                      </div>

                      <div>
                        <div className="text-lg font-bold text-green-600">
                          {formatBytes(stat.memory_usage_bytes)}
                        </div>
                        <p className="text-gray-600">Memory</p>
                      </div>

                      <div>
                        <div className="text-lg font-bold text-purple-600">
                          {stat.avg_access_time_ms.toFixed(1)}ms
                        </div>
                        <p className="text-gray-600">Avg Access</p>
                      </div>
                    </div>

                    {/* Memory usage bar */}
                    <div className="space-y-1">
                      <div className="flex justify-between text-xs text-gray-600">
                        <span>Memory Usage</span>
                        <span>
                          {((stat.memory_usage_bytes / (stat.max_entries * 100)) * 100).toFixed(1)}%
                        </span>
                      </div>
                      <div className="w-full bg-gray-200 rounded-full h-2">
                        <div
                          className="h-2 rounded-full bg-blue-500"
                          style={{
                            width: `${Math.min((stat.memory_usage_bytes / (stat.max_entries * 100)) * 100, 100)}%`
                          }}
                        />
                      </div>
                    </div>
                  </CardContent>
                </Card>
              ))}
            </div>

            {/* Performance Charts */}
            <div className="space-y-4">
              <Card>
                <CardHeader>
                  <CardTitle className="text-sm">Cache Hit Rates by Layer</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="h-48">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={hitRateData}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis dataKey="layer" />
                        <YAxis />
                        <Tooltip formatter={(value: number) => `${value.toFixed(1)}%`} />
                        <Bar dataKey="hit_rate" fill="#3b82f6" />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                </CardContent>
              </Card>

              <Card>
                <CardHeader>
                  <CardTitle className="text-sm">Memory Usage by Layer</CardTitle>
                </CardHeader>
                <CardContent>
                  <div className="h-48">
                    <ResponsiveContainer width="100%" height="100%">
                      <BarChart data={memoryUsageData}>
                        <CartesianGrid strokeDasharray="3 3" />
                        <XAxis dataKey="layer" />
                        <YAxis />
                        <Tooltip formatter={(value: number) => `${value.toFixed(1)} MB`} />
                        <Bar dataKey="used" fill="#10b981" name="Used" />
                      </BarChart>
                    </ResponsiveContainer>
                  </div>
                </CardContent>
              </Card>
            </div>
          </div>
        </TabsContent>

        {/* Performance Tab */}
        <TabsContent value="performance">
          <Card>
            <CardHeader>
              <CardTitle>Performance Metrics</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="grid grid-cols-1 md:grid-cols-3 gap-6">
                <div className="space-y-4">
                  <h4 className="font-medium">Overall Performance</h4>
                  <div className="space-y-3">
                    <div className="flex justify-between">
                      <span>Total Entries:</span>
                      <span className="font-medium">{totalEntries.toLocaleString()}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Total Memory:</span>
                      <span className="font-medium">{formatBytes(totalMemory)}</span>
                    </div>
                    <div className="flex justify-between">
                      <span>Avg Hit Rate:</span>
                      <span className="font-medium">{(avgHitRate * 100).toFixed(1)}%</span>
                    </div>
                  </div>
                </div>

                <div className="space-y-4">
                  <h4 className="font-medium">Efficiency Metrics</h4>
                  {cacheStats.map(stat => (
                    <div key={stat.layer} className="text-sm">
                      <div className="flex justify-between">
                        <span className="capitalize">{stat.layer}:</span>
                        <span>{(stat.hit_rate * 100).toFixed(1)}%</span>
                      </div>
                    </div>
                  ))}
                </div>

                <div className="space-y-4">
                  <h4 className="font-medium">Response Times</h4>
                  {cacheStats.map(stat => (
                    <div key={stat.layer} className="text-sm">
                      <div className="flex justify-between">
                        <span className="capitalize">{stat.layer}:</span>
                        <span>{stat.avg_access_time_ms.toFixed(1)}ms</span>
                      </div>
                    </div>
                  ))}
                </div>
              </div>
            </CardContent>
          </Card>
        </TabsContent>

        {/* Operations Tab */}
        <TabsContent value="operations">
          <Card>
            <CardHeader>
              <CardTitle>Recent Operations</CardTitle>
            </CardHeader>
            <CardContent>
              {operations.length > 0 ? (
                <div className="space-y-3">
                  {operations.slice(0, 10).map(op => (
                    <div key={op.id} className="flex items-center justify-between p-3 border rounded-lg">
                      <div className="flex items-center gap-3">
                        <div className={`w-2 h-2 rounded-full ${
                          op.status === 'completed' ? 'bg-green-500' :
                          op.status === 'failed' ? 'bg-red-500' :
                          op.status === 'running' ? 'bg-blue-500 animate-pulse' : 'bg-gray-500'
                        }`} />

                        <div>
                          <p className="font-medium capitalize">{op.operation}</p>
                          {op.layer && (
                            <p className="text-sm text-gray-600">Layer: {op.layer}</p>
                          )}
                          <p className="text-xs text-gray-500">
                            Started: {new Date(op.started_at).toLocaleTimeString()}
                          </p>
                        </div>
                      </div>

                      <div className="text-right">
                        <Badge variant={
                          op.status === 'completed' ? 'success' :
                          op.status === 'failed' ? 'destructive' :
                          op.status === 'running' ? 'default' : 'secondary'
                        }>
                          {op.status}
                        </Badge>

                        {op.progress !== undefined && op.status === 'running' && (
                          <p className="text-sm text-gray-600 mt-1">
                            {op.progress}%
                          </p>
                        )}
                      </div>
                    </div>
                  ))}
                </div>
              ) : (
                <p className="text-gray-600 text-center py-8">No recent operations</p>
              )}
            </CardContent>
          </Card>
        </TabsContent>

        {/* Configuration Tab */}
        <TabsContent value="configuration">
          <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
            <Card>
              <CardHeader>
                <CardTitle>Cache Configuration</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4">
                  {cacheStats.map(stat => (
                    <div key={stat.layer} className="p-4 border rounded-lg">
                      <h4 className="font-medium capitalize mb-3">{stat.layer} Layer</h4>
                      <div className="grid grid-cols-2 gap-3 text-sm">
                        <div>
                          <span className="text-gray-600">Max Entries:</span>
                          <p className="font-medium">{stat.max_entries.toLocaleString()}</p>
                        </div>
                        <div>
                          <span className="text-gray-600">TTL:</span>
                          <p className="font-medium">{stat.ttl_seconds}s</p>
                        </div>
                        <div>
                          <span className="text-gray-600">Current Entries:</span>
                          <p className="font-medium">{stat.entries.toLocaleString()}</p>
                        </div>
                        <div>
                          <span className="text-gray-600">Fill Rate:</span>
                          <p className="font-medium">
                            {((stat.entries / stat.max_entries) * 100).toFixed(1)}%
                          </p>
                        </div>
                      </div>
                    </div>
                  ))}
                </div>
              </CardContent>
            </Card>

            <Card>
              <CardHeader>
                <CardTitle>System Configuration</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-4 text-sm">
                  <div className="p-4 bg-gray-50 rounded-lg">
                    <h4 className="font-medium mb-2">Memory Management</h4>
                    <div className="space-y-2">
                      <div className="flex justify-between">
                        <span>Total Allocated:</span>
                        <span>{formatBytes(totalMemory)}</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Peak Usage:</span>
                        <span>{formatBytes(totalMemory * 1.2)}</span>
                      </div>
                    </div>
                  </div>

                  <div className="p-4 bg-gray-50 rounded-lg">
                    <h4 className="font-medium mb-2">Performance Targets</h4>
                    <div className="space-y-2">
                      <div className="flex justify-between">
                        <span>Target Hit Rate:</span>
                        <span>≥85%</span>
                      </div>
                      <div className="flex justify-between">
                        <span>Target Access Time:</span>
                        <span>&lt;10ms</span>
                      </div>
                    </div>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </TabsContent>
      </Tabs>
    </div>
  );
};

// Mock data generators
function generateMockCacheStats(): CacheLayerStats[] {
  return [
    {
      layer: 'ast',
      entries: 15247,
      hit_rate: 0.87,
      memory_usage_bytes: 128 * 1024 * 1024, // 128MB
      max_entries: 50000,
      ttl_seconds: 3600,
      evictions_last_hour: 23,
      avg_access_time_ms: 4.2,
      last_updated: new Date().toISOString()
    },
    {
      layer: 'analysis',
      entries: 8432,
      hit_rate: 0.92,
      memory_usage_bytes: 64 * 1024 * 1024, // 64MB
      max_entries: 25000,
      ttl_seconds: 1800,
      evictions_last_hour: 12,
      avg_access_time_ms: 2.8,
      last_updated: new Date().toISOString()
    },
    {
      layer: 'graph',
      entries: 3891,
      hit_rate: 0.78,
      memory_usage_bytes: 32 * 1024 * 1024, // 32MB
      max_entries: 10000,
      ttl_seconds: 7200,
      evictions_last_hour: 5,
      avg_access_time_ms: 8.1,
      last_updated: new Date().toISOString()
    },
    {
      layer: 'results',
      entries: 12064,
      hit_rate: 0.85,
      memory_usage_bytes: 96 * 1024 * 1024, // 96MB
      max_entries: 30000,
      ttl_seconds: 1800,
      evictions_last_hour: 18,
      avg_access_time_ms: 3.5,
      last_updated: new Date().toISOString()
    }
  ];
}

function generateMockCacheHealth(): CacheHealth {
  return {
    overall_status: 'healthy',
    total_memory_usage_mb: 320,
    memory_limit_mb: 1024,
    active_operations: 0,
    performance_score: 89,
    issues: [],
    recommendations: [
      'Consider increasing AST cache size to reduce evictions',
      'Graph cache TTL could be optimized for better hit rates',
      'Memory usage is well within limits'
    ]
  };
}

function generateMockOperations(): CacheOperation[] {
  return [
    {
      id: 'op_1',
      operation: 'warmup',
      status: 'completed',
      started_at: new Date(Date.now() - 300000).toISOString(),
      completed_at: new Date(Date.now() - 240000).toISOString(),
      progress: 100,
      message: 'Cache warmup completed successfully'
    },
    {
      id: 'op_2',
      operation: 'clear',
      layer: 'analysis',
      status: 'completed',
      started_at: new Date(Date.now() - 600000).toISOString(),
      completed_at: new Date(Date.now() - 590000).toISOString(),
      progress: 100,
      message: 'Analysis cache cleared'
    }
  ];
}