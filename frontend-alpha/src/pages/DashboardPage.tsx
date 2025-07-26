import React from 'react';
import { ExternalLink, MessageCircle } from 'lucide-react';
import { useAuth } from '../store/auth';
import Layout from '../components/layout/Layout';
import Card, { CardContent, CardHeader, CardTitle } from '../components/ui/Card';
import Button from '../components/ui/Button';

const DashboardPage: React.FC = () => {
  const { user } = useAuth();

  return (
    <Layout>
      <div className="space-y-8">
        {/* Welcome Section */}
        <div className="text-center">
          <h1 className="text-4xl font-bold text-gray-900 dark:text-white mb-4">
            Welcome to Uveddi Alpha Testing
          </h1>
          <p className="text-lg text-gray-600 dark:text-gray-400">
            Hello {user?.username}! Access the documentation and join our community.
          </p>
        </div>

        {/* Main Content Grid */}
        <div className="grid grid-cols-1 lg:grid-cols-3 gap-8">
          {/* Documentation Section - Takes up 2/3 of the space */}
          <div className="lg:col-span-2">
            <Card className="h-full">
              <CardHeader>
                <CardTitle className="flex items-center">
                  <ExternalLink className="w-5 h-5 mr-2" />
                  Interactive Documentation
                </CardTitle>
              </CardHeader>
              <CardContent className="p-0">
                <div className="relative w-full" style={{ height: '600px' }}>
                  <iframe
                    src="/index.html"
                    className="w-full h-full border-0 rounded-b-lg"
                    title="Uveddi Documentation"
                    sandbox="allow-scripts allow-same-origin allow-popups allow-forms"
                  />
                </div>
              </CardContent>
            </Card>
          </div>

          {/* Community Section - Takes up 1/3 of the space */}
          <div className="space-y-6">
            <Card>
              <CardHeader>
                <CardTitle className="flex items-center">
                  <MessageCircle className="w-5 h-5 mr-2" />
                  Join Our Community
                </CardTitle>
              </CardHeader>
              <CardContent>
                <div className="text-center space-y-4">
                  <p className="text-gray-600 dark:text-gray-400">
                    Connect with other alpha testers, share feedback, and get support.
                  </p>
                  <a
                    href="https://discord.com/channels/1391852224040403087/1398760378384781383"
                    target="_blank"
                    rel="noopener noreferrer"
                    className="inline-block"
                  >
                    <Button variant="primary" size="lg" className="w-full">
                      <MessageCircle className="w-5 h-5 mr-2" />
                      Join Discord Server
                    </Button>
                  </a>
                </div>
              </CardContent>
            </Card>

            {/* Quick Info Card */}
            <Card>
              <CardHeader>
                <CardTitle>Alpha Testing Info</CardTitle>
              </CardHeader>
              <CardContent>
                <div className="space-y-3 text-sm">
                  <div>
                    <h4 className="font-medium text-gray-900 dark:text-white">What to Test:</h4>
                    <p className="text-gray-600 dark:text-gray-400">
                      Explore the documentation, try the CLI tools, and report any issues.
                    </p>
                  </div>
                  <div>
                    <h4 className="font-medium text-gray-900 dark:text-white">Feedback:</h4>
                    <p className="text-gray-600 dark:text-gray-400">
                      Share your experience in our Discord channel.
                    </p>
                  </div>
                </div>
              </CardContent>
            </Card>
          </div>
        </div>
      </div>
    </Layout>
  );
};

export default DashboardPage;