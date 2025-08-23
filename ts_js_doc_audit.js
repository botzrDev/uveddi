#!/usr/bin/env node

/**
 * TypeScript/JavaScript Documentation Audit Script
 * 
 * Analyzes TypeScript and JavaScript files for documentation coverage
 * focusing on file headers and export documentation.
 */

const fs = require('fs');
const path = require('path');

class JSDocAudit {
    constructor() {
        this.filesAnalyzed = 0;
        this.filesWithHeaders = 0;
        this.filesMissingHeaders = [];
        this.totalExports = 0;
        this.documentedExports = 0;
        this.undocumentedExports = [];
        this.criticalFiles = [];
    }

    /**
     * Calculate module documentation coverage percentage
     * @returns {number} Percentage coverage
     */
    getModuleCoveragePercentage() {
        return this.filesAnalyzed === 0 ? 0 : 
            (this.filesWithHeaders / this.filesAnalyzed) * 100;
    }

    /**
     * Calculate export documentation coverage percentage
     * @returns {number} Percentage coverage
     */
    getExportCoveragePercentage() {
        return this.totalExports === 0 ? 0 : 
            (this.documentedExports / this.totalExports) * 100;
    }
}

/**
 * Check if a file is considered critical for documentation
 * @param {string} filePath - Path to the file
 * @returns {boolean} True if file is critical
 */
function isCriticalFile(filePath) {
    const criticalPatterns = [
        'src/App.tsx',
        'src/main.tsx',
        'src/index.ts',
        'server.js',
        'api.ts',
        'types/api.ts',
        'types/dashboard.ts',
        'services/api.ts',
        'components/index.ts',
        'utils/index.ts'
    ];
    
    return criticalPatterns.some(pattern => filePath.includes(pattern));
}

/**
 * Check if directory should be skipped during analysis
 * @param {string} dirPath - Directory path
 * @returns {boolean} True if directory should be skipped
 */
function shouldSkipDirectory(dirPath) {
    const dirName = path.basename(dirPath);
    const skipDirs = ['node_modules', 'dist', 'build', '__tests__', 'coverage', '.git'];
    return skipDirs.includes(dirName);
}

/**
 * Find all TypeScript and JavaScript files in a directory
 * @param {string} dir - Directory to search
 * @returns {string[]} Array of file paths
 */
function findTSJSFiles(dir) {
    const files = [];
    
    if (!fs.existsSync(dir)) {
        return files;
    }
    
    const entries = fs.readdirSync(dir, { withFileTypes: true });
    
    for (const entry of entries) {
        const fullPath = path.join(dir, entry.name);
        
        if (entry.isDirectory() && !shouldSkipDirectory(fullPath)) {
            files.push(...findTSJSFiles(fullPath));
        } else if (entry.isFile()) {
            const ext = path.extname(entry.name);
            if (['.ts', '.tsx', '.js', '.jsx'].includes(ext)) {
                files.push(fullPath);
            }
        }
    }
    
    return files;
}

/**
 * Check if a line contains a JSDoc comment
 * @param {string} line - Line of code
 * @returns {boolean} True if line contains JSDoc
 */
function isJSDocLine(line) {
    const trimmed = line.trim();
    return trimmed.startsWith('/**') || 
           trimmed.startsWith('*') || 
           trimmed.includes('*/') ||
           trimmed.startsWith('//');
}

/**
 * Extract file header JSDoc from content
 * @param {string} content - File content
 * @returns {object} Header analysis result
 */
function extractFileHeader(content) {
    const lines = content.split('\n');
    const headerLines = [];
    let inFileHeader = false;
    let inJSDoc = false;
    
    // Look for file header in first 30 lines
    for (let i = 0; i < Math.min(30, lines.length); i++) {
        const line = lines[i];
        const trimmed = line.trim();
        
        // Skip empty lines and imports at the start
        if (!trimmed || 
            trimmed.startsWith('import ') || 
            trimmed.startsWith('export ') ||
            trimmed.startsWith('const ') ||
            trimmed.startsWith('let ') ||
            trimmed.startsWith('var ')) {
            if (inJSDoc) break; // End of header block
            continue;
        }
        
        if (trimmed.startsWith('/**')) {
            inJSDoc = true;
            inFileHeader = true;
            headerLines.push(trimmed);
        } else if (inJSDoc && (trimmed.startsWith('*') || trimmed.includes('*/'))) {
            headerLines.push(trimmed);
            if (trimmed.includes('*/')) {
                break;
            }
        } else if (trimmed.startsWith('//') && !inJSDoc) {
            // Single line comments can also be file headers
            inFileHeader = true;
            headerLines.push(trimmed);
        } else if (inFileHeader && !trimmed.startsWith('//') && !isJSDocLine(line)) {
            // End of comment block
            break;
        }
    }
    
    return {
        hasHeader: headerLines.length > 0,
        headerContent: headerLines.join('\n')
    };
}

