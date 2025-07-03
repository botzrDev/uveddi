# Uveddi Color Palette

This document contains all the colors used in the Uveddi project, organized by purpose and usage.

## Primary Color Palette

### Primary Colors (Professional Navy Blue)
*Used for trust and stability - main brand colors*

| Shade | Hex Code | Usage |
|-------|----------|-------|
| primary-50 | `#f0f4ff` | Very light backgrounds, subtle highlights |
| primary-100 | `#e0e7ff` | Light backgrounds, hover states |
| primary-200 | `#c7d2fe` | Light accents, disabled states |
| primary-300 | `#a5b4fc` | Subtle borders, inactive elements |
| primary-400 | `#818cf8` | Secondary buttons, icons |
| primary-500 | `#6366f1` | **Main brand color** - primary buttons, links |
| primary-600 | `#4f46e5` | Primary button hover, active states |
| primary-700 | `#4338ca` | Dark primary elements, headers |
| primary-800 | `#3730a3` | Very dark primary, text on light backgrounds |
| primary-900 | `#312e81` | Darkest primary, high contrast text |
| primary-950 | `#1e1b4b` | Ultra dark primary, maximum contrast |

### Secondary Colors (Charcoal Gray)
*Used for backgrounds and text*

| Shade | Hex Code | Usage |
|-------|----------|-------|
| secondary-50 | `#f8fafc` | Lightest backgrounds, cards |
| secondary-100 | `#f1f5f9` | Light backgrounds, subtle surfaces |
| secondary-200 | `#e2e8f0` | Borders, dividers |
| secondary-300 | `#cbd5e1` | Light borders, inactive text |
| secondary-400 | `#94a3b8` | Placeholder text, secondary text |
| secondary-500 | `#64748b` | **Main secondary color** - body text |
| secondary-600 | `#475569` | Dark text, headings |
| secondary-700 | `#334155` | Very dark text, emphasis |
| secondary-800 | `#1e293b` | Dark backgrounds, dark mode |
| secondary-900 | `#0f172a` | Darkest backgrounds |
| secondary-950 | `#020617` | Ultra dark, maximum contrast backgrounds |

## Action Colors

### Success Colors (Vibrant Green)
*Used for low-friction CTAs (Get Started, Download)*

| Shade | Hex Code | Usage |
|-------|----------|-------|
| success-50 | `#f0fdf4` | Success message backgrounds |
| success-100 | `#dcfce7` | Light success states |
| success-200 | `#bbf7d0` | Success borders, highlights |
| success-300 | `#86efac` | Success icons, secondary elements |
| success-400 | `#4ade80` | Success buttons, positive actions |
| success-500 | `#22c55e` | **Main success color** - primary success |
| success-600 | `#16a34a` | Success hover states |
| success-700 | `#15803d` | Dark success elements |
| success-800 | `#166534` | Very dark success |
| success-900 | `#14532d` | Darkest success, text on light |

### Action Colors (Bright Orange)
*Used for high-commitment CTAs (Request Demo, Contact Sales)*

| Shade | Hex Code | Usage |
|-------|----------|-------|
| action-50 | `#fff7ed` | Action message backgrounds |
| action-100 | `#ffedd5` | Light action states |
| action-200 | `#fed7aa` | Action borders, highlights |
| action-300 | `#fdba74` | Action icons, secondary elements |
| action-400 | `#fb923c` | Action buttons, CTA elements |
| action-500 | `#f97316` | **Main action color** - primary CTAs |
| action-600 | `#ea580c` | Action hover states |
| action-700 | `#c2410c` | Dark action elements |
| action-800 | `#9a3412` | Very dark action |
| action-900 | `#7c2d12` | Darkest action, text on light |

### Accent Colors (Amber/Yellow)
*Used for accent highlights*

| Shade | Hex Code | Usage |
|-------|----------|-------|
| accent-50 | `#fef3c7` | Accent backgrounds, highlights |
| accent-100 | `#fde68a` | Light accent states |
| accent-200 | `#fcd34d` | Accent borders, warnings |
| accent-300 | `#fbbf24` | Accent icons, notifications |
| accent-400 | `#f59e0b` | Accent buttons, highlights |
| accent-500 | `#d97706` | **Main accent color** - primary accents |
| accent-600 | `#b45309` | Accent hover states |
| accent-700 | `#92400e` | Dark accent elements |
| accent-800 | `#78350f` | Very dark accent |
| accent-900 | `#451a03` | Darkest accent, text on light |

## Utility Colors

### Scrollbar Colors
*Used for custom scrollbar styling*

| Element | Light Mode | Dark Mode | Usage |
|---------|------------|-----------|-------|
| Track | `#f3f4f6` | `#1f2937` | Scrollbar track background |
| Thumb | `#d1d5db` | `#4b5563` | Scrollbar thumb |
| Thumb Hover | `#9ca3af` | `#6b7280` | Scrollbar thumb on hover |

### Legacy/Component Colors
*Colors found in App.css and other components*

| Color | Hex Code | Usage |
|-------|----------|-------|
| Logo React Hover | `#61dafbaa` | React logo hover effect (with transparency) |
| Logo Generic Hover | `#646cffaa` | Generic logo hover effect (with transparency) |
| Read Docs Text | `#888` | Documentation link text |

## Special Effects

### Glow Effects
*Used for interactive elements and focus states*

| Effect | Color | Usage |
|--------|-------|-------|
| Primary Glow | `rgba(99, 102, 241, 0.3)` | Primary button/element glow |
| Primary Glow Large | `rgba(99, 102, 241, 0.4)` | Enhanced primary glow |
| Success Glow | `rgba(34, 197, 94, 0.3)` | Success element glow |
| Action Glow | `rgba(249, 115, 22, 0.3)` | Action element glow |

## Color Usage Guidelines

### Primary Use Cases
- **Primary (Navy Blue)**: Main brand elements, primary buttons, navigation, logos
- **Secondary (Charcoal Gray)**: Text, backgrounds, borders, secondary UI elements
- **Success (Green)**: Success messages, positive actions, "Get Started" buttons
- **Action (Orange)**: High-commitment CTAs, "Request Demo", "Contact Sales"
- **Accent (Amber)**: Highlights, warnings, notifications, special emphasis

### Accessibility Notes
- All color combinations meet WCAG 2.1 AA contrast requirements
- Focus states use primary-400 (`#818cf8`) with 2px outline
- Reduced motion preferences are respected in animations
- Color is never the only means of conveying information

### Design System Integration
- Colors are defined in Tailwind CSS configuration
- CSS custom properties provide fallbacks
- Consistent naming convention: `[purpose]-[shade]`
- Responsive design considerations built into color choices

## Color Harmony
The palette uses a professional and trustworthy color scheme:
- **Navy Blue**: Conveys reliability, professionalism, and trust
- **Charcoal Gray**: Provides excellent readability and neutral balance
- **Vibrant Green**: Encourages positive action and growth
- **Bright Orange**: Creates urgency and drives high-value conversions
- **Amber/Yellow**: Adds warmth and highlights important information

This color palette is designed to work harmoniously across all components while maintaining strong visual hierarchy and accessibility standards.
