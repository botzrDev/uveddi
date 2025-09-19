//! Python-specific taint patterns

use crate::analysis::detectors::security::taint_analysis::types::{
    TaintSource, TaintSink, SanitizationPoint
};
use crate::analysis::detectors::security::types::SecurityIssueType;
use crate::ast::SourceLanguage;

/// Get Python-specific taint sources
pub fn get_python_sources() -> Vec<TaintSource> {
    vec![
        // Command line and environment
        TaintSource::new(
            "python_sys_argv".to_string(),
            "sys.argv".to_string(),
            "Command line arguments".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_os_environ".to_string(),
            "os.environ".to_string(),
            "Environment variables".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_os_getenv".to_string(),
            "os.getenv".to_string(),
            "Single environment variable".to_string(),
        ).with_language(SourceLanguage::Python),

        // User input
        TaintSource::new(
            "python_input".to_string(),
            "input(".to_string(),
            "User input via input()".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_raw_input".to_string(),
            "raw_input(".to_string(),
            "User input via raw_input() (Python 2)".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_getpass".to_string(),
            "getpass.getpass".to_string(),
            "Password input".to_string(),
        ).with_language(SourceLanguage::Python),

        // File I/O
        TaintSource::new(
            "python_file_read".to_string(),
            "open(".to_string(),
            "File opening/reading".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_pathlib_read".to_string(),
            "pathlib.Path.read_text".to_string(),
            "Pathlib file reading".to_string(),
        ).with_language(SourceLanguage::Python),

        // Network requests
        TaintSource::new(
            "python_requests_get".to_string(),
            "requests.get".to_string(),
            "HTTP GET request".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_urllib_request".to_string(),
            "urllib.request.urlopen".to_string(),
            "urllib HTTP request".to_string(),
        ).with_language(SourceLanguage::Python),

        // Web frameworks
        TaintSource::new(
            "python_flask_request".to_string(),
            "flask.request".to_string(),
            "Flask request object".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_django_request".to_string(),
            "django.http.HttpRequest".to_string(),
            "Django request object".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_bottle_request".to_string(),
            "bottle.request".to_string(),
            "Bottle request object".to_string(),
        ).with_language(SourceLanguage::Python),

        // Database results
        TaintSource::new(
            "python_sqlite_fetchall".to_string(),
            "sqlite3.Cursor.fetchall".to_string(),
            "SQLite query results".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSource::new(
            "python_pymongo_find".to_string(),
            "pymongo.collection.Collection.find".to_string(),
            "MongoDB query results".to_string(),
        ).with_language(SourceLanguage::Python),
    ]
}

/// Get Python-specific taint sinks
pub fn get_python_sinks() -> Vec<TaintSink> {
    vec![
        // Code execution
        TaintSink::new(
            "python_eval".to_string(),
            "eval(".to_string(),
            SecurityIssueType::Injection,
            "Python eval with user input".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_exec".to_string(),
            "exec(".to_string(),
            SecurityIssueType::Injection,
            "Python exec with user input".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_compile".to_string(),
            "compile(".to_string(),
            SecurityIssueType::Injection,
            "Python compile with user code".to_string(),
        ).with_language(SourceLanguage::Python),

        // Command execution
        TaintSink::new(
            "python_os_system".to_string(),
            "os.system".to_string(),
            SecurityIssueType::Injection,
            "OS system command".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_subprocess_run".to_string(),
            "subprocess.run".to_string(),
            SecurityIssueType::Injection,
            "Subprocess execution".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_subprocess_call".to_string(),
            "subprocess.call".to_string(),
            SecurityIssueType::Injection,
            "Subprocess call".to_string(),
        ).with_language(SourceLanguage::Python),

        // SQL injection
        TaintSink::new(
            "python_cursor_execute".to_string(),
            "cursor.execute".to_string(),
            SecurityIssueType::Injection,
            "Database cursor execute".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_sqlite_execute".to_string(),
            "sqlite3.Connection.execute".to_string(),
            SecurityIssueType::Injection,
            "SQLite execute".to_string(),
        ).with_language(SourceLanguage::Python),

        // File operations
        TaintSink::new(
            "python_file_write".to_string(),
            "file.write".to_string(),
            SecurityIssueType::PathTraversal,
            "File write operation".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_pathlib_write".to_string(),
            "pathlib.Path.write_text".to_string(),
            SecurityIssueType::PathTraversal,
            "Pathlib file write".to_string(),
        ).with_language(SourceLanguage::Python),

        // Template injection
        TaintSink::new(
            "python_template_render".to_string(),
            "jinja2.Template".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Jinja2 template rendering".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_flask_render_string".to_string(),
            "flask.render_template_string".to_string(),
            SecurityIssueType::CrossSiteScripting,
            "Flask template string rendering".to_string(),
        ).with_language(SourceLanguage::Python),

        // Dynamic imports
        TaintSink::new(
            "python_import".to_string(),
            "__import__".to_string(),
            SecurityIssueType::Injection,
            "Dynamic module import".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_importlib".to_string(),
            "importlib.import_module".to_string(),
            SecurityIssueType::Injection,
            "Importlib dynamic import".to_string(),
        ).with_language(SourceLanguage::Python),

        // Network requests (SSRF)
        TaintSink::new(
            "python_requests_request".to_string(),
            "requests.request".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "HTTP request with user URL".to_string(),
        ).with_language(SourceLanguage::Python),
        TaintSink::new(
            "python_urllib_urlopen".to_string(),
            "urllib.request.urlopen".to_string(),
            SecurityIssueType::ServerSideRequestForgery,
            "urllib URL opening".to_string(),
        ).with_language(SourceLanguage::Python),
    ]
}

/// Get Python-specific sanitizers
pub fn get_python_sanitizers() -> Vec<SanitizationPoint> {
    vec![
        // HTML/XSS prevention
        SanitizationPoint::new(
            "python_html_escape".to_string(),
            "html.escape".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.90),
        SanitizationPoint::new(
            "python_markupsafe_escape".to_string(),
            "markupsafe.escape".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.90),
        SanitizationPoint::new(
            "python_bleach_clean".to_string(),
            "bleach.clean".to_string(),
            vec![SecurityIssueType::CrossSiteScripting],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.95),

        // Command sanitization
        SanitizationPoint::new(
            "python_shlex_quote".to_string(),
            "shlex.quote".to_string(),
            vec![SecurityIssueType::Injection],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.85),

        // Path sanitization
        SanitizationPoint::new(
            "python_os_path_abspath".to_string(),
            "os.path.abspath".to_string(),
            vec![SecurityIssueType::PathTraversal],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.70),
        SanitizationPoint::new(
            "python_pathlib_resolve".to_string(),
            "pathlib.Path.resolve".to_string(),
            vec![SecurityIssueType::PathTraversal],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.80),

        // URL validation
        SanitizationPoint::new(
            "python_urllib_parse".to_string(),
            "urllib.parse.urlparse".to_string(),
            vec![SecurityIssueType::ServerSideRequestForgery],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.60),
        SanitizationPoint::new(
            "python_validators_url".to_string(),
            "validators.url".to_string(),
            vec![SecurityIssueType::ServerSideRequestForgery],
        ).with_language(SourceLanguage::Python)
        .with_effectiveness(0.75),
    ]
}