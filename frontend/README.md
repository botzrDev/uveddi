# Uveddi Interactive Reports Frontend

A modern React + TypeScript single-page application for visualizing Uveddi architectural analysis results.

## Features

- **Dashboard View**: High-level project metrics and issue summaries
- **Interactive Visualizations**: Dependency graphs, architectural diagrams, and charts
- **Findings Explorer**: Searchable and filterable list of code quality issues
- **Offline Support**: PWA capabilities for viewing reports without internet
- **Dark/Light Theme**: Automatic theme detection with manual toggle
- **Responsive Design**: Works on desktop and mobile devices

## Technology Stack

- **React 18**: Modern React with hooks and concurrent features
- **TypeScript**: Type-safe development
- **Vite**: Fast build tool and development server
- **Material-UI**: Comprehensive component library
- **React Query**: Server state management and caching
- **React Router**: Client-side routing
- **Chart.js**: Charts and metrics visualization
- **Cytoscape.js**: Interactive dependency graphs
- **Mermaid.js**: Diagram rendering
- **Workbox**: Service worker for PWA features

## Development

### Prerequisites

- Node.js 18+ 
- npm 9+

### Setup

```bash
# Install dependencies
npm install

# Start development server
npm run dev

# The app will be available at http://localhost:3000
# API requests are proxied to http://localhost:4000
```

### Development Commands

```bash
# Type checking
npm run type-check

# Linting
npm run lint

# Testing
npm run test
npm run test:ui

# Build for production
npm run build

# Preview production build
npm run preview
```

## API Integration

The frontend connects to the Uveddi REST API:

- **Base URL**: `/api/v1` (proxied to localhost:4000 in development)
- **Demo Report**: `GET /api/v1/reports/demo`
- **Report by ID**: `GET /api/v1/reports/:id`
- **Dependency Graph**: `GET /api/v1/reports/:id/graphs/dependency`
- **Health Check**: `GET /health`

## Project Structure

```
src/
├── components/          # Reusable React components
├── pages/              # Page-level components
├── hooks/              # Custom React hooks
├── services/           # API service layer
├── types/              # TypeScript type definitions
├── utils/              # Utility functions and theme
├── App.tsx             # Main application component
├── main.tsx            # Application entry point
└── index.css           # Global styles
```

## State Management

- **Server State**: React Query for API data caching and synchronization
- **UI State**: React Context for theme and global UI preferences
- **Component State**: Built-in React hooks (useState, useReducer)

## Routing

- `/` - Redirects to demo dashboard
- `/dashboard/:reportId` - Dashboard view for a specific report
- `/reports` - List of all available reports
- `/reports/:reportId` - Detailed report view

## PWA Features

The application includes Progressive Web App capabilities:

- **Service Worker**: Caches app shell and API responses
- **Offline Support**: Previously viewed reports available offline
- **Installable**: Can be installed as a native app
- **Background Sync**: Syncs data when connection is restored

## Theming

The application supports light and dark themes:

- **Auto-detection**: Uses system preference by default
- **Manual Toggle**: Theme switcher in the app bar
- **Persistence**: Theme preference saved to localStorage
- **Material Design**: Follows Material Design 3 guidelines

## Performance

- **Code Splitting**: Vendor and feature-based chunks
- **Lazy Loading**: Components loaded on demand
- **Caching**: Aggressive caching of static assets and API responses
- **Optimized Bundles**: Tree-shaking and minification

## Accessibility

- **Semantic HTML**: Proper heading hierarchy and landmarks
- **Keyboard Navigation**: Full keyboard accessibility
- **Screen Readers**: ARIA labels and descriptions
- **Color Contrast**: WCAG AA compliant color schemes
- **Focus Management**: Visible focus indicators

## Browser Support

- Chrome 90+
- Firefox 90+
- Safari 14+
- Edge 90+

## Contributing

1. Follow the existing code style and TypeScript patterns
2. Add tests for new components and utilities
3. Update type definitions when modifying API interfaces
4. Test accessibility features with screen readers
5. Ensure responsive design works on all device sizes

## Building for Production

```bash
# Build optimized bundle
npm run build

# The built files will be in the dist/ directory
# These can be served by the Rust backend or any static file server
```

## Integration with Rust Backend

The frontend is designed to be served by the Uveddi Rust backend:

```bash
# Start the backend server with SPA assets
uveddi ui serve --spa-assets ./frontend/dist --port 4000

# Or run in development mode with CORS enabled
uveddi ui serve --dev --port 4000
```

The Rust backend serves:
- API endpoints at `/api/v1/*`
- Static SPA assets at `/app/*`
- SPA fallback for client-side routing
- Health and metrics endpoints

## Environment Variables

The frontend can be configured with these environment variables:

- `VITE_API_BASE_URL`: Override API base URL (default: `/api/v1`)
- `VITE_ENABLE_DEV_TOOLS`: Enable React DevTools in production

## License

See the main project LICENSE file.