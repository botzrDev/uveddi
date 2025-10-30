#!/usr/bin/env python3
"""
Uveddi Plugin Generator

Generates a new plugin from template with customized configuration.
"""

import argparse
import json
import os
import shutil
import sys
from pathlib import Path
from typing import Dict, List, Any

def to_snake_case(name: str) -> str:
    """Convert name to snake_case."""
    import re
    name = re.sub('([a-z0-9])([A-Z])', r'\1_\2', name)
    return name.lower().replace('-', '_').replace(' ', '_')

def to_camel_case(name: str) -> str:
    """Convert name to CamelCase."""
    words = name.replace('-', '_').replace(' ', '_').split('_')
    return ''.join(word.capitalize() for word in words)

def to_kebab_case(name: str) -> str:
    """Convert name to kebab-case."""
    return to_snake_case(name).replace('_', '-')

def replace_template_variables(content: str, variables: Dict[str, Any]) -> str:
    """Replace Handlebars-style template variables in content."""
    import re
    
    # Simple variable replacement
    for key, value in variables.items():
        content = content.replace(f"{{{{{key}}}}}", str(value))
    
    # Handle arrays/lists with #each
    def replace_each(match):
        var_name = match.group(1)
        inner_content = match.group(2)
        
        if var_name in variables and isinstance(variables[var_name], list):
            result = []
            for i, item in enumerate(variables[var_name]):
                item_content = inner_content.replace("{{this}}", str(item))
                # Handle @last helper
                item_content = item_content.replace("{{#unless @last}}", "" if i == len(variables[var_name]) - 1 else "")
                item_content = item_content.replace("{{/unless}}", "" if i == len(variables[var_name]) - 1 else "")
                result.append(item_content)
            return ''.join(result)
        return ""
    
    content = re.sub(r'\{\{#each\s+(\w+)\}\}(.*?)\{\{/each\}\}', replace_each, content, flags=re.DOTALL)
    
    return content

def create_plugin(config: Dict[str, Any], output_dir: Path) -> None:
    """Create a new plugin from template."""
    template_dir = Path(__file__).parent.parent / "templates" / "plugin-template"
    
    if not template_dir.exists():
        print(f"Error: Template directory not found: {template_dir}")
        sys.exit(1)
    
    plugin_name = config["plugin_name"]
    plugin_dir = output_dir / plugin_name
    
    if plugin_dir.exists():
        response = input(f"Directory {plugin_dir} already exists. Overwrite? (y/N): ")
        if response.lower() != 'y':
            print("Cancelled.")
            return
        shutil.rmtree(plugin_dir)
    
    print(f"Creating plugin '{plugin_name}' in {plugin_dir}")
    
    # Prepare template variables
    variables = {
        **config,
        "plugin_struct": to_camel_case(plugin_name),
        "plugin_snake": to_snake_case(plugin_name),
        "plugin_kebab": to_kebab_case(plugin_name),
    }
    
    # Copy template files and replace variables
    for root, dirs, files in os.walk(template_dir):
        # Skip hidden directories
        dirs[:] = [d for d in dirs if not d.startswith('.')]
        
        rel_path = Path(root).relative_to(template_dir)
        dest_dir = plugin_dir / rel_path
        dest_dir.mkdir(parents=True, exist_ok=True)
        
        for file in files:
            if file.startswith('.'):
                continue
                
            src_file = Path(root) / file
            dest_file = dest_dir / file
            
            try:
                with open(src_file, 'r', encoding='utf-8') as f:
                    content = f.read()
                
                # Replace template variables
                content = replace_template_variables(content, variables)
                
                with open(dest_file, 'w', encoding='utf-8') as f:
                    f.write(content)
                    
                print(f"  Created: {dest_file}")
                
            except UnicodeDecodeError:
                # Binary file, just copy
                shutil.copy2(src_file, dest_file)
                print(f"  Copied: {dest_file}")
    
    print(f"\nPlugin '{plugin_name}' created successfully!")
    print(f"Next steps:")
    print(f"  cd {plugin_dir}")
    print(f"  cargo test")
    print(f"  wasm-pack build --target web")

def load_config_from_file(config_file: Path) -> Dict[str, Any]:
    """Load plugin configuration from JSON file."""
    try:
        with open(config_file, 'r', encoding='utf-8') as f:
            return json.load(f)
    except Exception as e:
        print(f"Error loading config file: {e}")
        sys.exit(1)

def interactive_config() -> Dict[str, Any]:
    """Create plugin configuration interactively."""
    print("Creating a new Uveddi plugin...")
    print()
    
    config = {}
    
    # Basic info
    config["plugin_name"] = input("Plugin name: ").strip()
    if not config["plugin_name"]:
        print("Error: Plugin name is required")
        sys.exit(1)
    
    config["version"] = input("Version [1.0.0]: ").strip() or "1.0.0"
    config["author"] = input("Author: ").strip()
    config["description"] = input("Description: ").strip()
    config["license"] = input("License [MIT]: ").strip() or "MIT"
    
    # Capabilities
    print("\nSupported languages (comma-separated, e.g., rust,python,javascript):")
    languages_input = input("Languages: ").strip()
    config["languages"] = [lang.strip() for lang in languages_input.split(",")] if languages_input else []
    
    print("\nDetector types (comma-separated, e.g., security,quality,performance):")
    detector_types_input = input("Detector types: ").strip()
    config["detector_types"] = [dt.strip() for dt in detector_types_input.split(",")] if detector_types_input else []
    
    # Permissions
    config["read_files"] = input("Allow reading files? (y/N): ").strip().lower() == 'y'
    config["write_files"] = input("Allow writing files? (y/N): ").strip().lower() == 'y'
    config["network_access"] = input("Allow network access? (y/N): ").strip().lower() == 'y'
    config["system_info"] = input("Allow system info access? (y/N): ").strip().lower() == 'y'
    
    # Resources
    config["max_memory_mb"] = int(input("Max memory (MB) [100]: ").strip() or "100")
    config["max_cpu_percent"] = int(input("Max CPU percent [50]: ").strip() or "50")
    config["timeout_seconds"] = int(input("Timeout (seconds) [30]: ").strip() or "30")
    
    # Custom dependencies
    print("\nCustom Cargo dependencies (one per line, empty to finish):")
    custom_deps = []
    while True:
        dep = input("Dependency: ").strip()
        if not dep:
            break
        custom_deps.append(dep)
    config["custom_dependencies"] = custom_deps
    
    # Features
    print("\nPlugin features (one per line, empty to finish):")
    features = []
    while True:
        feature = input("Feature: ").strip()
        if not feature:
            break
        features.append(feature)
    config["features"] = features
    
    return config

def main():
    parser = argparse.ArgumentParser(description="Generate a new Uveddi plugin from template")
    parser.add_argument("--config", "-c", type=Path, help="JSON configuration file")
    parser.add_argument("--output", "-o", type=Path, default=Path.cwd(), help="Output directory")
    parser.add_argument("--template", type=str, choices=["basic", "rule-engine", "ai-linter", "metrics"], 
                       default="basic", help="Plugin template type")
    
    args = parser.parse_args()
    
    if args.config:
        config = load_config_from_file(args.config)
    else:
        config = interactive_config()
    
    create_plugin(config, args.output)

if __name__ == "__main__":
    main()