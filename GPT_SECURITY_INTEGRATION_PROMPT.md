# GPT Development Prompt: Security Detector Integration with Reporting Systems

## Context and Objectives

You are a senior Rust developer working on the Uveddi architectural analysis tool. Your task is to ensure the comprehensive integration of the newly implemented Security Detector system with all reporting outputs and the React dashboard. The security detector is a multi-agent system that performs vulnerability analysis using OWASP Top 10 coverage, taint analysis, configuration analysis, and SCA (Software Composition Analysis).

## System Overview

### Current Architecture
```
┌─────────────────────────────────────────────────────────────────┐
│                    Uveddi Analysis Engine                      │
├─────────────────────────────────────────────────────────────────┤
│  ┌──────────────┐  ┌──────────────┐  ┌──────────────────────┐   │
│  │ Architecture │  │ Dead Code    │  │ Security Detector    │   │
│  │ Detector     │  │ Detector     │  │ (NEW - TO INTEGRATE) │   │
│  │              │  │              │  │                      │   │
│  └──────────────┘  └──────────────┘  └──────────────────────┘   │
└─────────────────────────────────────────────────────────────────┘
                               │
                ┌──────────────┼──────────────┐
                │              │              │
         ┌──────▼────┐  ┌──────▼────┐  ┌──────▼────┐
         │ HTML      │  │ JSON      │  │ React     │
         │ Reports   │  │ Reports   │  │ Dashboard │
         └───────────┘  └───────────┘  └───────────┘
```

### Integration Requirements
The Security Detector must be integrated with:
1. **HTML Report Generation** (`src/report/html/`)
2. **JSON Report Generation** (`src/report/json/`)
3. **React Dashboard** (`frontend/dashboard/`)
4. **API Endpoints** (`src/api/`)
5. **CLI Output** (`src/cli/`)
6. **SARIF Export** (for CI/CD integration)
7. **Database Storage** (`src/database/`)

## Task Breakdown

### Phase 1: Analysis Engine Integration

#### 1.1 Register Security Detector with Analysis Engine
**File**: `src/analysis/mod.rs`

**Requirements**:
- Add `MainSecurityDetector` to the list of available detectors
- Ensure it implements the `AnalysisDetector` trait properly
- Verify integration with the detector registry
- Add feature flag support (`security` feature)

**Key Implementation Points**:
```rust
// Example integration pattern
impl AnalysisEngine {
    pub fn with_security_detector(mut self) -> Self {
        #[cfg(feature = "security")]
        {
            let security_detector = MainSecurityDetector::new()
                .expect("Failed to initialize security detector");
            self.detectors.push(Box::new(security_detector));
        }
        self
    }
}
```

#### 1.2 Anti-Pattern Type Registration
**File**: `src/database/models.rs`

**Requirements**:
- Ensure all security anti-pattern types are properly registered
- Verify IDs don't conflict with existing anti-pattern types
- Add proper migrations if needed
- Test database schema compatibility

**Security Anti-Pattern Types to Register**:
- Broken Access Control (ID: 100)
- Cryptographic Failures (ID: 101)  
- Injection (ID: 102)
- Insecure Design (ID: 103)
- Security Misconfiguration (ID: 104)
- Vulnerable Components (ID: 105)
- Authentication Failures (ID: 106)
- Software Data Integrity Failures (ID: 107)
- Security Logging Failures (ID: 108)
- Server Side Request Forgery (ID: 109)
- Hardcoded Secrets (ID: 110)
- Insecure Dependencies (ID: 111)

### Phase 2: Database Integration

#### 2.1 Security Issue Storage
**Files**: `src/database/models.rs`, `src/database/connection.rs`

**Requirements**:
- Extend `ArchitecturalIssue` model to support security-specific metadata
- Add security-specific tables if needed:
  - `security_vulnerabilities` table for detailed vulnerability data
  - `taint_flows` table for data flow analysis results
  - `security_correlations` table for architectural correlations
- Implement proper serialization for `VulnerabilityMetadata`
- Add indexes for performance optimization

