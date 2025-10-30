#!/usr/bin/env python3
"""
Comprehensive Uveddi CLI Test Suite
Tests all CLI commands and options
Created: 2025-10-03
"""

import subprocess
import sys
import os
import tempfile
import shutil
from pathlib import Path

# ANSI colors
GREEN = '\033[0;32m'
RED = '\033[0;31m'
YELLOW = '\033[1;33m'
BLUE = '\033[0;34m'
NC = '\033[0m'

class CLITester:
    def __init__(self, binary_path):
        self.binary = binary_path
        self.passed = 0
        self.failed = 0
        self.skipped = 0
        self.test_dir = Path(tempfile.mkdtemp(prefix='uveddi-test-'))
        # Suppress log noise
        self.env = os.environ.copy()
        self.env['RUST_LOG'] = 'error'

    def test(self, name, args, should_fail=False, timeout=10):
        """Run a single test"""
        cmd = [self.binary] + args
        print(f"{BLUE}ℹ️  INFO:{NC} Running: {name}")

        try:
            result = subprocess.run(
                cmd,
                capture_output=True,
                timeout=timeout,
                env=self.env
            )
            success = (result.returncode == 0)

            if should_fail:
                if not success:
                    print(f"{GREEN}✅ PASS:{NC} {name} (failed as expected)")
                    self.passed += 1
                else:
                    print(f"{RED}❌ FAIL:{NC} {name} (should have failed)")
                    self.failed += 1
            else:
                if success:
                    print(f"{GREEN}✅ PASS:{NC} {name}")
                    self.passed += 1
                else:
                    print(f"{RED}❌ FAIL:{NC} {name}")
                    print(f"    Exit code: {result.returncode}")
                    if result.stderr:
                        print(f"    Stderr: {result.stderr.decode()[:200]}")
                    self.failed += 1

        except subprocess.TimeoutExpired:
            print(f"{RED}❌ FAIL:{NC} {name} (timeout)")
            self.failed += 1
        except Exception as e:
            print(f"{RED}❌ FAIL:{NC} {name} ({str(e)})")
            self.failed += 1

    def cleanup(self):
        """Clean up test directory"""
        if self.test_dir.exists():
            shutil.rmtree(self.test_dir)

    def summary(self):
        """Print test summary"""
        print()
        print("━" * 60)
        print("📋 Test Summary")
        print("━" * 60)
        print(f"{GREEN}✅ Passed:  {self.passed}{NC}")
        print(f"{RED}❌ Failed:  {self.failed}{NC}")
        print(f"{YELLOW}⏭️  Skipped: {self.skipped}{NC}")

        total = self.passed + self.failed + self.skipped
        if total > 0:
            rate = (self.passed * 100) // total
            print(f"\nTotal: {total} | Pass Rate: {rate}%")
        print()

        if self.failed == 0:
            print(f"{GREEN}🎉 All tests passed!{NC}")
            return 0
        else:
            print(f"{RED}⚠️  Some tests failed{NC}")
            return 1

