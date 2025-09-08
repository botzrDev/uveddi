// React Query hooks for report data fetching
import { useQuery, UseQueryResult } from '@tanstack/react-query';
import { apiService } from '@/services/api';
import type { InteractiveReport, DependencyGraph } from '@/types/api';

/**
 * Hook to fetch a specific report by ID
 */
export function useReport(id: string): UseQueryResult<InteractiveReport, Error> {
  return useQuery({
    queryKey: ['report', id],
    queryFn: () => apiService.getReport(id),
    staleTime: 5 * 60 * 1000, // 5 minutes
    gcTime: 10 * 60 * 1000, // 10 minutes (formerly cacheTime)
    retry: 2,
    retryDelay: (attemptIndex) => Math.min(1000 * 2 ** attemptIndex, 30000),
  });
}

/**
 * Hook to fetch the latest analysis report
 */
export function useLatestReport(): UseQueryResult<InteractiveReport, Error> {
  return useQuery({
    queryKey: ['report', 'latest'],
    queryFn: () => apiService.getLatestReport(),
    staleTime: 5 * 60 * 1000, // 5 minutes for analysis data
    gcTime: 10 * 60 * 1000, // 10 minutes
    retry: 2,
  });
}

/**
 * Hook to fetch dependency graph for a report
 */
export function useDependencyGraph(reportId: string): UseQueryResult<DependencyGraph, Error> {
  return useQuery({
    queryKey: ['dependencyGraph', reportId],
    queryFn: () => apiService.getDependencyGraph(reportId),
    staleTime: 10 * 60 * 1000, // 10 minutes
    gcTime: 20 * 60 * 1000, // 20 minutes
    retry: 2,
  });
}

/**
 * Hook to list available reports
 */
export function useReportList(params: {
  page?: number;
  limit?: number;
  sort?: string;
  order?: 'asc' | 'desc';
} = {}) {
  return useQuery({
    queryKey: ['reports', 'list', params],
    queryFn: () => apiService.listReports(params),
    staleTime: 2 * 60 * 1000, // 2 minutes
    gcTime: 5 * 60 * 1000, // 5 minutes
    retry: 2,
  });
}

/**
 * Hook to check API health
 */
export function useHealthStatus() {
  return useQuery({
    queryKey: ['health'],
    queryFn: () => apiService.getHealthStatus(),
    staleTime: 30 * 1000, // 30 seconds
    gcTime: 2 * 60 * 1000, // 2 minutes
    retry: 3,
    retryDelay: 1000,
  });
}

/**
 * Hook to get API metrics
 */
export function useMetrics() {
  return useQuery({
    queryKey: ['metrics'],
    queryFn: () => apiService.getMetrics(),
    staleTime: 30 * 1000, // 30 seconds
    gcTime: 2 * 60 * 1000, // 2 minutes
    retry: 1,
    refetchInterval: 60 * 1000, // Refetch every minute
  });
}