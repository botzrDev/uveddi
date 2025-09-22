pub mod rust;
pub mod python;
pub mod javascript;

pub use rust::{RustDependencyParser, CargoManifest};
pub use python::{PythonDependencyParser, PythonRequirement};
pub use javascript::{JavaScriptDependencyParser, PackageJson};

use crate::analysis::detectors::dependency::types::*;
use std::path::Path;

pub trait LanguageDependencyParser: Send + Sync {
    fn parse_manifest(
        &self,
        manifest_path: &Path,
    ) -> Result<Vec<DependencyInfo>, DependencyError>;

    fn parse_lockfile(
        &self,
        lockfile_path: &Path,
    ) -> Result<Vec<DependencyInfo>, DependencyError>;

    fn supported_manifests(&self) -> Vec<&'static str>;
    fn supported_lockfiles(&self) -> Vec<&'static str>;
    fn language_name(&self) -> &'static str;
}

pub struct LanguageParserRegistry {
    parsers: Vec<Box<dyn LanguageDependencyParser>>,
}

impl LanguageParserRegistry {
    pub fn new() -> Self {
        Self {
            parsers: vec![
                Box::new(RustDependencyParser::new()),
                Box::new(PythonDependencyParser::new()),
                Box::new(JavaScriptDependencyParser::new()),
            ],
        }
    }

    pub fn get_parser(&self, file_path: &Path) -> Option<&dyn LanguageDependencyParser> {
        let filename = file_path.file_name()?.to_str()?;

        for parser in &self.parsers {
            if parser.supported_manifests().contains(&filename) ||
               parser.supported_lockfiles().contains(&filename) {
                return Some(parser.as_ref());
            }
        }
        None
    }

    pub fn parse_all_manifests(
        &self,
        project_root: &Path,
    ) -> Result<Vec<DependencyInfo>, DependencyError> {
        let mut all_dependencies = Vec::new();

        for parser in &self.parsers {
            for manifest in parser.supported_manifests() {
                let manifest_path = project_root.join(manifest);
                if manifest_path.exists() {
                    match parser.parse_manifest(&manifest_path) {
                        Ok(mut deps) => all_dependencies.append(&mut deps),
                        Err(_) => continue, // Skip errors for optional manifests
                    }
                }
            }
        }

        Ok(all_dependencies)
    }
}

impl Default for LanguageParserRegistry {
    fn default() -> Self {
        Self::new()
    }
}