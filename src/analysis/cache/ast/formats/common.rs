use super::{javascript, python, rust};
use std::path::Path;

pub(crate) fn detect_language(path: &Path) -> String {
    let Some(extension) = path.extension().and_then(|ext| ext.to_str()) else {
        return "unknown".to_string();
    };

    let normalized = extension.to_lowercase();
    let extension_str = normalized.as_str();

    if rust::EXTENSIONS.contains(&extension_str) {
        return rust::LANGUAGE_NAME.to_string();
    }

    if python::EXTENSIONS.contains(&extension_str) {
        return python::LANGUAGE_NAME.to_string();
    }

    if javascript::EXTENSIONS.contains(&extension_str) {
        return extension_str.to_string();
    }

    normalized
}
