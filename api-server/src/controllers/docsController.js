const DocsService = require('../services/docsService');

class DocsController {
  constructor() {
    this.docsService = new DocsService();
  }

  getStructure = async (req, res) => {
    try {
      const structure = await this.docsService.getDocumentationStructure();
      res.json(structure);
    } catch (error) {
      console.error('Error getting docs structure:', error);
      res.status(500).json({
        error: 'Failed to load documentation structure',
        message: error.message
      });
    }
  };

  getFile = async (req, res) => {
    try {
      const filePath = req.params[0]; // Capture the wildcard path

      if (!filePath) {
        return res.status(400).json({
          error: 'File path is required'
        });
      }

      const fileContent = await this.docsService.getDocumentationFile(filePath);

      if (!fileContent) {
        return res.status(404).json({
          error: 'Documentation file not found'
        });
      }

      res.json(fileContent);
    } catch (error) {
      console.error('Error getting docs file:', error);
      res.status(500).json({
        error: 'Failed to load documentation file',
        message: error.message
      });
    }
  };

  searchDocs = async (req, res) => {
    try {
      const { q: query } = req.query;

      if (!query) {
        return res.status(400).json({
          error: 'Search query parameter "q" is required'
        });
      }

      const results = await this.docsService.searchDocumentation(query);
      res.json({ results, query });
    } catch (error) {
      console.error('Error searching docs:', error);
      res.status(500).json({
        error: 'Search failed',
        message: error.message
      });
    }
  };
}

module.exports = DocsController;