import React from 'react';
import { useMutation, useQuery } from '@tanstack/react-query';
import { authAPI, userAPI } from '../lib/api';
import { useAuth as useAuthStore } from '../store/auth';
import type { LoginRequest, RegisterRequest } from '../types/api';

export const useAuth = () => {
  const authStore = useAuthStore();

  // Login mutation
  const loginMutation = useMutation({
    mutationFn: authAPI.login,
    onSuccess: (data) => {
      authStore.login(data.user, data.access_token);
    },
    onError: (error: any) => {
      authStore.setError(error.detail || 'Login failed');
    },
  });

  // Register mutation
  const registerMutation = useMutation({
    mutationFn: authAPI.register,
    onSuccess: (data) => {
      authStore.login(data.user, data.access_token);
    },
    onError: (error: any) => {
      authStore.setError(error.detail || 'Registration failed');
    },
  });

  // Logout mutation
  const logoutMutation = useMutation({
    mutationFn: authAPI.logout,
    onSuccess: () => {
      authStore.logout();
    },
    onError: () => {
      // Logout locally even if server call fails
      authStore.logout();
    },
  });

  const login = (credentials: LoginRequest) => {
    authStore.setLoading(true);
    authStore.clearError();
    loginMutation.mutate(credentials);
  };

  const register = (userData: RegisterRequest) => {
    authStore.setLoading(true);
    authStore.clearError();
    registerMutation.mutate(userData);
  };

  const logout = () => {
    logoutMutation.mutate();
  };

  return {
    ...authStore,
    login,
    register,
    logout,
    isLoginLoading: loginMutation.isPending,
    isRegisterLoading: registerMutation.isPending,
    isLogoutLoading: logoutMutation.isPending,
  };
};

// Hook to fetch current user data
export const useCurrentUser = () => {
  const { isAuthenticated, setUser, setError } = useAuthStore();

  const query = useQuery({
    queryKey: ['currentUser'],
    queryFn: userAPI.getCurrentUser,
    enabled: isAuthenticated,
  });

  React.useEffect(() => {
    if (query.data) {
      setUser(query.data);
    }
    if (query.error) {
      setError((query.error as any).detail || 'Failed to fetch user data');
    }
  }, [query.data, query.error, setUser, setError]);

  return query;
};