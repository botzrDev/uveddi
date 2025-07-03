import { CheckCircle } from 'lucide-react';
import React from 'react';
import { Headline, Body } from '../ui/Typography';

const PricingSection: React.FC = () => {
  return (
    <section id="pricing" className="py-20 px-6 bg-secondary-900/20">
      <div className="container mx-auto">
        <div className="text-center mb-16">
          <Headline size="lg" className="mb-4">
            Flexible Pricing for Every Team
          </Headline>
          <Body size="lg" color="secondary">
            Start free, scale as you grow
          </Body>
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
  );
};

export default PricingSection;