/**
 * Extract export declarations from file content
 * @param {string} content - File content
 * @param {string} filePath - Path to file for context
 * @returns {object[]} Array of export declarations
 */
function extractExports(content, filePath) {
    const lines = content.split('\n');
    const exports = [];
    const exportPatterns = [
        /^export\s+(default\s+)?(function|class|interface|type|const|let|var|enum)\s+(\w+)/,
        /^export\s+default\s+(\w+)/,
        /^export\s*\{([^}]+)\}/,
        /^module\.exports\s*=|^exports\.(\w+)/
    ];
    
    for (let i = 0; i < lines.length; i++) {
        const line = lines[i].trim();
        
        for (const pattern of exportPatterns) {
            const match = line.match(pattern);
            if (match) {
                const exportName = match[3] || match[1] || 'default';
                
                // Check for JSDoc in previous lines (up to 5 lines before)
                const hasDoc = checkForJSDocBeforeLine(lines, i);
                const docContent = hasDoc ? extractJSDocBeforeLine(lines, i) : null;
                
                exports.push({
                    name: exportName,
                    type: determineExportType(line),
                    lineNumber: i + 1,
                    hasDoc,
                    docContent,
                    line: line
                });
                break;
            }
        }
    }
    
    return exports;
}

/**
 * Check for JSDoc comments before a specific line
 * @param {string[]} lines - All lines in the file
 * @param {number} lineIndex - Index of the line to check before
 * @returns {boolean} True if JSDoc found
 */
function checkForJSDocBeforeLine(lines, lineIndex) {
    const startIndex = Math.max(0, lineIndex - 5);
    
    for (let i = lineIndex - 1; i >= startIndex; i--) {
        const line = lines[i].trim();
        if (line.startsWith('/**') || line.startsWith('//')) {
            return true;
        }
        if (line && !line.startsWith('*') && !line.startsWith('//')) {
            break; // Non-comment line found
        }
    }
    return false;
}

/**
 * Extract JSDoc content before a specific line
 * @param {string[]} lines - All lines in the file
 * @param {number} lineIndex - Index of the line
 * @returns {string|null} JSDoc content or null
 */
function extractJSDocBeforeLine(lines, lineIndex) {
    const startIndex = Math.max(0, lineIndex - 10);
    const docLines = [];
    let collecting = false;
    
    for (let i = lineIndex - 1; i >= startIndex; i--) {
        const line = lines[i].trim();
        if (line.startsWith('/**') || line.startsWith('//')) {
            collecting = true;
            docLines.unshift(line);
        } else if (collecting && (line.startsWith('*') || line.includes('*/'))) {
            docLines.unshift(line);
        } else if (collecting && line) {
            break;
        }
    }
    
    return docLines.length > 0 ? docLines.join('\n') : null;
}

/**
 * Determine the type of export from the line
 * @param {string} line - Export line
 * @returns {string} Export type
 */
function determineExportType(line) {
    if (line.includes('function')) return 'function';
    if (line.includes('class')) return 'class';
    if (line.includes('interface')) return 'interface';
    if (line.includes('type')) return 'type';
    if (line.includes('enum')) return 'enum';
    if (line.includes('const') || line.includes('let') || line.includes('var')) return 'variable';
    if (line.includes('default')) return 'default';
    return 'unknown';
}

/**
 * Analyze a single TypeScript/JavaScript file
 * @param {string} filePath - Path to the file
 * @returns {object} Analysis result
 */
function analyzeFile(filePath) {
    try {
        const content = fs.readFileSync(filePath, 'utf-8');
        const headerAnalysis = extractFileHeader(content);
        const exports = extractExports(content, filePath);
        
        return {
            path: filePath,
            hasHeader: headerAnalysis.hasHeader,
            headerContent: headerAnalysis.headerContent,
            exports,
            error: null
        };
    } catch (error) {
        return {
            path: filePath,
            hasHeader: false,
            headerContent: null,
            exports: [],
            error: error.message
        };
    }
}

/**
 * Perform comprehensive TypeScript/JavaScript documentation audit
 * @param {string[]} directories - Directories to audit
 * @returns {JSDocAudit} Audit results
 */
