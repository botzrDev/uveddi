const express = require('express');
const ReportsController = require('../controllers/reportsController');

const router = express.Router();
const reportsController = new ReportsController();

// Reports API endpoints
router.get('/api/v1/reports/latest', reportsController.getLatestReport);
router.get('/api/v1/reports/demo', reportsController.getDemoReport);
router.get('/api/v1/reports/:id', reportsController.getReportById);
router.get('/api/v1/reports/:id/graphs/dependency', reportsController.getDependencyGraph);
router.get('/api/v1/reports/latest/export', reportsController.exportLatestReport);
router.get('/api/v1/reports/:id/export', reportsController.exportReportById);
router.get('/api/v1/reports', reportsController.getAllReports);
router.get('/api/v1/security/sarif', reportsController.getSarifReport);

module.exports = router;