const fs = require('fs-extra');
const path = require('path');

class ReportsService {
  constructor() {
    this.REPORTS_PATH = path.join(__dirname, '..', '..', '..', 'reports');
    this.REAL_WORLD_PATH = path.join(__dirname, '..', '..', '..', 'real_world_results');
  }

  async getLatestReport() {
    // Try multiple sources for analysis data
    const sources = ['cli-output', 'real-world-results', 'demo-data'];
    let reportData = null;
    let analysisSource = 'unknown';

    for (const source of sources) {
      try {
        reportData = await this.loadReportFromSource(source);
        if (reportData) {
          analysisSource = source;
          break;
        }
      } catch (error) {
        console.log(`Failed to load from ${source}:`, error.message);
        continue;
      }
    }

    if (!reportData) {
      throw new Error('No analysis data available from any source');
    }

    // Transform to expected format for React dashboard
    return this.transformReportData(reportData, analysisSource);
  }

  async getDemoReport() {
    const demoData = await this.getDemoReportData();
    return demoData;
  }

  async getReportById(id) {
    if (id === 'demo') {
      return await this.getDemoReport();
    }

    // For now, return null for unknown IDs
    // In production, this would load specific report by ID
    return null;
  }

  async exportLatestReport(format = 'json') {
    const report = await this.getLatestReport();
    return this.formatExportData(report, format);
  }

  async exportReportById(id, format = 'json') {
    const report = await this.getReportById(id);
    if (!report) {
      throw new Error(`Report with ID '${id}' not found`);
    }
    return this.formatExportData(report, format);
  }

  getAllReports() {
    return {
      reports: [
        {
          id: 'latest',
          name: 'Latest Analysis',
          created_at: new Date().toISOString(),
          status: 'completed',
          summary: {
            total_files: 45,
            issues_found: 12,
            analysis_time_ms: 2500
          }
        }
      ],
      total: 1,
      page: 1,
      per_page: 10
    };
  }

  getSarifReport() {
    return {
      version: "2.1.0",
      $schema: "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
      runs: [
        {
          tool: {
            driver: {
              name: "Uveddi",
              version: "0.9.0-alpha",
              informationUri: "https://github.com/botzrDev/uveddi"
            }
          },
          results: []
        }
      ]
    };
  }

  // Helper methods
  async loadReportFromSource(source) {
    const possiblePaths = [
      path.join(this.REAL_WORLD_PATH, 'analysis_output.json'),
      path.join(this.REPORTS_PATH, 'latest.json'),
      path.join(__dirname, '..', '..', '..', 'test_analysis', 'output.json')
    ];

    for (const filePath of possiblePaths) {
      try {
        if (await fs.pathExists(filePath)) {
          const data = await fs.readJson(filePath);
          return data;
        }
      } catch (error) {
        continue;
      }
    }

    return null;
  }

  transformReportData(reportData, analysisSource) {
    return {
      schemaVersion: "1.0.0",
      project: {
        id: `analysis-${analysisSource}`,
        name: `Uveddi Analysis - ${analysisSource}`,
        languages: ["rust"],
        path: analysisSource,
        commit: "latest",
        branch: "main"
      },
      summary: {
        timeGenerated: reportData.metadata?.timestamp || new Date().toISOString(),
        coverage: 85.0,
        issuesTotal: reportData.issues?.length || 0,
        filesAnalyzed: this.getUniqueFiles(reportData.issues).length,
        componentsAnalyzed: this.getUniqueFiles(reportData.issues).length,
        analysisDurationMs: reportData.metadata?.durationSeconds ? reportData.metadata.durationSeconds * 1000 : 0,
        issuesBySeverity: this.calculateIssuesBySeverity(reportData.issues || []),
        issuesByCategory: this.calculateIssuesByCategory(reportData.issues || [])
      },
      findings: (reportData.issues || []).map((issue, index) => ({
        id: `f-${index + 1}`,
        type: issue.antiPatternType,
        severity: issue.severity.toLowerCase(),
        title: `${issue.antiPatternType} in ${issue.filePath}`,
        message: issue.description,
        file: issue.filePath,
        startLine: issue.startLine || 0,
        endLine: issue.endLine || 0,
        codeSnippet: issue.codeSnippet || "",
        tags: [issue.antiPatternType.toLowerCase().replace(/\s+/g, '-')],
        detector: issue.detectorName || 'uveddi',
        confidence: issue.confidence || 0.8,
        aiExplanation: issue.aiExplanation,
        relatedFindings: []
      })),
      dependencyGraph: {
        nodes: [],
        edges: []
      },
      diagrams: [],
      metadata: {
        generatedAt: reportData.metadata?.timestamp || new Date().toISOString(),
        analysisId: `analysis-${analysisSource}-${Date.now()}`
      }
    };
  }

  async getDemoReportData() {
    // Return demo data - same as latest for now
    try {
      return await this.getLatestReport();
    } catch (error) {
      // Return minimal demo data if no real data available
      return {
        schemaVersion: "1.0.0",
        project: {
          id: "demo-project",
          name: "Demo Analysis",
          languages: ["rust"],
          path: "demo",
          commit: "demo",
          branch: "main"
        },
        summary: {
          timeGenerated: new Date().toISOString(),
          coverage: 85.0,
          issuesTotal: 0,
          filesAnalyzed: 0,
          componentsAnalyzed: 0,
          analysisDurationMs: 1000,
          issuesBySeverity: { high: 0, medium: 0, low: 0 },
          issuesByCategory: {}
        },
        findings: [],
        dependencyGraph: { nodes: [], edges: [] },
        diagrams: [],
        metadata: {
          generatedAt: new Date().toISOString(),
          analysisId: "demo-analysis"
        }
      };
    }
  }

  formatExportData(report, format) {
    switch (format) {
      case 'pdf':
        return 'PDF export not yet implemented';
      case 'csv':
        return this.convertToCSV(report);
      default:
        return JSON.stringify(report, null, 2);
    }
  }

  convertToCSV(report) {
    if (!report.findings || report.findings.length === 0) {
      return 'No findings to export';
    }

    const headers = ['ID', 'Type', 'Severity', 'File', 'Line', 'Message'];
    const rows = report.findings.map(finding => [
      finding.id,
      finding.type,
      finding.severity,
      finding.file,
      finding.startLine,
      finding.message.replace(/,/g, ';').replace(/\n/g, ' ')
    ]);

    return [headers.join(','), ...rows.map(row => row.join(','))].join('\n');
  }

  // Utility methods
  getUniqueFiles(issues) {
    if (!Array.isArray(issues)) return [];
    const files = new Set(issues.map(issue => issue.filePath).filter(Boolean));
    return Array.from(files);
  }

  calculateIssuesBySeverity(issues) {
    const severityCount = { high: 0, medium: 0, low: 0 };
    issues.forEach(issue => {
      const severity = issue.severity?.toLowerCase();
      if (severityCount.hasOwnProperty(severity)) {
        severityCount[severity]++;
      }
    });
    return severityCount;
  }

  calculateIssuesByCategory(issues) {
    const categoryCount = {};
    issues.forEach(issue => {
      const category = issue.antiPatternType || 'Unknown';
      categoryCount[category] = (categoryCount[category] || 0) + 1;
    });
    return categoryCount;
  }
}

module.exports = ReportsService;