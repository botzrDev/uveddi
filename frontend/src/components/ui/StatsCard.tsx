import React from 'react';

interface StatsCardProps {
  title: string;
  value: string | number;
  description?: string;
  trend?: {
    value: number;
    label: string;
    positive: boolean;
  };
  icon?: React.ReactNode;
  className?: string;
}

const StatsCard: React.FC<StatsCardProps> = ({
  title,
  value,
  description,
  trend,
  icon,
  className = '',
}) => {
  return (
    <div className={`bg-secondary-800 border border-secondary-700 rounded-xl p-6 hover:border-secondary-600 transition-colors ${className}`}>
      <div className="flex items-start justify-between">
        <div className="flex-1">
          <p className="text-secondary-400 text-sm font-medium mb-1">{title}</p>
          <p className="text-3xl font-bold text-white mb-1">{value}</p>
          {description && (
            <p className="text-secondary-300 text-sm">{description}</p>
          )}
          {trend && (
            <div className="flex items-center mt-2">
              <span className={`text-sm font-medium ${trend.positive ? 'text-primary-400' : 'text-red-400'}`}>
                {trend.positive ? '+' : '-'}{Math.abs(trend.value)}%
              </span>
              <span className="text-secondary-400 text-sm ml-1">{trend.label}</span>
            </div>
          )}
        </div>
        {icon && (
          <div className="text-primary-400 opacity-80">
            {icon}
          </div>
        )}
      </div>
    </div>
  );
};

export default StatsCard;
