import { Code2, GitBranch, Search, Settings, Shield, Zap } from 'lucide-react';
import React from 'react';
import FeatureCard from '../ui/FeatureCard';
import { Headline, Body } from '../ui/Typography';

const FeaturesSection: React.FC = () => {
  return (
    <section id="features" className="py-20 px-6 bg-secondary-800/30">
      <div className="container mx-auto">
        <div className="text-center mb-16">
          <Headline size="lg" className="mb-4">
            Community-Driven Architectural Analysis
          </Headline>
          <Body size="lg" color="tertiary" className="max-w-3xl mx-auto">
            Open source CLI powered by local AI analysis. No cloud dependencies, no data sharing, just pure architectural intelligence.
          </Body>
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
            title="Local AI Analysis"
            description="Powered by Ollama and local models. Your code never leaves your machine. Get AI-powered insights with complete privacy and control."
            gradient="from-blue-500 to-blue-600"
          />
          <FeatureCard
            icon={GitBranch}
            title="Dependency Analysis"
            description="Detect cyclic dependencies and tight coupling. Get clear markdown reports that help you understand and fix architectural issues."
            gradient="from-purple-500 to-purple-600"
          />
          <FeatureCard
            icon={Zap}
            title="Lightning-Fast CLI"
            description="Analyze massive codebases in seconds, not minutes. Uveddi's Rust-powered CLI fits seamlessly into your workflow and CI/CD."
            gradient="from-accent-500 to-accent-600"
          />
          <FeatureCard
            icon={Code2}
            title="Multi-Language Support"
            description="Analyze Rust, Python, and JavaScript codebases with the same tool. More languages coming through community contributions."
            gradient="from-primary-500 to-primary-600"
          />
          <FeatureCard
            icon={Settings}
            title="Simple CLI Integration"
            description="Drop into any workflow with a single command. Perfect for local development and basic CI/CD automation."
            gradient="from-indigo-500 to-indigo-600"
          />
        </div>
      </div>
    </section>
  );
};

export default FeaturesSection;