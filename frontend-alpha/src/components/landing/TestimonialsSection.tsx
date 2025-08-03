import { Shield, Star, TrendingUp, Zap } from 'lucide-react';
import React from 'react';
import { Headline, Body } from '../ui/Typography';

const TestimonialsSection: React.FC = () => {
  return (
    <section className="py-20 px-6">
      <div className="container mx-auto">
        <div className="text-center mb-16">
          <Headline size="lg" className="mb-4">
            Trusted by Developers Worldwide
          </Headline>
          <Body size="lg" color="secondary">
            See how uveddi is transforming code quality for teams everywhere
          </Body>
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
  );
};

export default TestimonialsSection;