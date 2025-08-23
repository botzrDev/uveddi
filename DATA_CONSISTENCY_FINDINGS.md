# Uveddi Data Consistency Analysis Report

## Executive Summary
Critical data contract mismatches have been identified between Uveddi's backend analysis engine and frontend dashboard. These inconsistencies prevent accurate display of analysis results in the React dashboard.

## 1. Data Flow Architecture

### Current Pipeline
```
Rust Analysis Engine (src/analysis/)
    ↓ generates ArchitecturalIssue structs
Analysis Results (database models)
    ↓ transforms to
Report Generation (src/report/) → Multiple formats
    ↓ served via
API Server (src/api/rest.rs) → REST endpoints
    ↓ consumed by
React Dashboard (frontend/) → User interface
```

## 2. Critical Issues Identified

### 🔴 Issue #1: Schema Mismatches Between Analysis Output and API Response

**Backend Analysis Output (JSON export):**
```json
{
  "issues": [
    {
      "antiPatternDescription": "Duplicate code blocks...",
      "antiPatternType": "Code Duplication",
      "codeSnippet": "...",
      "description": "Code duplication detected...",
      "endLine": 146,
      "filePath": "./src/api/server.rs",
      "severity": "low",  // lowercase
      "startLine": 106
    }
  ],
  "metadata": {
    "codebasePath": "./src",
    "durationSeconds": 0.0,
    "timestamp": "2025-08-23T..."
  },
  "summary": {
    "debtScore": 0,
    "filesAnalyzed": 0,  // WRONG - shows 0 despite analyzing files
    "issuesBySeverity": {
      "critical": 0,
      "high": 0,
      "low": 0,
      "medium": 0
    },
    "issuesTotal": 0
  }
}
```

**API Response (InteractiveReport):**
```json
{
  "schemaVersion": "1.0",
  "project": {
    "id": "demo-project",
    "name": "Demo Project",
    // ... more fields
  },
  "summary": {
    "coverage": 82.5,
    "issuesTotal": 7,
    "issuesBySeverity": {
      "critical": 1,
      "high": 3,
      "medium": 3,
      "low": 0
    },
    "issuesByCategory": {
      "anti-patterns": 4,
      "code-quality": 2,
      "security": 1
    },
    "filesAnalyzed": 42,
    "componentsAnalyzed": 15,
    "analysisDurationMs": 1250,
    "timeGenerated": "2025-08-23T..."
  },
  "findings": [  // Note: "findings" not "issues"
    {
      "id": "demo-001",
      "type": "GodObject",  // Note: "type" not "antiPatternType"
      "severity": "critical",  // lowercase
      "title": "Large class with too many responsibilities",
      "message": "...",
      "file": "src/user_manager.rs",  // Note: "file" not "filePath"
      "startLine": 45,
      "endLine": 287,
      "column": null,
      "codeSnippet": "...",
      "tags": ["anti-pattern", "maintainability"],
      "detector": "GodObjectDetector",
      "confidence": 0.89,
      "aiExplanation": "...",
      "recommendation": "...",
      "relatedFindings": []
    }
  ],
  "dependencyGraph": { /* ... */ },
  "diagrams": [ /* ... */ ],
  "metadata": { /* ... */ }
}
```

### 🔴 Issue #2: File Count Mismatch
- Console output: "Files analyzed: 365"
- JSON summary: `"filesAnalyzed": 0`
- API demo: `"filesAnalyzed": 42`

### 🔴 Issue #3: Severity Inconsistencies
- Backend uses: "Critical", "High", "Medium", "Low" (capitalized)
- Some outputs use: "critical", "high", "medium", "low" (lowercase)
- Frontend expects consistent casing

### 🔴 Issue #4: Field Name Inconsistencies

| Backend Field | API Field | Frontend Expects |
|--------------|-----------|------------------|
| `issues` | `findings` | `findings` |
| `antiPatternType` | `type` | `type` |
| `filePath` | `file` | `file` |
| `antiPatternDescription` | (embedded in message) | - |
| - | `title` | `title` (missing in backend) |
| - | `tags` | `tags` (missing in backend) |
| - | `confidence` | `confidence` (missing in backend) |

### 🟡 Issue #5: Missing Data in Backend Analysis
The backend analysis output is missing critical fields that the frontend expects:
- `schemaVersion` - Required for API contract versioning
- `project` metadata object
- `dependencyGraph` - Critical for visualization
- `diagrams` - Required for Mermaid diagrams
- Issue `id` field - Required for tracking
- Issue `tags` - Required for filtering
- Issue `confidence` scores

## 3. Root Causes

1. **Multiple Data Models**: Different structs used at different layers:
   - `ArchitecturalIssue` (database model)
   - `Finding` (interactive report model)
   - Raw JSON export format

2. **No Schema Validation**: No runtime validation ensuring data contracts are met

3. **Incomplete Data Transformation**: The conversion from database models to API responses is incomplete

4. **File Discovery Issue**: The analysis engine reports files analyzed in console but doesn't properly populate the summary

## 4. Impact

- Users cannot trust dashboard displays accurate analysis results
- Some detector results may not appear in dashboard
- Severity filtering doesn't work correctly
- File count always shows 0 in exports
- Dependency graphs may not render

## 5. Recommended Solutions

### Phase 1: Immediate Fixes
1. Fix file count tracking in analysis engine
2. Standardize severity casing across all outputs
3. Ensure all required fields are populated

### Phase 2: Schema Alignment
1. Update backend JSON export to match InteractiveReport schema
2. Add proper data transformation from ArchitecturalIssue to Finding
3. Implement schema validation at API boundaries

### Phase 3: Testing & Validation
1. Add integration tests comparing backend output with API response
2. Implement automated schema validation
3. Add regression tests for data consistency

## Next Steps

1. Fix the file count issue in the analysis engine
2. Implement proper data transformation in report generation
3. Add schema validation middleware to API
4. Update frontend to handle current API response format
5. Create comprehensive integration tests