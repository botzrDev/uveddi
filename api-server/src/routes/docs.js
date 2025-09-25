const express = require('express');
const DocsController = require('../controllers/docsController');

const router = express.Router();
const docsController = new DocsController();

// Documentation API endpoints
router.get('/api/docs/structure', docsController.getStructure);
router.get('/api/docs/file/*', docsController.getFile);
router.get('/api/docs/search', docsController.searchDocs);

module.exports = router;