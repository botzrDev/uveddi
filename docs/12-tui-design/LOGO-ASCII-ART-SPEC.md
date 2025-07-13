# Uveddi ASCII Logo & Graphics Specification

**Priority:** High  
**Implementation Phase:** Foundation (Phase 1)  
**Affects Tasks:** U1 (Main Menu), F2 (TEA Structure)

## 🎨 **Visual Design Requirements**

### **Logo Placement**
- **Position**: Top of main menu screen, centered
- **Duration**: Always visible on main menu
- **Animation**: Optional fade-in/typewriter effect on startup
- **Colors**: Gradient from cyan to yellow for "UVEDDI", subtle accents

### **ASCII Art Design**

#### **Primary Logo (Large)**
```
 ██    ██ ██    ██ ███████ ██████  ██████  ██ 
 ██    ██ ██    ██ ██      ██   ██ ██   ██ ██ 
 ██    ██ ██    ██ █████   ██   ██ ██   ██ ██ 
 ██    ██  ██  ██  ██      ██   ██ ██   ██ ██ 
  ██████    ████   ███████ ██████  ██████  ██ 
                                              
  ╔═══════════════════════════════════════╗   
  ║    Code Analysis & Quality Insights   ║   
  ╚═══════════════════════════════════════╝   
```

#### **Compact Logo (Small)**
```
 ╭─ UVEDDI ─╮
 │ Analysis │
 ╰─ ────── ─╯
```

#### **Terminal Size Responsive**
- **Width ≥ 80 chars**: Use primary logo
- **Width 60-79 chars**: Use compact logo  
- **Width < 60 chars**: Text-only "UVEDDI" with simple border

### **Color Scheme**
```rust
// Color definitions for logo
const LOGO_PRIMARY: Color = Color::Cyan;
const LOGO_ACCENT: Color = Color::Yellow;
const LOGO_BORDER: Color = Color::Blue;
const LOGO_SUBTITLE: Color = Color::Gray;
```

## 🏗️ **Implementation Details**

### **Logo Component Structure**

```rust
// src/tui/ui/components/logo.rs
pub struct UveddiLogo {
    /// Logo variant based on terminal size
    variant: LogoVariant,
    /// Animation state for startup effect
    animation_state: AnimationState,
    /// Colors for theming
    colors: LogoColors,
}

#[derive(Debug, Clone)]
pub enum LogoVariant {
    Large,      // Primary logo for wide terminals
    Compact,    // Compact logo for medium terminals  
    TextOnly,   // Simple text for narrow terminals
}

#[derive(Debug, Clone)]
pub struct LogoColors {
    pub primary: Color,    // Main "UVEDDI" text
    pub accent: Color,     // Decorative elements
    pub border: Color,     // Box borders
    pub subtitle: Color,   // Tagline text
}
```

### **Animation Options**

#### **Startup Animation (Optional)**
```rust
pub enum LogoAnimation {
    None,           // Instant display
    FadeIn,         // Gradual appearance
    TypeWriter,     // Character-by-character reveal
    SlideDown,      // Slide from top
}
```

#### **Idle Animation (Subtle)**
```rust
// Gentle color cycling or subtle glow effect
// Only when terminal supports advanced colors
pub fn update_idle_animation(&mut self, tick_count: u64) {
    if tick_count % 60 == 0 { // Every 1 second at 60fps
        self.cycle_accent_color();
    }
}
```

## 📐 **Layout Integration**

### **Main Menu Layout Update**

```rust
// Updated layout for main menu with logo
let chunks = Layout::default()
    .direction(Direction::Vertical)
    .constraints([
        Constraint::Length(8),   // Logo area (increased)
        Constraint::Min(8),      // Menu items  
        Constraint::Length(4),   // Footer/help
    ])
    .split(area);

// Render logo in header
self.render_logo(frame, chunks[0]);
```

### **Responsive Behavior**

```rust
impl UveddiLogo {
    pub fn select_variant(terminal_width: u16) -> LogoVariant {
        match terminal_width {
            width if width >= 80 => LogoVariant::Large,
            width if width >= 60 => LogoVariant::Compact,
            _ => LogoVariant::TextOnly,
        }
    }
}
```

## 🎯 **Task Updates Required**

### **Task U1: Main Menu Component**
**Additional Requirements:**
- [ ] Integrate UveddiLogo component in header
- [ ] Handle responsive logo sizing
- [ ] Implement logo color theming
- [ ] Add optional startup animation

**Updated Acceptance Criteria:**
- [ ] Logo displays prominently at top of main menu
- [ ] Logo adapts to different terminal sizes
- [ ] Logo colors follow theme system
- [ ] Logo animation (if enabled) plays on startup

### **Task F2: Basic TEA Structure**
**Additional Message Types:**
```rust
pub enum AppMessage {
    // ... existing messages
    
    // Logo-specific messages
    LogoAnimationComplete,
    LogoVariantChanged(LogoVariant),
    ThemeChanged(Theme),
}
```

**Updated App State:**
```rust
pub struct AppState {
    // ... existing fields
    
    /// Logo component state
    pub logo_state: UveddiLogo,
    /// Whether startup animation is enabled
    pub logo_animation_enabled: bool,
}
```

## 🖼️ **ASCII Art Assets**

