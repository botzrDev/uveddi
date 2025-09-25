const express = require('express');
const healthRoutes = require('./health');
const projectRoutes = require('./project');
const authRoutes = require('./auth');
const docsRoutes = require('./docs');
const reportsRoutes = require('./reports');

const router = express.Router();

// Mount route modules
router.use(healthRoutes);
router.use(projectRoutes);
router.use(authRoutes);
router.use(docsRoutes);
router.use(reportsRoutes);

module.exports = router;