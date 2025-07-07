# Multi-stage Docker build for Uveddi Rendering Service

# Stage 1: Node.js service setup
FROM node:18-alpine AS node-stage

WORKDIR /app

# Install dependencies
COPY rendering-service/package*.json ./
RUN npm ci --only=production

# Copy application code
COPY rendering-service/src ./src

# Install Playwright browsers
RUN npx playwright install chromium --with-deps

EXPOSE 3001

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=40s --retries=3 \
  CMD node -e "require('http').get('http://localhost:3001/health', (res) => { process.exit(res.statusCode === 200 ? 0 : 1); })"

CMD ["npm", "start"]