**Schema Extensions**:
```sql
-- Example additional columns for ArchitecturalIssue
ALTER TABLE architectural_issues ADD COLUMN confidence_score REAL;
ALTER TABLE architectural_issues ADD COLUMN vulnerability_type TEXT;
ALTER TABLE architectural_issues ADD COLUMN cwe_id TEXT;
ALTER TABLE architectural_issues ADD COLUMN cvss_score REAL;
ALTER TABLE architectural_issues ADD COLUMN attack_vector TEXT;

-- New security-specific tables
CREATE TABLE security_vulnerabilities (
    id INTEGER PRIMARY KEY,
    issue_id INTEGER REFERENCES architectural_issues(issue_id),
    owasp_category TEXT,
    remediation_steps TEXT,
    references TEXT, -- JSON array
    tags TEXT, -- JSON array
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);

CREATE TABLE taint_flows (
    id INTEGER PRIMARY KEY,
    vulnerability_id INTEGER REFERENCES security_vulnerabilities(id),
    source_name TEXT,
    source_line INTEGER,
    sink_name TEXT,
    sink_line INTEGER,
    confidence REAL,
    sanitizers TEXT, -- JSON array
    created_at TIMESTAMP DEFAULT CURRENT_TIMESTAMP
);
```

#### 2.2 Migration Scripts
**File**: `migrations/add_security_tables.sql`

**Requirements**:
- Create migration script for new security tables
- Ensure backward compatibility
- Add rollback script
- Test migration on existing databases

### Phase 3: API Integration

#### 3.1 Security Endpoints
**Files**: `src/api/routes/`, `src/api/handlers/`

**Requirements**:
- Add security-specific API endpoints:
  - `GET /api/v1/security/issues` - List security issues
  - `GET /api/v1/security/issues/{id}` - Get specific security issue
  - `GET /api/v1/security/summary` - Security summary statistics
  - `GET /api/v1/security/owasp-coverage` - OWASP Top 10 coverage
  - `GET /api/v1/security/taint-flows` - Taint analysis results
  - `POST /api/v1/security/analyze` - Trigger security analysis
  - `GET /api/v1/security/sarif` - Export SARIF format
- Implement proper error handling and validation
- Add OpenAPI documentation
- Include security in existing aggregate endpoints

**API Response Examples**:
```rust
#[derive(Serialize, Deserialize)]
pub struct SecurityIssueResponse {
    pub id: String,
    pub issue_type: String,
    pub severity: String,
    pub confidence_score: f64,
    pub location: LocationResponse,
    pub description: String,
    pub remediation: String,
    pub owasp_category: Option<String>,
    pub cwe_id: Option<String>,
    pub cvss_score: Option<f64>,
    pub references: Vec<String>,
    pub taint_flows: Option<Vec<TaintFlowResponse>>,
}

#[derive(Serialize, Deserialize)]
pub struct SecuritySummaryResponse {
    pub total_issues: usize,
    pub critical_count: usize,
    pub high_count: usize,
    pub medium_count: usize,
    pub low_count: usize,
    pub owasp_coverage: HashMap<String, OwaspCategoryStats>,
    pub confidence_distribution: HashMap<String, usize>,
    pub most_common_issues: Vec<IssueTypeStats>,
}
```

#### 3.2 Shared Types Integration
**File**: `src/api/types.rs`

**Requirements**:
- Add security-specific shared types
- Ensure serialization compatibility
- Integrate with existing type system
- Add proper documentation

### Phase 4: HTML Report Integration

#### 4.1 Security Section in HTML Reports
**Files**: `src/report/html/`, `templates/`

**Requirements**:
- Add comprehensive security section to HTML reports
- Include:
  - Executive security summary
  - OWASP Top 10 coverage matrix
  - Security issue list with filtering
  - Taint flow visualizations
  - Remediation guidance
  - Risk assessment charts
- Use existing CSS/JS frameworks
- Ensure responsive design
- Add export capabilities (PDF, etc.)

