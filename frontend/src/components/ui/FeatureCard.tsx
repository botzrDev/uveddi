import type { LucideIcon } from 'lucide-react';
import React from 'react';

interface FeatureCardProps {
  icon: LucideIcon;
  title: string;
  description: string;
  gradient?: string;
  className?: string;
}

const FeatureCard: React.FC<FeatureCardProps> = ({
  icon: Icon,
  title,
  description,
  gradient = 'from-primary-500 to-primary-600',
  className = '',
}) => {
  return (
    <div className={`group relative bg-secondary-800 hover:bg-secondary-750 border border-secondary-700 hover:border-secondary-600 rounded-xl p-6 transition-all duration-300 hover:scale-105 hover:shadow-glow ${className}`}>
      <div className={`inline-flex items-center justify-center w-12 h-12 bg-gradient-to-r ${gradient} rounded-lg mb-4 shadow-lg`}>
        <Icon className="w-6 h-6 text-white" />
      </div>
      <h3 className="text-xl font-semibold text-white mb-3 group-hover:text-primary-400 transition-colors">
        {title}
      </h3>
      <p className="text-secondary-300 leading-relaxed">
        {description}
      </p>
      <div className="absolute inset-0 bg-gradient-to-r from-primary-500/5 to-primary-600/5 rounded-xl opacity-0 group-hover:opacity-100 transition-opacity duration-300" />
    </div>
  );
};

export default FeatureCard;
