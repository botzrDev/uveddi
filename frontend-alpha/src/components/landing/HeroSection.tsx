import { Star } from 'lucide-react';
import React from 'react';
import AnimatedTerminal from '../ui/AnimatedTerminal';
import { Body, Display } from '../ui/Typography';

interface HeroSectionProps {
  terminalCommands: Array<{
    command: string;
    output: string[];
    delay: number;
  }>;
}

const HeroSection: React.FC<HeroSectionProps> = ({ terminalCommands }) => {
  const handleCopyCommand = () => {
    navigator.clipboard.writeText('curl -sSL https://uveddi.org/install.sh | bash');
  };

  return (
    <section className="relative py-24 px-6 overflow-hidden" data-testid="hero-section">
      <div className="absolute inset-0 bg-gradient-to-br from-secondary-950 via-secondary-900 to-primary-950/20" />
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary-900/20 via-transparent to-transparent" />
      
      <div className="container mx-auto relative z-10">
        <div className="text-center max-w-4xl mx-auto mb-16">
          <div className="inline-flex items-center bg-secondary-800/50 border border-secondary-700 rounded-full px-4 py-2 mb-8">
            <Star className="w-4 h-4 text-accent-400 mr-2" />
            <span className="text-sm text-secondary-300">Join the open source community building better software architecture</span>
          </div>

          <Display size="lg" className="mb-8 animate-slide-up">
            <span className="bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent">
              Open Source Architectural Analysis
            </span>
            <br />
            <span className="bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
              Built by Developers, for Developers
            </span>
          </Display>

          <Body size="lg" color="secondary" className="mb-6 leading-relaxed max-w-3xl mx-auto">
            Stop architectural drift before it becomes technical debt. Uveddi's community-driven CLI analyzes your codebase locally, detecting God Objects, cyclic dependencies, and other anti-patterns that slow your team down.
            <span className="text-primary-400 font-emphasis block mt-2">100% free. 100% local. 100% open source.</span>
          </Body>

          {/* CLI Install Command Section */}
          <div className="mb-6">
            <div className="bg-secondary-900 border border-secondary-700 rounded-lg px-6 py-4 inline-flex items-center mx-auto text-left shadow-lg">
              <span className="font-mono text-primary-400 text-base select-all">curl -sSL https://uveddi.org/install.sh | bash</span>
              <button 
                className="ml-4 px-3 py-1 bg-primary-500 hover:bg-primary-600 text-white text-xs rounded transition-all" 
                onClick={handleCopyCommand}
                data-testid="get-started-button"
              >
                Copy
              </button>
            </div>
            <p className="text-xs text-secondary-400 mt-2">Open source CLI - no account, no tracking, no limits.</p>
          </div>

        </div>

        {/* Terminal Demo */}
        <div className="max-w-4xl mx-auto">
          <AnimatedTerminal 
            commands={terminalCommands}
            className="shadow-2xl shadow-primary-500/20 border border-secondary-700/50"
            loop={true}
            data-testid="animated-terminal"
          />
        </div>
      </div>
    </section>
  );
};

export default HeroSection;