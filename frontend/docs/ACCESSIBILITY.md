# Accessibility Guide

This document outlines the accessibility features and testing procedures for the Uveddi Interactive Reports frontend application.

## Overview

The Uveddi frontend is designed to meet **WCAG 2.1 AA** standards and provides a fully accessible experience for users with disabilities. Our accessibility implementation includes support for:

- Screen readers
- Keyboard-only navigation
- High contrast displays
- Reduced motion preferences
- Voice control software
- Various assistive technologies

## Accessibility Features

### 1. Semantic HTML Structure

- **Proper heading hierarchy**: H1 for main page titles, H2 for section headings, etc.
- **Landmark elements**: `<header>`, `<main>`, `<nav>`, and `<footer>` for easy navigation
- **Semantic elements**: Buttons, forms, and interactive elements use appropriate HTML elements

### 2. Keyboard Navigation

All interactive elements are accessible via keyboard:

- **Tab order**: Logical focus flow through the interface
- **Skip links**: Jump to main content functionality
- **Focus indicators**: Visible focus outlines that meet 3:1 contrast ratio
- **Keyboard shortcuts**: Standard shortcuts for common actions
- **Modal focus management**: Focus trapping in dialogs and drawers

#### Keyboard Shortcuts

| Shortcut | Action |
|----------|--------|
| `Tab` | Navigate forward through interactive elements |
| `Shift + Tab` | Navigate backward through interactive elements |
| `Enter` or `Space` | Activate buttons and links |
| `Escape` | Close modals and dropdowns |
| `Arrow keys` | Navigate within menus and tabs |

### 3. Screen Reader Support

- **ARIA labels**: Descriptive labels for all interactive elements
- **ARIA roles**: Proper semantic roles for custom components
- **Live regions**: Dynamic content announcements
- **Alternative text**: Meaningful alt text for images and icons
- **Form labels**: Explicit labels for all form controls

### 4. Visual Design

#### Color and Contrast

- **Text contrast**: Minimum 4.5:1 ratio for normal text, 3:1 for large text
- **Non-text contrast**: 3:1 ratio for UI components and graphical elements
- **Color independence**: Information never conveyed by color alone
- **Theme support**: Both light and dark themes meet contrast requirements

#### Typography

- **Font sizes**: Minimum 16px base font size with relative sizing
- **Line spacing**: 1.5x line height for body text
- **Letter spacing**: Optimized for readability
- **Font weights**: Strategic use of font weights for hierarchy

### 5. Motion and Animation

- **Reduced motion**: Respects `prefers-reduced-motion` system setting
- **Animation controls**: Essential animations can be paused
- **Timeout controls**: No auto-advancing content without user control

### 6. Mobile Accessibility

- **Touch targets**: Minimum 44px × 44px touch targets
- **Zoom support**: Content readable at 200% zoom
- **Orientation support**: Works in both portrait and landscape
- **Voice over**: Compatible with mobile screen readers

## Testing Procedures

### Automated Testing

We use **jest-axe** for automated accessibility testing:

```bash
# Run all accessibility tests
npm run test:accessibility

# Run specific accessibility test suite
npm test -- --testPathPattern=accessibility

# Run accessibility tests with coverage
npm run test:coverage -- --testPathPattern=accessibility
```

#### Test Coverage

Our automated tests cover:

- ARIA attribute validation
- Color contrast ratios
- Keyboard navigation paths
- Form labeling compliance
- Semantic HTML structure
- Focus management

### Manual Testing

#### Screen Reader Testing

Tested with:
- **NVDA** (Windows)
- **JAWS** (Windows)
- **VoiceOver** (macOS/iOS)
- **TalkBack** (Android)

#### Keyboard Testing Checklist

- [ ] Tab through all interactive elements
- [ ] Verify focus is visible on all elements
- [ ] Test skip links functionality
- [ ] Ensure modal focus trapping works
- [ ] Verify escape key closes modals/menus

#### Visual Testing

- [ ] Test at 200% browser zoom
- [ ] Verify high contrast mode
- [ ] Test with custom system colors
- [ ] Check focus indicators visibility
- [ ] Verify text remains readable in all themes

### Browser Testing

Accessibility features tested in:

- **Chrome** 100+ (with various extensions)
- **Firefox** 100+
- **Safari** 15+
- **Edge** 100+

## Implementation Guidelines

### For Developers

#### Component Development

```typescript
// ✅ Good: Proper ARIA labeling
<Button 
  aria-label=\"Save dashboard layout\"
  onClick={handleSave}
>
  <SaveIcon aria-hidden=\"true\" />
  Save
</Button>

// ❌ Bad: Missing accessible name
<Button onClick={handleSave}>
  <SaveIcon />
</Button>
```

#### Form Controls

```typescript
// ✅ Good: Explicit labeling
<TextField
  id=\"widget-title\"
  label=\"Widget Title\"
  value={title}
  onChange={handleChange}
  aria-describedby=\"title-help\"
  required
/>
<Typography id=\"title-help\" variant=\"caption\">
  Enter a descriptive title for this widget
</Typography>

// ❌ Bad: Placeholder as label
<TextField
  placeholder=\"Enter widget title\"
  value={title}
  onChange={handleChange}
/>
```

#### Custom Components

