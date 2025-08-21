declare module 'jest-axe' {
  export function axe(container: Element): Promise<any>;
  export const toHaveNoViolations: jest.CustomMatcher;
}