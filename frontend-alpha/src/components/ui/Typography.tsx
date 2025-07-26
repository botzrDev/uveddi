/**
 * Typography Components for Uveddi Frontend
 * Pre-built components implementing the typography strategy
 */

import React from 'react';
import { cn } from '../../lib/utils';

// Base typography component props
interface BaseTypographyProps {
  className?: string;
  children: React.ReactNode;
}

// Display components for hero sections
interface DisplayProps extends BaseTypographyProps {
  size?: 'lg' | 'md' | 'sm';
  gradient?: boolean;
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
}

export const Display: React.FC<DisplayProps> = ({ 
  size = 'md', 
  gradient = false,
  as: Component = 'h1',
  className = '', 
  children 
}) => {
  const baseClasses = 'font-display font-display tracking-tight leading-tight';
  const sizeClasses = {
    lg: 'text-display-sm md:text-display-md lg:text-display-lg',
    md: 'text-display-sm md:text-display-md',
    sm: 'text-display-sm',
  };
  
  const gradientClasses = gradient 
    ? 'bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent'
    : 'text-white';

  return (
    <Component className={cn(baseClasses, sizeClasses[size], gradientClasses, className)}>
      {children}
    </Component>
  );
};

// Headline components for section titles
interface HeadlineProps extends BaseTypographyProps {
  size?: 'lg' | 'md' | 'sm';
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
  color?: 'primary' | 'secondary' | 'white' | 'gradient';
}

export const Headline: React.FC<HeadlineProps> = ({ 
  size = 'md',
  as: Component = 'h2',
  color = 'white',
  className = '', 
  children 
}) => {
  const baseClasses = 'font-display font-headline tracking-tight leading-tight';
  const sizeClasses = {
    lg: 'text-headline-lg',
    md: 'text-headline-md',
    sm: 'text-headline-sm',
  };
  
  const colorClasses = {
    primary: 'text-primary-400',
    secondary: 'text-secondary-200',
    white: 'text-white',
    gradient: 'bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent',
  };

  return (
    <Component className={cn(baseClasses, sizeClasses[size], colorClasses[color], className)}>
      {children}
    </Component>
  );
};

// Body text components
interface BodyProps extends BaseTypographyProps {
  size?: 'lg' | 'md' | 'sm';
  as?: 'p' | 'span' | 'div';
  color?: 'primary' | 'secondary' | 'tertiary' | 'muted';
  leading?: 'tight' | 'normal' | 'relaxed' | 'loose';
}

export const Body: React.FC<BodyProps> = ({ 
  size = 'md',
  as: Component = 'p',
  color = 'secondary',
  leading = 'normal',
  className = '', 
  children 
}) => {
  const baseClasses = 'font-sans font-body';
  const sizeClasses = {
    lg: 'text-body-lg',
    md: 'text-body-md',
    sm: 'text-body-sm',
  };
  
  const colorClasses = {
    primary: 'text-white',
    secondary: 'text-secondary-200',
    tertiary: 'text-secondary-300',
    muted: 'text-secondary-400',
  };
  
  const leadingClasses = {
    tight: 'leading-tight',
    normal: 'leading-normal',
    relaxed: 'leading-relaxed',
    loose: 'leading-loose',
  };

  return (
    <Component className={cn(baseClasses, sizeClasses[size], colorClasses[color], leadingClasses[leading], className)}>
      {children}
    </Component>
  );
};

// Caption component for small text
export const Caption: React.FC<BaseTypographyProps & { 
  color?: 'primary' | 'secondary' | 'tertiary' | 'muted';
}> = ({ 
  color = 'muted',
  className = '', 
  children 
}) => {
  const colorClasses = {
    primary: 'text-white',
    secondary: 'text-secondary-200',
    tertiary: 'text-secondary-300',
    muted: 'text-secondary-400',
  };

  return (
    <span className={cn('font-sans text-caption font-body leading-tight', colorClasses[color], className)}>
      {children}
    </span>
  );
};

// Code component
export const Code: React.FC<BaseTypographyProps> = ({ 
  className = '', 
  children 
}) => (
  <code className={cn('font-mono text-body-sm font-body leading-relaxed bg-secondary-800 px-2 py-1 rounded', className)}>
    {children}
  </code>
);

// Link component
interface LinkProps extends BaseTypographyProps {
  href?: string;
  external?: boolean;
  color?: 'primary' | 'secondary';
  underline?: boolean;
}

export const Link: React.FC<LinkProps> = ({ 
  href,
  external = false,
  color = 'primary',
  underline = true,
  className = '', 
  children 
}) => {
  const colorClasses = {
    primary: 'text-primary-400 hover:text-primary-300',
    secondary: 'text-secondary-300 hover:text-secondary-200',
  };
  
  const underlineClasses = underline ? 'underline-offset-4 hover:underline' : '';

  return (
    <a 
      href={href}
      target={external ? '_blank' : undefined}
      rel={external ? 'noopener noreferrer' : undefined}
      className={cn(
        'font-sans font-emphasis transition-colors',
        colorClasses[color],
        underlineClasses,
        className
      )}
    >
      {children}
    </a>
  );
};

// Button text component
interface ButtonTextProps extends BaseTypographyProps {
  size?: 'lg' | 'md' | 'sm';
}

export const ButtonText: React.FC<ButtonTextProps> = ({ 
  size = 'md',
  className = '', 
  children 
}) => {
  const sizeClasses = {
    lg: 'text-body-lg',
    md: 'text-body-md',
    sm: 'text-body-sm',
  };

  return (
    <span className={cn('font-sans font-emphasis tracking-wide', sizeClasses[size], className)}>
      {children}
    </span>
  );
};

// Overline component for labels
export const Overline: React.FC<BaseTypographyProps> = ({ 
  className = '', 
  children 
}) => (
  <span className={cn('font-sans text-caption font-emphasis tracking-widest uppercase leading-tight', className)}>
    {children}
  </span>
);

// Gradient text utility
export const GradientText: React.FC<BaseTypographyProps & {
  gradient?: 'primary' | 'secondary' | 'success' | 'brand';
}> = ({ 
  gradient = 'primary',
  className = '', 
  children 
}) => {
  const gradientClasses = {
    primary: 'bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent',
    secondary: 'bg-gradient-to-r from-secondary-200 to-secondary-300 bg-clip-text text-transparent',
    success: 'bg-gradient-to-r from-success-400 to-success-500 bg-clip-text text-transparent',
    brand: 'bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent',
  };

  return (
    <span className={cn(gradientClasses[gradient], className)}>
      {children}
    </span>
  );
};

// Export all components
export default {
  Display,
  Headline,
  Body,
  Caption,
  Code,
  Link,
  ButtonText,
  Overline,
  GradientText,
};
