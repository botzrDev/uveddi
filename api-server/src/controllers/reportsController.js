const ReportsService = require('../services/reportsService');

class ReportsController {
  constructor() {
    this.reportsService = new ReportsService();
  }

  getLatestReport = async (req, res) => {
    try {
      const report = await this.reportsService.getLatestReport();
      res.json(report);
    } catch (error) {
      console.error('Error loading latest report:', error);
      res.status(500).json({
        error: 'Failed to load latest analysis report',
        message: error.message
      });
    }
  };

  getDemoReport = async (req, res) => {
    try {
      const demoData = await this.reportsService.getDemoReport();
      res.json(demoData);
    } catch (error) {
      console.error('Error loading demo report:', error);
      res.status(500).json({
        error: 'Internal Server Error',
        message: 'Failed to load demo report',
        timestamp: new Date().toISOString()
      });
    }
  };

  getReportById = async (req, res) => {
    try {
      const { id } = req.params;

      if (id === 'latest') {
        return res.redirect('/api/v1/reports/latest');
      }

      const report = await this.reportsService.getReportById(id);

      if (!report) {
        return res.status(404).json({
          error: 'Report Not Found',
          message: `Report with ID '${id}' not found`,
          timestamp: new Date().toISOString()
        });
      }

      res.json(report);
    } catch (error) {
      console.error('Error loading report by ID:', error);
      res.status(500).json({
        error: 'Failed to load report',
        message: error.message
      });
    }
  };

  getDependencyGraph = (req, res) => {
    try {
      res.json({
        data: {
          nodes: [],
          edges: []
        }
      });
    } catch (error) {
      res.status(500).json({
        error: 'Failed to load dependency graph',
        message: error.message
      });
    }
  };

  exportLatestReport = async (req, res) => {
    try {
      const exportData = await this.reportsService.exportLatestReport(req.query.format);

      if (req.query.format === 'pdf') {
        res.setHeader('Content-Type', 'application/pdf');
        res.setHeader('Content-Disposition', 'attachment; filename="uveddi-report-latest.pdf"');
      } else if (req.query.format === 'csv') {
        res.setHeader('Content-Type', 'text/csv');
        res.setHeader('Content-Disposition', 'attachment; filename="uveddi-report-latest.csv"');
      } else {
        res.setHeader('Content-Type', 'application/json');
        res.setHeader('Content-Disposition', 'attachment; filename="uveddi-report-latest.json"');
      }

      res.send(exportData);
    } catch (error) {
      console.error('Error exporting latest report:', error);
      res.status(500).json({
        error: 'Export failed',
        message: error.message
      });
    }
  };

  exportReportById = async (req, res) => {
    try {
      const { id } = req.params;
      const exportData = await this.reportsService.exportReportById(id, req.query.format);

      if (req.query.format === 'pdf') {
        res.setHeader('Content-Type', 'application/pdf');
        res.setHeader('Content-Disposition', `attachment; filename="uveddi-report-${id}.pdf"`);
      } else if (req.query.format === 'csv') {
        res.setHeader('Content-Type', 'text/csv');
        res.setHeader('Content-Disposition', `attachment; filename="uveddi-report-${id}.csv"`);
      } else {
        res.setHeader('Content-Type', 'application/json');
        res.setHeader('Content-Disposition', `attachment; filename="uveddi-report-${id}.json"`);
      }

      res.send(exportData);
    } catch (error) {
      console.error('Error exporting report by ID:', error);
      res.status(500).json({
        error: 'Export failed',
        message: error.message
      });
    }
  };

  getAllReports = (req, res) => {
    try {
      const reports = this.reportsService.getAllReports();
      res.json(reports);
    } catch (error) {
      console.error('Error loading all reports:', error);
      res.status(500).json({
        error: 'Failed to load reports',
        message: error.message
      });
    }
  };

  getSarifReport = (req, res) => {
    try {
      const sarifData = this.reportsService.getSarifReport();
      res.json(sarifData);
    } catch (error) {
      console.error('Error generating SARIF report:', error);
      res.status(500).json({
        error: 'Failed to generate SARIF report',
        message: error.message
      });
    }
  };
}

module.exports = ReportsController;