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
            Actionable Intelligence, Not Just Data
          </Headline>
          <Body size="lg" color="tertiary" className="max-w-3xl mx-auto">
            Advanced static analysis meets AI-powered insights to give you the complete picture of your codebase health.
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
            description="Analyze massive codebases in seconds, not minutes. Uveddi's Rust-powered CLI fits seamlessly into your workflow and CI/CD."
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
  );
};

export default FeaturesSection;