function auditTSJSDocumentation(directories) {
    const audit = new JSDocAudit();
    
    for (const dir of directories) {
        console.log(`📁 Scanning ${dir}...`);
        const files = findTSJSFiles(dir);
        
        for (const filePath of files) {
            const analysis = analyzeFile(filePath);
            
            if (analysis.error) {
                continue;
            }
            
            audit.filesAnalyzed++;
            
            if (analysis.hasHeader) {
                audit.filesWithHeaders++;
            } else {
                audit.filesMissingHeaders.push({
                    path: filePath,
                    critical: isCriticalFile(filePath)
                });
            }
            
            if (isCriticalFile(filePath)) {
                audit.criticalFiles.push(filePath);
            }
            
            // Process exports
            for (const exportItem of analysis.exports) {
                audit.totalExports++;
                
                if (exportItem.hasDoc) {
                    audit.documentedExports++;
                } else {
                    audit.undocumentedExports.push({
                        file: filePath,
                        export: exportItem,
                        critical: isCriticalFile(filePath)
                    });
                }
            }
        }
    }
    
    return audit;
}

/**
 * Print comprehensive audit results
 * @param {JSDocAudit} audit - Audit results
 */
function printResults(audit) {
    console.log('\n📊 TYPESCRIPT/JAVASCRIPT DOCUMENTATION AUDIT RESULTS');
    console.log('===================================================');
    console.log('');
    
    console.log('📈 COVERAGE SUMMARY:');
    console.log(`  • Files analyzed: ${audit.filesAnalyzed}`);
    console.log(`  • File header coverage: ${audit.getModuleCoveragePercentage().toFixed(1)}% (${audit.filesWithHeaders}/${audit.filesAnalyzed})`);
    console.log(`  • Export documentation coverage: ${audit.getExportCoveragePercentage().toFixed(1)}% (${audit.documentedExports}/${audit.totalExports})`);
    console.log('');
    
    // Files missing headers
    if (audit.filesMissingHeaders.length > 0) {
        console.log(`❌ FILES MISSING FILE HEADERS (${audit.filesMissingHeaders.length}):`);
        
        // Sort by critical files first
        const sortedFiles = audit.filesMissingHeaders.sort((a, b) => b.critical - a.critical);
        
        for (const file of sortedFiles) {
            const priority = file.critical ? '🔴 CRITICAL' : '🟡 NORMAL';
            console.log(`  ${priority} ${file.path}`);
        }
        console.log('');
    }
    
    // Undocumented exports
    if (audit.undocumentedExports.length > 0) {
        console.log(`❌ UNDOCUMENTED EXPORTS (${audit.undocumentedExports.length}):`);
        
        // Group by file and sort by priority
        const groupedByFile = {};
        for (const item of audit.undocumentedExports) {
            if (!groupedByFile[item.file]) {
                groupedByFile[item.file] = [];
            }
            groupedByFile[item.file].push(item.export);
        }
        
        const sortedFiles = Object.keys(groupedByFile).sort((a, b) => {
            const aCritical = isCriticalFile(a);
            const bCritical = isCriticalFile(b);
            return bCritical - aCritical;
        });
        
        for (const filePath of sortedFiles) {
            const exports = groupedByFile[filePath];
            const priority = isCriticalFile(filePath) ? '🔴 CRITICAL' : '🟡 NORMAL';
            console.log(`  ${priority} ${filePath} (${exports.length} exports):`);
            
            for (const exportItem of exports) {
                console.log(`    • Line ${exportItem.lineNumber}: export ${exportItem.type} ${exportItem.name}`);
            }
        }
        console.log('');
    }
    
    // Recommendations
    console.log('💡 RECOMMENDATIONS:');
    console.log('  1. Add JSDoc file headers (/**) to files missing them');
    console.log('  2. Document exported functions, classes, and types');
    console.log('  3. Focus on critical files first (marked with 🔴)');
    console.log('  4. Use consistent JSDoc formatting for better tooling support');
    console.log('  5. Consider automated documentation generation tools');
    console.log('');
    
    if (audit.getModuleCoveragePercentage() < 70) {
        console.log('⚠️  File header coverage is below 70% - consider prioritizing');
    }
    if (audit.getExportCoveragePercentage() < 60) {
        console.log('⚠️  Export documentation coverage is below 60% - impacts API usability');
    }
}

// Main execution
function main() {
    const directories = [
        'frontend/src',
        'api-server',
        'rendering-service/src',
        'assets/js'
    ];
    
    console.log('🔍 Starting TypeScript/JavaScript Documentation Audit...');
    console.log(`📁 Analyzing directories: ${directories.join(', ')}`);
    console.log('');
    
    const audit = auditTSJSDocumentation(directories);
    printResults(audit);
}

if (require.main === module) {
    main();
}

module.exports = { auditTSJSDocumentation, JSDocAudit };