//! Dependency extraction tests for all supported languages (Rust, Python, JS/TS)

#[cfg(test)]
mod tests {
    use std::fs::File;
    use std::io::Write;
    use tempfile::tempdir;
    use uveddi::analysis::DependencyExtractor;
    use uveddi::ast::tree_sitter_impl::AstParser;

    fn create_temp_file(dir: &tempfile::TempDir, name: &str, content: &str) -> std::path::PathBuf {
        let file_path = dir.path().join(name);
        let mut file = File::create(&file_path).unwrap();
        writeln!(file, "{content}").unwrap();
        file_path
    }

    #[test]
    fn rust_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "a.rs", "mod b;\nuse std::fs;\n");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert!(dep_names.contains(&"b"));
    }

    #[test]
    fn python_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "a.py", "import os\nimport sys\n");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert!(dep_names.contains(&"os"));
        assert!(dep_names.contains(&"sys"));
    }

    #[test]
    fn js_dependency_extraction_basic() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(
            &dir,
            "a.js",
            "import b from './b.js';\nimport fs from 'fs';\n",
        );
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert_eq!(dep_names.len(), 2);
        assert!(dep_names.contains(&"b")); // Normalized from "./b.js"
        assert!(dep_names.contains(&"fs"));
    }

    #[test]
    fn rust_complex_dependency_extraction() {
        let dir = tempdir().unwrap();
        let content = r#"
use std::collections::{HashMap, HashSet};
use serde::{Deserialize, Serialize};
pub use crate::utils::*;
extern crate regex;

mod internal_module;
pub mod public_module;

#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
}
"#;
        let file_path = create_temp_file(&dir, "complex.rs", content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();

        // Should extract standard library dependencies
        assert!(dep_names
            .iter()
            .any(|name| name.contains("HashMap") || name.contains("collections")));
        // Should extract external crate dependencies
        assert!(
            dep_names.contains(&"serde") || dep_names.iter().any(|name| name.contains("serde"))
        );
        // Should extract internal module dependencies
        assert!(dep_names.contains(&"internal_module") || dep_names.contains(&"public_module"));
    }

    #[test]
    fn python_complex_dependency_extraction() {
        let dir = tempdir().unwrap();
        let content = r#"
import os
import sys
from collections import defaultdict, Counter
from typing import List, Dict, Optional
from .local_module import LocalClass
from ..parent_module import ParentFunction
import numpy as np
import pandas as pd

try:
    import optional_package
except ImportError:
    optional_package = None

def function_with_late_import():
    import json
    return json.loads("{}")
"#;
        let file_path = create_temp_file(&dir, "complex.py", content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();

        // Standard library imports
        assert!(dep_names.contains(&"os"));
        assert!(dep_names.contains(&"sys"));
        // From imports - should capture parent modules
        assert!(dep_names.iter().any(|name| name.contains("collections")));
        // Relative imports
        assert!(dep_names
            .iter()
            .any(|name| name.contains("local_module") || name.contains("parent_module")));
        // External packages
        assert!(dep_names.contains(&"numpy") || dep_names.contains(&"np"));
        // Late imports inside functions
        assert!(dep_names.contains(&"json"));
    }

    #[test]
    fn javascript_complex_dependency_extraction() {
        let dir = tempdir().unwrap();
        let content = r#"
import React, { useState, useEffect } from 'react';
import { BrowserRouter as Router, Routes, Route } from 'react-router-dom';
import axios from 'axios';
import * as utils from './utils';
import { API_BASE_URL } from '../config/constants';
import type { User, ApiResponse } from '../types/api';

const fs = require('fs');
const path = require('path');

// Dynamic import
async function loadModule() {
    const module = await import('./dynamic-module');
    return module.default;
}

// CommonJS require in function
function processFile() {
    const lodash = require('lodash');
    return lodash.map([1, 2, 3], x => x * 2);
}
"#;
        let file_path = create_temp_file(&dir, "complex.js", content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();

        // ES6 imports
        assert!(dep_names.contains(&"react"));
        assert!(
            dep_names.contains(&"react-router-dom")
                || dep_names.iter().any(|name| name.contains("react-router"))
        );
        assert!(dep_names.contains(&"axios"));
        // Local imports
        assert!(dep_names.contains(&"utils"));
        assert!(dep_names
            .iter()
            .any(|name| name.contains("constants") || name.contains("config")));
        // CommonJS requires
        assert!(dep_names.contains(&"fs"));
        assert!(dep_names.contains(&"path"));
        // Function-scoped requires
        assert!(dep_names.contains(&"lodash"));
    }

    #[test]
    fn typescript_dependency_extraction() {
        let dir = tempdir().unwrap();
        let content = r#"
import { Injectable } from '@angular/core';
import { HttpClient, HttpHeaders } from '@angular/common/http';
import { Observable } from 'rxjs';
import { map, catchError } from 'rxjs/operators';
import type { UserProfile } from './types/user';
import { environment } from '../environments/environment';

@Injectable({
  providedIn: 'root'
})
export class UserService {
  constructor(private http: HttpClient) {}
}
"#;
        let file_path = create_temp_file(&dir, "service.ts", content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();

        // Angular dependencies
        assert!(dep_names
            .iter()
            .any(|name| name.contains("angular") || name.contains("core")));
        // RxJS dependencies
        assert!(dep_names.contains(&"rxjs"));
        // Local type imports
        assert!(dep_names
            .iter()
            .any(|name| name.contains("types") || name.contains("user")));
        // Environment imports
        assert!(dep_names.iter().any(|name| name.contains("environment")));
    }

    #[test]
    fn empty_file_dependency_extraction() {
        let dir = tempdir().unwrap();
        let file_path = create_temp_file(&dir, "empty.rs", "");
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        assert!(deps.is_empty(), "Empty file should have no dependencies");
    }

    #[test]
    fn malformed_syntax_dependency_extraction() {
        let dir = tempdir().unwrap();
        // Create a file with syntax errors
        let file_path = create_temp_file(
            &dir,
            "malformed.rs",
            "use std::; // Invalid syntax\nimport missing_semicolon",
        );
        let mut parser = AstParser::new().unwrap();

        // Parser should handle malformed syntax gracefully
        let parse_result = parser.parse_file(&file_path);
        if let Ok(parsed) = parse_result {
            let extractor = DependencyExtractor::new().unwrap();
            let deps_result = extractor.extract_from_ast(&parsed);
            // Should either succeed with partial extraction or fail gracefully
            match deps_result {
                Ok(deps) => {
                    // Partial extraction is acceptable
                    println!("Partial extraction yielded {} dependencies", deps.len());
                }
                Err(e) => {
                    // Graceful failure is also acceptable
                    println!("Graceful failure on malformed syntax: {}", e);
                }
            }
        }
        // Test passes if no panic occurs
    }

    #[test]
    fn circular_dependency_detection() {
        let dir = tempdir().unwrap();

        // Create file A that depends on B
        let file_a = create_temp_file(&dir, "a.rs", "mod b;\nuse b::SomeStruct;");

        // Create file B that depends on A (circular)
        let file_b = create_temp_file(&dir, "b.rs", "use crate::a::AnotherStruct;");

        let mut parser = AstParser::new().unwrap();
        let extractor = DependencyExtractor::new().unwrap();

        // Extract dependencies from both files
        let parsed_a = parser.parse_file(&file_a).unwrap();
        let deps_a = extractor.extract_from_ast(&parsed_a).unwrap();

        let parsed_b = parser.parse_file(&file_b).unwrap();
        let deps_b = extractor.extract_from_ast(&parsed_b).unwrap();

        // Verify that each file's dependencies are detected
        let deps_a_names: Vec<_> = deps_a.iter().map(|d| d.to_module.as_str()).collect();
        let deps_b_names: Vec<_> = deps_b.iter().map(|d| d.to_module.as_str()).collect();

        assert!(deps_a_names.contains(&"b"));
        assert!(deps_b_names.iter().any(|name| name.contains("a")));
    }

    #[test]
    fn large_file_dependency_extraction() {
        let dir = tempdir().unwrap();

        // Generate a large file with many dependencies
        let mut content = String::new();
        for i in 0..100 {
            content.push_str(&format!("use crate::module_{}::SomeStruct{};\n", i, i));
        }
        content.push_str("fn main() {}\n");

        let file_path = create_temp_file(&dir, "large.rs", &content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();

        // Should extract all 100 module dependencies
        assert!(
            deps.len() >= 100,
            "Should extract dependencies from large file, got {}",
            deps.len()
        );

        // Verify some specific dependencies
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();
        assert!(dep_names.iter().any(|name| name.contains("module_0")));
        assert!(dep_names.iter().any(|name| name.contains("module_99")));
    }

    #[test]
    fn dependency_extraction_with_comments() {
        let dir = tempdir().unwrap();
        let content = r#"
// This is a comment with import fake_module;
/* Block comment with
   use another_fake::Module; */
use std::fs; // Real import
// use commented_out::Module;
"#;
        let file_path = create_temp_file(&dir, "with_comments.rs", content);
        let mut parser = AstParser::new().unwrap();
        let parsed = parser.parse_file(&file_path).unwrap();
        let extractor = DependencyExtractor::new().unwrap();
        let deps = extractor.extract_from_ast(&parsed).unwrap();
        let dep_names: Vec<_> = deps.iter().map(|d| d.to_module.as_str()).collect();

        // Should only extract real imports, not commented ones
        assert!(dep_names.iter().any(|name| name.contains("fs")));
        assert!(!dep_names.contains(&"fake_module"));
        assert!(!dep_names.contains(&"another_fake"));
        assert!(!dep_names.contains(&"commented_out"));
    }
}
