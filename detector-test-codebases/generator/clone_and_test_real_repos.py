# Clone and Test Real GitHub Repositories
import os
import subprocess
import sys
from pathlib import Path

# Curated list of repositories with known complexity issues
REAL_WORLD_REPOS = [
    # Small but complex Rust projects
    {
        'url': 'https://github.com/serde-rs/serde.git',
        'name': 'serde',
        'language': 'rust',
        'complexity': 'high',
        'expected_issues': ['large_classes', 'tight_coupling']
    },
    {
        'url': 'https://github.com/actix/actix-web.git', 
        'name': 'actix-web',
        'language': 'rust',
        'complexity': 'high',
        'expected_issues': ['god_objects', 'large_classes']
    },
    
    # Python projects with known legacy patterns
    {
        'url': 'https://github.com/pallets/flask.git',
        'name': 'flask', 
        'language': 'python',
        'complexity': 'medium',
        'expected_issues': ['magic_values', 'long_methods']
    },
    {
        'url': 'https://github.com/psf/requests.git',
        'name': 'requests',
        'language': 'python', 
        'complexity': 'medium',
        'expected_issues': ['dead_code', 'code_duplication']
    },
    
    # TypeScript/JavaScript with async complexity
    {
        'url': 'https://github.com/expressjs/express.git',
        'name': 'express',
        'language': 'javascript',
        'complexity': 'medium', 
        'expected_issues': ['tight_coupling', 'magic_values']
    },
]

def clone_repo(repo_config, base_dir):
    """Clone a repository with depth limit"""
    clone_path = Path(base_dir) / repo_config['name']
    
    if clone_path.exists():
        print(f"Repository {repo_config['name']} already exists, skipping...")
        return clone_path
        
    try:
        cmd = ['git', 'clone', '--depth', '1', repo_config['url'], str(clone_path)]
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=300)
        
        if result.returncode == 0:
            print(f"Successfully cloned {repo_config['name']}")
            return clone_path
        else:
            print(f"Failed to clone {repo_config['name']}: {result.stderr}")
            return None
            
    except subprocess.TimeoutExpired:
        print(f"Timeout cloning {repo_config['name']}")
        return None
    except Exception as e:
        print(f"Error cloning {repo_config['name']}: {e}")
        return None

def analyze_repo(uveddi_path, repo_path, repo_config):
    """Analyze a cloned repository"""
    try:
        cmd = [uveddi_path, 'analyze', str(repo_path), '--output-format', 'json']
        result = subprocess.run(cmd, capture_output=True, text=True, timeout=120)
        
        if result.returncode == 0 and result.stdout.strip():
            import json
            try:
                data = json.loads(result.stdout)
                return {
                    'success': True,
                    'detections': data.get('detections', []),
                    'analysis_time': data.get('analysis_time_ms', 0) / 1000,
                    'file_count': data.get('files_analyzed', 0)
                }
            except json.JSONDecodeError:
                return {'success': False, 'error': 'JSON parse error'}
        else:
            return {'success': False, 'error': result.stderr[:500]}
            
    except subprocess.TimeoutExpired:
        return {'success': False, 'error': 'Analysis timeout (120s)'}
    except Exception as e:
        return {'success': False, 'error': str(e)}

def main():
    if len(sys.argv) != 3:
        print("Usage: python3 clone_and_test_real_repos.py <uveddi_binary> <output_dir>")
        sys.exit(1)
        
    uveddi_path = sys.argv[1]
    output_dir = Path(sys.argv[2])
    
    if not os.path.exists(uveddi_path):
        print(f"Error: Uveddi binary not found at {uveddi_path}")
        sys.exit(1)
        
    output_dir.mkdir(parents=True, exist_ok=True)
    
    results = {
        'timestamp': str(Path(__file__).stat().st_mtime),
        'repositories': {}
    }
    
    print(f"Cloning and testing {len(REAL_WORLD_REPOS)} real-world repositories...")
    print("This may take several minutes...")
    
    for repo_config in REAL_WORLD_REPOS:
        print(f"\n=== Processing {repo_config['name']} ===")
        
        # Clone repository
        repo_path = clone_repo(repo_config, output_dir)
        if not repo_path:
            results['repositories'][repo_config['name']] = {
                'cloned': False,
                'analysis': {'success': False, 'error': 'Clone failed'}
            }
            continue
            
        # Analyze repository  
        print(f"Analyzing {repo_config['name']}...")
        analysis_result = analyze_repo(uveddi_path, repo_path, repo_config)
        
        results['repositories'][repo_config['name']] = {
            'cloned': True,
            'config': repo_config,
            'analysis': analysis_result
        }
        
        if analysis_result['success']:
            detections = len(analysis_result['detections'])
            print(f"  ✅ Analysis complete: {detections} detections found")
            print(f"     Files analyzed: {analysis_result.get('file_count', 'unknown')}")
            print(f"     Analysis time: {analysis_result.get('analysis_time', 0):.2f}s")
        else:
            print(f"  ❌ Analysis failed: {analysis_result['error']}")
    
    # Generate summary report
    report_file = output_dir / 'real_world_test_results.md'
    with open(report_file, 'w') as f:
        f.write("# Real-World Repository Analysis Results\n\n")
        
        successful = sum(1 for r in results['repositories'].values() if r['analysis']['success'])
        total = len(results['repositories'])
        
        f.write(f"**Repositories tested:** {total}\n")
        f.write(f"**Successful analyses:** {successful}\n\n")
        
        for repo_name, repo_result in results['repositories'].items():
            f.write(f"## {repo_name}\n")
            
            if repo_result['cloned']:
                config = repo_result['config']
                f.write(f"- **Language:** {config['language']}\n")
                f.write(f"- **Expected complexity:** {config['complexity']}\n")
                f.write(f"- **Expected issues:** {', '.join(config['expected_issues'])}\n")
                
                if repo_result['analysis']['success']:
                    analysis = repo_result['analysis']
                    detections = len(analysis['detections'])
                    f.write(f"- **Detections found:** {detections}\n")
                    f.write(f"- **Analysis time:** {analysis.get('analysis_time', 0):.2f}s\n")
                    
                    # Group detections by type
                    detection_types = {}
                    for detection in analysis['detections']:
                        det_type = detection.get('detector_type', 'Unknown')
                        detection_types[det_type] = detection_types.get(det_type, 0) + 1
                    
                    if detection_types:
                        f.write("- **Detection breakdown:**\n")
                        for det_type, count in sorted(detection_types.items()):
                            f.write(f"  - {det_type}: {count}\n")
                else:
                    f.write(f"- **Analysis failed:** {repo_result['analysis']['error']}\n")
            else:
                f.write("- **Status:** Clone failed\n")
            
            f.write("\n")
    
    print(f"\n=== REAL-WORLD TESTING COMPLETE ===")
    print(f"Results written to: {report_file}")
    print(f"Successfully analyzed: {successful}/{total} repositories")

if __name__ == '__main__':
    main()