**HTML Template Structure**:
```html
<!-- Security Section Template -->
<section id="security-analysis" class="report-section">
    <h2>Security Analysis</h2>
    
    <!-- Executive Summary -->
    <div class="security-summary">
        <div class="metric-cards">
            <div class="metric-card critical">
                <h3>Critical Issues</h3>
                <span class="count">{{critical_count}}</span>
            </div>
            <!-- More metric cards... -->
        </div>
    </div>
    
    <!-- OWASP Top 10 Coverage -->
    <div class="owasp-coverage">
        <h3>OWASP Top 10 2021 Coverage</h3>
        <table class="owasp-table">
            <!-- OWASP category rows -->
        </table>
    </div>
    
    <!-- Security Issues Table -->
    <div class="security-issues">
        <h3>Security Issues</h3>
        <table class="issues-table sortable filterable">
            <!-- Security issues with details -->
        </table>
    </div>
    
    <!-- Taint Flow Diagrams -->
    <div class="taint-flows">
        <h3>Data Flow Analysis</h3>
        <!-- Mermaid diagrams for taint flows -->
    </div>
</section>
```

#### 4.2 Security Visualizations
**Files**: `src/report/html/charts.rs`, `templates/charts/`

**Requirements**:
- Add security-specific charts and visualizations:
  - Security severity distribution (pie chart)
  - OWASP coverage heat map
  - Confidence score distribution
  - Issues over time (if historical data)
  - Risk vs. effort matrix
- Use Chart.js or similar library
- Ensure accessibility compliance
- Add interactive features

### Phase 5: JSON Report Integration

#### 5.1 Security Data in JSON Reports
**File**: `src/report/json/mod.rs`

**Requirements**:
- Extend JSON report structure to include security data
- Maintain backward compatibility
- Add security-specific report format
- Ensure proper serialization of all security types
- Add JSON schema validation

**JSON Structure**:
```json
{
  "analysis": {
    "security": {
      "summary": {
        "total_issues": 42,
        "critical_count": 3,
        "high_count": 8,
        "medium_count": 20,
        "low_count": 11,
        "confidence_distribution": {
          "high": 25,
          "medium": 12,
          "low": 5
        }
      },
      "owasp_coverage": {
        "A01_broken_access_control": {
          "issues_found": 5,
          "coverage_percentage": 90,
          "avg_confidence": 0.85
        }
      },
      "issues": [
        {
          "id": "sec_001",
          "type": "Injection",
          "severity": "Critical",
          "confidence_score": 0.95,
          "location": {
            "file": "src/auth.py",
            "line": 42,
            "function": "authenticate_user"
          },
          "description": "SQL injection vulnerability detected",
          "remediation": "Use parameterized queries",
          "metadata": {
            "cwe_id": "CWE-89",
            "cvss_score": 9.8,
            "owasp_category": "A03_Injection",
            "references": ["https://owasp.org/..."]
          },
          "taint_flows": [
            {
              "source": {
                "name": "user_input",
                "location": "src/auth.py:35",
                "type": "UserInput"
              },
              "sink": {
                "name": "sql_execute",
                "location": "src/auth.py:42",
                "type": "SqlQuery"
              },
              "confidence": 0.95,
              "sanitizers": []
            }
          ]
        }
      ],
      "correlations": [
        {
          "security_issue_id": "sec_001",
          "anti_pattern": "God Object",
          "correlation_strength": 0.8,
          "explanation": "Complex authentication logic increases injection risk"
        }
      ]
    }
  }
}
```

### Phase 6: React Dashboard Integration

#### 6.1 Security Dashboard Components
**Files**: `frontend/dashboard/src/components/security/`

**Requirements**:
- Create comprehensive security dashboard components:
  - `SecurityOverview.tsx` - Main security dashboard
  - `SecurityMetrics.tsx` - Key security metrics display
  - `OwaspCoverage.tsx` - OWASP Top 10 coverage visualization
  - `SecurityIssuesList.tsx` - Filterable security issues table
  - `TaintFlowDiagram.tsx` - Interactive taint flow visualization
  - `SecurityTrends.tsx` - Historical security trends
  - `RemediationGuidance.tsx` - Interactive remediation suggestions
- Use existing design system and components
- Implement proper error handling and loading states
- Add real-time updates capability
- Ensure accessibility compliance

