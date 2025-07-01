import React from 'react';
import { Link } from 'react-router-dom';

const LandingPage: React.FC = () => {
  return (
    <div className="bg-gray-900 text-white">
      <header className="sticky top-0 bg-gray-900 bg-opacity-90 backdrop-blur-md z-50">
        <nav className="container mx-auto px-6 py-4 flex justify-between items-center">
          <div className="text-2xl font-bold">uveddi</div>
          <div className="hidden md:flex items-center space-x-8">
            <a href="#features" className="hover:text-green-400">Features</a>
            <a href="#use-cases" className="hover:text-green-400">Use Cases</a>
            <a href="#pricing" className="hover:text-green-400">Pricing</a>
            <a href="#docs" className="hover:text-green-400">Docs</a>
          </div>
          <div className="flex items-center space-x-4">
            <Link to="/register" className="text-sm hover:text-green-400">Sign Up</Link>
            <Link to="/login" className="text-sm hover:text-green-400">Sign In</Link>
            <Link to="/register" className="bg-green-500 hover:bg-green-600 text-white font-bold py-2 px-4 rounded">
              Get Started Free
            </Link>
          </div>
        </nav>
      </header>

      <main>
        <section className="hero text-center py-20 px-6">
          <h1 className="text-5xl font-bold mb-4">uveddi: High-Level Code Analysis, Instantly.</h1>
          <p className="text-xl text-gray-400 mb-8">Transform complex codebases into actionable intelligence, securing and optimizing your projects from the command line.</p>
          <div className="flex justify-center space-x-4">
            <a href="#" className="bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded">Download uveddi CLI</a>
            <a href="#" className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-6 rounded">Request a Demo</a>
          </div>
          <div className="mt-12">
            <div className="bg-gray-800 border border-gray-700 rounded-lg p-4 max-w-4xl mx-auto text-left font-mono text-sm">
              <div className="flex items-center mb-2">
                <span className="text-green-400 mr-2">$</span>
                <span className="text-gray-300">uveddi analyze ./your-repo --level=high</span>
              </div>
              <div className="text-gray-400">
                <p>&gt; Analyzing codebase...</p>
                <p>&gt; Found <span className="text-yellow-400">3 critical</span> architectural issues.</p>
                <p className="text-green-400">&gt; Report generated at: reports/architectural_analysis.md</p>
              </div>
            </div>
          </div>
        </section>

        <section id="features" className="py-20 px-6">
          <div className="container mx-auto">
            <h2 className="text-4xl font-bold text-center mb-12">Actionable Intelligence, Not Just Data</h2>
            <div className="grid md:grid-cols-3 gap-8">
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-2xl font-bold mb-2">Architectural Anti-Patterns</h3>
                <p className="text-gray-400">Detect complex issues like Cyclic Dependencies, God Objects, and Leaky Abstractions before they compromise your codebase.</p>
              </div>
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-2xl font-bold mb-2">AI-Powered Insights</h3>
                <p className="text-gray-400">Leverage local or cloud-based AI to get human-readable explanations and actionable refactoring suggestions.</p>
              </div>
              <div className="bg-gray-800 p-6 rounded-lg">
                <h3 className="text-2xl font-bold mb-2">Visualize Your Architecture</h3>
                <p className="text-gray-400">Generate Mermaid.js diagrams directly in your reports to visualize dependencies and understand complex relationships.</p>
              </div>
            </div>
          </div>
        </section>

        <section id="use-cases" className="py-20 px-6 bg-gray-800">
          <div className="container mx-auto">
            <h2 className="text-4xl font-bold text-center mb-12">Shift-Left Your Architectural Review</h2>
            <div className="grid md:grid-cols-2 gap-8 items-center">
              <div>
                <h3 className="text-3xl font-bold mb-4">For the Modern Developer</h3>
                <p className="text-gray-400 mb-4">Integrate high-level analysis directly into your workflow. Catch architectural drift early, improve code quality, and spend less time on manual reviews. Uveddi empowers you to build better software, faster.</p>
                <ul className="space-y-2 text-gray-300">
                  <li>✓ Proactive Technical Debt Management</li>
                  <li>✓ Automated Code Reviews</li>
                  <li>✓ Real-time Tech Debt Prevention</li>
                </ul>
              </div>
              <div className="bg-gray-900 border border-gray-700 rounded-lg p-4 font-mono text-sm">
                <h4 className="text-lg font-bold mb-4 text-white">AI-Generated Report Snippet</h4>
                <p><span className="font-bold text-red-400">[Critical]</span> Cyclic Dependency Detected</p>
                <p className="text-gray-400">Module `A` has a circular dependency with Module `B`.</p>
                <div className="my-4 p-2 bg-gray-800 rounded">
                  <pre><code>graph TD\n    A --&gt; B\n    B --&gt; A</code></pre>
                </div>
                <p className="text-gray-300">Suggestion: Refactor to use dependency inversion...</p>
              </div>
            </div>
          </div>
        </section>

        <section id="integrations" className="py-20 px-6">
          <div className="container mx-auto text-center">
            <h2 className="text-4xl font-bold mb-4">Integrates With Your Workflow</h2>
            <p className="text-gray-400 mb-8">Uveddi works with your existing tools and platforms.</p>
            <div className="flex justify-center space-x-8">
              <span className="text-2xl font-bold text-gray-500">GitHub</span>
              <span className="text-2xl font-bold text-gray-500">GitLab</span>
              <span className="text-2xl font-bold text-gray-500">Jenkins</span>
              <span className="text-2xl font-bold text-gray-500">VS Code</span>
            </div>
          </div>
        </section>

        <section id="pricing" className="py-20 px-6 bg-gray-800">
          <div className="container mx-auto">
            <h2 className="text-4xl font-bold text-center mb-12">Flexible Pricing for Every Team</h2>
            <div className="grid md:grid-cols-2 gap-8 max-w-4xl mx-auto">
              <div className="bg-gray-900 p-8 rounded-lg border border-gray-700">
                <h3 className="text-2xl font-bold mb-4">Community</h3>
                <p className="text-5xl font-bold mb-4">$0</p>
                <p className="text-gray-400 mb-6">For individuals and small teams getting started with architectural analysis.</p>
                <ul className="space-y-2 text-gray-300 mb-6">
                  <li>✓ Local LLM Analysis</li>
                  <li>✓ Core Anti-Pattern Detection</li>
                  <li>✓ Markdown Reports</li>
                </ul>
                <Link to="/register" className="bg-green-500 hover:bg-green-600 text-white font-bold py-3 px-6 rounded w-full block text-center">Get Started</Link>
              </div>
              <div className="bg-gray-900 p-8 rounded-lg border border-green-500">
                <h3 className="text-2xl font-bold mb-4">Enterprise</h3>
                <p className="text-5xl font-bold mb-4">Custom</p>
                <p className="text-gray-400 mb-6">For organizations requiring advanced features, security, and support.</p>
                <ul className="space-y-2 text-gray-300 mb-6">
                  <li>✓ Cloud-Based LLM Analysis (OpenAI, Anthropic)</li>
                  <li>✓ CI/CD Integration</li>
                  <li>✓ Priority Support & SLA</li>
                </ul>
                <a href="#" className="bg-gray-700 hover:bg-gray-600 text-white font-bold py-3 px-6 rounded w-full block text-center">Contact Sales</a>
              </div>
            </div>
          </div>
        </section>
      </main>

      <footer className="bg-gray-900 text-center py-8">
        <div className="container mx-auto text-gray-500">
          <p>&copy; 2025 uveddi. All rights reserved.</p>
          <div className="flex justify-center space-x-4 mt-4">
            <a href="#" className="hover:text-white">Privacy Policy</a>
            <a href="#" className="hover:text-white">Terms of Service</a>
          </div>
        </div>
      </footer>
    </div>
  );
};

export default LandingPage;
