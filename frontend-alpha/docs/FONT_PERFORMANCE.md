# Font Performance Optimization Guide

## Current State
- Using Google Fonts CDN for Inter and JetBrains Mono
- Basic font-display: swap implementation

## Recommended Optimizations

### 1. Font Preloading (Immediate)
Add to your `index.html` for critical fonts:

```html
<link rel="preload" href="https://fonts.googleapis.com/css2?family=Inter:wght@300;400;500;600;700&display=swap" as="style" crossorigin>
<link rel="preload" href="https://fonts.googleapis.com/css2?family=JetBrains+Mono:wght@400;500;600;700&display=swap" as="style" crossorigin>
```

### 2. Font Subsetting (Phase 2)
Create subsets for English-only usage:
- Inter: ~50KB → ~15KB (70% reduction)
- JetBrains Mono: ~40KB → ~12KB (70% reduction)

### 3. Self-Hosting Migration (Phase 3)
Benefits:
- Eliminate DNS lookup to Google servers
- Better caching control
- GDPR compliance
- Faster loading for repeat visitors

### 4. Variable Font Upgrade (Future)
Consider migrating to variable fonts:
- Inter Variable: Single file for all weights
- JetBrains Mono Variable: Coming soon

## Implementation Priority
1. **Immediate**: Add font preloading to index.html
2. **Week 2**: Implement font subsetting
3. **Week 3**: Self-host optimized fonts
4. **Future**: Evaluate variable font migration

## Performance Targets
- Current: ~200KB font downloads
- Target: ~50KB font downloads (75% reduction)
- LCP improvement: 0.5-1.0 seconds
- CLS improvement: Eliminate font-swap layout shifts
