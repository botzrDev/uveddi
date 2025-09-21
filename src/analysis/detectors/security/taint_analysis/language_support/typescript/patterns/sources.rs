//! TypeScript/JavaScript source patterns

use crate::analysis::detectors::security::taint_analysis::types::TaintSource;
use crate::ast::SourceLanguage;

/// Get TypeScript/JavaScript-specific taint sources
pub fn get_typescript_sources() -> Vec<TaintSource> {
    let language = SourceLanguage::TypeScript; // Used for both TS and JS

    vec![
        // Command line and environment (Node.js)
        TaintSource::new(
            "js_process_argv".to_string(),
            "process.argv".to_string(),
            "Command line arguments".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_process_env".to_string(),
            "process.env".to_string(),
            "Environment variables".to_string(),
        )
        .with_language(language),
        // Browser input sources
        TaintSource::new(
            "js_location_search".to_string(),
            "location.search".to_string(),
            "URL search parameters".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_location_hash".to_string(),
            "location.hash".to_string(),
            "URL hash fragment".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_document_cookie".to_string(),
            "document.cookie".to_string(),
            "Browser cookies".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_localstorage".to_string(),
            "localStorage.getItem".to_string(),
            "Local storage data".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_sessionstorage".to_string(),
            "sessionStorage.getItem".to_string(),
            "Session storage data".to_string(),
        )
        .with_language(language),
        // Form and user input
        TaintSource::new(
            "js_input_value".to_string(),
            "HTMLInputElement.value".to_string(),
            "Form input values".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_textarea_value".to_string(),
            "HTMLTextAreaElement.value".to_string(),
            "Textarea values".to_string(),
        )
        .with_language(language),
        // Network requests
        TaintSource::new(
            "js_fetch_response".to_string(),
            "fetch(".to_string(),
            "Fetch API response".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_xhr_response".to_string(),
            "XMLHttpRequest.responseText".to_string(),
            "XMLHttpRequest response".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_axios_response".to_string(),
            "axios.get".to_string(),
            "Axios HTTP response".to_string(),
        )
        .with_language(language),
        // Web framework sources (Express.js, etc.)
        TaintSource::new(
            "js_req_body".to_string(),
            "req.body".to_string(),
            "Express request body".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_req_query".to_string(),
            "req.query".to_string(),
            "Express query parameters".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_req_params".to_string(),
            "req.params".to_string(),
            "Express route parameters".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_req_headers".to_string(),
            "req.headers".to_string(),
            "Express request headers".to_string(),
        )
        .with_language(language),
        // File I/O (Node.js)
        TaintSource::new(
            "js_fs_read".to_string(),
            "fs.readFileSync".to_string(),
            "File system read".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_fs_read_async".to_string(),
            "fs.readFile".to_string(),
            "Async file system read".to_string(),
        )
        .with_language(language),
        // Message events
        TaintSource::new(
            "js_message_event".to_string(),
            "MessageEvent.data".to_string(),
            "PostMessage data".to_string(),
        )
        .with_language(language),
        TaintSource::new(
            "js_websocket_message".to_string(),
            "WebSocket.onmessage".to_string(),
            "WebSocket message data".to_string(),
        )
        .with_language(language),
    ]
}
