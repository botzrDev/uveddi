#![cfg(feature = "memory-optimization")]

use crate::analysis::memory::zero_copy::{SerializableAst, ZeroCopyAstCache};
use crate::ast::tree_sitter::ParsedFile;
use std::path::Path;
use tracing::{debug, warn};

pub(crate) fn try_store_zero_copy(
    zero_copy_cache: Option<&ZeroCopyAstCache>,
    path: &Path,
    parsed_file: Option<&ParsedFile>,
    threshold: usize,
) {
    let Some(zero_copy_cache) = zero_copy_cache else {
        return;
    };

    let Some(parsed_file) = parsed_file else {
        debug!(
            file = %path.display(),
            "Parsed file not provided, skipping zero-copy cache storage"
        );
        return;
    };

    if parsed_file.source.len() < threshold {
        debug!(
            file = %path.display(),
            size = parsed_file.source.len(),
            threshold,
            "Skipping zero-copy storage due to threshold"
        );
        return;
    }

    match SerializableAst::from_parsed_file(parsed_file) {
        Ok(serializable_ast) => {
            if let Err(error) = zero_copy_cache.store(path, &serializable_ast) {
                warn!(file = %path.display(), %error, "Failed to store in zero-copy cache");
            } else {
                debug!(file = %path.display(), "Stored AST in zero-copy cache");
            }
        }
        Err(error) => {
            warn!(file = %path.display(), %error, "Failed to create serializable AST");
        }
    }
}

pub(crate) fn clear_zero_copy_cache(zero_copy_cache: Option<&ZeroCopyAstCache>) {
    if let Some(zero_copy_cache) = zero_copy_cache {
        if let Err(error) = zero_copy_cache.clear() {
            tracing::warn!(%error, "Failed to clear zero-copy cache");
        }
    }
}
