# Social Media Card Specifications

This directory contains social media card assets for Uveddi's documentation and web presence.

## Required Assets

### 1. Open Graph Image (`og-image.png`)
**Dimensions:** 1200x630 pixels  
**Format:** PNG  
**Purpose:** Facebook, LinkedIn, general social sharing

**Design Specifications:**
- **Background:** Gradient from `--uveddi-primary-600` (#4f46e5) to `--uveddi-primary-800` (#3730a3)
- **Logo:** White version of Uveddi logo, positioned left-center
- **Title:** "Uveddi" in JetBrains Mono, 72px, white, positioned right of logo
- **Tagline:** "AI-Powered Code Analysis That Stays Private" in Inter, 36px, `--uveddi-secondary-100` (#f1f5f9)
- **Subtitle:** "Architectural insights • Privacy-first • Local analysis" in Inter, 24px, `--uveddi-secondary-300` (#cbd5e1)
- **Accent:** Small orange dot (`--uveddi-action-500`) as visual accent

### 2. Twitter Card Image (`twitter-card.png`)
**Dimensions:** 1200x600 pixels  
**Format:** PNG  
**Purpose:** Twitter sharing

**Design Specifications:**
- **Background:** Same gradient as OG image
- **Layout:** Centered composition with logo above text
- **Logo:** White Uveddi logo, 80px height
- **Title:** "Uveddi" in JetBrains Mono, 64px, white
- **Tagline:** "AI-Powered Code Analysis" in Inter, 32px, `--uveddi-secondary-100`
- **Feature highlights:** Three key features in smaller text below

## Design Guidelines

### Typography
- **Primary font:** JetBrains Mono for brand name and technical elements
- **Secondary font:** Inter for descriptions and body text
- **Weights:** Bold (700) for titles, Medium (500) for taglines, Regular (400) for body

### Colors
Use the brand palette from `../colors/palette.css`:
- **Primary gradient:** `--uveddi-primary-600` to `--uveddi-primary-800`
- **Text:** White and light grays from secondary palette
- **Accent:** Orange from action palette for visual interest

### Logo Usage
- Use the white version (`../logos/logo-white.svg`) for contrast against dark background
- Maintain clear space around logo equal to logo height
- Logo should be prominent but not overwhelming

### Content Hierarchy
1. **Brand name** (largest, most prominent)
2. **Primary value proposition** (medium size, clear)
3. **Key features/benefits** (smaller, supporting)

## Implementation Notes

### Tools for Creation
- **Figma/Sketch:** For precise design control
- **Canva:** For quick iteration using brand assets
- **Code-based:** HTML/CSS with headless browser screenshot

### Optimization
- **File size:** Keep under 1MB for fast loading
- **Compression:** Use PNG with optimization
- **Accessibility:** Ensure sufficient color contrast (minimum 4.5:1)

### Testing
Test social cards across platforms:
- Facebook Sharing Debugger
- Twitter Card Validator  
- LinkedIn Post Inspector
- WhatsApp and other messaging apps

## Content Variations

### Primary Message (Default)
**Title:** "Uveddi"  
**Tagline:** "AI-Powered Code Analysis That Stays Private"  
**Features:** "Architectural insights • Privacy-first • Local analysis"

### Developer-Focused Variant
**Title:** "Uveddi"  
**Tagline:** "Find Code Issues Before They Find You"  
**Features:** "Multi-language • AI-enhanced • Zero data sharing"

### Enterprise Variant
**Title:** "Uveddi"  
**Tagline:** "Enterprise Code Quality at Scale"  
**Features:** "Compliance ready • Team collaboration • Advanced analytics"

## File Naming Convention
- `og-image.png` - Default Open Graph image
- `og-image-dev.png` - Developer-focused variant
- `og-image-enterprise.png` - Enterprise variant
- `twitter-card.png` - Default Twitter card
- `twitter-card-dev.png` - Developer-focused Twitter variant

This ensures consistent branding across all social platforms while maintaining the professional, privacy-focused identity of Uveddi.