### **Large Logo (Primary)**
```
████████╗   ██╗██╗   ██╗███████╗██████╗ ██████╗ ██╗
╚══██╔══╝   ██║██║   ██║██╔════╝██╔══██╗██╔══██╗██║
   ██║█████╗██║██║   ██║█████╗  ██║  ██║██║  ██║██║
   ██║╚════╝██║╚██╗ ██╔╝██╔══╝  ██║  ██║██║  ██║██║
   ██║      ██║ ╚████╔╝ ███████╗██████╔╝██████╔╝██║
   ╚═╝      ╚═╝  ╚═══╝  ╚══════╝╚═════╝ ╚═════╝ ╚═╝

╔════════════════════════════════════════════════╗
║        🔍 Code Analysis & Quality Insights     ║
║           Powered by AI & Static Analysis      ║
╚════════════════════════════════════════════════╝
```

### **Medium Logo (Compact)**
```
┌─ UVEDDI ─┐
│ Analysis │
│ & Insights│
└─ ────── ─┘
  🔍 🤖 📊
```

### **Small Logo (Text-Only)**
```
═══ UVEDDI ═══
Code Analysis
```

### **Decorative Elements**
```rust
const DECORATIVE_ELEMENTS: &[&str] = &[
    "🔍",  // Analysis/search
    "🤖",  // AI-powered  
    "📊",  // Data/metrics
    "🛠️",   // Tools
    "⚡",  // Fast/efficient
    "🎯",  // Precise/accurate
];
```

## 🎨 **Theming Integration**

### **Logo Theme Variants**
```rust
pub struct LogoTheme {
    pub primary: Style,      // Main UVEDDI text
    pub accent: Style,       // Decorative elements
    pub border: Style,       // Box drawing characters
    pub subtitle: Style,     // Tagline/description
    pub background: Option<Color>,
}

// Predefined themes
impl LogoTheme {
    pub fn default() -> Self {
        Self {
            primary: Style::default().fg(Color::Cyan).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::Yellow),
            border: Style::default().fg(Color::Blue),
            subtitle: Style::default().fg(Color::Gray).italic(),
            background: None,
        }
    }
    
    pub fn dark() -> Self {
        Self {
            primary: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::Yellow),
            border: Style::default().fg(Color::DarkGray),
            subtitle: Style::default().fg(Color::Gray),
            background: Some(Color::Black),
        }
    }
    
    pub fn monochrome() -> Self {
        Self {
            primary: Style::default().fg(Color::White).add_modifier(Modifier::BOLD),
            accent: Style::default().fg(Color::White),
            border: Style::default().fg(Color::White),
            subtitle: Style::default().fg(Color::Gray),
            background: None,
        }
    }
}
```

## 🔧 **Implementation Priority**

### **Phase 1: Basic Logo Display**
1. Create logo component with static display
2. Integrate with main menu layout
3. Implement responsive sizing

### **Phase 2: Theming & Polish**
1. Add color theming support
2. Implement theme switching
3. Test on various terminals

### **Phase 3: Animation (Optional)**
1. Add startup animation options
2. Implement idle effects
3. Performance optimization

## ✅ **Verification Criteria**

### **Visual Verification**
- [ ] Logo displays correctly on terminals 80x24 and larger
- [ ] Logo adapts appropriately to smaller terminal sizes
- [ ] Colors render correctly on different terminal types
- [ ] Logo remains readable in monochrome mode

### **Performance Verification**
- [ ] Logo rendering doesn't impact startup time
- [ ] Animation (if enabled) runs smoothly at 60fps
- [ ] No visual artifacts or flickering
- [ ] Memory usage remains reasonable

### **Integration Verification**
- [ ] Logo integrates seamlessly with main menu
- [ ] Theme changes apply to logo correctly
- [ ] Logo responds to terminal resize events
- [ ] Logo state persists correctly in app state

## 🎭 **Alternative ASCII Art Options**

### **Option A: Block Letters**
```
██    ██ ██    ██ ███████ ██████  ██████  ██ 
██    ██ ██    ██ ██      ██   ██ ██   ██ ██ 
██    ██ ██    ██ █████   ██   ██ ██   ██ ██ 
██    ██  ██  ██  ██      ██   ██ ██   ██ ██ 
 ██████    ████   ███████ ██████  ██████  ██ 
```

### **Option B: Line Art**
```
╦ ╦╦  ╦╔═╗╔╦╗╔╦╗╦
║ ║╚╗╔╝║╣  ║║ ║║║
╚═╝ ╚╝ ╚═╝═╩╝═╩╝╩
```

### **Option C: 3D Effect**
```
 __    __  __     __  ______  _____   _____   ______  
/\ \  /\ \/\ \   /\ \/\  ___\/\  __-./\  __-./\__  _\ 
\ \ \_\ \ \ \ \  \ \ \ \  __\\ \ \/\ \ \ \/\ \/_/\ \/ 
 \ \_____\ \_\ \  \ \_\ \_____\ \____-\ \____-  \ \_\ 
  \/_____/\/_/\/_/  \/_/\/_____/\/____/ \/____/   \/_/ 
```

This specification ensures the Uveddi TUI opens with a professional, branded experience that immediately communicates the tool's identity and purpose while maintaining responsive design across different terminal environments.