**Component Examples**:
```typescript
// SecurityOverview.tsx
interface SecurityOverviewProps {
  analysisId: string;
  refreshInterval?: number;
}

export const SecurityOverview: React.FC<SecurityOverviewProps> = ({
  analysisId,
  refreshInterval = 30000
}) => {
  const [securityData, setSecurityData] = useState<SecuritySummary | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<string | null>(null);

  // Implementation with proper error handling, loading states, etc.
  
  return (
    <div className="security-overview">
      <SecurityMetrics summary={securityData?.summary} />
      <OwaspCoverage coverage={securityData?.owasp_coverage} />
      <SecurityIssuesList issues={securityData?.issues} />
      <TaintFlowDiagram flows={securityData?.taint_flows} />
    </div>
  );
};

// SecurityMetrics.tsx
interface SecurityMetricsProps {
  summary: SecuritySummary;
}

export const SecurityMetrics: React.FC<SecurityMetricsProps> = ({ summary }) => {
  return (
    <div className="security-metrics grid grid-cols-5 gap-4">
      <MetricCard
        title="Critical Issues"
        value={summary.critical_count}
        severity="critical"
        icon={<AlertTriangle />}
      />
      <MetricCard
        title="High Risk"
        value={summary.high_count}
        severity="high"
        icon={<AlertCircle />}
      />
      {/* More metric cards... */}
    </div>
  );
};
```

#### 6.2 Security API Integration
**Files**: `frontend/dashboard/src/api/`, `frontend/dashboard/src/hooks/`

**Requirements**:
- Add security API client functions
- Create React hooks for security data fetching
- Implement proper caching and error handling
- Add real-time updates via WebSocket or polling
- Type safety with TypeScript interfaces

**API Client Example**:
```typescript
// api/security.ts
export class SecurityApi {
  private baseUrl: string;
  
  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }
  
  async getSecuritySummary(analysisId: string): Promise<SecuritySummary> {
    const response = await fetch(`${this.baseUrl}/api/v1/security/summary?analysis_id=${analysisId}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch security summary: ${response.statusText}`);
    }
    return response.json();
  }
  
  async getSecurityIssues(analysisId: string, filters?: SecurityFilters): Promise<SecurityIssue[]> {
    const queryParams = new URLSearchParams({
      analysis_id: analysisId,
      ...filters,
    });
    
    const response = await fetch(`${this.baseUrl}/api/v1/security/issues?${queryParams}`);
    if (!response.ok) {
      throw new Error(`Failed to fetch security issues: ${response.statusText}`);
    }
    return response.json();
  }
  
  async exportSarif(analysisId: string): Promise<Blob> {
    const response = await fetch(`${this.baseUrl}/api/v1/security/sarif?analysis_id=${analysisId}`);
    if (!response.ok) {
      throw new Error(`Failed to export SARIF: ${response.statusText}`);
    }
    return response.blob();
  }
}

// hooks/useSecurity.ts
export const useSecurity = (analysisId: string) => {
  const [data, setData] = useState<SecurityData | null>(null);
  const [loading, setLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);
  
  const fetchSecurityData = useCallback(async () => {
    try {
      setLoading(true);
      setError(null);
      
      const [summary, issues, owaspCoverage] = await Promise.all([
        securityApi.getSecuritySummary(analysisId),
        securityApi.getSecurityIssues(analysisId),
        securityApi.getOwaspCoverage(analysisId),
      ]);
      
      setData({ summary, issues, owaspCoverage });
    } catch (err) {
      setError(err instanceof Error ? err : new Error('Unknown error'));
    } finally {
      setLoading(false);
    }
  }, [analysisId]);
  
  useEffect(() => {
    fetchSecurityData();
  }, [fetchSecurityData]);
  
  return { data, loading, error, refetch: fetchSecurityData };
};
```

#### 6.3 Navigation and Routing
**Files**: `frontend/dashboard/src/App.tsx`, `frontend/dashboard/src/routes/`

**Requirements**:
- Add security routes to the application
- Update navigation to include security section
- Ensure proper breadcrumbs and navigation state
- Add security-specific URL parameters and filters

**Routing Structure**:
```typescript
// Add to existing routes
{
  path: "/analysis/:analysisId/security",
  element: <SecurityOverview />,
  children: [
    {
      path: "issues",
      element: <SecurityIssuesList />,
    },
    {
      path: "owasp",
      element: <OwaspCoverage />,
    },
    {
      path: "taint-flows",
      element: <TaintFlowDiagram />,
    },
    {
      path: "remediation",
      element: <RemediationGuidance />,
    },
  ],
}
```

