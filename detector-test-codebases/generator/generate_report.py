# Detector Test Results Generator
import json
import os
import sys
from datetime import datetime
import subprocess

def analyze_file(uveddi_path, file_path):
    """Analyze a single file and return detection results"""
    try:
        cmd = [uveddi_path, "analyze", file_path, "--output-format", "json"]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=30)
        
        if result.returncode == 0 and result.stdout.strip():
            try:
                # Extract JSON from output (Uveddi outputs JSON followed by status messages)
                output = result.stdout.strip()
                
                # Find the start and end of the JSON object
                json_start = output.find('{')
                if json_start == -1:
                    raise json.JSONDecodeError("No JSON found in output", output, 0)
                
                # Find the matching closing brace
                brace_count = 0
                json_end = json_start
                
                for i in range(json_start, len(output)):
                    if output[i] == '{':
                        brace_count += 1
                    elif output[i] == '}':
                        brace_count -= 1
                        if brace_count == 0:
                            json_end = i + 1
                            break
                
                json_text = output[json_start:json_end]
                data = json.loads(json_text)
                
                return {
                    'success': True,
                    'detections': data.get('issues', []),  # Fixed: Uveddi outputs 'issues', not 'detections'
                    'analysis_time': data.get('metadata', {}).get('durationSeconds', 0),  # Fixed: correct path to duration
                    'errors': [],
                    'metadata': data.get('metadata', {}),
                    'summary': data.get('summary', {})
                }
            except json.JSONDecodeError as e:
                return {
                    'success': False, 
                    'detections': [],
                    'errors': [f'JSON parse error: {str(e)} - Output: {result.stdout[:200]}...']
                }
        else:
            return {
                'success': False,
                'detections': [],
                'errors': [f'Analysis failed: {result.stderr[:200]}...']
            }
            
    except subprocess.TimeoutExpired:
        return {
            'success': False,
            'detections': [],
            'errors': ['Analysis timeout (30s)']
        }
    except Exception as e:
        return {
            'success': False,
            'detections': [],
            'errors': [f'Unexpected error: {str(e)}']
        }

def generate_comprehensive_report(test_dirs, uveddi_path):
    """Generate comprehensive test report"""
    
    report = {
        'timestamp': datetime.now().isoformat(),
        'uveddi_version': get_uveddi_version(uveddi_path),
        'test_results': {},
        'summary': {
            'total_files': 0,
            'successful_analyses': 0,
            'failed_analyses': 0,
            'total_detections': 0,
            'detections_by_type': {},
            'total_analysis_time': 0
        }
    }
    
    for test_dir in test_dirs:
        if not os.path.exists(test_dir):
            continue
            
        dir_name = os.path.basename(test_dir)
        report['test_results'][dir_name] = {
            'files': {},
            'summary': {
                'files_tested': 0,
                'successful': 0,
                'failed': 0,
                'detections': 0
            }
        }
        
        # Find all source files
        extensions = ['.rs', '.py', '.ts', '.js']
        source_files = []
        for root, dirs, files in os.walk(test_dir):
            for file in files:
                if any(file.endswith(ext) for ext in extensions):
                    source_files.append(os.path.join(root, file))
        
        for file_path in source_files:
            rel_path = os.path.relpath(file_path, test_dir)
            print(f"Analyzing: {rel_path}")
            
            result = analyze_file(uveddi_path, file_path)
            
            report['test_results'][dir_name]['files'][rel_path] = result
            report['test_results'][dir_name]['summary']['files_tested'] += 1
            report['summary']['total_files'] += 1
            
            if result['success']:
                report['test_results'][dir_name]['summary']['successful'] += 1
                report['summary']['successful_analyses'] += 1
                
                detections = len(result['detections'])
                report['test_results'][dir_name]['summary']['detections'] += detections
                report['summary']['total_detections'] += detections
                
                if 'analysis_time' in result:
                    report['summary']['total_analysis_time'] += result['analysis_time']
                
                # Count detection types
                for detection in result['detections']:
                    det_type = detection.get('detector_type', 'Unknown')
                    report['summary']['detections_by_type'][det_type] = \
                        report['summary']['detections_by_type'].get(det_type, 0) + 1
            else:
                report['test_results'][dir_name]['summary']['failed'] += 1
                report['summary']['failed_analyses'] += 1
    
    return report

