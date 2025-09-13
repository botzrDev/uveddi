import { useState, useEffect, useCallback } from 'react';

export interface ApiState<T> {
  data: T | null;
  loading: boolean;
  error: string | null;
  retry: () => void;
}

export function useApiData<T>(
  apiCall: () => Promise<T>,
  dependencies: any[] = []
): ApiState<T> {
  const [state, setState] = useState<ApiState<T>>({
    data: null,
    loading: true,
    error: null,
    retry: () => {}
  });

  const fetchData = useCallback(async () => {
    setState(prev => ({ ...prev, loading: true, error: null }));

    try {
      const data = await apiCall();
      setState(prev => ({ ...prev, data, loading: false }));
    } catch (error) {
      const errorMessage = error instanceof Error ? error.message : 'Unknown error occurred';
      setState(prev => ({
        ...prev,
        error: errorMessage,
        loading: false
      }));
    }
  }, [apiCall, ...dependencies]);

  useEffect(() => {
    fetchData();
  }, [fetchData]);

  return {
    ...state,
    retry: fetchData
  };
}