### Phase 7: CLI Integration

#### 7.1 Security Command Options
**File**: `src/cli/mod.rs`

**Requirements**:
- Add security-specific CLI flags and options:
  - `--security` - Enable security analysis
  - `--security-config <path>` - Custom security configuration
  - `--min-security-confidence <value>` - Minimum confidence threshold
  - `--security-output <format>` - Security-specific output format (sarif, json, html)
  - `--security-only` - Run only security analysis
  - `--owasp-categories <list>` - Specific OWASP categories to analyze
- Update help text and documentation
- Ensure backward compatibility

#### 7.2 Security Report Output
**File**: `src/cli/commands/analyze.rs`

**Requirements**:
- Add security section to terminal output
- Include color-coded severity levels
- Show summary statistics
- Provide remediation suggestions
- Add progress indicators for security analysis

**CLI Output Example**:
```
🔒 Security Analysis Results
═══════════════════════════

📊 Summary:
   Critical: 3    High: 8    Medium: 20    Low: 11

🎯 OWASP Top 10 Coverage:
   ✅ A01 Broken Access Control    (90% coverage, 5 issues)
   ⚠️  A02 Cryptographic Failures   (70% coverage, 2 issues)
   ❌ A03 Injection                (95% coverage, 8 issues)
   ...

🚨 Critical Issues:
   1. SQL Injection in src/auth.py:42
      └─ Confidence: 95% | Remediation: Use parameterized queries
   
   2. Command Injection in src/utils.py:15
      └─ Confidence: 89% | Remediation: Validate and sanitize input
   
   3. Hardcoded Secret in config/settings.py:8
      └─ Confidence: 100% | Remediation: Use environment variables

💡 Run with --security-output sarif to export for CI/CD integration
```

### Phase 8: SARIF Export Integration

#### 8.1 SARIF Format Implementation
**File**: `src/report/sarif/mod.rs`

**Requirements**:
- Implement comprehensive SARIF 2.1.0 format export
- Include all security issue metadata
- Map security issues to SARIF rules
- Add tool information and analysis metadata
- Ensure GitHub Security tab compatibility
- Include remediation guidance in SARIF format

**SARIF Structure**:
```json
{
  "$schema": "https://raw.githubusercontent.com/oasis-tcs/sarif-spec/master/Schemata/sarif-schema-2.1.0.json",
  "version": "2.1.0",
  "runs": [
    {
      "tool": {
        "driver": {
          "name": "Uveddi Security Detector",
          "version": "0.9.0",
          "informationUri": "https://github.com/uveddi/uveddi",
          "rules": [
            {
              "id": "sql-injection",
              "name": "SQL Injection",
              "shortDescription": {
                "text": "SQL injection vulnerability detected"
              },
              "fullDescription": {
                "text": "Application is vulnerable to SQL injection attacks through unsanitized user input"
              },
              "defaultConfiguration": {
                "level": "error"
              },
              "properties": {
                "tags": ["security", "injection", "owasp-a03"],
                "precision": "high"
              }
            }
          ]
        }
      },
      "results": [
        {
          "ruleId": "sql-injection",
          "message": {
            "text": "SQL injection vulnerability: User input directly concatenated into SQL query"
          },
          "level": "error",
          "locations": [
            {
              "physicalLocation": {
                "artifactLocation": {
                  "uri": "src/auth.py"
                },
                "region": {
                  "startLine": 42,
                  "endLine": 42,
                  "startColumn": 15,
                  "endColumn": 35
                }
              }
            }
          ],
          "fixes": [
            {
              "description": {
                "text": "Use parameterized queries to prevent SQL injection"
              },
              "artifactChanges": [
                {
                  "artifactLocation": {
                    "uri": "src/auth.py"
                  },
                  "replacements": [
                    {
                      "deletedRegion": {
                        "startLine": 42,
                        "endLine": 42
                      },
                      "insertedContent": {
                        "text": "cursor.execute(\"SELECT * FROM users WHERE id = ?\", (user_id,))"
                      }
                    }
                  ]
                }
              ]
            }
          ],
          "properties": {
            "confidence": 0.95,
            "severity": "critical",
            "cwe": "CWE-89",
            "owasp": "A03_Injection"
          }
        }
      ]
    }
  ]
}
```

