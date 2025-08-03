import React from 'react';
import { Link } from 'react-router-dom';
import { Plus, Activity, AlertCircle, CheckCircle, TrendingUp } from 'lucide-react';
import { useCurrentUser } from '../hooks/useAuth';
import { useOrganizationProjects } from '../hooks/useProjects';
import { useAuth } from '../store/auth';
import Layout from '../components/layout/Layout';
import Card, { CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import Button from '../components/ui/Button';

const DashboardPage: React.FC = () => {
  const { user } = useAuth();
  const { isLoading: userLoading } = useCurrentUser();
  const { data: projects, isLoading: projectsLoading } = useOrganizationProjects(
    user?.organization_id
  );

  if (userLoading) {
    return (
      <Layout>
        <div className="flex items-center justify-center h-64">
          <div className="animate-spin rounded-full h-32 w-32 border-b-2 border-green-600"></div>
        </div>
      </Layout>
    );
  }

  const stats = [
    {
      title: 'Total Projects',
      value: projects?.length || 0,
      icon: Activity,
      color: 'text-blue-600',
      bgColor: 'bg-blue-100 dark:bg-blue-900',
    },
    {
      title: 'Active Issues',
      value: '23', // This would come from API
      icon: AlertCircle,
      color: 'text-red-600',
      bgColor: 'bg-red-100 dark:bg-red-900',
    },
    {
      title: 'Resolved This Week',
      value: '12', // This would come from API
      icon: CheckCircle,
      color: 'text-green-600',
      bgColor: 'bg-green-100 dark:bg-green-900',
    },
    {
      title: 'Code Quality Score',
      value: '87%', // This would come from API
      icon: TrendingUp,
      color: 'text-purple-600',
      bgColor: 'bg-purple-100 dark:bg-purple-900',
    },
  ];

  return (
    <Layout>
      <div className="space-y-8">
        {/* Welcome Section */}
        <div className="flex justify-between items-center">
          <div>
            <h1 className="text-3xl font-bold text-gray-900 dark:text-white">
              Welcome back, {user?.username}!
            </h1>
            <p className="text-gray-600 dark:text-gray-400 mt-2">
              Here's what's happening with your projects today.
            </p>
          </div>
          <Link to="/projects/new">
            <Button variant="primary" size="lg">
              <Plus className="w-5 h-5 mr-2" />
              New Project
            </Button>
          </Link>
        </div>

        {/* Stats Grid */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-6">
          {stats.map((stat, index) => {
            const Icon = stat.icon;
            return (
              <Card key={index}>
                <CardContent className="flex items-center">
                  <div className={`p-3 rounded-lg ${stat.bgColor}`}>
                    <Icon className={`w-6 h-6 ${stat.color}`} />
                  </div>
                  <div className="ml-4">
                    <p className="text-sm font-medium text-gray-600 dark:text-gray-400">
                      {stat.title}
                    </p>
                    <p className="text-2xl font-bold text-gray-900 dark:text-white">
                      {stat.value}
                    </p>
                  </div>
                </CardContent>
              </Card>
            );
          })}
        </div>

        {/* Recent Projects */}
        <div className="grid grid-cols-1 lg:grid-cols-2 gap-8">
          <Card>
            <CardHeader>
              <CardTitle>Recent Projects</CardTitle>
            </CardHeader>
            <CardContent>
              {projectsLoading ? (
                <div className="flex items-center justify-center h-32">
                  <div className="animate-spin rounded-full h-8 w-8 border-b-2 border-green-600"></div>
                </div>
              ) : projects && projects.length > 0 ? (
                <div className="space-y-4">
                  {projects.slice(0, 5).map((project) => (
                    <div 
                      key={project.project_id}
                      className="flex items-center justify-between p-3 border border-gray-200 dark:border-gray-700 rounded-lg"
                    >
                      <div>
                        <h4 className="font-medium text-gray-900 dark:text-white">
                          {project.name}
                        </h4>
                        <p className="text-sm text-gray-500 dark:text-gray-400">
                          {new Date(project.created_at).toLocaleDateString()}
                        </p>
                      </div>
                      <Link to={`/projects/${project.project_id}`}>
                        <Button variant="outline" size="sm">
                          View
                        </Button>
                      </Link>
                    </div>
                  ))}
                  <Link to="/projects">
                    <Button variant="ghost" className="w-full">
                      View All Projects
                    </Button>
                  </Link>
                </div>
              ) : (
                <div className="text-center py-8">
                  <p className="text-gray-500 dark:text-gray-400 mb-4">
                    No projects yet. Get started by creating your first project.
                  </p>
                  <Link to="/projects/new">
                    <Button variant="primary">
                      <Plus className="w-4 h-4 mr-2" />
                      Create Project
                    </Button>
                  </Link>
                </div>
              )}
            </CardContent>
          </Card>

          {/* Recent Activity */}
          <Card>
            <CardHeader>
              <CardTitle>Recent Activity</CardTitle>
            </CardHeader>
            <CardContent>
              <div className="text-center py-8">
                <p className="text-gray-500 dark:text-gray-400">
                  No recent activity. Run your first analysis to see results here.
                </p>
              </div>
            </CardContent>
          </Card>
        </div>

        {/* Quick Actions */}
        <Card>
          <CardHeader>
            <CardTitle>Quick Actions</CardTitle>
          </CardHeader>
          <CardContent>
            <div className="grid grid-cols-1 md:grid-cols-3 gap-4">
              <Link to="/projects/new">
                <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                  <Plus className="w-8 h-8 text-green-600 mb-2" />
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    Create New Project
                  </h4>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    Set up a new codebase for analysis
                  </p>
                </div>
              </Link>
              <Link to="/cli-setup">
                <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                  <Activity className="w-8 h-8 text-blue-600 mb-2" />
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    Download CLI
                  </h4>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    Get the uveddi command-line tool
                  </p>
                </div>
              </Link>
              <Link to="/organization">
                <div className="p-4 border border-gray-200 dark:border-gray-700 rounded-lg hover:bg-gray-50 dark:hover:bg-gray-800 transition-colors">
                  <TrendingUp className="w-8 h-8 text-purple-600 mb-2" />
                  <h4 className="font-medium text-gray-900 dark:text-white">
                    Manage Team
                  </h4>
                  <p className="text-sm text-gray-500 dark:text-gray-400">
                    Invite team members and manage access
                  </p>
                </div>
              </Link>
            </div>
          </CardContent>
        </Card>
      </div>
    </Layout>
  );
};

export default DashboardPage;