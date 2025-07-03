import { CheckCircle } from 'lucide-react';
import React from 'react';
import { Headline, Body } from '../ui/Typography';

const PricingSection: React.FC = () => {
  return (
    <section id="pricing" className="py-20 px-6 bg-secondary-900/20">
      <div className="container mx-auto">
        <div className="text-center mb-16">
          <Headline size="lg" className="mb-4">
            Open Source. Always Free.
          </Headline>
          <Body size="lg" color="secondary">
            Built by the community, for the community
          </Body>
        </div>
        
        <div className="flex justify-center max-w-2xl mx-auto">
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
              Everything you need for architectural analysis. No limits, no tracking, no premium features locked away.
            </p>
            <ul className="space-y-3 text-secondary-200 mb-8">
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>Local AI Analysis (Ollama)</span>
              </li>
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>God Object & Cycle Detection</span>
              </li>
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>Code Duplication Analysis</span>
              </li>
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>Rust, Python, JavaScript Support</span>
              </li>
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>Markdown Reports</span>
              </li>
              <li className="flex items-center space-x-3">
                <CheckCircle className="w-5 h-5 text-success-400" />
                <span>Community Support & Contributions</span>
              </li>
            </ul>
            <button className="w-full px-6 py-3 bg-success-500 hover:bg-success-600 text-white font-semibold rounded-lg transition-all duration-300 hover:shadow-glow-success">
              Download & Get Started
            </button>
          </div>
        </div>
      </div>
    </section>
  );
};

export default PricingSection;