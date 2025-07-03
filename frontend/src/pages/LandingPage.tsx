import {
  AlertTriangle,
  ArrowRight,
  CheckCircle,
  Code2,
  ExternalLink,
  GitBranch,
  Github,
  GitlabIcon as Gitlab,
  MonitorSpeaker,
  Settings,
  Shield,
  TrendingUp
} from 'lucide-react';
import React from 'react';
import FeaturesSection from '../components/landing/FeaturesSection';
import HeroSection from '../components/landing/HeroSection';
import PricingSection from '../components/landing/PricingSection';
import TestimonialsSection from '../components/landing/TestimonialsSection';
import Footer from '../components/layout/Footer';
import Header from '../components/layout/Header';
import GradientButton from '../components/ui/GradientButton';
import StatsCard from '../components/ui/StatsCard';
import { Body, Caption, Headline } from '../components/ui/Typography';

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
    <div id="top" className="bg-secondary-950 text-white min-h-screen">
      {/* Use shared Header component */}
      <Header variant="landing" />

      <main>
        {/* Hero Section */}
        <HeroSection terminalCommands={terminalCommands} />

        {/* Trust Signals */}
        <section className="py-20 px-6 bg-secondary-900/30 border-y border-secondary-800/50">
          <div className="container mx-auto">
            <div className="text-center mb-12">
              <Caption color="tertiary" className="font-medium uppercase tracking-wider">Integrates with your existing workflow</Caption>
            </div>
            <div className="grid grid-cols-2 md:grid-cols-5 gap-8 items-center justify-items-center max-w-4xl mx-auto">
              <div className="flex flex-col items-center space-y-2 group">
                <Github className="w-10 h-10 text-secondary-400 group-hover:text-white transition-colors" />
                <span className="text-sm text-secondary-400 group-hover:text-secondary-300 transition-colors">GitHub</span>
              </div>
              <div className="flex flex-col items-center space-y-2 group">
                <Gitlab className="w-10 h-10 text-secondary-400 group-hover:text-white transition-colors" />
                <span className="text-sm text-secondary-400 group-hover:text-secondary-300 transition-colors">GitLab</span>
              </div>
              <div className="flex flex-col items-center space-y-2 group">
                <Settings className="w-10 h-10 text-secondary-400 group-hover:text-white transition-colors" />
                <span className="text-sm text-secondary-400 group-hover:text-secondary-300 transition-colors">VS Code</span>
              </div>
              <div className="flex flex-col items-center space-y-2 group">
                <GitBranch className="w-10 h-10 text-secondary-400 group-hover:text-white transition-colors" />
                <span className="text-sm text-secondary-400 group-hover:text-secondary-300 transition-colors">Jenkins</span>
              </div>
              <div className="flex flex-col items-center space-y-2 group">
                <MonitorSpeaker className="w-10 h-10 text-secondary-400 group-hover:text-white transition-colors" />
                <span className="text-sm text-secondary-400 group-hover:text-secondary-300 transition-colors">CircleCI</span>
              </div>
            </div>
            <div className="text-center mt-12">
              <Caption color="muted">Works with any Git repository � CLI-first approach � CI/CD ready</Caption>
            </div>
          </div>
        </section>

        {/* Problem/Solution Section */}
        <section className="py-20 px-6">
          <div className="container mx-auto">
            <div className="grid lg:grid-cols-2 gap-12 items-center">
              <div>
                <Headline size="lg" className="mb-6">
                  <span className="text-red-400">The Problem:</span> Code Quality Chaos
                </Headline>
                <div className="space-y-4 text-secondary-200">
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
                <Headline size="lg" className="mb-6">
                  <span className="text-success-400">The Solution:</span> Shift-Left Intelligence
                </Headline>
                <div className="space-y-4 text-secondary-200">
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-success-400 mt-1 flex-shrink-0" />
                    <p>Instant architectural analysis directly in your terminal</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-success-400 mt-1 flex-shrink-0" />
                    <p>AI-powered explanations make complex issues understandable</p>
                  </div>
                  <div className="flex items-start space-x-3">
                    <CheckCircle className="w-6 h-6 text-success-400 mt-1 flex-shrink-0" />
                    <p>Catch problems early, before they reach production</p>
                  </div>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Features Section */}
        <FeaturesSection />

        {/* Use Cases Section */}
        <section id="use-cases" className="py-20 px-6">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <Headline size="lg" className="mb-4">
                Shift-Left Your Architectural Review
              </Headline>
              <Body size="lg" color="tertiary">
                Empower developers to catch and fix issues early in the development cycle
              </Body>
            </div>
            
            <div className="grid lg:grid-cols-2 gap-12 items-center">
              <div>
                <Headline size="md" className="mb-6">For the Modern Developer</Headline>
                <Body size="lg" color="tertiary" className="mb-6 leading-relaxed">
                  Integrate high-level analysis directly into your workflow. Catch architectural drift early, 
                  improve code quality, and spend less time on manual reviews. 
                  <span className="text-primary-400 font-semibold">
                    Uveddi empowers you to build better software, faster.
                  </span>
                </Body>
                
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
            <Headline size="lg" className="mb-4">
              Integrates With Your Workflow
            </Headline>
            <Body size="lg" color="tertiary" className="mb-12">
              Uveddi works seamlessly with your existing tools and platforms
            </Body>
            
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

        {/* Testimonials Section */}
        <TestimonialsSection />

        {/* Pricing Section */}
        <PricingSection />

        {/* Final CTA Section */}
        <section className="py-20 px-6 bg-gradient-to-r from-primary-900/20 to-secondary-900">
          <div className="container mx-auto text-center">
            <Headline size="lg" className="mb-4">
              Ready to Take Control of Your Architecture?
            </Headline>
            <Body size="lg" color="tertiary" className="mb-8 max-w-2xl mx-auto">
              Join thousands of empowered developers who trust Uveddi to keep their codebases healthy, secure, and maintainable�so you can focus on building, not firefighting.
            </Body>
            <div className="flex flex-col sm:flex-row justify-center space-y-4 sm:space-y-0 sm:space-x-4">
              <GradientButton size="lg" className="group">
                Download CLI & Analyze Free
                <ArrowRight className="w-5 h-5 ml-2 group-hover:translate-x-1 transition-transform" />
              </GradientButton>
              <GradientButton variant="outline" size="lg">
                <ExternalLink className="w-5 h-5 mr-2" />
                View Documentation
              </GradientButton>
            </div>
            <Caption color="muted" className="mt-4">No credit card required � Works with any codebase � Get actionable results in minutes</Caption>
          </div>
        </section>

        {/* FAQ Section */}
        <section id="faq" className="py-16 px-6 bg-secondary-950 border-t border-secondary-800">
          <div className="container mx-auto max-w-3xl">
            <Headline size="md" color="primary" className="mb-8 text-center">Frequently Asked Questions</Headline>
            <div className="space-y-6">
              <div>
                <Headline size="sm" color="secondary" className="mb-2">Is Uveddi only for AI-generated code?</Headline>
                <Body color="muted">No. Uveddi analyzes both human-written and AI-generated code, surfacing architectural issues and technical debt in any codebase.</Body>
              </div>
              <div>
                <Headline size="sm" color="secondary" className="mb-2">Do I need to upload my code to the cloud?</Headline>
                <Body color="muted">No. Uveddi runs locally via CLI, so your code never leaves your machine unless you opt in to cloud features.</Body>
              </div>
              <div>
                <Headline size="sm" color="secondary" className="mb-2">How fast is the analysis?</Headline>
                <Body color="muted">Most codebases are analyzed in seconds, thanks to Uveddi's Rust-powered engine.</Body>
              </div>
              <div>
                <Headline size="sm" color="secondary" className="mb-2">What languages are supported?</Headline>
                <Body color="muted">JavaScript, TypeScript, Python, Rust, Java, and more coming soon.</Body>
              </div>
              <div>
                <Headline size="sm" color="secondary" className="mb-2">Is there a free tier?</Headline>
                <Body color="muted">Yes! The Community plan is free forever and includes core analysis features.</Body>
              </div>
            </div>
          </div>
        </section>
      </main>

      {/* Footer */}
      <Footer />
    </div>
  );
};

export default LandingPage;
