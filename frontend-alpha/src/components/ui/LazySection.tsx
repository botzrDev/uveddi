import React from 'react';
import { useIntersectionObserver } from '../../hooks/useIntersectionObserver';

interface LazySectionProps {
  children: React.ReactNode;
  className?: string;
  fallback?: React.ReactNode;
  threshold?: number;
  rootMargin?: string;
}

const LazySection: React.FC<LazySectionProps> = ({
  children,
  className = '',
  fallback = null,
  threshold = 0.1,
  rootMargin = '100px',
}) => {
  const { ref, hasIntersected } = useIntersectionObserver({
    threshold,
    rootMargin,
    triggerOnce: true,
  });

  return (
    <section ref={ref} className={className}>
      {hasIntersected ? children : fallback}
    </section>
  );
};

export default LazySection;