### Phase 9: Testing and Validation

#### 9.1 Integration Tests
**Files**: `tests/integration/security_reporting_test.rs`

**Requirements**:
- Create comprehensive integration tests for all reporting formats
- Test end-to-end workflow from detection to reporting
- Validate API responses and data consistency
- Test React dashboard functionality
- Verify SARIF export compliance
- Performance testing for large codebases

#### 9.2 Test Coverage
**Requirements**:
- Ensure minimum 90% test coverage for all integration code
- Test error handling and edge cases
- Validate backward compatibility
- Test with various security issue types and severities

### Phase 10: Documentation and Migration

#### 10.1 API Documentation Updates
**File**: `docs/08-api/openapi.yaml`

**Requirements**:
- Update OpenAPI specification with security endpoints
- Add request/response examples
- Document error codes and responses
- Include authentication requirements

#### 10.2 User Documentation
**Files**: `docs/03-user-guide/`, `docs/07-reference/`

**Requirements**:
- Update user guide with security analysis features
- Add CLI examples and usage patterns
- Document dashboard security features
- Include troubleshooting guide for security analysis

## Implementation Checklist

### Critical Path Items (Must Complete)
- [ ] Register SecurityDetector with AnalysisEngine
- [ ] Add security anti-pattern types to database
- [ ] Implement security API endpoints
- [ ] Add security section to HTML reports
- [ ] Extend JSON report format
- [ ] Create React dashboard security components
- [ ] Implement SARIF export
- [ ] Add CLI security options
- [ ] Create comprehensive integration tests

### Quality Assurance (Must Verify)
- [ ] All security issues appear in HTML reports
- [ ] React dashboard displays security data correctly
- [ ] API endpoints return proper security data
- [ ] SARIF export is valid and GitHub-compatible
- [ ] CLI shows security results appropriately
- [ ] Database stores security data correctly
- [ ] Performance impact is acceptable
- [ ] Error handling works properly
- [ ] Backward compatibility maintained

### Performance Requirements
- [ ] Security analysis adds <20% to overall analysis time
- [ ] Dashboard loads security data in <3 seconds
- [ ] API responses return in <1 second for typical codebases
- [ ] Memory usage increase <50MB for security features
- [ ] SARIF export completes in <10 seconds

### Security Requirements
- [ ] No sensitive data exposed in logs
- [ ] Proper input validation on all endpoints
- [ ] Rate limiting on security API endpoints
- [ ] Secure handling of vulnerability data
- [ ] Proper authentication/authorization checks

## Common Pitfalls to Avoid

1. **Data Inconsistency**: Ensure security data flows consistently through all reporting formats
2. **Performance Degradation**: Monitor impact on analysis performance and optimize accordingly
3. **UI/UX Issues**: Maintain consistent design patterns and user experience
4. **Type Safety**: Ensure proper TypeScript types for React components
5. **Error Handling**: Implement comprehensive error handling at all integration points
6. **Backward Compatibility**: Don't break existing functionality
7. **Test Coverage**: Maintain high test coverage throughout integration
8. **Documentation**: Keep all documentation updated and synchronized

## Success Criteria

The integration is successful when:
1. Security analysis results appear in all report formats (HTML, JSON, SARIF)
2. React dashboard displays comprehensive security information
3. API endpoints provide access to all security data
4. CLI includes security analysis in output
5. All existing functionality continues to work
6. Performance impact is within acceptable limits
7. Test coverage maintains >90% for new code
8. Documentation is complete and accurate

## Post-Integration Tasks

After completing the integration:
1. Performance optimization based on real-world usage
2. User feedback collection and iteration
3. Additional visualization improvements
4. Enhanced filtering and search capabilities
5. Integration with external security tools
6. Advanced analytics and trending features

This comprehensive integration will make Uveddi's security detector a first-class citizen in the analysis ecosystem, providing users with seamless access to security insights across all interfaces and report formats.