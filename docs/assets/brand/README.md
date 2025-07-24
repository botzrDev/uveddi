# 🎨 Uveddi Brand Assets

This directory contains all official brand assets for the Uveddi project, organized for easy access and consistent usage across all platforms and materials.

## 📁 Directory Structure

```
brand/
├── colors/          # Color palettes and CSS variables
├── logos/           # Logo files in various formats
├── favicons/        # Website icons and manifests
├── social/          # Social media card templates
├── mascot.md        # Archie the Code Owl guidelines
└── README.md        # This file
```

## 🎨 Brand Identity

### Core Values
- **Privacy-First**: All design choices reinforce our commitment to data privacy
- **Developer-Friendly**: Clean, technical aesthetic that appeals to developers
- **Professional**: Trustworthy and reliable visual identity
- **Accessible**: High contrast and readable across all platforms

### Visual Principles
- **Minimalist**: Clean lines and uncluttered layouts
- **Technical**: Monospace fonts and code-inspired elements
- **Trustworthy**: Professional color palette with navy blue primary
- **Friendly**: Warm accents and approachable mascot

## 🎯 Logo Usage

### Primary Logo (`logos/logo-primary.svg`)
- Use on light backgrounds
- Minimum size: 32px height
- Clear space: Equal to logo height on all sides
- Never stretch, skew, or modify proportions

### White Logo (`logos/logo-white.svg`)
- Use on dark backgrounds or brand colors
- Same sizing and spacing rules as primary
- Ideal for social media cards and dark themes

### Logo Don'ts
❌ Don't change colors  
❌ Don't add effects or shadows  
❌ Don't use on busy backgrounds  
❌ Don't place too close to other elements  

## 🌈 Color Palette

Our color system is defined in `colors/palette.css` with semantic naming:

### Primary Colors
- **Primary 600**: `#4f46e5` - Main brand color
- **Primary 800**: `#3730a3` - Dark variant for gradients
- **Primary 100**: `#e0e7ff` - Light backgrounds

### Secondary Colors
- **Secondary 900**: `#0f172a` - Primary text
- **Secondary 600**: `#475569` - Secondary text
- **Secondary 100**: `#f1f5f9` - Light backgrounds

### Accent Colors
- **Action 500**: `#f97316` - Call-to-action buttons
- **Success 600**: `#16a34a` - Success states
- **Warning 500**: `#d97706` - Warning states

## 🖼️ Favicon System

Complete favicon package in `favicons/` includes:
- `favicon.ico` - Traditional favicon
- `favicon.svg` - Modern SVG favicon
- `apple-touch-icon.png` - iOS home screen
- `web-app-manifest-*.png` - PWA icons
- `site.webmanifest` - Web app manifest

## 📱 Social Media Assets

Templates and specifications in `social/`:
- **Open Graph**: 1200x630px for Facebook, LinkedIn
- **Twitter Cards**: 1200x600px for Twitter
- **Template HTML**: `og-template.html` for generating cards

### Social Card Guidelines
- Use brand gradient background
- Include Uveddi logo prominently
- Maintain consistent typography hierarchy
- Ensure accessibility with sufficient contrast

## 🦉 Mascot: Archie the Code Owl

Our friendly mascot represents:
- **Wisdom**: Owls are symbols of knowledge and insight
- **Vigilance**: Always watching for code issues
- **Privacy**: Protective nature aligns with our values
- **Guidance**: Helpful companion for developers

See `mascot.md` for complete usage guidelines.

## 📐 Typography

### Primary Font: JetBrains Mono
- Use for: Brand name, code examples, technical content
- Weights: 400 (regular), 500 (medium), 700 (bold)
- Reinforces technical, developer-focused identity

### Secondary Font: Inter
- Use for: Body text, descriptions, UI elements
- Weights: 300 (light), 400 (regular), 500 (medium), 600 (semibold), 700 (bold)
- Excellent readability and modern appearance

## 🎨 Design Templates

### Documentation Headers
```html
<div class="uveddi-header">
  <div class="logo">U</div>
  <h1>Page Title</h1>
  <p class="tagline">Descriptive subtitle</p>
</div>
```

### Callout Boxes
```html
<div class="callout info">
  <span class="icon">💡</span>
  <div class="content">
    <strong>Tip:</strong> Helpful information here
  </div>
</div>
```

### Code Blocks
```html
<pre class="uveddi-code">
  <code class="language-rust">
    // Code example with syntax highlighting
  </code>
  <button class="copy-button">📋 Copy</button>
</pre>
```

## 🚀 Implementation Guidelines

### Web Implementation
1. Include `colors/palette.css` for consistent colors
2. Use CSS custom properties for theming
3. Implement dark mode with provided color overrides
4. Ensure favicon package is properly linked

### Print Materials
- Use high-resolution logo files
- Maintain brand colors in CMYK: C:69 M:65 Y:0 K:10
- Include sufficient white space
- Use brand fonts or approved alternatives

### Presentations
- Start with brand slide template
- Use consistent color scheme throughout
- Include Archie for friendly, approachable tone
- Maintain professional appearance

## ✅ Brand Checklist

Before publishing any branded material:

- [ ] Logo used correctly (size, spacing, colors)
- [ ] Brand colors from approved palette
- [ ] Typography follows guidelines
- [ ] Sufficient contrast for accessibility
- [ ] Consistent with Uveddi voice and tone
- [ ] Privacy-first messaging reinforced
- [ ] Technical accuracy maintained
- [ ] Archie used appropriately (if included)

## 📞 Brand Support

For questions about brand usage:
- **Documentation**: This README and linked files
- **Issues**: [GitHub Issues](https://github.com/botzrDev/uveddi/issues) with "branding" label
- **Email**: [brand@uveddi.dev](mailto:brand@uveddi.dev) for sensitive inquiries

## 📄 License

All brand assets are part of the Uveddi project and are licensed under the MIT License. You may use these assets when:
- Contributing to the Uveddi project
- Creating educational content about Uveddi
- Building integrations or extensions for Uveddi

For commercial use outside of these contexts, please contact us for permission.

---

**Remember**: Consistent branding builds trust and recognition. When in doubt, prioritize clarity and accessibility over visual complexity.