```typescript
// ✅ Good: Proper role and keyboard support
<Paper
  component=\"button\"
  role=\"button\"
  tabIndex={0}
  onClick={handleClick}
  onKeyDown={(e) => {
    if (e.key === 'Enter' || e.key === ' ') {
      e.preventDefault();
      handleClick();
    }
  }}
  aria-label={`Add ${widgetType.name} widget`}
>
  {/* Widget content */}
</Paper>
```

### CSS Guidelines

#### Focus Indicators

```css
/* Enhanced focus styles for accessibility */
.MuiButton-root:focus-visible {
  outline: 3px solid var(--uveddi-primary-600) !important;
  outline-offset: 2px !important;
  box-shadow: 0 0 0 1px rgba(255, 255, 255, 0.8), 
              0 0 0 4px var(--uveddi-primary-600);
}
```

#### Reduced Motion

```css
@media (prefers-reduced-motion: reduce) {
  *,
  *::before,
  *::after {
    animation-duration: 0.01ms !important;
    animation-iteration-count: 1 !important;
    transition-duration: 0.01ms !important;
    scroll-behavior: auto !important;
  }
}
```

#### High Contrast Support

```css
@media (prefers-contrast: high) {
  :root {
    --uveddi-text-primary: #000000;
    --uveddi-border: #000000;
  }
  
  button, .MuiButton-root {
    border: 2px solid currentColor !important;
  }
}
```

## Common Issues and Solutions

### Issue: Focus Not Visible

**Solution**: Ensure focus indicators have sufficient contrast and are not removed by CSS.

```css
/* ❌ Don't do this */
button:focus {
  outline: none;
}

/* ✅ Do this instead */
button:focus-visible {
  outline: 2px solid #0066cc;
  outline-offset: 2px;
}
```

### Issue: Inaccessible Custom Components

**Solution**: Add proper ARIA attributes and keyboard event handlers.

```typescript
// Before: Inaccessible div button
<div onClick={handleClick}>Click me</div>

// After: Accessible button
<div
  role=\"button\"
  tabIndex={0}
  onClick={handleClick}
  onKeyDown={(e) => e.key === 'Enter' && handleClick()}
  aria-label=\"Descriptive action\"
>
  Click me
</div>
```

### Issue: Missing Form Labels

**Solution**: Always provide explicit labels for form controls.

```typescript
// ✅ Explicit label association
<label htmlFor=\"email\">Email Address</label>
<input id=\"email\" type=\"email\" required />

// ✅ Or use aria-label
<input 
  type=\"email\" 
  aria-label=\"Email Address\"
  required 
/>
```

## Testing Tools

### Browser Extensions

- **axe DevTools**: Comprehensive accessibility testing
- **WAVE**: Web accessibility evaluation
- **Lighthouse**: Includes accessibility audit
- **Color Oracle**: Color blindness simulator

### Command Line Tools

```bash
# Install accessibility testing dependencies
npm install --save-dev @axe-core/cli jest-axe

# Run axe tests on built application
npx axe-cli http://localhost:3000

# Generate accessibility report
npx axe-cli --reporter html --output-dir reports/accessibility http://localhost:3000
```

## Continuous Integration

Our CI pipeline includes:

1. **Automated accessibility tests** on every pull request
2. **Lighthouse accessibility audits** on staging deployments
3. **axe-core scanning** of critical user journeys
4. **Color contrast validation** for all themes

### GitHub Actions Integration

```yaml
name: Accessibility Tests
on: [push, pull_request]

jobs:
  a11y-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v3
      - name: Setup Node.js
        uses: actions/setup-node@v3
        with:
          node-version: '18'
      - name: Install dependencies
        run: npm ci
      - name: Run accessibility tests
        run: npm run test:accessibility
      - name: Run axe-cli
        run: |
          npm run build
          npm run preview &
          npx wait-on http://localhost:4173
          npx axe-cli http://localhost:4173 --reporter junit --output-file a11y-results.xml
```

## Resources

### Standards and Guidelines

- [WCAG 2.1 Guidelines](https://www.w3.org/WAI/WCAG21/quickref/)
- [WAI-ARIA Authoring Practices](https://www.w3.org/WAI/ARIA/apg/)
- [Material-UI Accessibility Guide](https://mui.com/material-ui/guides/accessibility/)
- [React Accessibility Documentation](https://react.dev/learn/accessibility)

### Testing Resources

- [WebAIM Screen Reader Testing](https://webaim.org/articles/screenreader_testing/)
- [Keyboard Testing Guide](https://webaim.org/articles/keyboard/)
- [Color Contrast Analyzers](https://webaim.org/resources/contrastchecker/)

### Learning Materials

- [WebAIM Accessibility Training](https://webaim.org/training/)
- [A11y Project Checklist](https://www.a11yproject.com/checklist/)
- [Inclusive Design Principles](https://inclusivedesignprinciples.org/)

## Support

For accessibility questions or to report accessibility issues:

1. **Create an issue** in the project repository
2. **Include details** about the barrier or enhancement
3. **Specify assistive technology** used (if applicable)
4. **Provide reproduction steps** when possible

We are committed to maintaining and improving the accessibility of Uveddi Interactive Reports. Feedback and contributions are always welcome!