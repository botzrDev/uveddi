const express = require('express');
const router = express.Router();

// Get project information
router.get('/api/project/info', (req, res) => {
  res.json({
    name: 'Uveddi',
    version: '0.9.0-alpha',
    description: 'AI-Powered Code Analysis Tool',
    repository: 'https://github.com/botzrDev/uveddi',
    documentation: 'https://botzrdev.github.io/uveddi/',
    features: [
      'Multi-language analysis',
      'AI-powered insights',
      'Privacy-focused local analysis',
      'Extensible plugin architecture',
      'Terminal User Interface (TUI)',
      'Comprehensive reporting'
    ],
    environment: 'alpha'
  });
});

module.exports = router;