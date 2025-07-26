# Typography Implementation Guide

## Overview
This guide provides step-by-step instructions for implementing the new Uveddi typography system across your React frontend.

## Quick Start

### 1. Import Typography Components
```tsx
import { Display, Headline, Body, Caption, Code, Link, GradientText } from '../components/ui/Typography';
```

### 2. Basic Usage Examples

#### Hero Section
```tsx
// Before
<h1 className="text-6xl font-bold text-white">
  Your Headline
</h1>

// After
<Display size="lg" gradient>
  Your Headline
</Display>
```

#### Section Headings
```tsx
// Before
<h2 className="text-3xl font-semibold text-white mb-4">
  Section Title
</h2>

// After
<Headline size="lg" color="white" className="mb-4">
  Section Title
</Headline>
```

#### Body Text
```tsx
// Before
<p className="text-lg text-gray-300 leading-relaxed">
  Your paragraph text here.
</p>

// After
<Body size="lg" color="tertiary" leading="relaxed">
  Your paragraph text here.
</Body>
```

#### Code Snippets
```tsx
// Before
<code className="bg-gray-800 px-2 py-1 rounded text-sm">
  npm install uveddi
</code>

// After
<Code>npm install uveddi</Code>
```

### 3. Advanced Usage

#### Responsive Typography
```tsx
// Automatically responsive display text
<Display size="lg" className="text-center">
  Responsive Hero
</Display>

// Custom responsive behavior
<Headline 
  size="md" 
  className="text-headline-sm md:text-headline-md lg:text-headline-lg"
>
  Custom Responsive
</Headline>
```

#### Gradient Text
```tsx
<GradientText gradient="primary">
  Gradient Brand Text
</GradientText>

// Or inline with components
<Headline size="lg" color="gradient">
  Gradient Headline
</Headline>
```

#### Mixed Typography
```tsx
<Display size="lg" className="mb-6">
  <GradientText gradient="brand">Code Analysis</GradientText>
  <br />
  <GradientText gradient="primary">Instantly Intelligent</GradientText>
</Display>
```

## Component Reference

### Display
For hero sections and major page titles.
- **Sizes**: `lg`, `md`, `sm`
- **Props**: `size`, `gradient`, `as`, `className`
- **Usage**: Page heroes, landing page titles

### Headline
For section headings and subheadings.
- **Sizes**: `lg`, `md`, `sm`
- **Colors**: `primary`, `secondary`, `white`, `gradient`
- **Props**: `size`, `as`, `color`, `className`
- **Usage**: Section titles, card headings, form labels

### Body
For paragraph text and general content.
- **Sizes**: `lg`, `md`, `sm`
- **Colors**: `primary`, `secondary`, `tertiary`, `muted`
- **Leading**: `tight`, `normal`, `relaxed`, `loose`
- **Props**: `size`, `as`, `color`, `leading`, `className`
- **Usage**: Paragraphs, descriptions, content text

### Caption
For small text and metadata.
- **Colors**: `primary`, `secondary`, `tertiary`, `muted`
- **Usage**: Image captions, timestamps, metadata

### Code
For inline code and technical text.
- **Usage**: Code snippets, file names, commands

### Link
For hyperlinks and interactive text.
- **Colors**: `primary`, `secondary`
- **Props**: `href`, `external`, `color`, `underline`
- **Usage**: Navigation links, external links, CTAs

## Migration Strategy

### Phase 1: Core Pages (Week 1)
1. **LandingPage.tsx** ✅ (Already started)
2. **Header.tsx** ✅ (Already started)
3. **FeatureCard.tsx** ✅ (Already started)

### Phase 2: Remaining Components (Week 2)
1. **Button.tsx** - Update button text styling
2. **Card.tsx** - Update card headings and content
3. **StatsCard.tsx** - Update numbers and labels
4. **AnimatedTerminal.tsx** - Update terminal text

### Phase 3: Additional Pages (Week 3)
1. **DashboardPage.tsx** - Update dashboard headings
2. **LoginPage.tsx** - Update form labels and titles
3. **RegisterPage.tsx** - Update form styling
4. **TestPage.tsx** - Update test content

## Performance Checklist

### ✅ Completed
- [x] Font preloading in index.html
- [x] Typography CSS variables
- [x] Responsive font sizes
- [x] Accessibility focus states
- [x] Reduced motion support

### 🔄 In Progress
- [ ] Font subsetting implementation
- [ ] Self-hosting migration
- [ ] Variable font evaluation

### 📋 Pending
- [ ] Font loading optimization
- [ ] WOFF2 conversion
- [ ] CDN configuration
- [ ] Performance monitoring

## Testing

### Visual Testing
1. Test all typography scales on different screen sizes
2. Verify color contrast ratios meet WCAG AA standards
3. Check typography in both light and dark modes
4. Test with different zoom levels (100%, 125%, 150%)

### Performance Testing
1. Measure font loading times
2. Check Core Web Vitals impact
3. Test with slow network conditions
4. Verify font-display behavior

### Accessibility Testing
1. Test with screen readers
2. Check keyboard navigation
3. Verify color contrast compliance
4. Test with high contrast mode

## Troubleshooting

### Common Issues

#### Font Not Loading
- Check font import in index.css
- Verify font-display: swap is set
- Check network tab for font requests

#### Inconsistent Sizing
- Ensure using typography components consistently
- Check for conflicting Tailwind classes
- Verify responsive breakpoints

#### Performance Issues
- Implement font preloading
- Consider font subsetting
- Check for unused font weights

## Support

For questions or issues with the typography system:
1. Check this guide first
2. Review component documentation
3. Test with provided examples
4. Report issues with specific use cases
