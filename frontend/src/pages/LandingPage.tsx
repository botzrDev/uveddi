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
  Search,
  Settings,
  Shield,
  Star,
  TrendingUp,
  Zap
} from 'lucide-react';
import React from 'react';
import Header from '../components/layout/Header';
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
    <div id="top" className="bg-secondary-950 text-white min-h-screen">
      {/* Use shared Header component */}
      <Header variant="landing" />

      <main>
        {/* Hero Section */}
        <section className="relative py-24 px-6 overflow-hidden">
          <div className="absolute inset-0 bg-gradient-to-br from-secondary-950 via-secondary-900 to-primary-950/20" />
          <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary-900/20 via-transparent to-transparent" />
          
          <div className="container mx-auto relative z-10">
            <div className="text-center max-w-4xl mx-auto mb-16">
              <div className="inline-flex items-center bg-secondary-800/50 border border-secondary-700 rounded-full px-4 py-2 mb-8">
                <Star className="w-4 h-4 text-accent-400 mr-2" />
                <span className="text-sm text-secondary-300">Trusted by developers for high-level code analysis</span>
              </div>
              {/* === HERO HEADLINE A/B TEST === */}
              {/* Option A: Architectural Debt Angle */}
              {/* <h1 className="font-display text-display-sm md:text-display-md lg:text-display-lg font-display mb-8 animate-slide-up leading-tight tracking-tight">
                <span className="bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent">
                  Stop Fixing Symptoms.
                </span>
                <br />
                <span className="bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
                  Eradicate Architectural Debt.
                </span>
              </h1> */}
              {/* Option B: AI Code Architect Angle */}
              <h1 className="font-display text-display-sm md:text-display-md lg:text-display-lg font-display mb-8 animate-slide-up leading-tight tracking-tight">
                <span className="bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent">
                  Your AI Co-pilot Writes the Code.
                </span>
                <br />
                <span className="bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
                  Uveddi Ensures the Architecture is Sound.
                </span>
              </h1>
              {/* === END HERO HEADLINE A/B TEST === */}

              <p className="font-sans text-body-lg font-body text-secondary-200 mb-6 leading-relaxed max-w-3xl mx-auto">
                AI generates code at lightning speed, but it can't see the bigger picture. Uveddi is the essential verification layer that analyzes your codebase's architecture, ensuring AI-generated code is maintainable, scalable, and won't create long-term technical debt.
                <span className="text-primary-400 font-emphasis block mt-2">Accelerate onboarding by 30%. Ship features 2x faster. Cut technical debt at the root.</span>
              </p>

              {/* CLI Install Command Section */}
              <div className="mb-6">
                <div className="bg-secondary-900 border border-secondary-700 rounded-lg px-6 py-4 inline-flex items-center mx-auto text-left shadow-lg">
                  <span className="font-mono text-primary-400 text-base select-all">curl -sSL https://uveddi.dev/install.sh | bash</span>
                  <button className="ml-4 px-3 py-1 bg-primary-500 hover:bg-primary-600 text-white text-xs rounded transition-all" onClick={() => {navigator.clipboard.writeText('curl -sSL https://uveddi.dev/install.sh | bash')}}>Copy</button>
                </div>
                <p className="text-xs text-secondary-400 mt-2">Get started in under 2 minutes. No account or credit card required.</p>
              </div>

              {/* Single focused CTA */}
              <div className="mb-8">
                <button className="group px-8 py-4 bg-success-500 hover:bg-success-600 text-white text-lg font-semibold rounded-xl transition-all duration-300 hover:shadow-glow-success hover:scale-105 flex items-center mx-auto">
                  <Download className="w-6 h-6 mr-3 group-hover:animate-bounce-gentle" />
                  Download CLI & Analyze Free
                  <ArrowRight className="w-5 h-5 ml-3 group-hover:translate-x-1 transition-transform" />
                </button>
                <p className="text-sm text-secondary-400 mt-3">Works with any codebase • Instant results • CLI-first</p>
              </div>
            </div>

            {/* Terminal Demo */}
            <div className="max-w-4xl mx-auto">
              <AnimatedTerminal 
                commands={terminalCommands}
                className="shadow-2xl shadow-primary-500/20 border border-secondary-700/50"
                loop={true}
              />
            </div>
          </div>
        </section>

        {/* Trust Signals */}
        <section className="py-20 px-6 bg-secondary-900/30 border-y border-secondary-800/50">
          <div className="container mx-auto">
            <div className="text-center mb-12">
              <p className="text-secondary-300 text-sm font-medium uppercase tracking-wider">Integrates with your existing workflow</p>
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
              <p className="text-secondary-400 text-sm">Works with any Git repository • CLI-first approach • CI/CD ready</p>
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
                <h2 className="text-4xl font-bold mb-6">
                  <span className="text-success-400">The Solution:</span> Shift-Left Intelligence
                </h2>
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
                title="Architectural Bottleneck Detection"
                description="Stop firefighting symptoms. Instantly surface the root causes—like Cyclic Dependencies and God Objects—that slow your team and block delivery."
                gradient="from-red-500 to-red-600"
              />
              <FeatureCard
                icon={Shield}
                title="AI-Powered Security Insights"
                description="Catch vulnerabilities before they become incidents. Uveddi explains risks in plain English and recommends fixes, empowering you to act with confidence."
                gradient="from-blue-500 to-blue-600"
              />
              <FeatureCard
                icon={GitBranch}
                title="Visual Dependency Maps"
                description="See the big picture. Instantly generate interactive diagrams to understand and untangle complex relationships. Accelerate onboarding by 30%."
                gradient="from-purple-500 to-purple-600"
              />
              <FeatureCard
                icon={Zap}
                title="Lightning-Fast CLI"
                description="Analyze massive codebases in seconds, not minutes. Uveddi’s Rust-powered CLI fits seamlessly into your workflow and CI/CD."
                gradient="from-accent-500 to-accent-600"
              />
              <FeatureCard
                icon={Code2}
                title="Multi-Language, One Tool"
                description="Unify your analysis. Scan JavaScript, TypeScript, Python, Rust, Java, and more—no context switching, no extra setup."
                gradient="from-primary-500 to-primary-600"
              />
              <FeatureCard
                icon={Settings}
                title="Seamless CI/CD Integration"
                description="Automate architectural checks in every pipeline. Prevent costly issues from ever reaching production."
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

        {/* Testimonials Section */}
        <section className="py-20 px-6">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold mb-4">
                Trusted by Developers Worldwide
              </h2>
              <p className="text-xl text-secondary-200">
                See how uveddi is transforming code quality for teams everywhere
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-6xl mx-auto">
              {/* Testimonial 1: Architect Persona */}
              <div className="bg-secondary-800/50 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-all duration-300">
                <div className="flex items-start space-x-4 mb-6">
                  <div className="w-12 h-12 bg-gradient-to-r from-primary-500 to-primary-600 rounded-full flex items-center justify-center text-white font-bold text-lg">
                    JD
                  </div>
                  <div>
                    <h4 className="font-semibold text-white">Jessica Davis</h4>
                    <p className="text-secondary-300 text-sm">Principal Architect</p>
                    <div className="flex items-center space-x-2 mt-1">
                      <div className="w-6 h-6 bg-blue-600 rounded flex items-center justify-center">
                        <span className="text-white text-xs font-bold">A</span>
                      </div>
                      <span className="text-secondary-400 text-sm">Acme Corp</span>
                    </div>
                  </div>
                </div>
                <div className="mb-4">
                  <div className="flex text-accent-400 mb-2">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <blockquote className="text-secondary-200 italic">
                    "Uveddi helped us <span className="text-success-400 font-semibold">reduce onboarding time by 40%</span> and eliminated critical architectural debt that slowed our releases."
                  </blockquote>
                </div>
                <div className="text-sm text-secondary-400">
                  2x faster feature delivery, 0 major regressions in 12 months
                </div>
              </div>

              {/* Testimonial 2: DevOps Persona */}
              <div className="bg-secondary-800/50 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-all duration-300">
                <div className="flex items-start space-x-4 mb-6">
                  <div className="w-12 h-12 bg-gradient-to-r from-success-500 to-success-600 rounded-full flex items-center justify-center text-white font-bold text-lg">
                    RK
                  </div>
                  <div>
                    <h4 className="font-semibold text-white">Rahul Kumar</h4>
                    <p className="text-secondary-300 text-sm">DevOps Lead</p>
                    <div className="flex items-center space-x-2 mt-1">
                      <div className="w-6 h-6 bg-green-600 rounded flex items-center justify-center">
                        <span className="text-white text-xs font-bold">S</span>
                      </div>
                      <span className="text-secondary-400 text-sm">ScaleUp</span>
                    </div>
                  </div>
                </div>
                <div className="mb-4">
                  <div className="flex text-accent-400 mb-2">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <blockquote className="text-secondary-200 italic">
                    "CI/CD integration with Uveddi <span className="text-success-400 font-semibold">prevented 5 production incidents</span> last quarter. Our team now fixes issues before they ever reach users."
                  </blockquote>
                </div>
                <div className="text-sm text-secondary-400">
                  $200K+ saved in incident response costs
                </div>
              </div>

              {/* Testimonial 3: CTO Persona */}
              <div className="bg-secondary-800/50 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-all duration-300">
                <div className="flex items-start space-x-4 mb-6">
                  <div className="w-12 h-12 bg-gradient-to-r from-action-500 to-action-600 rounded-full flex items-center justify-center text-white font-bold text-lg">
                    LC
                  </div>
                  <div>
                    <h4 className="font-semibold text-white">Lina Chen</h4>
                    <p className="text-secondary-300 text-sm">CTO</p>
                    <div className="flex items-center space-x-2 mt-1">
                      <div className="w-6 h-6 bg-purple-600 rounded flex items-center justify-center">
                        <span className="text-white text-xs font-bold">E</span>
                      </div>
                      <span className="text-secondary-400 text-sm">EnterpriseX</span>
                    </div>
                  </div>
                </div>
                <div className="mb-4">
                  <div className="flex text-accent-400 mb-2">
                    {[...Array(5)].map((_, i) => (
                      <Star key={i} className="w-4 h-4 fill-current" />
                    ))}
                  </div>
                  <blockquote className="text-secondary-200 italic">
                    "With Uveddi, we <span className="text-success-400 font-semibold">cut technical debt by 60%</span> and improved team morale. The AI explanations make complex issues actionable for everyone."
                  </blockquote>
                </div>
                <div className="text-sm text-secondary-400">
                  60% less tech debt, 3x faster onboarding
                </div>
              </div>
            </div>

            <div className="text-center mt-12">
              <div className="flex flex-col md:flex-row justify-center items-center space-y-4 md:space-y-0 md:space-x-8 text-secondary-300">
                <div className="flex items-center space-x-2">
                  <TrendingUp className="w-5 h-5 text-success-400" />
                  <span>10,000+ repositories analyzed</span>
                </div>
                <div className="flex items-center space-x-2">
                  <Shield className="w-5 h-5 text-success-400" />
                  <span>500+ vulnerabilities caught early</span>
                </div>
                <div className="flex items-center space-x-2">
                  <Zap className="w-5 h-5 text-success-400" />
                  <span>Average 60% faster code reviews</span>
                </div>
              </div>
            </div>
          </div>
        </section>

        {/* Pricing Section */}
        <section id="pricing" className="py-20 px-6 bg-secondary-900/20">
          <div className="container mx-auto">
            <div className="text-center mb-16">
              <h2 className="text-4xl font-bold mb-4">
                Flexible Pricing for Every Team
              </h2>
              <p className="text-xl text-secondary-200">
                Start free, scale as you grow
              </p>
            </div>
            
            <div className="grid md:grid-cols-2 lg:grid-cols-3 gap-8 max-w-6xl mx-auto">
              {/* Community - FREE TIER HIGHLIGHTED */}
              <div className="bg-secondary-800 border-2 border-success-500 rounded-xl p-8 relative overflow-hidden shadow-lg shadow-success-500/20 transform hover:scale-105 transition-all duration-300">
                <div className="absolute top-0 right-0 bg-success-500 text-white px-4 py-2 text-sm font-bold">
                  FREE FOREVER
                </div>
                <h3 className="text-2xl font-bold mb-2 text-success-400">Community</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold text-success-400">$0</span>
                  <span className="text-secondary-300 ml-2">/forever</span>
                </div>
                <p className="text-secondary-200 mb-6">
                  Perfect for individuals and small teams getting started with architectural analysis.
                </p>
                <ul className="space-y-3 text-secondary-200 mb-8">
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-success-400" />
                    <span>Local LLM Analysis</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-success-400" />
                    <span>Core Anti-Pattern Detection</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-success-400" />
                    <span>Markdown Reports</span>
                  </li>
                  <li className="flex items-center space-x-3">
                    <CheckCircle className="w-5 h-5 text-success-400" />
                    <span>Community Support</span>
                  </li>
                </ul>
                <button className="w-full px-6 py-3 bg-success-500 hover:bg-success-600 text-white font-semibold rounded-lg transition-all duration-300 hover:shadow-glow-success">
                  Get Started Free
                </button>
              </div>

              {/* Pro */}
              <div className="bg-secondary-800 border border-primary-500/50 rounded-xl p-8 relative overflow-hidden hover:border-primary-500 transition-all duration-300">
                <div className="absolute top-0 right-0 bg-primary-500 text-white px-3 py-1 text-sm font-semibold">
                  POPULAR
                </div>
                <h3 className="text-2xl font-bold mb-2">Pro</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold">$29</span>
                  <span className="text-secondary-300 ml-2">/month</span>
                </div>
                <p className="text-secondary-200 mb-6">
                  Enhanced features for professional developers and growing teams.
                </p>
                <ul className="space-y-3 text-secondary-200 mb-8">
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
                <button className="w-full px-6 py-3 bg-primary-500 hover:bg-primary-600 text-white font-semibold rounded-lg transition-all duration-300 hover:shadow-glow">
                  Start Pro Trial
                </button>
              </div>

              {/* Enterprise */}
              <div className="bg-secondary-800 border border-secondary-700 rounded-xl p-8 hover:border-secondary-600 transition-colors">
                <h3 className="text-2xl font-bold mb-2">Enterprise</h3>
                <div className="flex items-baseline mb-4">
                  <span className="text-5xl font-bold">Custom</span>
                </div>
                <p className="text-secondary-200 mb-6">
                  Advanced features, security, and support for large organizations.
                </p>
                <ul className="space-y-3 text-secondary-200 mb-8">
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
                <button className="w-full px-6 py-3 bg-action-500 hover:bg-action-600 text-white font-semibold rounded-lg transition-all duration-300 hover:shadow-glow-action">
                  Contact Sales
                </button>
              </div>
            </div>
          </div>
        </section>

        {/* Final CTA Section */}
        <section className="py-20 px-6 bg-gradient-to-r from-primary-900/20 to-secondary-900">
          <div className="container mx-auto text-center">
            <h2 className="text-4xl font-bold mb-4">
              Ready to Take Control of Your Architecture?
            </h2>
            <p className="text-xl text-secondary-300 mb-8 max-w-2xl mx-auto">
              Join thousands of empowered developers who trust Uveddi to keep their codebases healthy, secure, and maintainable—so you can focus on building, not firefighting.
            </p>
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
            <p className="text-sm text-secondary-400 mt-4">No credit card required • Works with any codebase • Get actionable results in minutes</p>
          </div>
        </section>

        {/* FAQ Section */}
        <section id="faq" className="py-16 px-6 bg-secondary-950 border-t border-secondary-800">
          <div className="container mx-auto max-w-3xl">
            <h3 className="text-2xl font-bold mb-8 text-primary-400 text-center">Frequently Asked Questions</h3>
            <div className="space-y-6">
              <div>
                <h4 className="font-semibold text-secondary-200 mb-2">Is Uveddi only for AI-generated code?</h4>
                <p className="text-secondary-400">No. Uveddi analyzes both human-written and AI-generated code, surfacing architectural issues and technical debt in any codebase.</p>
              </div>
              <div>
                <h4 className="font-semibold text-secondary-200 mb-2">Do I need to upload my code to the cloud?</h4>
                <p className="text-secondary-400">No. Uveddi runs locally via CLI, so your code never leaves your machine unless you opt in to cloud features.</p>
              </div>
              <div>
                <h4 className="font-semibold text-secondary-200 mb-2">How fast is the analysis?</h4>
                <p className="text-secondary-400">Most codebases are analyzed in seconds, thanks to Uveddi’s Rust-powered engine.</p>
              </div>
              <div>
                <h4 className="font-semibold text-secondary-200 mb-2">What languages are supported?</h4>
                <p className="text-secondary-400">JavaScript, TypeScript, Python, Rust, Java, and more coming soon.</p>
              </div>
              <div>
                <h4 className="font-semibold text-secondary-200 mb-2">Is there a free tier?</h4>
                <p className="text-secondary-400">Yes! The Community plan is free forever and includes core analysis features.</p>
              </div>
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
