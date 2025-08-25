/**
 * Accessibility testing utilities using jest-axe
 */

import { configureAxe } from 'jest-axe';

// Configure axe for consistent testing
export const axe = configureAxe({
  rules: {
    // Ensure color contrast meets WCAG AA standards
    'color-contrast': { enabled: true },
    // Ensure proper heading hierarchy
    'heading-order': { enabled: true },
    // Ensure images have alt text
    'image-alt': { enabled: true },
    // Ensure form inputs are labeled
    'label': { enabled: true },
    // Ensure interactive elements are keyboard accessible
    'keyboard': { enabled: true },
    // Ensure focus indicators are visible
    'focus-order-semantics': { enabled: true },
    // Ensure semantic HTML is used correctly
    'landmark-one-main': { enabled: true },
    'landmark-complementary-is-top-level': { enabled: true },
    'landmark-main-is-top-level': { enabled: true },
    // Ensure ARIA attributes are valid
    'aria-valid-attr-value': { enabled: true },
    'aria-valid-attr': { enabled: true },
    'aria-required-attr': { enabled: true },
    'aria-required-children': { enabled: true },
    'aria-required-parent': { enabled: true },
  },
  tags: ['wcag2a', 'wcag2aa', 'wcag21aa'],
  // Include best practices but not experimental rules
  disableOtherRules: false,
});

/**
 * Test helper for accessibility violations
 */
export const expectNoAccessibilityViolations = async (container: Element) => {
  const results = await axe(container);
  expect(results).toHaveNoViolations();
};

/**
 * Get accessibility violations for debugging
 */
export const getAccessibilityViolations = async (container: Element) => {
  const results = await axe(container);
  return results.violations;
};

/**
 * Custom matcher type definitions for TypeScript
 */
declare global {
  namespace jest {
    interface Matchers<R> {
      toHaveNoViolations(): R;
    }
  }
}