def main():
    # Find binary
    binary = os.environ.get('UVEDDI_BIN', './target/debug/uveddi')
    if not os.path.exists(binary):
        print(f"{RED}Error: Binary not found at {binary}{NC}")
        print("Build first: cargo build --bin uveddi --features standard")
        return 1

    tester = CLITester(binary)

    print("🧪 Uveddi CLI Test Suite")
    print("=" * 60)
    print(f"Binary: {binary}")
    print(f"Test Dir: {tester.test_dir}")
    print()

    try:
        # ===================================================================
        # SECTION 1: Basic CLI Options
        # ===================================================================
        print("━" * 60)
        print("SECTION 1: Basic CLI Options")
        print("━" * 60)

        tester.test("Version (--version)", ["--version"])
        tester.test("Version (-V)", ["-V"])
        tester.test("Help (--help)", ["--help"])
        tester.test("Help (-h)", ["-h"])

        # ===================================================================
        # SECTION 2: Analyze Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 2: Analyze Command")
        print("━" * 60)

        # Create test project
        project_dir = tester.test_dir / "project"
        project_dir.mkdir()
        (project_dir / "main.rs").write_text('''
fn main() {
    println!("test");
}
        ''')

        tester.test("Analyze --help", ["analyze", "--help"])
        tester.test("Analyze alias (a --help)", ["a", "--help"])
        tester.test("Analyze basic", ["analyze", str(project_dir), "--output-format", "json"], timeout=30)
        tester.test("Analyze with Markdown", ["analyze", str(project_dir), "--output-format", "markdown"], timeout=30)
        tester.test("Analyze with timeout", ["analyze", str(project_dir), "--timeout", "5"], timeout=15)
        tester.test("Analyze non-existent", ["analyze", "/nonexistent"], should_fail=True)

        # ===================================================================
        # SECTION 3: Config Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 3: Config Command")
        print("━" * 60)

        tester.test("Config --help", ["config", "--help"])
        tester.test("Config alias (cfg --help)", ["cfg", "--help"])
        tester.test("Config show", ["config", "show"])

        # ===================================================================
        # SECTION 4: Doctor Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 4: Doctor Command")
        print("━" * 60)

        tester.test("Doctor --help", ["doctor", "--help"])
        tester.test("Doctor alias (dr --help)", ["dr", "--help"])
        tester.test("Doctor check", ["doctor"])

        # ===================================================================
        # SECTION 5: Help Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 5: Help Command")
        print("━" * 60)

        tester.test("Help command", ["help"])
        tester.test("Help analyze", ["help", "analyze"])
        tester.test("Help config", ["help", "config"])

        # ===================================================================
        # SECTION 6: Hooks Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 6: Hooks Command")
        print("━" * 60)

        tester.test("Hooks --help", ["hooks", "--help"])

        # Create git repo
        git_repo = tester.test_dir / "git-repo"
        git_repo.mkdir()
        subprocess.run(["git", "init"], cwd=git_repo, capture_output=True, env=tester.env)

        # Test hooks list (requires being in a git repo)
        original_cwd = os.getcwd()
        os.chdir(git_repo)
        tester.test("Hooks list", ["hooks", "list"])
        os.chdir(original_cwd)

        # ===================================================================
        # SECTION 7: Init Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 7: Init Command")
        print("━" * 60)

        tester.test("Init --help", ["init", "--help"])

        init_dir = tester.test_dir / "init-test"
        init_dir.mkdir()
        os.chdir(init_dir)
        tester.test("Init non-interactive", ["init", "--non-interactive"])
        os.chdir(original_cwd)

        # ===================================================================
        # SECTION 8: UI Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 8: UI Command")
        print("━" * 60)

        tester.test("UI --help", ["ui", "--help"])

        # ===================================================================
        # SECTION 9: CI Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 9: CI Command")
        print("━" * 60)

        tester.test("CI --help", ["ci", "--help"])

        # ===================================================================
        # SECTION 10: TUI Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 10: TUI Command")
        print("━" * 60)

        tester.test("TUI --help", ["tui", "--help"])

        # ===================================================================
        # SECTION 11: Serve Command
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 11: Serve Command")
        print("━" * 60)

        tester.test("Serve --help", ["serve", "--help"])

        # ===================================================================
        # SECTION 12: Error Handling
        # ===================================================================
        print()
        print("━" * 60)
        print("SECTION 12: Error Handling")
        print("━" * 60)

        tester.test("Invalid command", ["invalid-command"], should_fail=True)
        tester.test("Analyze invalid format", ["analyze", str(project_dir), "--output-format", "invalid"], should_fail=True, timeout=15)

    finally:
        tester.cleanup()

    return tester.summary()

if __name__ == "__main__":
    sys.exit(main())
