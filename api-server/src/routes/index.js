const express = require('express');
const healthRoutes = require('./health');
const projectRoutes = require('./project');
const authRoutes = require('./auth');
const docsRoutes = require('./docs');

const router = express.Router();

// Mount route modules
router.use(healthRoutes);
router.use(projectRoutes);
router.use(authRoutes);
router.use(docsRoutes);

module.exports = router;