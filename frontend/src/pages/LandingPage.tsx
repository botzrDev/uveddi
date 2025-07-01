import {
  AlertTriangle,
  ArrowRight,
  CheckCircle,
  Code2,
  Download,
  ExternalLink,
  GitBranch,
  Github,
  GitlabIcon as Gitlab,
  MonitorSpeaker,
  Play,
  Search,
  Settings,
  Shield,
  Star,
  TrendingUp,
  Zap
} from 'lucide-react';
import React from 'react';
import { Link } from 'react-router-dom';
import AnimatedTerminal from '../components/ui/AnimatedTerminal';
import FeatureCard from '../components/ui/FeatureCard';
import GradientButton from '../components/ui/GradientButton';
import StatsCard from '../components/ui/StatsCard';

const LandingPage: React.FC = () => {
  const terminalCommands = [
    {
      command: 'uveddi analyze ./your-repo --level=high',
      output: [
        '> Analyzing codebase architecture...',
        '> Found 3 critical architectural issues',
        '> Detected 15 potential security vulnerabilities',
        '> Generated analysis report: ./reports/architectural_analysis.md',
        '',
        '✅ Analysis complete in 2.3s'
      ],
      delay: 2000,
    },
    {
      command: 'uveddi explain --issue="cyclic-dependency" --ai',
      output: [
        '> Using AI to explain architectural issue...',
        '',
        '🔍 Cyclic Dependency Analysis:',
        '   Your modules A ↔ B create a circular dependency.',
        '   This increases coupling and makes testing difficult.',
        '',
        '💡 Suggested fix: Use dependency inversion pattern',
        '   Create an interface to break the cycle.',
      ],
      delay: 3000,
    }
  ];

  return (
    <div className="bg-secondary-900 bg-gray-900 text-white min-h-screen">
      {/* Header */}
      <header className="sticky top-0 bg-secondary-900/80 backdrop-blur-md z-50 border-b border-secondary-800">
        <nav className="container mx-auto px-6 py-4">
          <div className="flex justify-between items-center">
            <div className="flex items-center space-x-2">
              <div className="w-8 h-8 bg-gradient-to-r from-primary-500 to-primary-600 rounded-lg flex items-center justify-center">
                <Code2 className="w-5 h-5 text-white" />
              </div>
              <span className="text-2xl font-bold bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
                uveddi
              </span>
            </div>
            <div className="hidden md:flex items-center space-x-8">
              <a href="#features" className="text-secondary-300 hover:text-primary-400 transition-colors">Features</a>
              <a href="#use-cases" className="text-secondary-300 hover:text-primary-400 transition-colors">Use Cases</a>
              <a href="#integrations" className="text-secondary-300 hover:text-primary-400 transition-colors">Integrations</a>
              <a href="#pricing" className="text-secondary-300 hover:text-primary-400 transition-colors">Pricing</a>
              <a href="#docs" className="text-secondary-300 hover:text-primary-400 transition-colors">Docs</a>
            </div>
            <div className="flex items-center space-x-4">
              <Link to="/login" className="text-secondary-300 hover:text-primary-400 transition-colors">
                Sign In
              </Link>
              <GradientButton href="/register">
                Get Started Free
              </GradientButton>
            </div>
          </div>
        </nav>
      </header>

      <main>
        {/* Hero Section */}
        <section className="relative py-20 px-6 overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-secondary-900 from-gray-900 via-secondary-800 via-gray-800 to-secondary-900 to-gray-900" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary-900/20 from-green-900/20 via-transparent to-transparent" />
          
          <div className="container mx-auto relative z-10">
            <div className="text-center max-w-4xl mx-auto mb-12">
              <div className="inline-flex items-center bg-secondary-800 bg-gray-800 border border-secondary-700 border-gray-700 rounded-full px-4 py-2 mb-6">
                <Star className="w-4 h-4 text-accent-400 text-yellow-400 mr-2" />
                <span className="text-sm text-secondary-300 text-gray-300">Trusted by developers for high-level code analysis</span>
              </div>
              
              <h1 className="text-5xl md:text-7xl font-bold mb-6 animate-slide-up">
                <span className="bg-gradient-to-r from-white to-secondary-300 to-gray-300 bg-clip-text text-transparent">
                  Code Analysis
                </span>
                <br />
                <span className="bg-gradient-to-r from-primary-400 from-green-400 to-primary-500 to-green-500 bg-clip-text text-transparent">
                  Instantly Intelligent
                </span>
              </h1>
              
              <p className="text-xl text-secondary-300 text-gray-300 mb-8 leading-relaxed max-w-3xl mx-auto">
                Transform complex codebases into actionable intelligence. Detect architectural anti-patterns, 
                security vulnerabilities, and technical debt before they compromise your projects.
                <span className="text-primary-400 text-green-400 font-semibold"> AI-powered insights for human-written and AI-generated code.</span>
              </p>
              
              <div className="flex flex-col sm:flex-row justify-center space-y-4 sm:space-y-0 sm:space-x-4 mb-12">
                <GradientButton size="lg" className="group">
                  <Download className="w-5 h-5 mr-2 group-hover:animate-bounce-gentle" />
                  Download CLI
                </GradientButton>
                <GradientButton variant="secondary" size="lg" className="group">
                  <Play className="w-5 h-5 mr-2" />
                  Watch Demo
                </GradientButton>
              </div>
            </div>

            {/* Terminal Demo */}
            <div className="max-w-4xl mx-auto">
              <AnimatedTerminal 
                commands={terminalCommands}
                className="shadow-2xl shadow-primary-500/20"
                loop={true}
              />
            </div>
          </div>
        </section>

        {/* Trust Signals */}
        <section className="py-16 px-6 bg-secondary-800/50">
          <div className="container mx-auto">
            <div className="text-center mb-8">
              <p className="text-secondary-400 text-sm font-medium">Trusted by developers at</p>
            </div>
            <div className="flex justify-center items-center space-x-12 opacity-60">
              <Github className="w-8 h-8" />
              <Gitlab className="w-8 h-8" />
              <span className="text-xl font-bold">VS Code</span>
              <span className="text-xl font-bold">Jenkins</span>
              <span className="text-xl font-bold">CircleCI</span>
            </div>
          </div>
        </section>

        {/* Problem/Solution Section */}
        <section className="py-20 px-6">
          <div className="container mx-auto">
            <div className="grid lg:grid-cols-2 gap-12 items-center">
              <div>
                <h2 className="text-4xl font-bold mb-6">
                  <span className="text-red-400">The Problem:</span> Code Quality Chaos
                </h2>
                <div className="space-y-4 text-secondary-300">
                  <div className="flex items-start space-x-3">
                    <AlertTriangle className="w-6 h-6 text-red-400 mt-1 flex-shrink-0" />
                    <p>Manual code reviews take hours and miss critical architectural issues</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <AlertTriangle className="w-6 h-6 text-red-400 mt-1 flex-shrink-0" />
                    <p>AI-generated code lacks quality validation and security oversight</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <AlertTriangle className="w-6 h-6 text-red-400 mt-1 flex-shrink-0" />
                    <p>Technical debt accumulates silently until it becomes expensive to fix</p>
                  </div>
                </div>
              </div>
              
              <div>
                <h2 className="text-4xl font-bold mb-6">
                  <span className="text-primary-400">The Solution:</span> Shift-Left Intelligence
                </h2>
                <div className="space-y-4 text-secondary-300">
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-primary-400 mt-1 flex-shrink-0" />
                    <p>Instant architectural analysis directly in your terminal</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-primary-400 mt-1 flex-shrink-0" />
                    <p>AI-powered explanations make complex issues understandable</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-primary-400 mt-1 flex-shrink-0" />
                    <p>Catch problems early, before they reach production</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <section id="features" className="py-20 px-6 bg-secondary-800/30">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold mb-4">
                Actionable Intelligence, Not Just Data
              </h2>
              <p className="text-xl text-secondary-300 max-w-3xl mx-auto">
                Advanced static analysis meets AI-powered insights to give you the complete picture of your codebase health.
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8">
              <FeatureCard
                icon={Search}
                title="Architectural Anti-Patterns"
                description="Detect complex issues like Cyclic Dependencies, God Objects, and Leaky Abstractions before they compromise your codebase."
                gradient="from-red-500 to-red-600"
              />
              <FeatureCard
                icon={Shield}
                title="AI-Powered Security"
                description="Leverage local or cloud-based AI to get human-readable security explanations and actionable refactoring suggestions."
                gradient="from-blue-500 to-blue-600"
              />
              <FeatureCard
                icon={GitBranch}
                title="Dependency Visualization"
                description="Generate Mermaid.js diagrams directly in your reports to visualize dependencies and understand complex relationships."
                gradient="from-purple-500 to-purple-600"
              />
              <FeatureCard
                icon={Zap}
                title="Lightning Fast"
                description="Built with Rust for maximum performance. Analyze entire codebases in seconds, not minutes."
                gradient="from-accent-500 to-accent-600"
              />
              <FeatureCard
                icon={Code2}
                title="Multi-Language Support"
                description="Comprehensive analysis for JavaScript, TypeScript, Python, Rust, Java, and more languages coming soon."
                gradient="from-primary-500 to-primary-600"
              />
              <FeatureCard
                icon={Settings}
                title="CI/CD Integration"
                description="Seamlessly integrate with GitHub Actions, GitLab CI, Jenkins, and other popular CI/CD platforms."
                gradient="from-indigo-500 to-indigo-600"
              />
            </div>
          </div>
        </section>

        {/* Use Cases Section */}
        <section id="use-cases" className="py-20 px-6">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold mb-4">
                Shift-Left Your Architectural Review
              </h2>
              <p className="text-xl text-secondary-300">
                Empower developers to catch and fix issues early in the development cycle
              </p>
            </div>
            
            <div className="grid lg:grid-cols-2 gap-12 items-center">
              <div>
                <h3 className="text-3xl font-bold mb-6">For the Modern Developer</h3>
                <p className="text-secondary-300 mb-6 text-lg leading-relaxed">
                  Integrate high-level analysis directly into your workflow. Catch architectural drift early, 
                  improve code quality, and spend less time on manual reviews. 
                  <span className="text-primary-400 font-semibold">
                    Uveddi empowers you to build better software, faster.
                  </span>
                </p>
                
                <div className="grid sm:grid-cols-2 gap-4 mb-8">
                  <StatsCard
                    title="Faster Reviews"
                    value="15x"
                    description="Reduce manual review time"
                    icon={<TrendingUp className="w-6 h-6" />}
                  />
                  <StatsCard
                    title="Early Detection"
                    value="80%"
                    description="Issues caught pre-deployment"
                    icon={<Shield className="w-6 h-6" />}
                  />
                </div>
                
                <div className="space-y-3">
                  <div className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span className="text-secondary-300">Proactive Technical Debt Management</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span className="text-secondary-300">Automated Code Reviews</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span className="text-secondary-300">Real-time Tech Debt Prevention</span>
                  </div>
                  <div className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span className="text-secondary-300">AI-Generated Code Safeguards</span>
                  </div>
                </div>
              </div>
              
              <div className="bg-secondary-800 border border-secondary-700 rounded-xl p-6">
                <h4 className="text-lg font-semibold mb-4 text-primary-400">AI-Generated Report Snippet</h4>
                <div className="space-y-3 font-mono text-sm">
                  <div className="flex items-start space-x-2">
                    <span className="text-red-400 font-bold">[CRITICAL]</span>
                    <span className="text-white">Cyclic Dependency Detected</span>
                  </div>
                  <p className="text-secondary-300 ml-8">
                    Module `UserService` has a circular dependency with `AuthService`.
                  </p>
                  
                  <div className="bg-secondary-900 border border-secondary-600 rounded p-3 my-4">
                    <pre className="text-primary-400 text-xs">
{`graph TD
    UserService --> AuthService
    AuthService --> UserService
    style UserService fill:#ef4444
    style AuthService fill:#ef4444`}
                    </pre>
                  </div>
                  
                  <div className="text-secondary-300">
                    <p className="mb-2"><span className="text-primary-400">💡 AI Suggestion:</span></p>
                    <p className="ml-4">Refactor to use dependency inversion pattern. Create an `IAuthProvider` interface to break the cycle and improve testability.</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Integrations Section */}
        <section id="integrations" className="py-20 px-6 bg-secondary-800/30">
          <div className="container mx-auto text-center">
            <h2 className="text-4xl font-bold mb-4">
              Integrates With Your Workflow
            </h2>
            <p className="text-xl text-secondary-300 mb-12">
              Uveddi works seamlessly with your existing tools and platforms
            </p>
            
            <div className="grid grid-cols-2 md:grid-cols-4 lg:grid-cols-6 gap-8 items-center">
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <Github className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">GitHub</span>
              </div>
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <Gitlab className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">GitLab</span>
              </div>
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <Settings className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">Jenkins</span>
              </div>
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <Code2 className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">VS Code</span>
              </div>
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <MonitorSpeaker className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">CircleCI</span>
              </div>
              <div className="flex flex-col items-center space-y-2 hover:scale-110 transition-transform">
                <GitBranch className="w-12 h-12 text-secondary-400 hover:text-primary-400 transition-colors" />
                <span className="text-sm text-secondary-400">Bitbucket</span>
              </div>
            </div>
          </div>
        </section>

        {/* Pricing Section */}
        <section id="pricing" className="py-20 px-6">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold mb-4">
                Flexible Pricing for Every Team
              </h2>
              <p className="text-xl text-secondary-300">
                Start free, scale as you grow
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-6xl mx-auto">
              {/* Community */}
              <div className="bg-secondary-800 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-colors">
                <h3 className="text-2xl font-bold mb-2">Community</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold">$0</span>
                  <span className="text-secondary-400 ml-2">/forever</span>
                </div>
                <p className="text-secondary-300 mb-6">
                  Perfect for individuals and small teams getting started with architectural analysis.
                </p>
                <ul className="space-y-3 text-secondary-300 mb-8">
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Local LLM Analysis</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Core Anti-Pattern Detection</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Markdown Reports</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Community Support</span>
                  </li>
                </ul>
                <GradientButton className="w-full">
                  Get Started Free
                </GradientButton>
              </div>

              {/* Pro */}
              <div className="bg-secondary-800 border-2 border-primary-500 rounded-xl p-8 relative overflow-hidden">
                <div className="absolute top-0 right-0 bg-primary-500 text-white px-3 py-1 text-sm font-semibold">
                  POPULAR
                </div>
                <h3 className="text-2xl font-bold mb-2">Pro</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold">$29</span>
                  <span className="text-secondary-400 ml-2">/month</span>
                </div>
                <p className="text-secondary-300 mb-6">
                  Enhanced features for professional developers and growing teams.
                </p>
                <ul className="space-y-3 text-secondary-300 mb-8">
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Everything in Community</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Cloud-Based AI Analysis</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Advanced Security Scanning</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>CI/CD Integration</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Priority Support</span>
                  </li>
                </ul>
                <GradientButton className="w-full">
                  Start Pro Trial
                </GradientButton>
              </div>

              {/* Enterprise */}
              <div className="bg-secondary-800 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-colors">
                <h3 className="text-2xl font-bold mb-2">Enterprise</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold">Custom</span>
                </div>
                <p className="text-secondary-300 mb-6">
                  Advanced features, security, and support for large organizations.
                </p>
                <ul className="space-y-3 text-secondary-300 mb-8">
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Everything in Pro</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>On-Premise Deployment</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>SSO & Advanced Security</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>Custom Integrations</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-primary-400" />
                    <span>24/7 Support & SLA</span>
                  </li>
                </ul>
                <GradientButton variant="secondary" className="w-full">
                  Contact Sales
                </GradientButton>
              </div>
            </div>
          </div>
        </section>

        {/* Final CTA Section */}
        <section className="py-20 px-6 bg-gradient-to-r from-primary-900/20 to-secondary-900">
          <div className="container mx-auto text-center">
            <h2 className="text-4xl font-bold mb-4">
              Ready to Transform Your Code Analysis?
            </h2>
            <p className="text-xl text-secondary-300 mb-8 max-w-2xl mx-auto">
              Join thousands of developers who trust uveddi to keep their codebases healthy, 
              secure, and maintainable.
            </p>
            <div className="flex flex-col sm:flex-row justify-center space-y-4 sm:space-y-0 sm:space-x-4">
              <GradientButton size="lg" className="group">
                Download CLI
                <ArrowRight className="w-5 h-5 ml-2 group-hover:translate-x-1 transition-transform" />
              </GradientButton>
              <GradientButton variant="outline" size="lg">
                <ExternalLink className="w-5 h-5 mr-2" />
                View Documentation
              </GradientButton>
            </div>
          </div>
        </section>
      </main>

      {/* Footer */}
      <footer className="bg-secondary-900 border-t border-secondary-800 py-12">
        <div className="container mx-auto px-6">
          <div className="grid md:grid-cols-4 gap-8">
            <div>
              <div className="flex items-center space-x-2 mb-4">
                <div className="w-8 h-8 bg-gradient-to-r from-primary-500 to-primary-600 rounded-lg flex items-center justify-center">
                  <Code2 className="w-5 h-5 text-white" />
                </div>
                <span className="text-xl font-bold bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
                  uveddi
                </span>
              </div>
              <p className="text-secondary-400 text-sm">
                High-level code analysis for the modern developer.
              </p>
            </div>
            
            <div>
              <h4 className="font-semibold text-white mb-4">Product</h4>
              <ul className="space-y-2 text-secondary-400 text-sm">
                <li><a href="#" className="hover:text-primary-400 transition-colors">Features</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Pricing</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Integrations</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">API</a></li>
              </ul>
            </div>
            
            <div>
              <h4 className="font-semibold text-white mb-4">Resources</h4>
              <ul className="space-y-2 text-secondary-400 text-sm">
                <li><a href="#" className="hover:text-primary-400 transition-colors">Documentation</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Blog</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Community</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Support</a></li>
              </ul>
            </div>
            
            <div>
              <h4 className="font-semibold text-white mb-4">Company</h4>
              <ul className="space-y-2 text-secondary-400 text-sm">
                <li><a href="#" className="hover:text-primary-400 transition-colors">About</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Privacy Policy</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Terms of Service</a></li>
                <li><a href="#" className="hover:text-primary-400 transition-colors">Contact</a></li>
              </ul>
            </div>
          </div>
          
          <div className="border-t border-secondary-800 mt-8 pt-8 text-center text-secondary-400 text-sm">
            <p>&copy; 2025 uveddi. All rights reserved.</p>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default LandingPage;
