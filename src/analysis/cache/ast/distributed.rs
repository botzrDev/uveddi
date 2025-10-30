use std::path::Path;

/// Placeholder distributed cache coordinator hooks.
#[allow(dead_code)]
pub(crate) fn notify_entry_stored(path: &Path) {
    tracing::trace!(file = %path.display(), "Distributed cache notified of store");
}

#[allow(dead_code)]
pub(crate) fn notify_entry_invalidated(path: &Path) {
    tracing::trace!(file = %path.display(), "Distributed cache notified of invalidation");
}
