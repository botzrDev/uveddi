import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { organizationAPI } from '../lib/api';
import type { OrganizationCreate } from '../types/api';

export const useOrganization = (organizationId?: number) => {
  return useQuery({
    queryKey: ['organization', organizationId],
    queryFn: () => organizationAPI.getOrganization(organizationId!),
    enabled: !!organizationId,
  });
};

export const useOrganizationUsers = (organizationId?: number) => {
  return useQuery({
    queryKey: ['organizationUsers', organizationId],
    queryFn: () => organizationAPI.getOrganizationUsers(organizationId!),
    enabled: !!organizationId,
  });
};

export const useCreateOrganization = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (orgData: OrganizationCreate) => 
      organizationAPI.createOrganization(orgData),
    onSuccess: (newOrganization) => {
      // Invalidate and refetch organization queries
      queryClient.invalidateQueries({ queryKey: ['organization'] });
      queryClient.setQueryData(
        ['organization', newOrganization.organization_id], 
        newOrganization
      );
    },
  });
};