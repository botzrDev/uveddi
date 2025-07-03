import { ArrowRight, Download, Star } from 'lucide-react';
import React from 'react';
import AnimatedTerminal from '../ui/AnimatedTerminal';
import { Display, Headline, Body, Caption } from '../ui/Typography';

interface HeroSectionProps {
  terminalCommands: Array<{
    command: string;
    output: string[];
    delay: number;
  }>;
}

const HeroSection: React.FC<HeroSectionProps> = ({ terminalCommands }) => {
  const handleCopyCommand = () => {
    navigator.clipboard.writeText('curl -sSL https://uveddi.dev/install.sh | bash');
  };

  return (
    <section className="relative py-24 px-6 overflow-hidden">
      <div className="absolute inset-0 bg-gradient-to-br from-secondary-950 via-secondary-900 to-primary-950/20" />
      <div className="absolute inset-0 bg-[radial-gradient(ellipse_at_center,_var(--tw-gradient-stops))] from-primary-900/20 via-transparent to-transparent" />
      
      <div className="container mx-auto relative z-10">
        <div className="text-center max-w-4xl mx-auto mb-16">
          <div className="inline-flex items-center bg-secondary-800/50 border border-secondary-700 rounded-full px-4 py-2 mb-8">
            <Star className="w-4 h-4 text-accent-400 mr-2" />
            <span className="text-sm text-secondary-300">Trusted by developers for high-level code analysis</span>
          </div>

          <Display size="lg" className="mb-8 animate-slide-up">
            <span className="bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent">
              Your AI Co-pilot Writes the Code.
            </span>
            <br />
            <span className="bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent">
              Uveddi Ensures the Architecture is Sound.
            </span>
          </Display>

          <Body size="lg" color="secondary" className="mb-6 leading-relaxed max-w-3xl mx-auto">
            AI generates code at lightning speed, but it can't see the bigger picture. Uveddi is the essential verification layer that analyzes your codebase's architecture, ensuring AI-generated code is maintainable, scalable, and won't create long-term technical debt.
            <span className="text-primary-400 font-emphasis block mt-2">Accelerate onboarding by 30%. Ship features 2x faster. Cut technical debt at the root.</span>
          </Body>

          {/* CLI Install Command Section */}
          <div className="mb-6">
            <div className="bg-secondary-900 border border-secondary-700 rounded-lg px-6 py-4 inline-flex items-center mx-auto text-left shadow-lg">
              <span className="font-mono text-primary-400 text-base select-all">curl -sSL https://uveddi.dev/install.sh | bash</span>
              <button 
                className="ml-4 px-3 py-1 bg-primary-500 hover:bg-primary-600 text-white text-xs rounded transition-all" 
                onClick={handleCopyCommand}
              >
                Copy
              </button>
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
  );
};

export default HeroSection;