def get_uveddi_version(uveddi_path):
    """Get Uveddi version"""
    try:
        result = subprocess.run([uveddi_path, "--version"], capture_output=True, text=True)
        if result.returncode == 0:
            return result.stdout.strip()
    except:
        pass
    return "Unknown"

def generate_markdown_report(report):
    """Generate markdown report from JSON data"""
    
    md = f"""# Uveddi Extreme Detector Test Report

**Generated:** {report['timestamp']}  
**Uveddi Version:** {report['uveddi_version']}  

## Summary

- **Total Files Tested:** {report['summary']['total_files']}
- **Successful Analyses:** {report['summary']['successful_analyses']}
- **Failed Analyses:** {report['summary']['failed_analyses']}
- **Total Detections:** {report['summary']['total_detections']}
- **Total Analysis Time:** {report['summary']['total_analysis_time']:.2f}s

### Detections by Type
"""
    
    for det_type, count in sorted(report['summary']['detections_by_type'].items()):
        md += f"- **{det_type}:** {count}\n"
    
    md += "\n## Detailed Results\n\n"
    
    for dir_name, dir_results in report['test_results'].items():
        md += f"### {dir_name}\n\n"
        md += f"- Files tested: {dir_results['summary']['files_tested']}\n"
        md += f"- Successful: {dir_results['summary']['successful']}\n" 
        md += f"- Failed: {dir_results['summary']['failed']}\n"
        md += f"- Total detections: {dir_results['summary']['detections']}\n\n"
        
        # Show individual file results
        for file_path, file_result in dir_results['files'].items():
            status = "✅" if file_result['success'] else "❌"
            detections = len(file_result['detections']) if file_result['success'] else 0
            
            md += f"#### {status} {file_path}\n"
            if file_result['success']:
                md += f"- Detections: {detections}\n"
                if file_result['detections']:
                    for detection in file_result['detections'][:3]:  # Limit to first 3
                        md += f"  - {detection.get('detector_type', 'Unknown')}: {detection.get('message', 'No message')}\n"
                    if len(file_result['detections']) > 3:
                        md += f"  - ... and {len(file_result['detections']) - 3} more\n"
            else:
                md += f"- Errors: {', '.join(file_result['errors'][:2])}\n"
            md += "\n"
    
    return md

def main():
    if len(sys.argv) < 3:
        print("Usage: python3 generate_report.py <uveddi_binary> <test_dir1> [test_dir2] ...")
        sys.exit(1)
    
    uveddi_path = sys.argv[1]
    test_dirs = sys.argv[2:]
    
    if not os.path.exists(uveddi_path):
        print(f"Error: Uveddi binary not found at {uveddi_path}")
        sys.exit(1)
    
    print("Generating comprehensive test report...")
    report = generate_comprehensive_report(test_dirs, uveddi_path)
    
    # Save JSON report
    with open('comprehensive_test_report.json', 'w') as f:
        json.dump(report, f, indent=2)
    
    # Save markdown report  
    md_report = generate_markdown_report(report)
    with open('comprehensive_test_report.md', 'w') as f:
        f.write(md_report)
    
    print(f"Reports generated:")
    print(f"- JSON: comprehensive_test_report.json")
    print(f"- Markdown: comprehensive_test_report.md")
    print(f"\nSummary: {report['summary']['successful_analyses']}/{report['summary']['total_files']} files analyzed successfully")
    print(f"Total detections: {report['summary']['total_detections']}")

if __name__ == '__main__':
    main()
