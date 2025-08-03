import { useMutation, useQuery, useQueryClient } from '@tanstack/react-query';
import { projectAPI } from '../lib/api';
import type { ProjectCreate } from '../types/api';

export const useProject = (projectId?: number) => {
  return useQuery({
    queryKey: ['project', projectId],
    queryFn: () => projectAPI.getProject(projectId!),
    enabled: !!projectId,
  });
};

export const useOrganizationProjects = (
  organizationId?: number,
  limit: number = 100,
  offset: number = 0
) => {
  return useQuery({
    queryKey: ['organizationProjects', organizationId, limit, offset],
    queryFn: () => projectAPI.getOrganizationProjects(organizationId!, limit, offset),
    enabled: !!organizationId,
  });
};

export const useCreateProject = () => {
  const queryClient = useQueryClient();

  return useMutation({
    mutationFn: (projectData: ProjectCreate) => 
      projectAPI.createProject(projectData),
    onSuccess: (newProject) => {
      // Invalidate organization projects queries
      queryClient.invalidateQueries({ 
        queryKey: ['organizationProjects', newProject.organization_id] 
      });
      queryClient.setQueryData(
        ['project', newProject.project_id], 
        newProject
      );
    },
  });
};