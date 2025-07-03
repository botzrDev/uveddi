/**
 * Typography utilities for Uveddi Frontend
 * Implements the typography strategy with semantic class names
 */

import React from 'react';
import { cn } from './utils';

// Typography variant definitions
export const typography = {
  // Display styles for hero sections and major headings
  display: {
    lg: 'font-display text-display-lg font-display tracking-tight',
    md: 'font-display text-display-md font-display tracking-tight',
    sm: 'font-display text-display-sm font-display tracking-tight',
  },
  
  // Headline styles for section headings
  headline: {
    lg: 'font-display text-headline-lg font-headline tracking-tight',
    md: 'font-display text-headline-md font-headline tracking-tight',
    sm: 'font-display text-headline-sm font-headline tracking-tight',
  },
  
  // Body text styles
  body: {
    lg: 'font-sans text-body-lg font-body leading-relaxed',
    md: 'font-sans text-body-md font-body leading-normal',
    sm: 'font-sans text-body-sm font-body leading-normal',
  },
  
  // Utility text styles
  caption: 'font-sans text-caption font-body leading-tight',
  overline: 'font-sans text-overline font-emphasis tracking-widest uppercase',
  
  // Code and technical text
  code: 'font-mono text-body-sm font-body leading-relaxed',
  
  // Interactive elements
  button: {
    lg: 'font-sans text-body-lg font-emphasis tracking-wide',
    md: 'font-sans text-body-md font-emphasis tracking-wide',
    sm: 'font-sans text-body-sm font-emphasis tracking-wide',
  },
  
  // Links
  link: 'font-sans font-emphasis underline-offset-4 hover:underline transition-colors',
};

// Responsive typography helpers
export const responsiveTypography = {
  // Mobile-first display styles
  'display-responsive': 'text-display-sm md:text-display-md lg:text-display-lg',
  'headline-responsive': 'text-headline-sm md:text-headline-md lg:text-headline-lg',
  'body-responsive': 'text-body-md lg:text-body-lg',
};

// Typography component props
export interface TypographyProps {
  variant?: keyof typeof typography.display | keyof typeof typography.headline | keyof typeof typography.body;
  className?: string;
  children: React.ReactNode;
}

// Typography components
export const Display: React.FC<TypographyProps & { size?: 'lg' | 'md' | 'sm' }> = ({ 
  size = 'md', 
  className = '', 
  children 
}) => (
  <h1 className={cn(typography.display[size], className)}>
    {children}
  </h1>
);

export const Headline: React.FC<TypographyProps & { 
  size?: 'lg' | 'md' | 'sm';
  as?: 'h1' | 'h2' | 'h3' | 'h4' | 'h5' | 'h6';
}> = ({ 
  size = 'md',
  as: Component = 'h2',
  className = '', 
  children 
}) => (
  <Component className={cn(typography.headline[size], className)}>
    {children}
  </Component>
);

export const Body: React.FC<TypographyProps & { 
  size?: 'lg' | 'md' | 'sm';
  as?: 'p' | 'span' | 'div';
}> = ({ 
  size = 'md',
  as: Component = 'p',
  className = '', 
  children 
}) => (
  <Component className={cn(typography.body[size], className)}>
    {children}
  </Component>
);

export const Caption: React.FC<Omit<TypographyProps, 'variant'>> = ({ 
  className = '', 
  children 
}) => (
  <span className={cn(typography.caption, className)}>
    {children}
  </span>
);

export const Code: React.FC<Omit<TypographyProps, 'variant'>> = ({ 
  className = '', 
  children 
}) => (
  <code className={cn(typography.code, className)}>
    {children}
  </code>
);

// Text color utilities optimized for accessibility
export const textColors = {
  // Primary text colors
  primary: 'text-white',
  secondary: 'text-secondary-200',
  tertiary: 'text-secondary-300',
  muted: 'text-secondary-400',
  
  // Semantic colors
  success: 'text-success-400',
  warning: 'text-accent-400',
  error: 'text-red-400',
  info: 'text-primary-400',
  
  // Interactive states
  link: 'text-primary-400 hover:text-primary-300',
  'link-secondary': 'text-secondary-300 hover:text-secondary-200',
};

// Gradient text utilities
export const gradientText = {
  primary: 'bg-gradient-to-r from-primary-400 to-primary-500 bg-clip-text text-transparent',
  secondary: 'bg-gradient-to-r from-secondary-200 to-secondary-300 bg-clip-text text-transparent',
  success: 'bg-gradient-to-r from-success-400 to-success-500 bg-clip-text text-transparent',
  brand: 'bg-gradient-to-r from-white to-secondary-200 bg-clip-text text-transparent',
};

// Accessibility-focused contrast ratios
export const accessibilityHelpers = {
  // Ensures WCAG AA compliance
  'high-contrast': 'contrast-125',
  'focus-visible': 'focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-primary-400',
  'reduced-motion': 'motion-reduce:transition-none motion-reduce:animation-none',
};

// Export all utilities
export const typo = {
  ...typography,
  ...responsiveTypography,
  ...textColors,
  ...gradientText,
  ...accessibilityHelpers,
};
