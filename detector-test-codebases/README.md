# Detector Test Codebases for Uveddi

This folder contains extreme test codebases designed to stress Uveddi's detectors.

Structure:
- rust-test-cases/
- python-test-cases/
- javascript-test-cases/
- typescript-test-cases/
- mixed-language-project/
- benchmark-codebases/

Each language folder contains categorized test cases (god_objects, dead_code, etc.).

Usage:
- Add more test files according to the categories.
- Run Uveddi analysis against this folder to validate detectors.

Notes:
- Files here intentionally contain anti-patterns, huge files, and tricky constructs.
- Keep performance tests gated or compressed if you don't want to commit massive files to git.

Generator
---------
An automated generator is available at `detector-test-codebases/generator/generate.py` to produce large extreme test files for Rust, Python, and TypeScript.

Quick example (from repository root):

```bash
python3 detector-test-codebases/generator/generate.py --lang rust --type god_object --fields 75 --methods 150 --outdir detector-test-codebases/benchmark-codebases/generated_sample
```

To clone real GitHub codebases for detector testing, use the simple helper `detector-test-codebases/generator/clone_repos.sh` with a repo list file (see `repos.example.txt`).
