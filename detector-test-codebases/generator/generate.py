#!/usr/bin/env python3
"""Generate extreme test files for multiple languages.
Usage examples:
  python3 generate.py --lang rust --type god_object --fields 50 --methods 150 --outdir ../benchmark-codebases/generated_sample
  python3 generate.py --lang python --type god_object --attrs 50 --methods 100 --outdir ../benchmark-codebases/generated_sample

It can also read a repos file and optionally clone them (requires git installed):
  bash clone_repos.sh repos.txt output_dir

This script is intentionally conservative when writing files; use outdir inside detector-test-codebases and add generated artifacts to .gitignore.
"""
import argparse
import os
import textwrap


def ensure_dir(path):
    os.makedirs(path, exist_ok=True)


def generate_rust_god_object(path, name, fields, methods):
    ensure_dir(path)
    fname = os.path.join(path, f"{name}.rs")
    with open(fname, 'w') as f:
        f.write('// Generated Rust god object\n')
        f.write(f'pub struct {name} {{\n')
        for i in range(1, fields+1):
            f.write(f'    pub field_{i:03}: usize,\n')
        f.write('}\n\n')
        f.write(f"impl {name} {{\n")
        f.write(f"    pub fn new() -> Self {{\n")
        f.write(f"        Self {{\n")
        for i in range(1, fields+1):
            f.write(f"            field_{i:03}: 0,\n")
        f.write("        }\n    }\n\n")
        # methods
        for i in range(1, methods+1):
            # keep methods trivial to keep compile time low
            idx = (i - 1) % fields + 1
            f.write(f"    pub fn method_{i:03}(&self) -> usize {{ self.field_{idx:03} }}\n")
        f.write('}\n')
    print('Wrote', fname)


def generate_python_god_object(path, name, attrs, methods):
    ensure_dir(path)
    fname = os.path.join(path, f"{name}.py")
    with open(fname, 'w') as f:
        f.write('# Generated Python god object\n')
        f.write(f'class {name}:\n')
        f.write('    def __init__(self):\n')
        for i in range(1, attrs+1):
            f.write(f'        self.attr_{i:03} = None\n')
        f.write('\n')
        for i in range(1, methods+1):
            idx = (i - 1) % attrs + 1
            f.write(f'    def method_{i:03}(self):\n')
            f.write(f'        return self.attr_{idx:03}\n\n')
    print('Wrote', fname)


def generate_ts_god_object(path, name, props, methods):
    ensure_dir(path)
    fname = os.path.join(path, f"{name}.ts")
    with open(fname, 'w') as f:
        f.write('// Generated TypeScript god object\n')
        f.write(f'export class {name} {{\n')
        f.write('    constructor() {\n')
        for i in range(1, props+1):
            f.write(f"        (this as any)['prop_{i:03}'] = null;\n")
        f.write('    }\n\n')
        for i in range(1, methods+1):
            idx = (i - 1) % props + 1
            f.write(f'    async method_{i:03}() {{ return (this as any)[\'prop_{idx:03}\']; }}\n')
        f.write('}\n')
    print('Wrote', fname)


def main():
    p = argparse.ArgumentParser()
    p.add_argument('--lang', required=True, choices=['rust','python','ts'])
    p.add_argument('--type', default='god_object', choices=['god_object'])
    p.add_argument('--fields', type=int, default=50, help='for rust: number of fields')
    p.add_argument('--attrs', type=int, default=50, help='for python: number of attributes')
    p.add_argument('--props', type=int, default=40, help='for ts: number of properties')
    p.add_argument('--methods', type=int, default=100)
    p.add_argument('--outdir', required=True)
    p.add_argument('--name', default='GeneratedGod')
    args = p.parse_args()

    outdir = os.path.abspath(args.outdir)
    if args.lang == 'rust':
        generate_rust_god_object(outdir, args.name, args.fields, args.methods)
    elif args.lang == 'python':
        generate_python_god_object(outdir, args.name, args.attrs, args.methods)
    elif args.lang == 'ts':
        generate_ts_god_object(outdir, args.name, args.props, args.methods)

if __name__ == '__main__':
    main()
