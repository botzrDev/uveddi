//! TypeScript/JavaScript sink patterns

use crate::analysis::detectors::security::taint_analysis::types::TaintSink;
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::SourceLanguage;

/// Get TypeScript/JavaScript-specific taint sinks
pub fn get_typescript_sinks() -> Vec<TaintSink> {
    let language = SourceLanguage::TypeScript;

    vec![
        // DOM manipulation (XSS)
        TaintSink::new(
            "js_inner_html".to_string(),
            "innerHTML".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "DOM innerHTML manipulation".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_outer_html".to_string(),
            "outerHTML".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "DOM outerHTML manipulation".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_document_write".to_string(),
            "document.write".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Document write operation".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_insertadjacenthtml".to_string(),
            "insertAdjacentHTML".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Adjacent HTML insertion".to_string(),
        )
        .with_language(language),
        // Code execution
        TaintSink::new(
            "js_eval".to_string(),
            "eval(".to_string(),
            SecurityIssueType::Injection,
            "JavaScript eval execution".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_function_constructor".to_string(),
            "Function(".to_string(),
            SecurityIssueType::Injection,
            "Function constructor".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_set_timeout".to_string(),
            "setTimeout(".to_string(),
            SecurityIssueType::Injection,
            "setTimeout with code string".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_set_interval".to_string(),
            "setInterval(".to_string(),
            SecurityIssueType::Injection,
            "setInterval with code string".to_string(),
        )
        .with_language(language),
        // Node.js command execution
        TaintSink::new(
            "js_child_exec".to_string(),
            "child_process.exec".to_string(),
            SecurityIssueType::Injection,
            "Child process execution".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_child_spawn".to_string(),
            "child_process.spawn".to_string(),
            SecurityIssueType::Injection,
            "Child process spawn".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_child_exec_file".to_string(),
            "child_process.execFile".to_string(),
            SecurityIssueType::Injection,
            "Child process execFile".to_string(),
        )
        .with_language(language),
        // File operations (Node.js)
        TaintSink::new(
            "js_fs_write".to_string(),
            "fs.writeFileSync".to_string(),
            SecurityIssueType::PathTraversal,
            "File system write".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_fs_write_async".to_string(),
            "fs.writeFile".to_string(),
            SecurityIssueType::PathTraversal,
            "Async file system write".to_string(),
        )
        .with_language(language),
        // Network requests (SSRF)
        TaintSink::new(
            "js_fetch_request".to_string(),
            "fetch(".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "Fetch request with user URL".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_xhr_open".to_string(),
            "XMLHttpRequest.open".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "XMLHttpRequest to user URL".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_websocket_connect".to_string(),
            "new WebSocket(".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "WebSocket connection to user URL".to_string(),
        )
        .with_language(language),
        // Response manipulation
        TaintSink::new(
            "js_res_send".to_string(),
            "res.send".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Express response send".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_res_write".to_string(),
            "res.write".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Express response write".to_string(),
        )
        .with_language(language),
        // Database queries (if using raw queries)
        TaintSink::new(
            "js_sql_query".to_string(),
            "query(".to_string(),
            SecurityIssueType::Injection,
            "Database query execution".to_string(),
        )
        .with_language(language),
        // Dynamic module loading
        TaintSink::new(
            "js_require".to_string(),
            "require(".to_string(),
            SecurityIssueType::Injection,
            "Dynamic module require".to_string(),
        )
        .with_language(language),
        TaintSink::new(
            "js_import".to_string(),
            "import(".to_string(),
            SecurityIssueType::Injection,
            "Dynamic import".to_string(),
        )
        .with_language(language),
    ]
}
