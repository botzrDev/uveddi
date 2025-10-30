#!/usr/bin/env python3
"""
UV-60: Scalability Testing for Various Project Sizes
Tests performance with different project sizes (1k, 5k, 10k+ files) as specified in acceptance criteria.
"""

import os
import sys
import json
import time
import subprocess
import tempfile
import shutil
from pathlib import Path
from typing import Dict, Any, List, Tuple
from datetime import datetime
import statistics
import psutil

class ScalabilityTester:
    def __init__(self):
        self.test_sizes = [1000, 5000, 10000]
        self.output_dir = "scalability-test-results"
        self.temp_projects = {}
        
        # Performance expectations from benchmark-config.toml
        self.expectations = {
            1000: {"max_time_ms": 100, "max_memory_mb": 50, "max_cpu_percent": 50},
            5000: {"max_time_ms": 500, "max_memory_mb": 200, "max_cpu_percent": 70},
            10000: {"max_time_ms": 2000, "max_memory_mb": 500, "max_cpu_percent": 85}
        }
        
        os.makedirs(self.output_dir, exist_ok=True)
    
    def generate_test_file(self, file_index: int, complexity: str = "medium") -> str:
        """Generate a Rust source file for testing."""
        if complexity == "simple":
            return f"""
// Generated test file {file_index}
pub fn function_{file_index}() -> i32 {{
    {file_index} + 42
}}

pub struct SimpleStruct{file_index} {{
    value: i32,
}}

impl SimpleStruct{file_index} {{
    pub fn new(value: i32) -> Self {{
        Self {{ value }}
    }}
    
    pub fn get_value(&self) -> i32 {{
        self.value
    }}
}}
"""
        elif complexity == "complex":
            methods = []
            for i in range(20):
                methods.append(f"""
    pub fn method_{i}(&self) -> i32 {{
        match self.field_0 % {i + 1} {{
            0 => self.field_1 + {i},
            1 => self.field_2 * {i},
            _ => self.field_3 - {i},
        }}
    }}""")
            
            return f"""
// Generated complex test file {file_index}
use std::collections::HashMap;

pub struct ComplexStruct{file_index} {{
    field_0: i32,
    field_1: i32,
    field_2: i32,
    field_3: i32,
    data: HashMap<String, i32>,
}}

impl ComplexStruct{file_index} {{
    pub fn new() -> Self {{
        let mut data = HashMap::new();
        for i in 0..10 {{
            data.insert(format!("key_{{}}", i), i * {file_index});
        }}
        
        Self {{
            field_0: {file_index},
            field_1: {file_index} * 2,
            field_2: {file_index} * 3,
            field_3: {file_index} * 4,
            data,
        }}
    }}
    {''.join(methods)}
}}

pub enum TestEnum{file_index} {{
    Variant1(i32),
    Variant2(String),
    Variant3 {{ x: f64, y: f64 }},
}}

pub trait TestTrait{file_index} {{
    fn test_method(&self) -> i32;
}}

impl TestTrait{file_index} for ComplexStruct{file_index} {{
    fn test_method(&self) -> i32 {{
        self.field_0 + self.field_1
    }}
}}
"""
        else:  # medium complexity
            return f"""
// Generated medium test file {file_index}
use std::vec::Vec;

pub struct MediumStruct{file_index} {{
    id: i32,
    name: String,
    values: Vec<i32>,
}}

impl MediumStruct{file_index} {{
    pub fn new(id: i32, name: String) -> Self {{
        Self {{
            id,
            name,
            values: vec![1, 2, 3, 4, 5],
        }}
    }}
    
    pub fn add_value(&mut self, value: i32) {{
        self.values.push(value);
    }}
    
    pub fn get_sum(&self) -> i32 {{
        self.values.iter().sum()
    }}
    
    pub fn get_average(&self) -> f64 {{
        if self.values.is_empty() {{
            0.0
        }} else {{
            self.get_sum() as f64 / self.values.len() as f64
        }}
    }}
    
    pub fn filter_values(&self, min: i32) -> Vec<i32> {{
        self.values.iter().filter(|&&x| x >= min).cloned().collect()
    }}
}}

pub fn process_data_{file_index}(data: &[i32]) -> i32 {{
    data.iter()
        .filter(|&&x| x % 2 == 0)
        .map(|&x| x * 2)
        .sum()
}}

#[cfg(test)]
mod tests {{
    use super::*;
    
    #[test]
    fn test_medium_struct_{file_index}() {{
        let mut ms = MediumStruct{file_index}::new({file_index}, "test".to_string());
        ms.add_value(10);
        assert!(ms.get_sum() > 0);
    }}
}}
"""
    
    def create_test_project(self, file_count: int) -> str:
        """Create a temporary Rust project with the specified number of files."""
        print(f"📁 Creating test project with {file_count} files...")
        
        # Create temporary directory
        temp_dir = tempfile.mkdtemp(prefix=f"scalability_test_{file_count}_")
        src_dir = os.path.join(temp_dir, "src")
        os.makedirs(src_dir)
        
        # Create Cargo.toml
        cargo_toml = f"""[package]
name = "scalability_test_{file_count}"
version = "0.1.0"
edition = "2021"

[dependencies]
"""
        with open(os.path.join(temp_dir, "Cargo.toml"), "w") as f:
            f.write(cargo_toml)
        
        # Create lib.rs
        lib_rs = "// Generated library for scalability testing\n\n"
        for i in range(file_count):
            module_name = f"module_{i}"
            lib_rs += f"pub mod {module_name};\n"
        
        with open(os.path.join(src_dir, "lib.rs"), "w") as f:
            f.write(lib_rs)
        
        # Create individual source files
        for i in range(file_count):
            complexity = ["simple", "medium", "complex"][i % 3]
            content = self.generate_test_file(i, complexity)
            
            with open(os.path.join(src_dir, f"module_{i}.rs"), "w") as f:
                f.write(content)
        
        self.temp_projects[file_count] = temp_dir
        print(f"✅ Test project created at {temp_dir}")
        return temp_dir
    
    def measure_system_resources(self) -> Dict[str, float]:
        """Measure current system resource usage."""
        process = psutil.Process()
        
        return {
            "cpu_percent": process.cpu_percent(),
            "memory_mb": process.memory_info().rss / 1024 / 1024,
            "system_cpu_percent": psutil.cpu_percent(),
            "system_memory_percent": psutil.virtual_memory().percent
        }
    
    def run_rust_analysis(self, project_path: str) -> Dict[str, Any]:
        """Run Rust analysis on the test project and measure performance."""
        print(f"🔍 Running analysis on {project_path}...")
        
        start_resources = self.measure_system_resources()
        start_time = time.time()
        
        try:
            # Change to project directory
            original_cwd = os.getcwd()
            os.chdir(project_path)
            
            # Run cargo check to parse and analyze the project
            result = subprocess.run(
                ["cargo", "check", "--message-format=json"],
                capture_output=True,
                text=True,
                timeout=300  # 5 minute timeout
            )
            
            end_time = time.time()
            end_resources = self.measure_system_resources()
            
            execution_time_ms = (end_time - start_time) * 1000
            
            # Calculate resource usage delta
            cpu_delta = max(0, end_resources["cpu_percent"] - start_resources["cpu_percent"])
            memory_delta = max(0, end_resources["memory_mb"] - start_resources["memory_mb"])
            
            analysis_result = {
                "success": result.returncode == 0,
                "execution_time_ms": execution_time_ms,
                "execution_time_s": end_time - start_time,
                "cpu_usage_percent": cpu_delta,
                "memory_usage_mb": memory_delta,
                "exit_code": result.returncode,
                "stdout_lines": len(result.stdout.split('\\n')) if result.stdout else 0,
                "stderr_lines": len(result.stderr.split('\\n')) if result.stderr else 0
            }
            
            if result.returncode != 0:
                analysis_result["error"] = result.stderr[:500]  # Truncate error
            
            return analysis_result
            
        except subprocess.TimeoutExpired:
            return {
                "success": False,
                "execution_time_ms": 300000,  # 5 minutes
                "execution_time_s": 300,
                "error": "Analysis timed out after 5 minutes",
                "cpu_usage_percent": 0,
                "memory_usage_mb": 0
            }
        except Exception as e:
            return {
                "success": False,
                "execution_time_ms": 0,
                "execution_time_s": 0,
                "error": str(e),
                "cpu_usage_percent": 0,
                "memory_usage_mb": 0
            }
        finally:
            os.chdir(original_cwd)
    
    def run_javascript_analysis(self, file_count: int) -> Dict[str, Any]:
        """Run JavaScript rendering service analysis with load proportional to file count."""
        print(f"🎨 Running rendering service analysis for {file_count} file simulation...")
        
        # Calculate load based on file count
        # More files = more complex diagrams to render
        iterations = min(50, max(10, file_count // 100))
        concurrent_requests = min(20, max(5, file_count // 500))
        
        try:
            # Check if rendering service is available
            import requests
            health_response = requests.get("http://localhost:3001/health", timeout=5)
            if health_response.status_code != 200:
                return {
                    "success": False,
                    "error": "Rendering service not available",
                    "execution_time_ms": 0,
                    "cpu_usage_percent": 0,
                    "memory_usage_mb": 0
                }
        except Exception:
            return {
                "success": False,
                "error": "Could not connect to rendering service",
                "execution_time_ms": 0,
                "cpu_usage_percent": 0,
                "memory_usage_mb": 0
            }
        
        # Generate complex diagram proportional to file count
        complexity_factor = min(10, file_count // 1000 + 1)
        diagram_content = "graph TD\\n"
        
        for i in range(complexity_factor * 10):
            diagram_content += f"    A{i}[Node {i}] --> B{i}[Process {i}]\\n"
            diagram_content += f"    B{i} --> C{i}[Result {i}]\\n"
        
        start_resources = self.measure_system_resources()
        start_time = time.time()
        
        try:
            import asyncio
            import aiohttp
            
            async def render_diagram():
                async with aiohttp.ClientSession() as session:
                    tasks = []
                    for _ in range(concurrent_requests):
                        task = session.post(
                            "http://localhost:3001/render",
                            json={
                                "mermaid_code": diagram_content,
                                "format": "svg",
                                "width": 1200,
                                "height": 800
                            }
                        )
                        tasks.append(task)
                    
                    responses = await asyncio.gather(*tasks, return_exceptions=True)
                    return responses
            
            # Run the async rendering
            loop = asyncio.new_event_loop()
            asyncio.set_event_loop(loop)
            responses = loop.run_until_complete(render_diagram())
            loop.close()
            
            end_time = time.time()
            end_resources = self.measure_system_resources()
            
            execution_time_ms = (end_time - start_time) * 1000
            cpu_delta = max(0, end_resources["cpu_percent"] - start_resources["cpu_percent"])
            memory_delta = max(0, end_resources["memory_mb"] - start_resources["memory_mb"])
            
            successful_responses = sum(1 for r in responses if not isinstance(r, Exception))
            
            return {
                "success": successful_responses > 0,
                "execution_time_ms": execution_time_ms,
                "execution_time_s": end_time - start_time,
                "cpu_usage_percent": cpu_delta,
                "memory_usage_mb": memory_delta,
                "successful_requests": successful_responses,
                "total_requests": len(responses),
                "success_rate": (successful_responses / len(responses)) * 100 if responses else 0
            }
            
        except Exception as e:
            return {
                "success": False,
                "error": str(e),
                "execution_time_ms": 0,
                "cpu_usage_percent": 0,
                "memory_usage_mb": 0
            }
    
    def evaluate_performance(self, file_count: int, rust_result: Dict[str, Any], js_result: Dict[str, Any]) -> Dict[str, Any]:
        """Evaluate performance against expectations and assign grades."""
        expectations = self.expectations.get(file_count, self.expectations[10000])
        
        evaluation = {
            "file_count": file_count,
            "expectations": expectations,
            "rust_performance": {},
            "js_performance": {},
            "overall_grade": "F",
            "meets_expectations": False
        }
        
        # Evaluate Rust performance
        if rust_result.get("success", False):
            rust_time_ok = rust_result["execution_time_ms"] <= expectations["max_time_ms"]
            rust_memory_ok = rust_result["memory_usage_mb"] <= expectations["max_memory_mb"]
            rust_cpu_ok = rust_result["cpu_usage_percent"] <= expectations["max_cpu_percent"]
            
            evaluation["rust_performance"] = {
                "execution_time_ok": rust_time_ok,
                "memory_usage_ok": rust_memory_ok,
                "cpu_usage_ok": rust_cpu_ok,
                "grade": "A" if all([rust_time_ok, rust_memory_ok, rust_cpu_ok]) else ("B" if sum([rust_time_ok, rust_memory_ok, rust_cpu_ok]) >= 2 else "C")
            }
        else:
            evaluation["rust_performance"] = {
                "execution_time_ok": False,
                "memory_usage_ok": False,
                "cpu_usage_ok": False,
                "grade": "F"
            }
        
        # Evaluate JavaScript performance
        if js_result.get("success", False):
            js_time_ok = js_result["execution_time_ms"] <= expectations["max_time_ms"] * 2  # More lenient for JS
            js_memory_ok = js_result["memory_usage_mb"] <= expectations["max_memory_mb"]
            js_success_ok = js_result.get("success_rate", 0) >= 90
            
            evaluation["js_performance"] = {
                "execution_time_ok": js_time_ok,
                "memory_usage_ok": js_memory_ok,
                "success_rate_ok": js_success_ok,
                "grade": "A" if all([js_time_ok, js_memory_ok, js_success_ok]) else ("B" if sum([js_time_ok, js_memory_ok, js_success_ok]) >= 2 else "C")
            }
        else:
            evaluation["js_performance"] = {
                "execution_time_ok": False,
                "memory_usage_ok": False,
                "success_rate_ok": False,
                "grade": "F"
            }
        
        # Calculate overall grade
        rust_grade = evaluation["rust_performance"]["grade"]
        js_grade = evaluation["js_performance"]["grade"]
        
        grade_scores = {"A": 4, "B": 3, "C": 2, "D": 1, "F": 0}
        avg_score = (grade_scores[rust_grade] + grade_scores[js_grade]) / 2
        
        if avg_score >= 3.5:
            evaluation["overall_grade"] = "A"
        elif avg_score >= 2.5:
            evaluation["overall_grade"] = "B"
        elif avg_score >= 1.5:
            evaluation["overall_grade"] = "C"
        elif avg_score >= 0.5:
            evaluation["overall_grade"] = "D"
        else:
            evaluation["overall_grade"] = "F"
        
        evaluation["meets_expectations"] = evaluation["overall_grade"] in ["A", "B"]
        
        return evaluation
    
    def run_scalability_tests(self) -> Dict[str, Any]:
        """Run complete scalability test suite."""
        print("🚀 Starting UV-60 Scalability Testing...")
        
        test_results = {
            "timestamp": datetime.utcnow().isoformat(),
            "test_sizes": self.test_sizes,
            "results": {},
            "summary": {
                "total_tests": len(self.test_sizes),
                "passed_tests": 0,
                "failed_tests": 0,
                "overall_grade": "F"
            }
        }
        
        for file_count in self.test_sizes:
            print(f"\\n📊 Testing with {file_count} files...")
            
            # Create test project
            project_path = self.create_test_project(file_count)
            
            # Run Rust analysis
            rust_result = self.run_rust_analysis(project_path)
            
            # Run JavaScript analysis
            js_result = self.run_javascript_analysis(file_count)
            
            # Evaluate performance
            evaluation = self.evaluate_performance(file_count, rust_result, js_result)
            
            test_results["results"][str(file_count)] = {
                "rust_analysis": rust_result,
                "js_analysis": js_result,
                "evaluation": evaluation
            }
            
            # Update summary
            if evaluation["meets_expectations"]:
                test_results["summary"]["passed_tests"] += 1
            else:
                test_results["summary"]["failed_tests"] += 1
            
            # Print results
            print(f"   Rust: {rust_result['execution_time_ms']:.1f}ms, Grade: {evaluation['rust_performance']['grade']}")
            print(f"   JS: {js_result['execution_time_ms']:.1f}ms, Grade: {evaluation['js_performance']['grade']}")
            print(f"   Overall: {evaluation['overall_grade']} ({'PASS' if evaluation['meets_expectations'] else 'FAIL'})")
        
        # Calculate overall grade
        passed_ratio = test_results["summary"]["passed_tests"] / test_results["summary"]["total_tests"]
        if passed_ratio >= 0.9:
            test_results["summary"]["overall_grade"] = "A"
        elif passed_ratio >= 0.7:
            test_results["summary"]["overall_grade"] = "B"
        elif passed_ratio >= 0.5:
            test_results["summary"]["overall_grade"] = "C"
        else:
            test_results["summary"]["overall_grade"] = "F"
        
        # Save results
        results_file = os.path.join(self.output_dir, "scalability-test-results.json")
        with open(results_file, "w") as f:
            json.dump(test_results, f, indent=2)
        
        print(f"\\n✅ Scalability testing complete:")
        print(f"   Overall Grade: {test_results['summary']['overall_grade']}")
        print(f"   Passed: {test_results['summary']['passed_tests']}/{test_results['summary']['total_tests']}")
        print(f"   Results saved: {results_file}")
        
        return test_results
    
    def cleanup(self):
        """Clean up temporary test projects."""
        for file_count, temp_dir in self.temp_projects.items():
            try:
                shutil.rmtree(temp_dir)
                print(f"🗑️  Cleaned up {file_count}-file test project")
            except Exception as e:
                print(f"⚠️  Could not clean up {temp_dir}: {e}")

def main():
    """Main entry point for scalability testing."""
    tester = ScalabilityTester()
    
    try:
        results = tester.run_scalability_tests()
        success = results["summary"]["overall_grade"] in ["A", "B", "C"]
        sys.exit(0 if success else 1)
    except KeyboardInterrupt:
        print("\\n⚠️  Scalability testing interrupted")
        sys.exit(1)
    except Exception as e:
        print(f"❌ Scalability testing failed: {e}")
        sys.exit(1)
    finally:
        tester.cleanup()

if __name__ == "__main__":
    main()