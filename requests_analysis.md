# Uveddi Architectural Analysis Report

_Generated on 2025-07-31 08:26:06_

## Executive Summary

This report analyzes the codebase at `alpha_testing/requests` and identified **167 architectural issues** across **22 files**.

- **High Severity**: 162 issues
- **Medium Severity**: 5 issues
- **Low Severity**: 0 issues

The analysis took 0.00 seconds to complete.

## Issues by Severity

### 🔴 High Severity Issues

| File | Issue |
|------|-------|
| `models.py:84-84`| God Object detected: 'RequestEncodingMixin' has 3 methods and 24 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 1 complex |
| `models.py:230-230`| God Object detected: 'Request' has 3 methods and 16 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 1 complex |
| `models.py:313-313`| God Object detected: 'PreparedRequest' has 13 methods and 66 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 9 trivial, 4 complex |
| `models.py:640-640`| God Object detected: 'Response' has 23 methods and 47 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 16 trivial, 7 complex |
| `models.py:86-86`| Potentially dead code: function 'path_url' is not used in this file (confidence: 90.0%) |
| `models.py:218-218`| Potentially dead code: function 'deregister_hook' is not used in this file (confidence: 90.0%) |
| `models.py:755-755`| Potentially dead code: function 'ok' is not used in this file (confidence: 90.0%) |
| `models.py:770-770`| Potentially dead code: function 'is_redirect' is not used in this file (confidence: 90.0%) |
| `models.py:777-777`| Potentially dead code: function 'is_permanent_redirect' is not used in this file (confidence: 90.0%) |
| `models.py:785-785`| Potentially dead code: function 'next' is not used in this file (confidence: 90.0%) |
| `models.py:790-790`| Potentially dead code: function 'apparent_encoding' is not used in this file (confidence: 90.0%) |
| `models.py:857-857`| Potentially dead code: function 'iter_lines' is not used in this file (confidence: 90.0%) |
| `models.py:891-891`| Potentially dead code: function 'content' is not used in this file (confidence: 90.0%) |
| `models.py:910-910`| Potentially dead code: function 'text' is not used in this file (confidence: 90.0%) |
| `models.py:947-947`| Potentially dead code: function 'json' is not used in this file (confidence: 90.0%) |
| `models.py:983-983`| Potentially dead code: function 'links' is not used in this file (confidence: 90.0%) |
| `models.py:84-84`| Potentially dead code: class 'RequestEncodingMixin' is not used in this file (confidence: 90.0%) |
| `models.py:206-206`| Potentially dead code: class 'RequestHooksMixin' is not used in this file (confidence: 90.0%) |
| `models.py:230-230`| Potentially dead code: class 'Request' is not used in this file (confidence: 90.0%) |
| `models.py:640-640`| Potentially dead code: class 'Response' is not used in this file (confidence: 90.0%) |
| `models.py`| Component 'caller' has 163 dependencies, exceeding critical threshold of 15 |
| `sessions.py:106-106`| God Object detected: 'SessionRedirectMixin' has 6 methods and 43 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 2 complex |
| `sessions.py:356-356`| God Object detected: 'Session' has 19 methods and 52 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 13 trivial, 6 complex |
| `sessions.py:604-604`| Potentially dead code: function 'options' is not used in this file (confidence: 90.0%) |
| `sessions.py:615-615`| Potentially dead code: function 'head' is not used in this file (confidence: 90.0%) |
| `sessions.py:626-626`| Potentially dead code: function 'post' is not used in this file (confidence: 90.0%) |
| `sessions.py:639-639`| Potentially dead code: function 'put' is not used in this file (confidence: 90.0%) |
| `sessions.py:651-651`| Potentially dead code: function 'patch' is not used in this file (confidence: 90.0%) |
| `sessions.py:663-663`| Potentially dead code: function 'delete' is not used in this file (confidence: 90.0%) |
| `sessions.py:819-819`| Potentially dead code: function 'session' is not used in this file (confidence: 90.0%) |
| `sessions.py:106-106`| Potentially dead code: class 'SessionRedirectMixin' is not used in this file (confidence: 90.0%) |
| `sessions.py`| Component 'caller' has 127 dependencies, exceeding critical threshold of 15 |
| `_internal_utils.py:25-25`| Potentially dead code: function 'to_native_string' is not used in this file (confidence: 90.0%) |
| `_internal_utils.py:38-38`| Potentially dead code: function 'unicode_is_ascii' is not used in this file (confidence: 90.0%) |
| `exceptions.py:12-12`| Potentially dead code: class 'RequestException' is not used in this file (confidence: 90.0%) |
| `exceptions.py:27-27`| Potentially dead code: class 'InvalidJSONError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:31-31`| Potentially dead code: class 'JSONDecodeError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:55-55`| Potentially dead code: class 'HTTPError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:59-59`| Potentially dead code: class 'ConnectionError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:63-63`| Potentially dead code: class 'ProxyError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:67-67`| Potentially dead code: class 'SSLError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:71-71`| Potentially dead code: class 'Timeout' is not used in this file (confidence: 90.0%) |
| `exceptions.py:80-80`| Potentially dead code: class 'ConnectTimeout' is not used in this file (confidence: 90.0%) |
| `exceptions.py:87-87`| Potentially dead code: class 'ReadTimeout' is not used in this file (confidence: 90.0%) |
| `exceptions.py:91-91`| Potentially dead code: class 'URLRequired' is not used in this file (confidence: 90.0%) |
| `exceptions.py:95-95`| Potentially dead code: class 'TooManyRedirects' is not used in this file (confidence: 90.0%) |
| `exceptions.py:99-99`| Potentially dead code: class 'MissingSchema' is not used in this file (confidence: 90.0%) |
| `exceptions.py:103-103`| Potentially dead code: class 'InvalidSchema' is not used in this file (confidence: 90.0%) |
| `exceptions.py:107-107`| Potentially dead code: class 'InvalidURL' is not used in this file (confidence: 90.0%) |
| `exceptions.py:111-111`| Potentially dead code: class 'InvalidHeader' is not used in this file (confidence: 90.0%) |
| `exceptions.py:115-115`| Potentially dead code: class 'InvalidProxyURL' is not used in this file (confidence: 90.0%) |
| `exceptions.py:119-119`| Potentially dead code: class 'ChunkedEncodingError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:123-123`| Potentially dead code: class 'ContentDecodingError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:127-127`| Potentially dead code: class 'StreamConsumedError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:131-131`| Potentially dead code: class 'RetryError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:135-135`| Potentially dead code: class 'UnrewindableBodyError' is not used in this file (confidence: 90.0%) |
| `exceptions.py:142-142`| Potentially dead code: class 'RequestsWarning' is not used in this file (confidence: 90.0%) |
| `exceptions.py:146-146`| Potentially dead code: class 'FileModeWarning' is not used in this file (confidence: 90.0%) |
| `exceptions.py:150-150`| Potentially dead code: class 'RequestsDependencyWarning' is not used in this file (confidence: 90.0%) |
| `structures.py:13-13`| God Object detected: 'CaseInsensitiveDict' has 10 methods and 4 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 7 trivial, 3 complex |
| `structures.py:76-76`| Potentially dead code: function 'copy' is not used in this file (confidence: 90.0%) |
| `structures.py:83-83`| Potentially dead code: class 'LookupDict' is not used in this file (confidence: 90.0%) |
| `structures.py`| Component 'caller' has 17 dependencies, exceeding critical threshold of 15 |
| `__init__.py`| Component 'caller' has 28 dependencies, exceeding critical threshold of 15 |
| `api.py:62-62`| Potentially dead code: function 'get' is not used in this file (confidence: 90.0%) |
| `api.py:76-76`| Potentially dead code: function 'options' is not used in this file (confidence: 90.0%) |
| `api.py:88-88`| Potentially dead code: function 'head' is not used in this file (confidence: 90.0%) |
| `api.py:103-103`| Potentially dead code: function 'post' is not used in this file (confidence: 90.0%) |
| `api.py:118-118`| Potentially dead code: function 'put' is not used in this file (confidence: 90.0%) |
| `api.py:133-133`| Potentially dead code: function 'patch' is not used in this file (confidence: 90.0%) |
| `api.py:148-148`| Potentially dead code: function 'delete' is not used in this file (confidence: 90.0%) |
| `utils.py:127-127`| Potentially dead code: function 'dict_to_sequence' is not used in this file (confidence: 90.0%) |
| `utils.py:136-136`| Potentially dead code: function 'super_len' is not used in this file (confidence: 90.0%) |
| `utils.py:207-207`| Potentially dead code: function 'get_netrc_auth' is not used in this file (confidence: 90.0%) |
| `utils.py:251-251`| Potentially dead code: function 'guess_filename' is not used in this file (confidence: 90.0%) |
| `utils.py:258-258`| Potentially dead code: function 'extract_zipped_paths' is not used in this file (confidence: 90.0%) |
| `utils.py:308-308`| Potentially dead code: function 'from_key_val_list' is not used in this file (confidence: 90.0%) |
| `utils.py:335-335`| Potentially dead code: function 'to_key_val_list' is not used in this file (confidence: 90.0%) |
| `utils.py:365-365`| Potentially dead code: function 'parse_list_header' is not used in this file (confidence: 90.0%) |
| `utils.py:397-397`| Potentially dead code: function 'parse_dict_header' is not used in this file (confidence: 90.0%) |
| `utils.py:457-457`| Potentially dead code: function 'dict_from_cookiejar' is not used in this file (confidence: 90.0%) |
| `utils.py:468-468`| Potentially dead code: function 'add_dict_to_cookiejar' is not used in this file (confidence: 90.0%) |
| `utils.py:479-479`| Potentially dead code: function 'get_encodings_from_content' is not used in this file (confidence: 90.0%) |
| `utils.py:554-554`| Potentially dead code: function 'stream_decode_response_unicode' is not used in this file (confidence: 90.0%) |
| `utils.py:571-571`| Potentially dead code: function 'iter_slices' is not used in this file (confidence: 90.0%) |
| `utils.py:581-581`| Potentially dead code: function 'get_unicode_from_response' is not used in this file (confidence: 90.0%) |
| `utils.py:650-650`| Potentially dead code: function 'requote_uri' is not used in this file (confidence: 90.0%) |
| `utils.py:828-828`| Potentially dead code: function 'select_proxy' is not used in this file (confidence: 90.0%) |
| `utils.py:854-854`| Potentially dead code: function 'resolve_proxies' is not used in this file (confidence: 90.0%) |
| `utils.py:890-890`| Potentially dead code: function 'default_headers' is not used in this file (confidence: 90.0%) |
| `utils.py:904-904`| Potentially dead code: function 'parse_header_links' is not used in this file (confidence: 90.0%) |
| `utils.py:947-947`| Potentially dead code: function 'guess_json_utf' is not used in this file (confidence: 90.0%) |
| `utils.py:979-979`| Potentially dead code: function 'prepend_scheme_if_needed' is not used in this file (confidence: 90.0%) |
| `utils.py:1008-1008`| Potentially dead code: function 'get_auth_from_url' is not used in this file (confidence: 90.0%) |
| `utils.py:1024-1024`| Potentially dead code: function 'check_header_validity' is not used in this file (confidence: 90.0%) |
| `utils.py:1054-1054`| Potentially dead code: function 'urldefragauth' is not used in this file (confidence: 90.0%) |
| `utils.py:1071-1071`| Potentially dead code: function 'rewind_body' is not used in this file (confidence: 90.0%) |
| `utils.py`| Component 'caller' has 174 dependencies, exceeding critical threshold of 15 |
| `hooks.py:15-15`| Potentially dead code: function 'default_hooks' is not used in this file (confidence: 90.0%) |
| `hooks.py:22-22`| Potentially dead code: function 'dispatch_hook' is not used in this file (confidence: 90.0%) |
| `cookies.py:23-23`| God Object detected: 'MockRequest' has 14 methods and 6 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 9 trivial, 5 complex |
| `cookies.py:176-176`| God Object detected: 'RequestsCookieJar' has 24 methods and 13 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 16 trivial, 8 complex |
| `cookies.py:40-40`| Potentially dead code: function 'get_type' is not used in this file (confidence: 90.0%) |
| `cookies.py:49-49`| Potentially dead code: function 'get_full_url' is not used in this file (confidence: 90.0%) |
| `cookies.py:72-72`| Potentially dead code: function 'has_header' is not used in this file (confidence: 90.0%) |
| `cookies.py:75-75`| Potentially dead code: function 'get_header' is not used in this file (confidence: 90.0%) |
| `cookies.py:78-78`| Potentially dead code: function 'add_header' is not used in this file (confidence: 90.0%) |
| `cookies.py:84-84`| Potentially dead code: function 'add_unredirected_header' is not used in this file (confidence: 90.0%) |
| `cookies.py:91-91`| Potentially dead code: function 'unverifiable' is not used in this file (confidence: 90.0%) |
| `cookies.py:95-95`| Potentially dead code: function 'origin_req_host' is not used in this file (confidence: 90.0%) |
| `cookies.py:99-99`| Potentially dead code: function 'host' is not used in this file (confidence: 90.0%) |
| `cookies.py:117-117`| Potentially dead code: function 'info' is not used in this file (confidence: 90.0%) |
| `cookies.py:124-124`| Potentially dead code: function 'extract_cookies_to_jar' is not used in this file (confidence: 90.0%) |
| `cookies.py:140-140`| Potentially dead code: function 'get_cookie_header' is not used in this file (confidence: 90.0%) |
| `cookies.py:234-234`| Potentially dead code: function 'keys' is not used in this file (confidence: 90.0%) |
| `cookies.py:251-251`| Potentially dead code: function 'values' is not used in this file (confidence: 90.0%) |
| `cookies.py:268-268`| Potentially dead code: function 'items' is not used in this file (confidence: 90.0%) |
| `cookies.py:285-285`| Potentially dead code: function 'list_paths' is not used in this file (confidence: 90.0%) |
| `cookies.py:306-306`| Potentially dead code: function 'get_dict' is not used in this file (confidence: 90.0%) |
| `cookies.py:542-542`| Potentially dead code: function 'merge_cookies' is not used in this file (confidence: 90.0%) |
| `cookies.py`| Component 'caller' has 98 dependencies, exceeding critical threshold of 15 |
| `auth.py:107-107`| God Object detected: 'HTTPDigestAuth' has 12 methods and 57 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 8 trivial, 4 complex |
| `auth.py:145-145`| Potentially dead code: function 'md5_utf8' is not used in this file (confidence: 90.0%) |
| `auth.py:153-153`| Potentially dead code: function 'sha_utf8' is not used in this file (confidence: 90.0%) |
| `auth.py:161-161`| Potentially dead code: function 'sha256_utf8' is not used in this file (confidence: 90.0%) |
| `auth.py:169-169`| Potentially dead code: function 'sha512_utf8' is not used in this file (confidence: 90.0%) |
| `auth.py:236-236`| Potentially dead code: function 'handle_redirect' is not used in this file (confidence: 90.0%) |
| `auth.py:241-241`| Potentially dead code: function 'handle_401' is not used in this file (confidence: 90.0%) |
| `auth.py:69-69`| Potentially dead code: class 'AuthBase' is not used in this file (confidence: 90.0%) |
| `auth.py:76-76`| Potentially dead code: class 'HTTPBasicAuth' is not used in this file (confidence: 90.0%) |
| `auth.py:99-99`| Potentially dead code: class 'HTTPProxyAuth' is not used in this file (confidence: 90.0%) |
| `auth.py:107-107`| Potentially dead code: class 'HTTPDigestAuth' is not used in this file (confidence: 90.0%) |
| `auth.py`| Component 'caller' has 62 dependencies, exceeding critical threshold of 15 |
| `adapters.py:143-143`| God Object detected: 'HTTPAdapter' has 15 methods and 76 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 10 trivial, 5 complex |
| `adapters.py:119-119`| Potentially dead code: function 'send' is not used in this file (confidence: 90.0%) |
| `adapters.py:138-138`| Potentially dead code: function 'close' is not used in this file (confidence: 90.0%) |
| `adapters.py:472-472`| Potentially dead code: function 'get_connection' is not used in this file (confidence: 90.0%) |
| `adapters.py:513-513`| Potentially dead code: function 'close' is not used in this file (confidence: 90.0%) |
| `adapters.py:590-590`| Potentially dead code: function 'send' is not used in this file (confidence: 90.0%) |
| `adapters.py:113-113`| Potentially dead code: class 'BaseAdapter' is not used in this file (confidence: 90.0%) |
| `adapters.py:143-143`| Potentially dead code: class 'HTTPAdapter' is not used in this file (confidence: 90.0%) |
| `adapters.py`| Component 'caller' has 92 dependencies, exceeding critical threshold of 15 |
| `flask_theme_support.py:7-7`| Potentially dead code: class 'FlaskyStyle' is not used in this file (confidence: 90.0%) |
| `test_utils.py:50-50`| God Object detected: 'TestSuperLen' has 17 methods and 11 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 11 trivial, 6 complex |
| `test_utils.py`| Component 'caller' has 157 dependencies, exceeding critical threshold of 15 |
| `test_requests.py:82-82`| God Object detected: 'TestRequests' has 213 methods and 566 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 149 trivial, 64 complex |
| `test_requests.py:2277-2277`| God Object detected: 'TestCaseInsensitiveDict' has 17 methods and 37 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 11 trivial, 6 complex |
| `test_requests.py:2555-2555`| God Object detected: 'RedirectSession' has 4 methods and 13 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 2 complex |
| `test_requests.py:2706-2706`| God Object detected: 'TestPreparingURLs' has 18 methods and 41 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 12 trivial, 6 complex |
| `test_requests.py`| Component 'caller' has 1039 dependencies, exceeding critical threshold of 15 |
| `compat.py:14-14`| Potentially dead code: function 'u' is not used in this file (confidence: 90.0%) |
| `conftest.py`| Component 'caller' has 15 dependencies, exceeding critical threshold of 15 |
| `server.py:25-25`| God Object detected: 'Server' has 11 methods and 16 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 7 trivial, 4 complex |
| `server.py:53-53`| Potentially dead code: function 'text_response_handler' is not used in this file (confidence: 90.0%) |
| `server.py:62-62`| Potentially dead code: function 'basic_response_server' is not used in this file (confidence: 90.0%) |
| `server.py:67-67`| Potentially dead code: function 'run' is not used in this file (confidence: 90.0%) |
| `server.py:138-138`| Potentially dead code: class 'TLSServer' is not used in this file (confidence: 90.0%) |
| `server.py`| Component 'caller' has 30 dependencies, exceeding critical threshold of 15 |
| `test_testserver.py:11-11`| God Object detected: 'TestTestServer' has 12 methods and 29 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 8 trivial, 4 complex |
| `test_testserver.py`| Component 'caller' has 59 dependencies, exceeding critical threshold of 15 |
| `test_lowlevel.py`| Component 'caller' has 113 dependencies, exceeding critical threshold of 15 |
| `utils.py:6-6`| Potentially dead code: function 'override_environ' is not used in this file (confidence: 90.0%) |

### 🟠 Medium Severity Issues

| File | Issue |
|------|-------|
| `cookies.py:366-366`| Potentially dead code: function '_find' is not used in this file (confidence: 70.0%) |
| `cookies.py:440-440`| Potentially dead code: function '_copy_cookie_jar' is not used in this file (confidence: 70.0%) |
| `test_structures.py:6-6`| God Object detected: 'TestCaseInsensitiveDict' has 8 methods and 4 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 5 trivial, 3 complex |
| `test_requests.py:890-890`| God Object detected: 'CustomMapping' has 6 methods and 2 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 2 complex |
| `test_requests.py:2483-2483`| God Object detected: 'TestTimeout' has 7 methods and 2 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 3 complex |



## Detailed Analysis

### Code Duplication

Multiple instances of similar code that should be refactored.

#### Issue #1: Potentially dead code: function 'path_url' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 86-86

**Code Snippet**:

```
path_url
```


#### Issue #2: Potentially dead code: function 'deregister_hook' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 218-218

**Code Snippet**:

```
deregister_hook
```


#### Issue #3: Potentially dead code: function 'ok' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 755-755

**Code Snippet**:

```
ok
```


#### Issue #4: Potentially dead code: function 'is_redirect' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 770-770

**Code Snippet**:

```
is_redirect
```


#### Issue #5: Potentially dead code: function 'is_permanent_redirect' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 777-777

**Code Snippet**:

```
is_permanent_redirect
```


#### Issue #6: Potentially dead code: function 'next' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 785-785

**Code Snippet**:

```
next
```


#### Issue #7: Potentially dead code: function 'apparent_encoding' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 790-790

**Code Snippet**:

```
apparent_encoding
```


#### Issue #8: Potentially dead code: function 'iter_lines' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 857-857

**Code Snippet**:

```
iter_lines
```


#### Issue #9: Potentially dead code: function 'content' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 891-891

**Code Snippet**:

```
content
```


#### Issue #10: Potentially dead code: function 'text' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 910-910

**Code Snippet**:

```
text
```


#### Issue #11: Potentially dead code: function 'json' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 947-947

**Code Snippet**:

```
json
```


#### Issue #12: Potentially dead code: function 'links' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 983-983

**Code Snippet**:

```
links
```


#### Issue #13: Potentially dead code: class 'RequestEncodingMixin' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 84-84

**Code Snippet**:

```
RequestEncodingMixin
```


#### Issue #14: Potentially dead code: class 'RequestHooksMixin' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 206-206

**Code Snippet**:

```
RequestHooksMixin
```


#### Issue #15: Potentially dead code: class 'Request' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 230-230

**Code Snippet**:

```
Request
```


#### Issue #16: Potentially dead code: class 'Response' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 640-640

**Code Snippet**:

```
Response
```


#### Issue #17: Potentially dead code: function 'options' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 604-604

**Code Snippet**:

```
options
```


#### Issue #18: Potentially dead code: function 'head' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 615-615

**Code Snippet**:

```
head
```


#### Issue #19: Potentially dead code: function 'post' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 626-626

**Code Snippet**:

```
post
```


#### Issue #20: Potentially dead code: function 'put' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 639-639

**Code Snippet**:

```
put
```


#### Issue #21: Potentially dead code: function 'patch' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 651-651

**Code Snippet**:

```
patch
```


#### Issue #22: Potentially dead code: function 'delete' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 663-663

**Code Snippet**:

```
delete
```


#### Issue #23: Potentially dead code: function 'session' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 819-819

**Code Snippet**:

```
session
```


#### Issue #24: Potentially dead code: class 'SessionRedirectMixin' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: High
- **Location**: Lines 106-106

**Code Snippet**:

```
SessionRedirectMixin
```


#### Issue #25: Potentially dead code: function 'to_native_string' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/_internal_utils.py`
- **Severity**: High
- **Location**: Lines 25-25

**Code Snippet**:

```
to_native_string
```


#### Issue #26: Potentially dead code: function 'unicode_is_ascii' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/_internal_utils.py`
- **Severity**: High
- **Location**: Lines 38-38

**Code Snippet**:

```
unicode_is_ascii
```


#### Issue #27: Potentially dead code: class 'RequestException' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 12-12

**Code Snippet**:

```
RequestException
```


#### Issue #28: Potentially dead code: class 'InvalidJSONError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 27-27

**Code Snippet**:

```
InvalidJSONError
```


#### Issue #29: Potentially dead code: class 'JSONDecodeError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 31-31

**Code Snippet**:

```
JSONDecodeError
```


#### Issue #30: Potentially dead code: class 'HTTPError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 55-55

**Code Snippet**:

```
HTTPError
```


#### Issue #31: Potentially dead code: class 'ConnectionError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 59-59

**Code Snippet**:

```
ConnectionError
```


#### Issue #32: Potentially dead code: class 'ProxyError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 63-63

**Code Snippet**:

```
ProxyError
```


#### Issue #33: Potentially dead code: class 'SSLError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 67-67

**Code Snippet**:

```
SSLError
```


#### Issue #34: Potentially dead code: class 'Timeout' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 71-71

**Code Snippet**:

```
Timeout
```


#### Issue #35: Potentially dead code: class 'ConnectTimeout' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 80-80

**Code Snippet**:

```
ConnectTimeout
```


#### Issue #36: Potentially dead code: class 'ReadTimeout' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 87-87

**Code Snippet**:

```
ReadTimeout
```


#### Issue #37: Potentially dead code: class 'URLRequired' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 91-91

**Code Snippet**:

```
URLRequired
```


#### Issue #38: Potentially dead code: class 'TooManyRedirects' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 95-95

**Code Snippet**:

```
TooManyRedirects
```


#### Issue #39: Potentially dead code: class 'MissingSchema' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 99-99

**Code Snippet**:

```
MissingSchema
```


#### Issue #40: Potentially dead code: class 'InvalidSchema' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 103-103

**Code Snippet**:

```
InvalidSchema
```


#### Issue #41: Potentially dead code: class 'InvalidURL' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 107-107

**Code Snippet**:

```
InvalidURL
```


#### Issue #42: Potentially dead code: class 'InvalidHeader' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 111-111

**Code Snippet**:

```
InvalidHeader
```


#### Issue #43: Potentially dead code: class 'InvalidProxyURL' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 115-115

**Code Snippet**:

```
InvalidProxyURL
```


#### Issue #44: Potentially dead code: class 'ChunkedEncodingError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 119-119

**Code Snippet**:

```
ChunkedEncodingError
```


#### Issue #45: Potentially dead code: class 'ContentDecodingError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 123-123

**Code Snippet**:

```
ContentDecodingError
```


#### Issue #46: Potentially dead code: class 'StreamConsumedError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 127-127

**Code Snippet**:

```
StreamConsumedError
```


#### Issue #47: Potentially dead code: class 'RetryError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 131-131

**Code Snippet**:

```
RetryError
```


#### Issue #48: Potentially dead code: class 'UnrewindableBodyError' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 135-135

**Code Snippet**:

```
UnrewindableBodyError
```


#### Issue #49: Potentially dead code: class 'RequestsWarning' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 142-142

**Code Snippet**:

```
RequestsWarning
```


#### Issue #50: Potentially dead code: class 'FileModeWarning' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 146-146

**Code Snippet**:

```
FileModeWarning
```


#### Issue #51: Potentially dead code: class 'RequestsDependencyWarning' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/exceptions.py`
- **Severity**: High
- **Location**: Lines 150-150

**Code Snippet**:

```
RequestsDependencyWarning
```


#### Issue #52: Potentially dead code: function 'copy' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/structures.py`
- **Severity**: High
- **Location**: Lines 76-76

**Code Snippet**:

```
copy
```


#### Issue #53: Potentially dead code: class 'LookupDict' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/structures.py`
- **Severity**: High
- **Location**: Lines 83-83

**Code Snippet**:

```
LookupDict
```


#### Issue #54: Potentially dead code: function 'get' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 62-62

**Code Snippet**:

```
get
```


#### Issue #55: Potentially dead code: function 'options' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 76-76

**Code Snippet**:

```
options
```


#### Issue #56: Potentially dead code: function 'head' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 88-88

**Code Snippet**:

```
head
```


#### Issue #57: Potentially dead code: function 'post' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 103-103

**Code Snippet**:

```
post
```


#### Issue #58: Potentially dead code: function 'put' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 118-118

**Code Snippet**:

```
put
```


#### Issue #59: Potentially dead code: function 'patch' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 133-133

**Code Snippet**:

```
patch
```


#### Issue #60: Potentially dead code: function 'delete' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/api.py`
- **Severity**: High
- **Location**: Lines 148-148

**Code Snippet**:

```
delete
```


#### Issue #61: Potentially dead code: function 'dict_to_sequence' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 127-127

**Code Snippet**:

```
dict_to_sequence
```


#### Issue #62: Potentially dead code: function 'super_len' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 136-136

**Code Snippet**:

```
super_len
```


#### Issue #63: Potentially dead code: function 'get_netrc_auth' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 207-207

**Code Snippet**:

```
get_netrc_auth
```


#### Issue #64: Potentially dead code: function 'guess_filename' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 251-251

**Code Snippet**:

```
guess_filename
```


#### Issue #65: Potentially dead code: function 'extract_zipped_paths' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 258-258

**Code Snippet**:

```
extract_zipped_paths
```


#### Issue #66: Potentially dead code: function 'from_key_val_list' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 308-308

**Code Snippet**:

```
from_key_val_list
```


#### Issue #67: Potentially dead code: function 'to_key_val_list' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 335-335

**Code Snippet**:

```
to_key_val_list
```


#### Issue #68: Potentially dead code: function 'parse_list_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 365-365

**Code Snippet**:

```
parse_list_header
```


#### Issue #69: Potentially dead code: function 'parse_dict_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 397-397

**Code Snippet**:

```
parse_dict_header
```


#### Issue #70: Potentially dead code: function 'dict_from_cookiejar' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 457-457

**Code Snippet**:

```
dict_from_cookiejar
```


#### Issue #71: Potentially dead code: function 'add_dict_to_cookiejar' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 468-468

**Code Snippet**:

```
add_dict_to_cookiejar
```


#### Issue #72: Potentially dead code: function 'get_encodings_from_content' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 479-479

**Code Snippet**:

```
get_encodings_from_content
```


#### Issue #73: Potentially dead code: function 'stream_decode_response_unicode' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 554-554

**Code Snippet**:

```
stream_decode_response_unicode
```


#### Issue #74: Potentially dead code: function 'iter_slices' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 571-571

**Code Snippet**:

```
iter_slices
```


#### Issue #75: Potentially dead code: function 'get_unicode_from_response' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 581-581

**Code Snippet**:

```
get_unicode_from_response
```


#### Issue #76: Potentially dead code: function 'requote_uri' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 650-650

**Code Snippet**:

```
requote_uri
```


#### Issue #77: Potentially dead code: function 'select_proxy' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 828-828

**Code Snippet**:

```
select_proxy
```


#### Issue #78: Potentially dead code: function 'resolve_proxies' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 854-854

**Code Snippet**:

```
resolve_proxies
```


#### Issue #79: Potentially dead code: function 'default_headers' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 890-890

**Code Snippet**:

```
default_headers
```


#### Issue #80: Potentially dead code: function 'parse_header_links' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 904-904

**Code Snippet**:

```
parse_header_links
```


#### Issue #81: Potentially dead code: function 'guess_json_utf' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 947-947

**Code Snippet**:

```
guess_json_utf
```


#### Issue #82: Potentially dead code: function 'prepend_scheme_if_needed' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 979-979

**Code Snippet**:

```
prepend_scheme_if_needed
```


#### Issue #83: Potentially dead code: function 'get_auth_from_url' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 1008-1008

**Code Snippet**:

```
get_auth_from_url
```


#### Issue #84: Potentially dead code: function 'check_header_validity' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 1024-1024

**Code Snippet**:

```
check_header_validity
```


#### Issue #85: Potentially dead code: function 'urldefragauth' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 1054-1054

**Code Snippet**:

```
urldefragauth
```


#### Issue #86: Potentially dead code: function 'rewind_body' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: High
- **Location**: Lines 1071-1071

**Code Snippet**:

```
rewind_body
```


#### Issue #87: Potentially dead code: function 'default_hooks' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/hooks.py`
- **Severity**: High
- **Location**: Lines 15-15

**Code Snippet**:

```
default_hooks
```


#### Issue #88: Potentially dead code: function 'dispatch_hook' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/hooks.py`
- **Severity**: High
- **Location**: Lines 22-22

**Code Snippet**:

```
dispatch_hook
```


#### Issue #89: Potentially dead code: function 'get_type' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 40-40

**Code Snippet**:

```
get_type
```


#### Issue #90: Potentially dead code: function 'get_full_url' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 49-49

**Code Snippet**:

```
get_full_url
```


#### Issue #91: Potentially dead code: function 'has_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 72-72

**Code Snippet**:

```
has_header
```


#### Issue #92: Potentially dead code: function 'get_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 75-75

**Code Snippet**:

```
get_header
```


#### Issue #93: Potentially dead code: function 'add_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 78-78

**Code Snippet**:

```
add_header
```


#### Issue #94: Potentially dead code: function 'add_unredirected_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 84-84

**Code Snippet**:

```
add_unredirected_header
```


#### Issue #95: Potentially dead code: function 'unverifiable' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 91-91

**Code Snippet**:

```
unverifiable
```


#### Issue #96: Potentially dead code: function 'origin_req_host' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 95-95

**Code Snippet**:

```
origin_req_host
```


#### Issue #97: Potentially dead code: function 'host' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 99-99

**Code Snippet**:

```
host
```


#### Issue #98: Potentially dead code: function 'info' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 117-117

**Code Snippet**:

```
info
```


#### Issue #99: Potentially dead code: function 'extract_cookies_to_jar' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 124-124

**Code Snippet**:

```
extract_cookies_to_jar
```


#### Issue #100: Potentially dead code: function 'get_cookie_header' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 140-140

**Code Snippet**:

```
get_cookie_header
```


#### Issue #101: Potentially dead code: function 'keys' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 234-234

**Code Snippet**:

```
keys
```


#### Issue #102: Potentially dead code: function 'values' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 251-251

**Code Snippet**:

```
values
```


#### Issue #103: Potentially dead code: function 'items' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 268-268

**Code Snippet**:

```
items
```


#### Issue #104: Potentially dead code: function 'list_paths' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 285-285

**Code Snippet**:

```
list_paths
```


#### Issue #105: Potentially dead code: function 'get_dict' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 306-306

**Code Snippet**:

```
get_dict
```


#### Issue #106: Potentially dead code: function '_find' is not used in this file (confidence: 70.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: Medium
- **Location**: Lines 366-366

**Code Snippet**:

```
_find
```


#### Issue #107: Potentially dead code: function '_copy_cookie_jar' is not used in this file (confidence: 70.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: Medium
- **Location**: Lines 440-440

**Code Snippet**:

```
_copy_cookie_jar
```


#### Issue #108: Potentially dead code: function 'merge_cookies' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: High
- **Location**: Lines 542-542

**Code Snippet**:

```
merge_cookies
```


#### Issue #109: Potentially dead code: function 'md5_utf8' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 145-145

**Code Snippet**:

```
md5_utf8
```


#### Issue #110: Potentially dead code: function 'sha_utf8' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 153-153

**Code Snippet**:

```
sha_utf8
```


#### Issue #111: Potentially dead code: function 'sha256_utf8' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 161-161

**Code Snippet**:

```
sha256_utf8
```


#### Issue #112: Potentially dead code: function 'sha512_utf8' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 169-169

**Code Snippet**:

```
sha512_utf8
```


#### Issue #113: Potentially dead code: function 'handle_redirect' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 236-236

**Code Snippet**:

```
handle_redirect
```


#### Issue #114: Potentially dead code: function 'handle_401' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 241-241

**Code Snippet**:

```
handle_401
```


#### Issue #115: Potentially dead code: class 'AuthBase' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 69-69

**Code Snippet**:

```
AuthBase
```


#### Issue #116: Potentially dead code: class 'HTTPBasicAuth' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 76-76

**Code Snippet**:

```
HTTPBasicAuth
```


#### Issue #117: Potentially dead code: class 'HTTPProxyAuth' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 99-99

**Code Snippet**:

```
HTTPProxyAuth
```


#### Issue #118: Potentially dead code: class 'HTTPDigestAuth' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: High
- **Location**: Lines 107-107

**Code Snippet**:

```
HTTPDigestAuth
```


#### Issue #119: Potentially dead code: function 'send' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 119-119

**Code Snippet**:

```
send
```


#### Issue #120: Potentially dead code: function 'close' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 138-138

**Code Snippet**:

```
close
```


#### Issue #121: Potentially dead code: function 'get_connection' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 472-472

**Code Snippet**:

```
get_connection
```


#### Issue #122: Potentially dead code: function 'close' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 513-513

**Code Snippet**:

```
close
```


#### Issue #123: Potentially dead code: function 'send' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 590-590

**Code Snippet**:

```
send
```


#### Issue #124: Potentially dead code: class 'BaseAdapter' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 113-113

**Code Snippet**:

```
BaseAdapter
```


#### Issue #125: Potentially dead code: class 'HTTPAdapter' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: High
- **Location**: Lines 143-143

**Code Snippet**:

```
HTTPAdapter
```


#### Issue #126: Potentially dead code: class 'FlaskyStyle' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/docs/_themes/flask_theme_support.py`
- **Severity**: High
- **Location**: Lines 7-7

**Code Snippet**:

```
FlaskyStyle
```


#### Issue #127: Potentially dead code: function 'u' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/compat.py`
- **Severity**: High
- **Location**: Lines 14-14

**Code Snippet**:

```
u
```


#### Issue #128: Potentially dead code: function 'text_response_handler' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: High
- **Location**: Lines 53-53

**Code Snippet**:

```
text_response_handler
```


#### Issue #129: Potentially dead code: function 'basic_response_server' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: High
- **Location**: Lines 62-62

**Code Snippet**:

```
basic_response_server
```


#### Issue #130: Potentially dead code: function 'run' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: High
- **Location**: Lines 67-67

**Code Snippet**:

```
run
```


#### Issue #131: Potentially dead code: class 'TLSServer' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: High
- **Location**: Lines 138-138

**Code Snippet**:

```
TLSServer
```


#### Issue #132: Potentially dead code: function 'override_environ' is not used in this file (confidence: 90.0%)

- **File**: `alpha_testing/requests/tests/utils.py`
- **Severity**: High
- **Location**: Lines 6-6

**Code Snippet**:

```
override_environ
```


### God Object

A class that has too many responsibilities and is difficult to maintain.

#### Issue #1: God Object detected: 'RequestEncodingMixin' has 3 methods and 24 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 1 complex

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: Critical
- **Location**: Lines 84-84

**Code Snippet**:

```
class RequestEncodingMixin:
    @property
    def path_url(self):
        """Build the path URL to use."""

        url = []

        p = urlsplit(self.url)

        path = p.path
        if not path:
            path = "/"

        url.append(path)

        query = p.query
        if query:
            url.append("?")
            url.append(query)

        return "".join(url)

    @staticmethod
    def _encode_params(data):
        """Encode parameters in a piece of data.

        Will successfully encode parameters when passed as a dict or a list of
        2-tuples. Order is retained if data is a list of 2-tuples but arbitrary
        if parameters are supplied as a dict.
        """

        if isinstance(data, (str, bytes)):
            return data
        elif hasattr(data, "read"):
            return data
        elif hasattr(data, "__iter__"):
            result = []
            for k, vs in to_key_val_list(data):
                if isinstance(vs, basestring) or not hasattr(vs, "__iter__"):
                    vs = [vs]
                for v in vs:
                    if v is not None:
                        result.append(
                            (
                                k.encode("utf-8") if isinstance(k, str) else k,
                                v.encode("utf-8") if isinstance(v, str) else v,
                            )
                        )
            return urlencode(result, doseq=True)
        else:
            return data

    @staticmethod
    def _encode_files(files, data):
        """Build the body for a multipart/form-data request.

        Will successfully encode files when passed as a dict or a list of
        tuples. Order is retained if data is a list of tuples but arbitrary
        if parameters are supplied as a dict.
        The tuples may be 2-tuples (filename, fileobj), 3-tuples (filename, fileobj, contentype)
        or 4-tuples (filename, fileobj, contentype, custom_headers).
        """
        if not files:
            raise ValueError("Files must be provided.")
        elif isinstance(data, basestring):
            raise ValueError("Data must not be a string.")

        new_fields = []
        fields = to_key_val_list(data or {})
        files = to_key_val_list(files or {})

        for field, val in fields:
            if isinstance(val, basestring) or not hasattr(val, "__iter__"):
                val = [val]
            for v in val:
                if v is not None:
                    # Don't call str() on bytestrings: in Py3 it all goes wrong.
                    if not isinstance(v, bytes):
                        v = str(v)

                    new_fields.append(
                        (
                            field.decode("utf-8")
                            if isinstance(field, bytes)
                            else field,
                            v.encode("utf-8") if isinstance(v, str) else v,
                        )
                    )

        for k, v in files:
            # support for explicit filename
            ft = None
            fh = None
            if isinstance(v, (tuple, list)):
                if len(v) == 2:
                    fn, fp = v
                elif len(v) == 3:
                    fn, fp, ft = v
                else:
                    fn, fp, ft, fh = v
            else:
                fn = guess_filename(v) or k
                fp = v

            if isinstance(fp, (str, bytes, bytearray)):
                fdata = fp
            elif hasattr(fp, "read"):
                fdata = fp.read()
            elif fp is None:
                continue
            else:
                fdata = fp

            rf = RequestField(name=k, data=fdata, filename=fn, headers=fh)
            rf.make_multipart(content_type=ft)
            new_fields.append(rf)

        body, content_type = encode_multipart_formdata(new_fields)

        return body, content_type
```


#### Issue #2: God Object detected: 'Request' has 3 methods and 16 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 1 complex

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: High
- **Location**: Lines 230-230

**Code Snippet**:

```
class Request(RequestHooksMixin):
    """A user-created :class:`Request <Request>` object.

    Used to prepare a :class:`PreparedRequest <PreparedRequest>`, which is sent to the server.

    :param method: HTTP method to use.
    :param url: URL to send.
    :param headers: dictionary of headers to send.
    :param files: dictionary of {filename: fileobject} files to multipart upload.
    :param data: the body to attach to the request. If a dictionary or
        list of tuples ``[(key, value)]`` is provided, form-encoding will
        take place.
    :param json: json for the body to attach to the request (if files or data is not specified).
    :param params: URL parameters to append to the URL. If a dictionary or
        list of tuples ``[(key, value)]`` is provided, form-encoding will
        take place.
    :param auth: Auth handler or (user, pass) tuple.
    :param cookies: dictionary or CookieJar of cookies to attach to this request.
    :param hooks: dictionary of callback hooks, for internal usage.

    Usage::

      >>> import requests
      >>> req = requests.Request('GET', 'https://httpbin.org/get')
      >>> req.prepare()
      <PreparedRequest [GET]>
    """

    def __init__(
        self,
        method=None,
        url=None,
        headers=None,
        files=None,
        data=None,
        params=None,
        auth=None,
        cookies=None,
        hooks=None,
        json=None,
    ):
        # Default empty dicts for dict params.
        data = [] if data is None else data
        files = [] if files is None else files
        headers = {} if headers is None else headers
        params = {} if params is None else params
        hooks = {} if hooks is None else hooks

        self.hooks = default_hooks()
        for k, v in list(hooks.items()):
            self.register_hook(event=k, hook=v)

        self.method = method
        self.url = url
        self.headers = headers
        self.files = files
        self.data = data
        self.json = json
        self.params = params
        self.auth = auth
        self.cookies = cookies

    def __repr__(self):
        return f"<Request [{self.method}]>"

    def prepare(self):
        """Constructs a :class:`PreparedRequest <PreparedRequest>` for transmission and returns it."""
        p = PreparedRequest()
        p.prepare(
            method=self.method,
            url=self.url,
            headers=self.headers,
            files=self.files,
            data=self.data,
            json=self.json,
            params=self.params,
            auth=self.auth,
            cookies=self.cookies,
            hooks=self.hooks,
        )
        return p
```


#### Issue #3: God Object detected: 'PreparedRequest' has 13 methods and 66 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 9 trivial, 4 complex

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: Critical
- **Location**: Lines 313-313

**Code Snippet**:

```
class PreparedRequest(RequestEncodingMixin, RequestHooksMixin):
    """The fully mutable :class:`PreparedRequest <PreparedRequest>` object,
    containing the exact bytes that will be sent to the server.

    Instances are generated from a :class:`Request <Request>` object, and
    should not be instantiated manually; doing so may produce undesirable
    effects.

    Usage::

      >>> import requests
      >>> req = requests.Request('GET', 'https://httpbin.org/get')
      >>> r = req.prepare()
      >>> r
      <PreparedRequest [GET]>

      >>> s = requests.Session()
      >>> s.send(r)
      <Response [200]>
    """

    def __init__(self):
        #: HTTP verb to send to the server.
        self.method = None
        #: HTTP URL to send the request to.
        self.url = None
        #: dictionary of HTTP headers.
        self.headers = None
        # The `CookieJar` used to create the Cookie header will be stored here
        # after prepare_cookies is called
        self._cookies = None
        #: request body to send to the server.
        self.body = None
        #: dictionary of callback hooks, for internal usage.
        self.hooks = default_hooks()
        #: integer denoting starting position of a readable file-like body.
        self._body_position = None

    def prepare(
        self,
        method=None,
        url=None,
        headers=None,
        files=None,
        data=None,
        params=None,
        auth=None,
        cookies=None,
        hooks=None,
        json=None,
    ):
        """Prepares the entire request with the given parameters."""

        self.prepare_method(method)
        self.prepare_url(url, params)
        self.prepare_headers(headers)
        self.prepare_cookies(cookies)
        self.prepare_body(data, files, json)
        self.prepare_auth(auth, url)

        # Note that prepare_auth must be last to enable authentication schemes
        # such as OAuth to work on a fully prepared request.

        # This MUST go after prepare_auth. Authenticators could add a hook
        self.prepare_hooks(hooks)

    def __repr__(self):
        return f"<PreparedRequest [{self.method}]>"

    def copy(self):
        p = PreparedRequest()
        p.method = self.method
        p.url = self.url
        p.headers = self.headers.copy() if self.headers is not None else None
        p._cookies = _copy_cookie_jar(self._cookies)
        p.body = self.body
        p.hooks = self.hooks
        p._body_position = self._body_position
        return p

    def prepare_method(self, method):
        """Prepares the given HTTP method."""
        self.method = method
        if self.method is not None:
            self.method = to_native_string(self.method.upper())

    @staticmethod
    def _get_idna_encoded_host(host):
        import idna

        try:
            host = idna.encode(host, uts46=True).decode("utf-8")
        except idna.IDNAError:
            raise UnicodeError
        return host

    def prepare_url(self, url, params):
        """Prepares the given HTTP URL."""
        #: Accept objects that have string representations.
        #: We're unable to blindly call unicode/str functions
        #: as this will include the bytestring indicator (b'')
        #: on python 3.x.
        #: https://github.com/psf/requests/pull/2238
        if isinstance(url, bytes):
            url = url.decode("utf8")
        else:
            url = str(url)

        # Remove leading whitespaces from url
        url = url.lstrip()

        # Don't do any URL preparation for non-HTTP schemes like `mailto`,
        # `data` etc to work around exceptions from `url_parse`, which
        # handles RFC 3986 only.
        if ":" in url and not url.lower().startswith("http"):
            self.url = url
            return

        # Support for unicode domain names and paths.
        try:
            scheme, auth, host, port, path, query, fragment = parse_url(url)
        except LocationParseError as e:
            raise InvalidURL(*e.args)

        if not scheme:
            raise MissingSchema(
                f"Invalid URL {url!r}: No scheme supplied. "
                f"Perhaps you meant https://{url}?"
            )

        if not host:
            raise InvalidURL(f"Invalid URL {url!r}: No host supplied")

        # In general, we want to try IDNA encoding the hostname if the string contains
        # non-ASCII characters. This allows users to automatically get the correct IDNA
        # behaviour. For strings containing only ASCII characters, we need to also verify
        # it doesn't start with a wildcard (*), before allowing the unencoded hostname.
        if not unicode_is_ascii(host):
            try:
                host = self._get_idna_encoded_host(host)
            except UnicodeError:
                raise InvalidURL("URL has an invalid label.")
        elif host.startswith(("*", ".")):
            raise InvalidURL("URL has an invalid label.")

        # Carefully reconstruct the network location
        netloc = auth or ""
        if netloc:
            netloc += "@"
        netloc += host
        if port:
            netloc += f":{port}"

        # Bare domains aren't valid URLs.
        if not path:
            path = "/"

        if isinstance(params, (str, bytes)):
            params = to_native_string(params)

        enc_params = self._encode_params(params)
        if enc_params:
            if query:
                query = f"{query}&{enc_params}"
            else:
                query = enc_params

        url = requote_uri(urlunparse([scheme, netloc, path, None, query, fragment]))
        self.url = url

    def prepare_headers(self, headers):
        """Prepares the given HTTP headers."""

        self.headers = CaseInsensitiveDict()
        if headers:
            for header in headers.items():
                # Raise exception on invalid header value.
                check_header_validity(header)
                name, value = header
                self.headers[to_native_string(name)] = value

    def prepare_body(self, data, files, json=None):
        """Prepares the given HTTP body data."""

        # Check if file, fo, generator, iterator.
        # If not, run through normal process.

        # Nottin' on you.
        body = None
        content_type = None

        if not data and json is not None:
            # urllib3 requires a bytes-like body. Python 2's json.dumps
            # provides this natively, but Python 3 gives a Unicode string.
            content_type = "application/json"

            try:
                body = complexjson.dumps(json, allow_nan=False)
            except ValueError as ve:
                raise InvalidJSONError(ve, request=self)

            if not isinstance(body, bytes):
                body = body.encode("utf-8")

        is_stream = all(
            [
                hasattr(data, "__iter__"),
                not isinstance(data, (basestring, list, tuple, Mapping)),
            ]
        )

        if is_stream:
            try:
                length = super_len(data)
            except (TypeError, AttributeError, UnsupportedOperation):
                length = None

            body = data

            if getattr(body, "tell", None) is not None:
                # Record the current file position before reading.
                # This will allow us to rewind a file in the event
                # of a redirect.
                try:
                    self._body_position = body.tell()
                except OSError:
                    # This differentiates from None, allowing us to catch
                    # a failed `tell()` later when trying to rewind the body
                    self._body_position = object()

            if files:
                raise NotImplementedError(
                    "Streamed bodies and files are mutually exclusive."
                )

            if length:
                self.headers["Content-Length"] = builtin_str(length)
            else:
                self.headers["Transfer-Encoding"] = "chunked"
        else:
            # Multi-part file uploads.
            if files:
                (body, content_type) = self._encode_files(files, data)
            else:
                if data:
                    body = self._encode_params(data)
                    if isinstance(data, basestring) or hasattr(data, "read"):
                        content_type = None
                    else:
                        content_type = "application/x-www-form-urlencoded"

            self.prepare_content_length(body)

            # Add content-type if it wasn't explicitly provided.
            if content_type and ("content-type" not in self.headers):
                self.headers["Content-Type"] = content_type

        self.body = body

    def prepare_content_length(self, body):
        """Prepare Content-Length header based on request method and body"""
        if body is not None:
            length = super_len(body)
            if length:
                # If length exists, set it. Otherwise, we fallback
                # to Transfer-Encoding: chunked.
                self.headers["Content-Length"] = builtin_str(length)
        elif (
            self.method not in ("GET", "HEAD")
            and self.headers.get("Content-Length") is None
        ):
            # Set Content-Length to 0 for methods that can have a body
            # but don't provide one. (i.e. not GET or HEAD)
            self.headers["Content-Length"] = "0"

    def prepare_auth(self, auth, url=""):
        """Prepares the given HTTP auth data."""

        # If no Auth is explicitly provided, extract it from the URL first.
        if auth is None:
            url_auth = get_auth_from_url(self.url)
            auth = url_auth if any(url_auth) else None

        if auth:
            if isinstance(auth, tuple) and len(auth) == 2:
                # special-case basic HTTP auth
                auth = HTTPBasicAuth(*auth)

            # Allow auth to make its changes.
            r = auth(self)

            # Update self to reflect the auth changes.
            self.__dict__.update(r.__dict__)

            # Recompute Content-Length
            self.prepare_content_length(self.body)

    def prepare_cookies(self, cookies):
        """Prepares the given HTTP cookie data.

        This function eventually generates a ``Cookie`` header from the
        given cookies using cookielib. Due to cookielib's design, the header
        will not be regenerated if it already exists, meaning this function
        can only be called once for the life of the
        :class:`PreparedRequest <PreparedRequest>` object. Any subsequent calls
        to ``prepare_cookies`` will have no actual effect, unless the "Cookie"
        header is removed beforehand.
        """
        if isinstance(cookies, cookielib.CookieJar):
            self._cookies = cookies
        else:
            self._cookies = cookiejar_from_dict(cookies)

        cookie_header = get_cookie_header(self._cookies, self)
        if cookie_header is not None:
            self.headers["Cookie"] = cookie_header

    def prepare_hooks(self, hooks):
        """Prepares the given hooks."""
        # hooks can be passed as None to the prepare method and to this
        # method. To prevent iterating over None, simply use an empty list
        # if hooks is False-y
        hooks = hooks or []
        for event in hooks:
            self.register_hook(event, hooks[event])
```


#### Issue #4: God Object detected: 'Response' has 23 methods and 47 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 16 trivial, 7 complex

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: Critical
- **Location**: Lines 640-640

**Code Snippet**:

```
class Response:
    """The :class:`Response <Response>` object, which contains a
    server's response to an HTTP request.
    """

    __attrs__ = [
        "_content",
        "status_code",
        "headers",
        "url",
        "history",
        "encoding",
        "reason",
        "cookies",
        "elapsed",
        "request",
    ]

    def __init__(self):
        self._content = False
        self._content_consumed = False
        self._next = None

        #: Integer Code of responded HTTP Status, e.g. 404 or 200.
        self.status_code = None

        #: Case-insensitive Dictionary of Response Headers.
        #: For example, ``headers['content-encoding']`` will return the
        #: value of a ``'Content-Encoding'`` response header.
        self.headers = CaseInsensitiveDict()

        #: File-like object representation of response (for advanced usage).
        #: Use of ``raw`` requires that ``stream=True`` be set on the request.
        #: This requirement does not apply for use internally to Requests.
        self.raw = None

        #: Final URL location of Response.
        self.url = None

        #: Encoding to decode with when accessing r.text.
        self.encoding = None

        #: A list of :class:`Response <Response>` objects from
        #: the history of the Request. Any redirect responses will end
        #: up here. The list is sorted from the oldest to the most recent request.
        self.history = []

        #: Textual reason of responded HTTP Status, e.g. "Not Found" or "OK".
        self.reason = None

        #: A CookieJar of Cookies the server sent back.
        self.cookies = cookiejar_from_dict({})

        #: The amount of time elapsed between sending the request
        #: and the arrival of the response (as a timedelta).
        #: This property specifically measures the time taken between sending
        #: the first byte of the request and finishing parsing the headers. It
        #: is therefore unaffected by consuming the response content or the
        #: value of the ``stream`` keyword argument.
        self.elapsed = datetime.timedelta(0)

        #: The :class:`PreparedRequest <PreparedRequest>` object to which this
        #: is a response.
        self.request = None

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def __getstate__(self):
        # Consume everything; accessing the content attribute makes
        # sure the content has been fully read.
        if not self._content_consumed:
            self.content

        return {attr: getattr(self, attr, None) for attr in self.__attrs__}

    def __setstate__(self, state):
        for name, value in state.items():
            setattr(self, name, value)

        # pickled objects do not have .raw
        setattr(self, "_content_consumed", True)
        setattr(self, "raw", None)

    def __repr__(self):
        return f"<Response [{self.status_code}]>"

    def __bool__(self):
        """Returns True if :attr:`status_code` is less than 400.

        This attribute checks if the status code of the response is between
        400 and 600 to see if there was a client error or a server error. If
        the status code, is between 200 and 400, this will return True. This
        is **not** a check to see if the response code is ``200 OK``.
        """
        return self.ok

    def __nonzero__(self):
        """Returns True if :attr:`status_code` is less than 400.

        This attribute checks if the status code of the response is between
        400 and 600 to see if there was a client error or a server error. If
        the status code, is between 200 and 400, this will return True. This
        is **not** a check to see if the response code is ``200 OK``.
        """
        return self.ok

    def __iter__(self):
        """Allows you to use a response as an iterator."""
        return self.iter_content(128)

    @property
    def ok(self):
        """Returns True if :attr:`status_code` is less than 400, False if not.

        This attribute checks if the status code of the response is between
        400 and 600 to see if there was a client error or a server error. If
        the status code is between 200 and 400, this will return True. This
        is **not** a check to see if the response code is ``200 OK``.
        """
        try:
            self.raise_for_status()
        except HTTPError:
            return False
        return True

    @property
    def is_redirect(self):
        """True if this Response is a well-formed HTTP redirect that could have
        been processed automatically (by :meth:`Session.resolve_redirects`).
        """
        return "location" in self.headers and self.status_code in REDIRECT_STATI

    @property
    def is_permanent_redirect(self):
        """True if this Response one of the permanent versions of redirect."""
        return "location" in self.headers and self.status_code in (
            codes.moved_permanently,
            codes.permanent_redirect,
        )

    @property
    def next(self):
        """Returns a PreparedRequest for the next request in a redirect chain, if there is one."""
        return self._next

    @property
    def apparent_encoding(self):
        """The apparent encoding, provided by the charset_normalizer or chardet libraries."""
        if chardet is not None:
            return chardet.detect(self.content)["encoding"]
        else:
            # If no character detection library is available, we'll fall back
            # to a standard Python utf-8 str.
            return "utf-8"

    def iter_content(self, chunk_size=1, decode_unicode=False):
        """Iterates over the response data.  When stream=True is set on the
        request, this avoids reading the content at once into memory for
        large responses.  The chunk size is the number of bytes it should
        read into memory.  This is not necessarily the length of each item
        returned as decoding can take place.

        chunk_size must be of type int or None. A value of None will
        function differently depending on the value of `stream`.
        stream=True will read data as it arrives in whatever size the
        chunks are received. If stream=False, data is returned as
        a single chunk.

        If decode_unicode is True, content will be decoded using the best
        available encoding based on the response.
        """

        def generate():
            # Special case for urllib3.
            if hasattr(self.raw, "stream"):
                try:
                    yield from self.raw.stream(chunk_size, decode_content=True)
                except ProtocolError as e:
                    raise ChunkedEncodingError(e)
                except DecodeError as e:
                    raise ContentDecodingError(e)
                except ReadTimeoutError as e:
                    raise ConnectionError(e)
                except SSLError as e:
                    raise RequestsSSLError(e)
            else:
                # Standard file-like object.
                while True:
                    chunk = self.raw.read(chunk_size)
                    if not chunk:
                        break
                    yield chunk

            self._content_consumed = True

        if self._content_consumed and isinstance(self._content, bool):
            raise StreamConsumedError()
        elif chunk_size is not None and not isinstance(chunk_size, int):
            raise TypeError(
                f"chunk_size must be an int, it is instead a {type(chunk_size)}."
            )
        # simulate reading small chunks of the content
        reused_chunks = iter_slices(self._content, chunk_size)

        stream_chunks = generate()

        chunks = reused_chunks if self._content_consumed else stream_chunks

        if decode_unicode:
            chunks = stream_decode_response_unicode(chunks, self)

        return chunks

    def iter_lines(
        self, chunk_size=ITER_CHUNK_SIZE, decode_unicode=False, delimiter=None
    ):
        """Iterates over the response data, one line at a time.  When
        stream=True is set on the request, this avoids reading the
        content at once into memory for large responses.

        .. note:: This method is not reentrant safe.
        """

        pending = None

        for chunk in self.iter_content(
            chunk_size=chunk_size, decode_unicode=decode_unicode
        ):
            if pending is not None:
                chunk = pending + chunk

            if delimiter:
                lines = chunk.split(delimiter)
            else:
                lines = chunk.splitlines()

            if lines and lines[-1] and chunk and lines[-1][-1] == chunk[-1]:
                pending = lines.pop()
            else:
                pending = None

            yield from lines

        if pending is not None:
            yield pending

    @property
    def content(self):
        """Content of the response, in bytes."""

        if self._content is False:
            # Read the contents.
            if self._content_consumed:
                raise RuntimeError("The content for this response was already consumed")

            if self.status_code == 0 or self.raw is None:
                self._content = None
            else:
                self._content = b"".join(self.iter_content(CONTENT_CHUNK_SIZE)) or b""

        self._content_consumed = True
        # don't need to release the connection; that's been handled by urllib3
        # since we exhausted the data.
        return self._content

    @property
    def text(self):
        """Content of the response, in unicode.

        If Response.encoding is None, encoding will be guessed using
        ``charset_normalizer`` or ``chardet``.

        The encoding of the response content is determined based solely on HTTP
        headers, following RFC 2616 to the letter. If you can take advantage of
        non-HTTP knowledge to make a better guess at the encoding, you should
        set ``r.encoding`` appropriately before accessing this property.
        """

        # Try charset from content-type
        content = None
        encoding = self.encoding

        if not self.content:
            return ""

        # Fallback to auto-detected encoding.
        if self.encoding is None:
            encoding = self.apparent_encoding

        # Decode unicode from given encoding.
        try:
            content = str(self.content, encoding, errors="replace")
        except (LookupError, TypeError):
            # A LookupError is raised if the encoding was not found which could
            # indicate a misspelling or similar mistake.
            #
            # A TypeError can be raised if encoding is None
            #
            # So we try blindly encoding.
            content = str(self.content, errors="replace")

        return content

    def json(self, **kwargs):
        r"""Decodes the JSON response body (if any) as a Python object.

        This may return a dictionary, list, etc. depending on what is in the response.

        :param \*\*kwargs: Optional arguments that ``json.loads`` takes.
        :raises requests.exceptions.JSONDecodeError: If the response body does not
            contain valid json.
        """

        if not self.encoding and self.content and len(self.content) > 3:
            # No encoding set. JSON RFC 4627 section 3 states we should expect
            # UTF-8, -16 or -32. Detect which one to use; If the detection or
            # decoding fails, fall back to `self.text` (using charset_normalizer to make
            # a best guess).
            encoding = guess_json_utf(self.content)
            if encoding is not None:
                try:
                    return complexjson.loads(self.content.decode(encoding), **kwargs)
                except UnicodeDecodeError:
                    # Wrong UTF codec detected; usually because it's not UTF-8
                    # but some other 8-bit codec.  This is an RFC violation,
                    # and the server didn't bother to tell us what codec *was*
                    # used.
                    pass
                except JSONDecodeError as e:
                    raise RequestsJSONDecodeError(e.msg, e.doc, e.pos)

        try:
            return complexjson.loads(self.text, **kwargs)
        except JSONDecodeError as e:
            # Catch JSON-related errors and raise as requests.JSONDecodeError
            # This aliases json.JSONDecodeError and simplejson.JSONDecodeError
            raise RequestsJSONDecodeError(e.msg, e.doc, e.pos)

    @property
    def links(self):
        """Returns the parsed header links of the response, if any."""

        header = self.headers.get("link")

        resolved_links = {}

        if header:
            links = parse_header_links(header)

            for link in links:
                key = link.get("rel") or link.get("url")
                resolved_links[key] = link

        return resolved_links

    def raise_for_status(self):
        """Raises :class:`HTTPError`, if one occurred."""

        http_error_msg = ""
        if isinstance(self.reason, bytes):
            # We attempt to decode utf-8 first because some servers
            # choose to localize their reason strings. If the string
            # isn't utf-8, we fall back to iso-8859-1 for all other
            # encodings. (See PR #3538)
            try:
                reason = self.reason.decode("utf-8")
            except UnicodeDecodeError:
                reason = self.reason.decode("iso-8859-1")
        else:
            reason = self.reason

        if 400 <= self.status_code < 500:
            http_error_msg = (
                f"{self.status_code} Client Error: {reason} for url: {self.url}"
            )

        elif 500 <= self.status_code < 600:
            http_error_msg = (
                f"{self.status_code} Server Error: {reason} for url: {self.url}"
            )

        if http_error_msg:
            raise HTTPError(http_error_msg, response=self)

    def close(self):
        """Releases the connection back to the pool. Once this method has been
        called the underlying ``raw`` object must not be accessed again.

        *Note: Should not normally need to be called explicitly.*
        """
        if not self._content_consumed:
            self.raw.close()

        release_conn = getattr(self.raw, "release_conn", None)
        if release_conn is not None:
            release_conn()
```


#### Issue #5: Component 'caller' has 163 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/models.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #6: God Object detected: 'SessionRedirectMixin' has 6 methods and 43 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 2 complex

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: Critical
- **Location**: Lines 106-106

**Code Snippet**:

```
class SessionRedirectMixin:
    def get_redirect_target(self, resp):
        """Receives a Response. Returns a redirect URI or ``None``"""
        # Due to the nature of how requests processes redirects this method will
        # be called at least once upon the original response and at least twice
        # on each subsequent redirect response (if any).
        # If a custom mixin is used to handle this logic, it may be advantageous
        # to cache the redirect location onto the response object as a private
        # attribute.
        if resp.is_redirect:
            location = resp.headers["location"]
            # Currently the underlying http module on py3 decode headers
            # in latin1, but empirical evidence suggests that latin1 is very
            # rarely used with non-ASCII characters in HTTP headers.
            # It is more likely to get UTF8 header rather than latin1.
            # This causes incorrect handling of UTF8 encoded location headers.
            # To solve this, we re-encode the location in latin1.
            location = location.encode("latin1")
            return to_native_string(location, "utf8")
        return None

    def should_strip_auth(self, old_url, new_url):
        """Decide whether Authorization header should be removed when redirecting"""
        old_parsed = urlparse(old_url)
        new_parsed = urlparse(new_url)
        if old_parsed.hostname != new_parsed.hostname:
            return True
        # Special case: allow http -> https redirect when using the standard
        # ports. This isn't specified by RFC 7235, but is kept to avoid
        # breaking backwards compatibility with older versions of requests
        # that allowed any redirects on the same host.
        if (
            old_parsed.scheme == "http"
            and old_parsed.port in (80, None)
            and new_parsed.scheme == "https"
            and new_parsed.port in (443, None)
        ):
            return False

        # Handle default port usage corresponding to scheme.
        changed_port = old_parsed.port != new_parsed.port
        changed_scheme = old_parsed.scheme != new_parsed.scheme
        default_port = (DEFAULT_PORTS.get(old_parsed.scheme, None), None)
        if (
            not changed_scheme
            and old_parsed.port in default_port
            and new_parsed.port in default_port
        ):
            return False

        # Standard case: root URI must match
        return changed_port or changed_scheme

    def resolve_redirects(
        self,
        resp,
        req,
        stream=False,
        timeout=None,
        verify=True,
        cert=None,
        proxies=None,
        yield_requests=False,
        **adapter_kwargs,
    ):
        """Receives a Response. Returns a generator of Responses or Requests."""

        hist = []  # keep track of history

        url = self.get_redirect_target(resp)
        previous_fragment = urlparse(req.url).fragment
        while url:
            prepared_request = req.copy()

            # Update history and keep track of redirects.
            # resp.history must ignore the original request in this loop
            hist.append(resp)
            resp.history = hist[1:]

            try:
                resp.content  # Consume socket so it can be released
            except (ChunkedEncodingError, ContentDecodingError, RuntimeError):
                resp.raw.read(decode_content=False)

            if len(resp.history) >= self.max_redirects:
                raise TooManyRedirects(
                    f"Exceeded {self.max_redirects} redirects.", response=resp
                )

            # Release the connection back into the pool.
            resp.close()

            # Handle redirection without scheme (see: RFC 1808 Section 4)
            if url.startswith("//"):
                parsed_rurl = urlparse(resp.url)
                url = ":".join([to_native_string(parsed_rurl.scheme), url])

            # Normalize url case and attach previous fragment if needed (RFC 7231 7.1.2)
            parsed = urlparse(url)
            if parsed.fragment == "" and previous_fragment:
                parsed = parsed._replace(fragment=previous_fragment)
            elif parsed.fragment:
                previous_fragment = parsed.fragment
            url = parsed.geturl()

            # Facilitate relative 'location' headers, as allowed by RFC 7231.
            # (e.g. '/path/to/resource' instead of 'http://domain.tld/path/to/resource')
            # Compliant with RFC3986, we percent encode the url.
            if not parsed.netloc:
                url = urljoin(resp.url, requote_uri(url))
            else:
                url = requote_uri(url)

            prepared_request.url = to_native_string(url)

            self.rebuild_method(prepared_request, resp)

            # https://github.com/psf/requests/issues/1084
            if resp.status_code not in (
                codes.temporary_redirect,
                codes.permanent_redirect,
            ):
                # https://github.com/psf/requests/issues/3490
                purged_headers = ("Content-Length", "Content-Type", "Transfer-Encoding")
                for header in purged_headers:
                    prepared_request.headers.pop(header, None)
                prepared_request.body = None

            headers = prepared_request.headers
            headers.pop("Cookie", None)

            # Extract any cookies sent on the response to the cookiejar
            # in the new request. Because we've mutated our copied prepared
            # request, use the old one that we haven't yet touched.
            extract_cookies_to_jar(prepared_request._cookies, req, resp.raw)
            merge_cookies(prepared_request._cookies, self.cookies)
            prepared_request.prepare_cookies(prepared_request._cookies)

            # Rebuild auth and proxy information.
            proxies = self.rebuild_proxies(prepared_request, proxies)
            self.rebuild_auth(prepared_request, resp)

            # A failed tell() sets `_body_position` to `object()`. This non-None
            # value ensures `rewindable` will be True, allowing us to raise an
            # UnrewindableBodyError, instead of hanging the connection.
            rewindable = prepared_request._body_position is not None and (
                "Content-Length" in headers or "Transfer-Encoding" in headers
            )

            # Attempt to rewind consumed file-like object.
            if rewindable:
                rewind_body(prepared_request)

            # Override the original request.
            req = prepared_request

            if yield_requests:
                yield req
            else:
                resp = self.send(
                    req,
                    stream=stream,
                    timeout=timeout,
                    verify=verify,
                    cert=cert,
                    proxies=proxies,
                    allow_redirects=False,
                    **adapter_kwargs,
                )

                extract_cookies_to_jar(self.cookies, prepared_request, resp.raw)

                # extract redirect url, if any, for the next loop
                url = self.get_redirect_target(resp)
                yield resp

    def rebuild_auth(self, prepared_request, response):
        """When being redirected we may want to strip authentication from the
        request to avoid leaking credentials. This method intelligently removes
        and reapplies authentication where possible to avoid credential loss.
        """
        headers = prepared_request.headers
        url = prepared_request.url

        if "Authorization" in headers and self.should_strip_auth(
            response.request.url, url
        ):
            # If we get redirected to a new host, we should strip out any
            # authentication headers.
            del headers["Authorization"]

        # .netrc might have more auth for us on our new host.
        new_auth = get_netrc_auth(url) if self.trust_env else None
        if new_auth is not None:
            prepared_request.prepare_auth(new_auth)

    def rebuild_proxies(self, prepared_request, proxies):
        """This method re-evaluates the proxy configuration by considering the
        environment variables. If we are redirected to a URL covered by
        NO_PROXY, we strip the proxy configuration. Otherwise, we set missing
        proxy keys for this URL (in case they were stripped by a previous
        redirect).

        This method also replaces the Proxy-Authorization header where
        necessary.

        :rtype: dict
        """
        headers = prepared_request.headers
        scheme = urlparse(prepared_request.url).scheme
        new_proxies = resolve_proxies(prepared_request, proxies, self.trust_env)

        if "Proxy-Authorization" in headers:
            del headers["Proxy-Authorization"]

        try:
            username, password = get_auth_from_url(new_proxies[scheme])
        except KeyError:
            username, password = None, None

        # urllib3 handles proxy authorization for us in the standard adapter.
        # Avoid appending this to TLS tunneled requests where it may be leaked.
        if not scheme.startswith("https") and username and password:
            headers["Proxy-Authorization"] = _basic_auth_str(username, password)

        return new_proxies

    def rebuild_method(self, prepared_request, response):
        """When being redirected we may want to change the method of the request
        based on certain specs or browser behavior.
        """
        method = prepared_request.method

        # https://tools.ietf.org/html/rfc7231#section-6.4.4
        if response.status_code == codes.see_other and method != "HEAD":
            method = "GET"

        # Do what the browsers do, despite standards...
        # First, turn 302s into GETs.
        if response.status_code == codes.found and method != "HEAD":
            method = "GET"

        # Second, if a POST is responded to with a 301, turn it into a GET.
        # This bizarre behaviour is explained in Issue 1704.
        if response.status_code == codes.moved and method == "POST":
            method = "GET"

        prepared_request.method = method
```


#### Issue #7: God Object detected: 'Session' has 19 methods and 52 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 13 trivial, 6 complex

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: Critical
- **Location**: Lines 356-356

**Code Snippet**:

```
class Session(SessionRedirectMixin):
    """A Requests session.

    Provides cookie persistence, connection-pooling, and configuration.

    Basic Usage::

      >>> import requests
      >>> s = requests.Session()
      >>> s.get('https://httpbin.org/get')
      <Response [200]>

    Or as a context manager::

      >>> with requests.Session() as s:
      ...     s.get('https://httpbin.org/get')
      <Response [200]>
    """

    __attrs__ = [
        "headers",
        "cookies",
        "auth",
        "proxies",
        "hooks",
        "params",
        "verify",
        "cert",
        "adapters",
        "stream",
        "trust_env",
        "max_redirects",
    ]

    def __init__(self):
        #: A case-insensitive dictionary of headers to be sent on each
        #: :class:`Request <Request>` sent from this
        #: :class:`Session <Session>`.
        self.headers = default_headers()

        #: Default Authentication tuple or object to attach to
        #: :class:`Request <Request>`.
        self.auth = None

        #: Dictionary mapping protocol or protocol and host to the URL of the proxy
        #: (e.g. {'http': 'foo.bar:3128', 'http://host.name': 'foo.bar:4012'}) to
        #: be used on each :class:`Request <Request>`.
        self.proxies = {}

        #: Event-handling hooks.
        self.hooks = default_hooks()

        #: Dictionary of querystring data to attach to each
        #: :class:`Request <Request>`. The dictionary values may be lists for
        #: representing multivalued query parameters.
        self.params = {}

        #: Stream response content default.
        self.stream = False

        #: SSL Verification default.
        #: Defaults to `True`, requiring requests to verify the TLS certificate at the
        #: remote end.
        #: If verify is set to `False`, requests will accept any TLS certificate
        #: presented by the server, and will ignore hostname mismatches and/or
        #: expired certificates, which will make your application vulnerable to
        #: man-in-the-middle (MitM) attacks.
        #: Only set this to `False` for testing.
        self.verify = True

        #: SSL client certificate default, if String, path to ssl client
        #: cert file (.pem). If Tuple, ('cert', 'key') pair.
        self.cert = None

        #: Maximum number of redirects allowed. If the request exceeds this
        #: limit, a :class:`TooManyRedirects` exception is raised.
        #: This defaults to requests.models.DEFAULT_REDIRECT_LIMIT, which is
        #: 30.
        self.max_redirects = DEFAULT_REDIRECT_LIMIT

        #: Trust environment settings for proxy configuration, default
        #: authentication and similar.
        self.trust_env = True

        #: A CookieJar containing all currently outstanding cookies set on this
        #: session. By default it is a
        #: :class:`RequestsCookieJar <requests.cookies.RequestsCookieJar>`, but
        #: may be any other ``cookielib.CookieJar`` compatible object.
        self.cookies = cookiejar_from_dict({})

        # Default connection adapters.
        self.adapters = OrderedDict()
        self.mount("https://", HTTPAdapter())
        self.mount("http://", HTTPAdapter())

    def __enter__(self):
        return self

    def __exit__(self, *args):
        self.close()

    def prepare_request(self, request):
        """Constructs a :class:`PreparedRequest <PreparedRequest>` for
        transmission and returns it. The :class:`PreparedRequest` has settings
        merged from the :class:`Request <Request>` instance and those of the
        :class:`Session`.

        :param request: :class:`Request` instance to prepare with this
            session's settings.
        :rtype: requests.PreparedRequest
        """
        cookies = request.cookies or {}

        # Bootstrap CookieJar.
        if not isinstance(cookies, cookielib.CookieJar):
            cookies = cookiejar_from_dict(cookies)

        # Merge with session cookies
        merged_cookies = merge_cookies(
            merge_cookies(RequestsCookieJar(), self.cookies), cookies
        )

        # Set environment's basic authentication if not explicitly set.
        auth = request.auth
        if self.trust_env and not auth and not self.auth:
            auth = get_netrc_auth(request.url)

        p = PreparedRequest()
        p.prepare(
            method=request.method.upper(),
            url=request.url,
            files=request.files,
            data=request.data,
            json=request.json,
            headers=merge_setting(
                request.headers, self.headers, dict_class=CaseInsensitiveDict
            ),
            params=merge_setting(request.params, self.params),
            auth=merge_setting(auth, self.auth),
            cookies=merged_cookies,
            hooks=merge_hooks(request.hooks, self.hooks),
        )
        return p

    def request(
        self,
        method,
        url,
        params=None,
        data=None,
        headers=None,
        cookies=None,
        files=None,
        auth=None,
        timeout=None,
        allow_redirects=True,
        proxies=None,
        hooks=None,
        stream=None,
        verify=None,
        cert=None,
        json=None,
    ):
        """Constructs a :class:`Request <Request>`, prepares it and sends it.
        Returns :class:`Response <Response>` object.

        :param method: method for the new :class:`Request` object.
        :param url: URL for the new :class:`Request` object.
        :param params: (optional) Dictionary or bytes to be sent in the query
            string for the :class:`Request`.
        :param data: (optional) Dictionary, list of tuples, bytes, or file-like
            object to send in the body of the :class:`Request`.
        :param json: (optional) json to send in the body of the
            :class:`Request`.
        :param headers: (optional) Dictionary of HTTP Headers to send with the
            :class:`Request`.
        :param cookies: (optional) Dict or CookieJar object to send with the
            :class:`Request`.
        :param files: (optional) Dictionary of ``'filename': file-like-objects``
            for multipart encoding upload.
        :param auth: (optional) Auth tuple or callable to enable
            Basic/Digest/Custom HTTP Auth.
        :param timeout: (optional) How many seconds to wait for the server to send
            data before giving up, as a float, or a :ref:`(connect timeout,
            read timeout) <timeouts>` tuple.
        :type timeout: float or tuple
        :param allow_redirects: (optional) Set to True by default.
        :type allow_redirects: bool
        :param proxies: (optional) Dictionary mapping protocol or protocol and
            hostname to the URL of the proxy.
        :param hooks: (optional) Dictionary mapping hook name to one event or
            list of events, event must be callable.
        :param stream: (optional) whether to immediately download the response
            content. Defaults to ``False``.
        :param verify: (optional) Either a boolean, in which case it controls whether we verify
            the server's TLS certificate, or a string, in which case it must be a path
            to a CA bundle to use. Defaults to ``True``. When set to
            ``False``, requests will accept any TLS certificate presented by
            the server, and will ignore hostname mismatches and/or expired
            certificates, which will make your application vulnerable to
            man-in-the-middle (MitM) attacks. Setting verify to ``False``
            may be useful during local development or testing.
        :param cert: (optional) if String, path to ssl client cert file (.pem).
            If Tuple, ('cert', 'key') pair.
        :rtype: requests.Response
        """
        # Create the Request.
        req = Request(
            method=method.upper(),
            url=url,
            headers=headers,
            files=files,
            data=data or {},
            json=json,
            params=params or {},
            auth=auth,
            cookies=cookies,
            hooks=hooks,
        )
        prep = self.prepare_request(req)

        proxies = proxies or {}

        settings = self.merge_environment_settings(
            prep.url, proxies, stream, verify, cert
        )

        # Send the request.
        send_kwargs = {
            "timeout": timeout,
            "allow_redirects": allow_redirects,
        }
        send_kwargs.update(settings)
        resp = self.send(prep, **send_kwargs)

        return resp

    def get(self, url, **kwargs):
        r"""Sends a GET request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        kwargs.setdefault("allow_redirects", True)
        return self.request("GET", url, **kwargs)

    def options(self, url, **kwargs):
        r"""Sends a OPTIONS request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        kwargs.setdefault("allow_redirects", True)
        return self.request("OPTIONS", url, **kwargs)

    def head(self, url, **kwargs):
        r"""Sends a HEAD request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        kwargs.setdefault("allow_redirects", False)
        return self.request("HEAD", url, **kwargs)

    def post(self, url, data=None, json=None, **kwargs):
        r"""Sends a POST request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param data: (optional) Dictionary, list of tuples, bytes, or file-like
            object to send in the body of the :class:`Request`.
        :param json: (optional) json to send in the body of the :class:`Request`.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        return self.request("POST", url, data=data, json=json, **kwargs)

    def put(self, url, data=None, **kwargs):
        r"""Sends a PUT request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param data: (optional) Dictionary, list of tuples, bytes, or file-like
            object to send in the body of the :class:`Request`.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        return self.request("PUT", url, data=data, **kwargs)

    def patch(self, url, data=None, **kwargs):
        r"""Sends a PATCH request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param data: (optional) Dictionary, list of tuples, bytes, or file-like
            object to send in the body of the :class:`Request`.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        return self.request("PATCH", url, data=data, **kwargs)

    def delete(self, url, **kwargs):
        r"""Sends a DELETE request. Returns :class:`Response` object.

        :param url: URL for the new :class:`Request` object.
        :param \*\*kwargs: Optional arguments that ``request`` takes.
        :rtype: requests.Response
        """

        return self.request("DELETE", url, **kwargs)

    def send(self, request, **kwargs):
        """Send a given PreparedRequest.

        :rtype: requests.Response
        """
        # Set defaults that the hooks can utilize to ensure they always have
        # the correct parameters to reproduce the previous request.
        kwargs.setdefault("stream", self.stream)
        kwargs.setdefault("verify", self.verify)
        kwargs.setdefault("cert", self.cert)
        if "proxies" not in kwargs:
            kwargs["proxies"] = resolve_proxies(request, self.proxies, self.trust_env)

        # It's possible that users might accidentally send a Request object.
        # Guard against that specific failure case.
        if isinstance(request, Request):
            raise ValueError("You can only send PreparedRequests.")

        # Set up variables needed for resolve_redirects and dispatching of hooks
        allow_redirects = kwargs.pop("allow_redirects", True)
        stream = kwargs.get("stream")
        hooks = request.hooks

        # Get the appropriate adapter to use
        adapter = self.get_adapter(url=request.url)

        # Start time (approximately) of the request
        start = preferred_clock()

        # Send the request
        r = adapter.send(request, **kwargs)

        # Total elapsed time of the request (approximately)
        elapsed = preferred_clock() - start
        r.elapsed = timedelta(seconds=elapsed)

        # Response manipulation hooks
        r = dispatch_hook("response", hooks, r, **kwargs)

        # Persist cookies
        if r.history:
            # If the hooks create history then we want those cookies too
            for resp in r.history:
                extract_cookies_to_jar(self.cookies, resp.request, resp.raw)

        extract_cookies_to_jar(self.cookies, request, r.raw)

        # Resolve redirects if allowed.
        if allow_redirects:
            # Redirect resolving generator.
            gen = self.resolve_redirects(r, request, **kwargs)
            history = [resp for resp in gen]
        else:
            history = []

        # Shuffle things around if there's history.
        if history:
            # Insert the first (original) request at the start
            history.insert(0, r)
            # Get the last request made
            r = history.pop()
            r.history = history

        # If redirects aren't being followed, store the response on the Request for Response.next().
        if not allow_redirects:
            try:
                r._next = next(
                    self.resolve_redirects(r, request, yield_requests=True, **kwargs)
                )
            except StopIteration:
                pass

        if not stream:
            r.content

        return r

    def merge_environment_settings(self, url, proxies, stream, verify, cert):
        """
        Check the environment and merge it with some settings.

        :rtype: dict
        """
        # Gather clues from the surrounding environment.
        if self.trust_env:
            # Set environment's proxies.
            no_proxy = proxies.get("no_proxy") if proxies is not None else None
            env_proxies = get_environ_proxies(url, no_proxy=no_proxy)
            for k, v in env_proxies.items():
                proxies.setdefault(k, v)

            # Look for requests environment configuration
            # and be compatible with cURL.
            if verify is True or verify is None:
                verify = (
                    os.environ.get("REQUESTS_CA_BUNDLE")
                    or os.environ.get("CURL_CA_BUNDLE")
                    or verify
                )

        # Merge all the kwargs.
        proxies = merge_setting(proxies, self.proxies)
        stream = merge_setting(stream, self.stream)
        verify = merge_setting(verify, self.verify)
        cert = merge_setting(cert, self.cert)

        return {"proxies": proxies, "stream": stream, "verify": verify, "cert": cert}

    def get_adapter(self, url):
        """
        Returns the appropriate connection adapter for the given URL.

        :rtype: requests.adapters.BaseAdapter
        """
        for prefix, adapter in self.adapters.items():
            if url.lower().startswith(prefix.lower()):
                return adapter

        # Nothing matches :-/
        raise InvalidSchema(f"No connection adapters were found for {url!r}")

    def close(self):
        """Closes all adapters and as such the session"""
        for v in self.adapters.values():
            v.close()

    def mount(self, prefix, adapter):
        """Registers a connection adapter to a prefix.

        Adapters are sorted in descending order by prefix length.
        """
        self.adapters[prefix] = adapter
        keys_to_move = [k for k in self.adapters if len(k) < len(prefix)]

        for key in keys_to_move:
            self.adapters[key] = self.adapters.pop(key)

    def __getstate__(self):
        state = {attr: getattr(self, attr, None) for attr in self.__attrs__}
        return state

    def __setstate__(self, state):
        for attr, value in state.items():
            setattr(self, attr, value)
```


#### Issue #8: Component 'caller' has 127 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/sessions.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #9: God Object detected: 'CaseInsensitiveDict' has 10 methods and 4 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 7 trivial, 3 complex

- **File**: `alpha_testing/requests/src/requests/structures.py`
- **Severity**: High
- **Location**: Lines 13-13

**Code Snippet**:

```
class CaseInsensitiveDict(MutableMapping):
    """A case-insensitive ``dict``-like object.

    Implements all methods and operations of
    ``MutableMapping`` as well as dict's ``copy``. Also
    provides ``lower_items``.

    All keys are expected to be strings. The structure remembers the
    case of the last key to be set, and ``iter(instance)``,
    ``keys()``, ``items()``, ``iterkeys()``, and ``iteritems()``
    will contain case-sensitive keys. However, querying and contains
    testing is case insensitive::

        cid = CaseInsensitiveDict()
        cid['Accept'] = 'application/json'
        cid['aCCEPT'] == 'application/json'  # True
        list(cid) == ['Accept']  # True

    For example, ``headers['content-encoding']`` will return the
    value of a ``'Content-Encoding'`` response header, regardless
    of how the header name was originally stored.

    If the constructor, ``.update``, or equality comparison
    operations are given keys that have equal ``.lower()``s, the
    behavior is undefined.
    """

    def __init__(self, data=None, **kwargs):
        self._store = OrderedDict()
        if data is None:
            data = {}
        self.update(data, **kwargs)

    def __setitem__(self, key, value):
        # Use the lowercased key for lookups, but store the actual
        # key alongside the value.
        self._store[key.lower()] = (key, value)

    def __getitem__(self, key):
        return self._store[key.lower()][1]

    def __delitem__(self, key):
        del self._store[key.lower()]

    def __iter__(self):
        return (casedkey for casedkey, mappedvalue in self._store.values())

    def __len__(self):
        return len(self._store)

    def lower_items(self):
        """Like iteritems(), but with all lowercase keys."""
        return ((lowerkey, keyval[1]) for (lowerkey, keyval) in self._store.items())

    def __eq__(self, other):
        if isinstance(other, Mapping):
            other = CaseInsensitiveDict(other)
        else:
            return NotImplemented
        # Compare insensitively
        return dict(self.lower_items()) == dict(other.lower_items())

    # Copy is required
    def copy(self):
        return CaseInsensitiveDict(self._store.values())

    def __repr__(self):
        return str(dict(self.items()))
```


#### Issue #10: Component 'caller' has 17 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/structures.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #11: Component 'caller' has 28 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/__init__.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #12: Component 'caller' has 174 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/utils.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #13: God Object detected: 'MockRequest' has 14 methods and 6 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 9 trivial, 5 complex

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: Critical
- **Location**: Lines 23-23

**Code Snippet**:

```
class MockRequest:
    """Wraps a `requests.Request` to mimic a `urllib2.Request`.

    The code in `http.cookiejar.CookieJar` expects this interface in order to correctly
    manage cookie policies, i.e., determine whether a cookie can be set, given the
    domains of the request and the cookie.

    The original request object is read-only. The client is responsible for collecting
    the new headers via `get_new_headers()` and interpreting them appropriately. You
    probably want `get_cookie_header`, defined below.
    """

    def __init__(self, request):
        self._r = request
        self._new_headers = {}
        self.type = urlparse(self._r.url).scheme

    def get_type(self):
        return self.type

    def get_host(self):
        return urlparse(self._r.url).netloc

    def get_origin_req_host(self):
        return self.get_host()

    def get_full_url(self):
        # Only return the response's URL if the user hadn't set the Host
        # header
        if not self._r.headers.get("Host"):
            return self._r.url
        # If they did set it, retrieve it and reconstruct the expected domain
        host = to_native_string(self._r.headers["Host"], encoding="utf-8")
        parsed = urlparse(self._r.url)
        # Reconstruct the URL as we expect it
        return urlunparse(
            [
                parsed.scheme,
                host,
                parsed.path,
                parsed.params,
                parsed.query,
                parsed.fragment,
            ]
        )

    def is_unverifiable(self):
        return True

    def has_header(self, name):
        return name in self._r.headers or name in self._new_headers

    def get_header(self, name, default=None):
        return self._r.headers.get(name, self._new_headers.get(name, default))

    def add_header(self, key, val):
        """cookiejar has no legitimate use for this method; add it back if you find one."""
        raise NotImplementedError(
            "Cookie headers should be added with add_unredirected_header()"
        )

    def add_unredirected_header(self, name, value):
        self._new_headers[name] = value

    def get_new_headers(self):
        return self._new_headers

    @property
    def unverifiable(self):
        return self.is_unverifiable()

    @property
    def origin_req_host(self):
        return self.get_origin_req_host()

    @property
    def host(self):
        return self.get_host()
```


#### Issue #14: God Object detected: 'RequestsCookieJar' has 24 methods and 13 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 16 trivial, 8 complex

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: Critical
- **Location**: Lines 176-176

**Code Snippet**:

```
class RequestsCookieJar(cookielib.CookieJar, MutableMapping):
    """Compatibility class; is a http.cookiejar.CookieJar, but exposes a dict
    interface.

    This is the CookieJar we create by default for requests and sessions that
    don't specify one, since some clients may expect response.cookies and
    session.cookies to support dict operations.

    Requests does not use the dict interface internally; it's just for
    compatibility with external client code. All requests code should work
    out of the box with externally provided instances of ``CookieJar``, e.g.
    ``LWPCookieJar`` and ``FileCookieJar``.

    Unlike a regular CookieJar, this class is pickleable.

    .. warning:: dictionary operations that are normally O(1) may be O(n).
    """

    def get(self, name, default=None, domain=None, path=None):
        """Dict-like get() that also supports optional domain and path args in
        order to resolve naming collisions from using one cookie jar over
        multiple domains.

        .. warning:: operation is O(n), not O(1).
        """
        try:
            return self._find_no_duplicates(name, domain, path)
        except KeyError:
            return default

    def set(self, name, value, **kwargs):
        """Dict-like set() that also supports optional domain and path args in
        order to resolve naming collisions from using one cookie jar over
        multiple domains.
        """
        # support client code that unsets cookies by assignment of a None value:
        if value is None:
            remove_cookie_by_name(
                self, name, domain=kwargs.get("domain"), path=kwargs.get("path")
            )
            return

        if isinstance(value, Morsel):
            c = morsel_to_cookie(value)
        else:
            c = create_cookie(name, value, **kwargs)
        self.set_cookie(c)
        return c

    def iterkeys(self):
        """Dict-like iterkeys() that returns an iterator of names of cookies
        from the jar.

        .. seealso:: itervalues() and iteritems().
        """
        for cookie in iter(self):
            yield cookie.name

    def keys(self):
        """Dict-like keys() that returns a list of names of cookies from the
        jar.

        .. seealso:: values() and items().
        """
        return list(self.iterkeys())

    def itervalues(self):
        """Dict-like itervalues() that returns an iterator of values of cookies
        from the jar.

        .. seealso:: iterkeys() and iteritems().
        """
        for cookie in iter(self):
            yield cookie.value

    def values(self):
        """Dict-like values() that returns a list of values of cookies from the
        jar.

        .. seealso:: keys() and items().
        """
        return list(self.itervalues())

    def iteritems(self):
        """Dict-like iteritems() that returns an iterator of name-value tuples
        from the jar.

        .. seealso:: iterkeys() and itervalues().
        """
        for cookie in iter(self):
            yield cookie.name, cookie.value

    def items(self):
        """Dict-like items() that returns a list of name-value tuples from the
        jar. Allows client-code to call ``dict(RequestsCookieJar)`` and get a
        vanilla python dict of key value pairs.

        .. seealso:: keys() and values().
        """
        return list(self.iteritems())

    def list_domains(self):
        """Utility method to list all the domains in the jar."""
        domains = []
        for cookie in iter(self):
            if cookie.domain not in domains:
                domains.append(cookie.domain)
        return domains

    def list_paths(self):
        """Utility method to list all the paths in the jar."""
        paths = []
        for cookie in iter(self):
            if cookie.path not in paths:
                paths.append(cookie.path)
        return paths

    def multiple_domains(self):
        """Returns True if there are multiple domains in the jar.
        Returns False otherwise.

        :rtype: bool
        """
        domains = []
        for cookie in iter(self):
            if cookie.domain is not None and cookie.domain in domains:
                return True
            domains.append(cookie.domain)
        return False  # there is only one domain in jar

    def get_dict(self, domain=None, path=None):
        """Takes as an argument an optional domain and path and returns a plain
        old Python dict of name-value pairs of cookies that meet the
        requirements.

        :rtype: dict
        """
        dictionary = {}
        for cookie in iter(self):
            if (domain is None or cookie.domain == domain) and (
                path is None or cookie.path == path
            ):
                dictionary[cookie.name] = cookie.value
        return dictionary

    def __contains__(self, name):
        try:
            return super().__contains__(name)
        except CookieConflictError:
            return True

    def __getitem__(self, name):
        """Dict-like __getitem__() for compatibility with client code. Throws
        exception if there are more than one cookie with name. In that case,
        use the more explicit get() method instead.

        .. warning:: operation is O(n), not O(1).
        """
        return self._find_no_duplicates(name)

    def __setitem__(self, name, value):
        """Dict-like __setitem__ for compatibility with client code. Throws
        exception if there is already a cookie of that name in the jar. In that
        case, use the more explicit set() method instead.
        """
        self.set(name, value)

    def __delitem__(self, name):
        """Deletes a cookie given a name. Wraps ``http.cookiejar.CookieJar``'s
        ``remove_cookie_by_name()``.
        """
        remove_cookie_by_name(self, name)

    def set_cookie(self, cookie, *args, **kwargs):
        if (
            hasattr(cookie.value, "startswith")
            and cookie.value.startswith('"')
            and cookie.value.endswith('"')
        ):
            cookie.value = cookie.value.replace('\\"', "")
        return super().set_cookie(cookie, *args, **kwargs)

    def update(self, other):
        """Updates this jar with cookies from another CookieJar or dict-like"""
        if isinstance(other, cookielib.CookieJar):
            for cookie in other:
                self.set_cookie(copy.copy(cookie))
        else:
            super().update(other)

    def _find(self, name, domain=None, path=None):
        """Requests uses this method internally to get cookie values.

        If there are conflicting cookies, _find arbitrarily chooses one.
        See _find_no_duplicates if you want an exception thrown if there are
        conflicting cookies.

        :param name: a string containing name of cookie
        :param domain: (optional) string containing domain of cookie
        :param path: (optional) string containing path of cookie
        :return: cookie.value
        """
        for cookie in iter(self):
            if cookie.name == name:
                if domain is None or cookie.domain == domain:
                    if path is None or cookie.path == path:
                        return cookie.value

        raise KeyError(f"name={name!r}, domain={domain!r}, path={path!r}")

    def _find_no_duplicates(self, name, domain=None, path=None):
        """Both ``__get_item__`` and ``get`` call this function: it's never
        used elsewhere in Requests.

        :param name: a string containing name of cookie
        :param domain: (optional) string containing domain of cookie
        :param path: (optional) string containing path of cookie
        :raises KeyError: if cookie is not found
        :raises CookieConflictError: if there are multiple cookies
            that match name and optionally domain and path
        :return: cookie.value
        """
        toReturn = None
        for cookie in iter(self):
            if cookie.name == name:
                if domain is None or cookie.domain == domain:
                    if path is None or cookie.path == path:
                        if toReturn is not None:
                            # if there are multiple cookies that meet passed in criteria
                            raise CookieConflictError(
                                f"There are multiple cookies with name, {name!r}"
                            )
                        # we will eventually return this as long as no cookie conflict
                        toReturn = cookie.value

        if toReturn:
            return toReturn
        raise KeyError(f"name={name!r}, domain={domain!r}, path={path!r}")

    def __getstate__(self):
        """Unlike a normal CookieJar, this class is pickleable."""
        state = self.__dict__.copy()
        # remove the unpickleable RLock object
        state.pop("_cookies_lock")
        return state

    def __setstate__(self, state):
        """Unlike a normal CookieJar, this class is pickleable."""
        self.__dict__.update(state)
        if "_cookies_lock" not in self.__dict__:
            self._cookies_lock = threading.RLock()

    def copy(self):
        """Return a copy of this RequestsCookieJar."""
        new_cj = RequestsCookieJar()
        new_cj.set_policy(self.get_policy())
        new_cj.update(self)
        return new_cj

    def get_policy(self):
        """Return the CookiePolicy instance used."""
        return self._policy
```


#### Issue #15: Component 'caller' has 98 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/cookies.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #16: God Object detected: 'HTTPDigestAuth' has 12 methods and 57 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 8 trivial, 4 complex

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: Critical
- **Location**: Lines 107-107

**Code Snippet**:

```
class HTTPDigestAuth(AuthBase):
    """Attaches HTTP Digest Authentication to the given Request object."""

    def __init__(self, username, password):
        self.username = username
        self.password = password
        # Keep state in per-thread local storage
        self._thread_local = threading.local()

    def init_per_thread_state(self):
        # Ensure state is initialized just once per-thread
        if not hasattr(self._thread_local, "init"):
            self._thread_local.init = True
            self._thread_local.last_nonce = ""
            self._thread_local.nonce_count = 0
            self._thread_local.chal = {}
            self._thread_local.pos = None
            self._thread_local.num_401_calls = None

    def build_digest_header(self, method, url):
        """
        :rtype: str
        """

        realm = self._thread_local.chal["realm"]
        nonce = self._thread_local.chal["nonce"]
        qop = self._thread_local.chal.get("qop")
        algorithm = self._thread_local.chal.get("algorithm")
        opaque = self._thread_local.chal.get("opaque")
        hash_utf8 = None

        if algorithm is None:
            _algorithm = "MD5"
        else:
            _algorithm = algorithm.upper()
        # lambdas assume digest modules are imported at the top level
        if _algorithm == "MD5" or _algorithm == "MD5-SESS":

            def md5_utf8(x):
                if isinstance(x, str):
                    x = x.encode("utf-8")
                return hashlib.md5(x).hexdigest()

            hash_utf8 = md5_utf8
        elif _algorithm == "SHA":

            def sha_utf8(x):
                if isinstance(x, str):
                    x = x.encode("utf-8")
                return hashlib.sha1(x).hexdigest()

            hash_utf8 = sha_utf8
        elif _algorithm == "SHA-256":

            def sha256_utf8(x):
                if isinstance(x, str):
                    x = x.encode("utf-8")
                return hashlib.sha256(x).hexdigest()

            hash_utf8 = sha256_utf8
        elif _algorithm == "SHA-512":

            def sha512_utf8(x):
                if isinstance(x, str):
                    x = x.encode("utf-8")
                return hashlib.sha512(x).hexdigest()

            hash_utf8 = sha512_utf8

        KD = lambda s, d: hash_utf8(f"{s}:{d}")  # noqa:E731

        if hash_utf8 is None:
            return None

        # XXX not implemented yet
        entdig = None
        p_parsed = urlparse(url)
        #: path is request-uri defined in RFC 2616 which should not be empty
        path = p_parsed.path or "/"
        if p_parsed.query:
            path += f"?{p_parsed.query}"

        A1 = f"{self.username}:{realm}:{self.password}"
        A2 = f"{method}:{path}"

        HA1 = hash_utf8(A1)
        HA2 = hash_utf8(A2)

        if nonce == self._thread_local.last_nonce:
            self._thread_local.nonce_count += 1
        else:
            self._thread_local.nonce_count = 1
        ncvalue = f"{self._thread_local.nonce_count:08x}"
        s = str(self._thread_local.nonce_count).encode("utf-8")
        s += nonce.encode("utf-8")
        s += time.ctime().encode("utf-8")
        s += os.urandom(8)

        cnonce = hashlib.sha1(s).hexdigest()[:16]
        if _algorithm == "MD5-SESS":
            HA1 = hash_utf8(f"{HA1}:{nonce}:{cnonce}")

        if not qop:
            respdig = KD(HA1, f"{nonce}:{HA2}")
        elif qop == "auth" or "auth" in qop.split(","):
            noncebit = f"{nonce}:{ncvalue}:{cnonce}:auth:{HA2}"
            respdig = KD(HA1, noncebit)
        else:
            # XXX handle auth-int.
            return None

        self._thread_local.last_nonce = nonce

        # XXX should the partial digests be encoded too?
        base = (
            f'username="{self.username}", realm="{realm}", nonce="{nonce}", '
            f'uri="{path}", response="{respdig}"'
        )
        if opaque:
            base += f', opaque="{opaque}"'
        if algorithm:
            base += f', algorithm="{algorithm}"'
        if entdig:
            base += f', digest="{entdig}"'
        if qop:
            base += f', qop="auth", nc={ncvalue}, cnonce="{cnonce}"'

        return f"Digest {base}"

    def handle_redirect(self, r, **kwargs):
        """Reset num_401_calls counter on redirects."""
        if r.is_redirect:
            self._thread_local.num_401_calls = 1

    def handle_401(self, r, **kwargs):
        """
        Takes the given response and tries digest-auth, if needed.

        :rtype: requests.Response
        """

        # If response is not 4xx, do not auth
        # See https://github.com/psf/requests/issues/3772
        if not 400 <= r.status_code < 500:
            self._thread_local.num_401_calls = 1
            return r

        if self._thread_local.pos is not None:
            # Rewind the file position indicator of the body to where
            # it was to resend the request.
            r.request.body.seek(self._thread_local.pos)
        s_auth = r.headers.get("www-authenticate", "")

        if "digest" in s_auth.lower() and self._thread_local.num_401_calls < 2:
            self._thread_local.num_401_calls += 1
            pat = re.compile(r"digest ", flags=re.IGNORECASE)
            self._thread_local.chal = parse_dict_header(pat.sub("", s_auth, count=1))

            # Consume content and release the original connection
            # to allow our new request to reuse the same one.
            r.content
            r.close()
            prep = r.request.copy()
            extract_cookies_to_jar(prep._cookies, r.request, r.raw)
            prep.prepare_cookies(prep._cookies)

            prep.headers["Authorization"] = self.build_digest_header(
                prep.method, prep.url
            )
            _r = r.connection.send(prep, **kwargs)
            _r.history.append(r)
            _r.request = prep

            return _r

        self._thread_local.num_401_calls = 1
        return r

    def __call__(self, r):
        # Initialize per-thread state, if needed
        self.init_per_thread_state()
        # If we have a saved nonce, skip the 401
        if self._thread_local.last_nonce:
            r.headers["Authorization"] = self.build_digest_header(r.method, r.url)
        try:
            self._thread_local.pos = r.body.tell()
        except AttributeError:
            # In the case of HTTPDigestAuth being reused and the body of
            # the previous request was a file-like object, pos has the
            # file position of the previous body. Ensure it's set to
            # None.
            self._thread_local.pos = None
        r.register_hook("response", self.handle_401)
        r.register_hook("response", self.handle_redirect)
        self._thread_local.num_401_calls = 1

        return r

    def __eq__(self, other):
        return all(
            [
                self.username == getattr(other, "username", None),
                self.password == getattr(other, "password", None),
            ]
        )

    def __ne__(self, other):
        return not self == other
```


#### Issue #17: Component 'caller' has 62 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/auth.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #18: God Object detected: 'HTTPAdapter' has 15 methods and 76 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 10 trivial, 5 complex

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: Critical
- **Location**: Lines 143-143

**Code Snippet**:

```
class HTTPAdapter(BaseAdapter):
    """The built-in HTTP Adapter for urllib3.

    Provides a general-case interface for Requests sessions to contact HTTP and
    HTTPS urls by implementing the Transport Adapter interface. This class will
    usually be created by the :class:`Session <Session>` class under the
    covers.

    :param pool_connections: The number of urllib3 connection pools to cache.
    :param pool_maxsize: The maximum number of connections to save in the pool.
    :param max_retries: The maximum number of retries each connection
        should attempt. Note, this applies only to failed DNS lookups, socket
        connections and connection timeouts, never to requests where data has
        made it to the server. By default, Requests does not retry failed
        connections. If you need granular control over the conditions under
        which we retry a request, import urllib3's ``Retry`` class and pass
        that instead.
    :param pool_block: Whether the connection pool should block for connections.

    Usage::

      >>> import requests
      >>> s = requests.Session()
      >>> a = requests.adapters.HTTPAdapter(max_retries=3)
      >>> s.mount('http://', a)
    """

    __attrs__ = [
        "max_retries",
        "config",
        "_pool_connections",
        "_pool_maxsize",
        "_pool_block",
    ]

    def __init__(
        self,
        pool_connections=DEFAULT_POOLSIZE,
        pool_maxsize=DEFAULT_POOLSIZE,
        max_retries=DEFAULT_RETRIES,
        pool_block=DEFAULT_POOLBLOCK,
    ):
        if max_retries == DEFAULT_RETRIES:
            self.max_retries = Retry(0, read=False)
        else:
            self.max_retries = Retry.from_int(max_retries)
        self.config = {}
        self.proxy_manager = {}

        super().__init__()

        self._pool_connections = pool_connections
        self._pool_maxsize = pool_maxsize
        self._pool_block = pool_block

        self.init_poolmanager(pool_connections, pool_maxsize, block=pool_block)

    def __getstate__(self):
        return {attr: getattr(self, attr, None) for attr in self.__attrs__}

    def __setstate__(self, state):
        # Can't handle by adding 'proxy_manager' to self.__attrs__ because
        # self.poolmanager uses a lambda function, which isn't pickleable.
        self.proxy_manager = {}
        self.config = {}

        for attr, value in state.items():
            setattr(self, attr, value)

        self.init_poolmanager(
            self._pool_connections, self._pool_maxsize, block=self._pool_block
        )

    def init_poolmanager(
        self, connections, maxsize, block=DEFAULT_POOLBLOCK, **pool_kwargs
    ):
        """Initializes a urllib3 PoolManager.

        This method should not be called from user code, and is only
        exposed for use when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param connections: The number of urllib3 connection pools to cache.
        :param maxsize: The maximum number of connections to save in the pool.
        :param block: Block when no free connections are available.
        :param pool_kwargs: Extra keyword arguments used to initialize the Pool Manager.
        """
        # save these values for pickling
        self._pool_connections = connections
        self._pool_maxsize = maxsize
        self._pool_block = block

        self.poolmanager = PoolManager(
            num_pools=connections,
            maxsize=maxsize,
            block=block,
            **pool_kwargs,
        )

    def proxy_manager_for(self, proxy, **proxy_kwargs):
        """Return urllib3 ProxyManager for the given proxy.

        This method should not be called from user code, and is only
        exposed for use when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param proxy: The proxy to return a urllib3 ProxyManager for.
        :param proxy_kwargs: Extra keyword arguments used to configure the Proxy Manager.
        :returns: ProxyManager
        :rtype: urllib3.ProxyManager
        """
        if proxy in self.proxy_manager:
            manager = self.proxy_manager[proxy]
        elif proxy.lower().startswith("socks"):
            username, password = get_auth_from_url(proxy)
            manager = self.proxy_manager[proxy] = SOCKSProxyManager(
                proxy,
                username=username,
                password=password,
                num_pools=self._pool_connections,
                maxsize=self._pool_maxsize,
                block=self._pool_block,
                **proxy_kwargs,
            )
        else:
            proxy_headers = self.proxy_headers(proxy)
            manager = self.proxy_manager[proxy] = proxy_from_url(
                proxy,
                proxy_headers=proxy_headers,
                num_pools=self._pool_connections,
                maxsize=self._pool_maxsize,
                block=self._pool_block,
                **proxy_kwargs,
            )

        return manager

    def cert_verify(self, conn, url, verify, cert):
        """Verify a SSL certificate. This method should not be called from user
        code, and is only exposed for use when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param conn: The urllib3 connection object associated with the cert.
        :param url: The requested URL.
        :param verify: Either a boolean, in which case it controls whether we verify
            the server's TLS certificate, or a string, in which case it must be a path
            to a CA bundle to use
        :param cert: The SSL certificate to verify.
        """
        if url.lower().startswith("https") and verify:
            cert_loc = None

            # Allow self-specified cert location.
            if verify is not True:
                cert_loc = verify

            if not cert_loc:
                cert_loc = extract_zipped_paths(DEFAULT_CA_BUNDLE_PATH)

            if not cert_loc or not os.path.exists(cert_loc):
                raise OSError(
                    f"Could not find a suitable TLS CA certificate bundle, "
                    f"invalid path: {cert_loc}"
                )

            conn.cert_reqs = "CERT_REQUIRED"

            if not os.path.isdir(cert_loc):
                conn.ca_certs = cert_loc
            else:
                conn.ca_cert_dir = cert_loc
        else:
            conn.cert_reqs = "CERT_NONE"
            conn.ca_certs = None
            conn.ca_cert_dir = None

        if cert:
            if not isinstance(cert, basestring):
                conn.cert_file = cert[0]
                conn.key_file = cert[1]
            else:
                conn.cert_file = cert
                conn.key_file = None
            if conn.cert_file and not os.path.exists(conn.cert_file):
                raise OSError(
                    f"Could not find the TLS certificate file, "
                    f"invalid path: {conn.cert_file}"
                )
            if conn.key_file and not os.path.exists(conn.key_file):
                raise OSError(
                    f"Could not find the TLS key file, invalid path: {conn.key_file}"
                )

    def build_response(self, req, resp):
        """Builds a :class:`Response <requests.Response>` object from a urllib3
        response. This should not be called from user code, and is only exposed
        for use when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`

        :param req: The :class:`PreparedRequest <PreparedRequest>` used to generate the response.
        :param resp: The urllib3 response object.
        :rtype: requests.Response
        """
        response = Response()

        # Fallback to None if there's no status_code, for whatever reason.
        response.status_code = getattr(resp, "status", None)

        # Make headers case-insensitive.
        response.headers = CaseInsensitiveDict(getattr(resp, "headers", {}))

        # Set encoding.
        response.encoding = get_encoding_from_headers(response.headers)
        response.raw = resp
        response.reason = response.raw.reason

        if isinstance(req.url, bytes):
            response.url = req.url.decode("utf-8")
        else:
            response.url = req.url

        # Add new cookies from the server.
        extract_cookies_to_jar(response.cookies, req, resp)

        # Give the Response some context.
        response.request = req
        response.connection = self

        return response

    def build_connection_pool_key_attributes(self, request, verify, cert=None):
        """Build the PoolKey attributes used by urllib3 to return a connection.

        This looks at the PreparedRequest, the user-specified verify value,
        and the value of the cert parameter to determine what PoolKey values
        to use to select a connection from a given urllib3 Connection Pool.

        The SSL related pool key arguments are not consistently set. As of
        this writing, use the following to determine what keys may be in that
        dictionary:

        * If ``verify`` is ``True``, ``"ssl_context"`` will be set and will be the
          default Requests SSL Context
        * If ``verify`` is ``False``, ``"ssl_context"`` will not be set but
          ``"cert_reqs"`` will be set
        * If ``verify`` is a string, (i.e., it is a user-specified trust bundle)
          ``"ca_certs"`` will be set if the string is not a directory recognized
          by :py:func:`os.path.isdir`, otherwise ``"ca_cert_dir"`` will be
          set.
        * If ``"cert"`` is specified, ``"cert_file"`` will always be set. If
          ``"cert"`` is a tuple with a second item, ``"key_file"`` will also
          be present

        To override these settings, one may subclass this class, call this
        method and use the above logic to change parameters as desired. For
        example, if one wishes to use a custom :py:class:`ssl.SSLContext` one
        must both set ``"ssl_context"`` and based on what else they require,
        alter the other keys to ensure the desired behaviour.

        :param request:
            The PreparedReqest being sent over the connection.
        :type request:
            :class:`~requests.models.PreparedRequest`
        :param verify:
            Either a boolean, in which case it controls whether
            we verify the server's TLS certificate, or a string, in which case it
            must be a path to a CA bundle to use.
        :param cert:
            (optional) Any user-provided SSL certificate for client
            authentication (a.k.a., mTLS). This may be a string (i.e., just
            the path to a file which holds both certificate and key) or a
            tuple of length 2 with the certificate file path and key file
            path.
        :returns:
            A tuple of two dictionaries. The first is the "host parameters"
            portion of the Pool Key including scheme, hostname, and port. The
            second is a dictionary of SSLContext related parameters.
        """
        return _urllib3_request_context(request, verify, cert, self.poolmanager)

    def get_connection_with_tls_context(self, request, verify, proxies=None, cert=None):
        """Returns a urllib3 connection for the given request and TLS settings.
        This should not be called from user code, and is only exposed for use
        when subclassing the :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param request:
            The :class:`PreparedRequest <PreparedRequest>` object to be sent
            over the connection.
        :param verify:
            Either a boolean, in which case it controls whether we verify the
            server's TLS certificate, or a string, in which case it must be a
            path to a CA bundle to use.
        :param proxies:
            (optional) The proxies dictionary to apply to the request.
        :param cert:
            (optional) Any user-provided SSL certificate to be used for client
            authentication (a.k.a., mTLS).
        :rtype:
            urllib3.ConnectionPool
        """
        proxy = select_proxy(request.url, proxies)
        try:
            host_params, pool_kwargs = self.build_connection_pool_key_attributes(
                request,
                verify,
                cert,
            )
        except ValueError as e:
            raise InvalidURL(e, request=request)
        if proxy:
            proxy = prepend_scheme_if_needed(proxy, "http")
            proxy_url = parse_url(proxy)
            if not proxy_url.host:
                raise InvalidProxyURL(
                    "Please check proxy URL. It is malformed "
                    "and could be missing the host."
                )
            proxy_manager = self.proxy_manager_for(proxy)
            conn = proxy_manager.connection_from_host(
                **host_params, pool_kwargs=pool_kwargs
            )
        else:
            # Only scheme should be lower case
            conn = self.poolmanager.connection_from_host(
                **host_params, pool_kwargs=pool_kwargs
            )

        return conn

    def get_connection(self, url, proxies=None):
        """DEPRECATED: Users should move to `get_connection_with_tls_context`
        for all subclasses of HTTPAdapter using Requests>=2.32.2.

        Returns a urllib3 connection for the given URL. This should not be
        called from user code, and is only exposed for use when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param url: The URL to connect to.
        :param proxies: (optional) A Requests-style dictionary of proxies used on this request.
        :rtype: urllib3.ConnectionPool
        """
        warnings.warn(
            (
                "`get_connection` has been deprecated in favor of "
                "`get_connection_with_tls_context`. Custom HTTPAdapter subclasses "
                "will need to migrate for Requests>=2.32.2. Please see "
                "https://github.com/psf/requests/pull/6710 for more details."
            ),
            DeprecationWarning,
        )
        proxy = select_proxy(url, proxies)

        if proxy:
            proxy = prepend_scheme_if_needed(proxy, "http")
            proxy_url = parse_url(proxy)
            if not proxy_url.host:
                raise InvalidProxyURL(
                    "Please check proxy URL. It is malformed "
                    "and could be missing the host."
                )
            proxy_manager = self.proxy_manager_for(proxy)
            conn = proxy_manager.connection_from_url(url)
        else:
            # Only scheme should be lower case
            parsed = urlparse(url)
            url = parsed.geturl()
            conn = self.poolmanager.connection_from_url(url)

        return conn

    def close(self):
        """Disposes of any internal state.

        Currently, this closes the PoolManager and any active ProxyManager,
        which closes any pooled connections.
        """
        self.poolmanager.clear()
        for proxy in self.proxy_manager.values():
            proxy.clear()

    def request_url(self, request, proxies):
        """Obtain the url to use when making the final request.

        If the message is being sent through a HTTP proxy, the full URL has to
        be used. Otherwise, we should only use the path portion of the URL.

        This should not be called from user code, and is only exposed for use
        when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param request: The :class:`PreparedRequest <PreparedRequest>` being sent.
        :param proxies: A dictionary of schemes or schemes and hosts to proxy URLs.
        :rtype: str
        """
        proxy = select_proxy(request.url, proxies)
        scheme = urlparse(request.url).scheme

        is_proxied_http_request = proxy and scheme != "https"
        using_socks_proxy = False
        if proxy:
            proxy_scheme = urlparse(proxy).scheme.lower()
            using_socks_proxy = proxy_scheme.startswith("socks")

        url = request.path_url
        if url.startswith("//"):  # Don't confuse urllib3
            url = f"/{url.lstrip('/')}"

        if is_proxied_http_request and not using_socks_proxy:
            url = urldefragauth(request.url)

        return url

    def add_headers(self, request, **kwargs):
        """Add any headers needed by the connection. As of v2.0 this does
        nothing by default, but is left for overriding by users that subclass
        the :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        This should not be called from user code, and is only exposed for use
        when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param request: The :class:`PreparedRequest <PreparedRequest>` to add headers to.
        :param kwargs: The keyword arguments from the call to send().
        """
        pass

    def proxy_headers(self, proxy):
        """Returns a dictionary of the headers to add to any request sent
        through a proxy. This works with urllib3 magic to ensure that they are
        correctly sent to the proxy, rather than in a tunnelled request if
        CONNECT is being used.

        This should not be called from user code, and is only exposed for use
        when subclassing the
        :class:`HTTPAdapter <requests.adapters.HTTPAdapter>`.

        :param proxy: The url of the proxy being used for this request.
        :rtype: dict
        """
        headers = {}
        username, password = get_auth_from_url(proxy)

        if username:
            headers["Proxy-Authorization"] = _basic_auth_str(username, password)

        return headers

    def send(
        self, request, stream=False, timeout=None, verify=True, cert=None, proxies=None
    ):
        """Sends PreparedRequest object. Returns Response object.

        :param request: The :class:`PreparedRequest <PreparedRequest>` being sent.
        :param stream: (optional) Whether to stream the request content.
        :param timeout: (optional) How long to wait for the server to send
            data before giving up, as a float, or a :ref:`(connect timeout,
            read timeout) <timeouts>` tuple.
        :type timeout: float or tuple or urllib3 Timeout object
        :param verify: (optional) Either a boolean, in which case it controls whether
            we verify the server's TLS certificate, or a string, in which case it
            must be a path to a CA bundle to use
        :param cert: (optional) Any user-provided SSL certificate to be trusted.
        :param proxies: (optional) The proxies dictionary to apply to the request.
        :rtype: requests.Response
        """

        try:
            conn = self.get_connection_with_tls_context(
                request, verify, proxies=proxies, cert=cert
            )
        except LocationValueError as e:
            raise InvalidURL(e, request=request)

        self.cert_verify(conn, request.url, verify, cert)
        url = self.request_url(request, proxies)
        self.add_headers(
            request,
            stream=stream,
            timeout=timeout,
            verify=verify,
            cert=cert,
            proxies=proxies,
        )

        chunked = not (request.body is None or "Content-Length" in request.headers)

        if isinstance(timeout, tuple):
            try:
                connect, read = timeout
                timeout = TimeoutSauce(connect=connect, read=read)
            except ValueError:
                raise ValueError(
                    f"Invalid timeout {timeout}. Pass a (connect, read) timeout tuple, "
                    f"or a single float to set both timeouts to the same value."
                )
        elif isinstance(timeout, TimeoutSauce):
            pass
        else:
            timeout = TimeoutSauce(connect=timeout, read=timeout)

        try:
            resp = conn.urlopen(
                method=request.method,
                url=url,
                body=request.body,
                headers=request.headers,
                redirect=False,
                assert_same_host=False,
                preload_content=False,
                decode_content=False,
                retries=self.max_retries,
                timeout=timeout,
                chunked=chunked,
            )

        except (ProtocolError, OSError) as err:
            raise ConnectionError(err, request=request)

        except MaxRetryError as e:
            if isinstance(e.reason, ConnectTimeoutError):
                # TODO: Remove this in 3.0.0: see #2811
                if not isinstance(e.reason, NewConnectionError):
                    raise ConnectTimeout(e, request=request)

            if isinstance(e.reason, ResponseError):
                raise RetryError(e, request=request)

            if isinstance(e.reason, _ProxyError):
                raise ProxyError(e, request=request)

            if isinstance(e.reason, _SSLError):
                # This branch is for urllib3 v1.22 and later.
                raise SSLError(e, request=request)

            raise ConnectionError(e, request=request)

        except ClosedPoolError as e:
            raise ConnectionError(e, request=request)

        except _ProxyError as e:
            raise ProxyError(e)

        except (_SSLError, _HTTPError) as e:
            if isinstance(e, _SSLError):
                # This branch is for urllib3 versions earlier than v1.22
                raise SSLError(e, request=request)
            elif isinstance(e, ReadTimeoutError):
                raise ReadTimeout(e, request=request)
            elif isinstance(e, _InvalidHeader):
                raise InvalidHeader(e, request=request)
            else:
                raise

        return self.build_response(request, resp)
```


#### Issue #19: Component 'caller' has 92 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/src/requests/adapters.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #20: God Object detected: 'TestSuperLen' has 17 methods and 11 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 11 trivial, 6 complex

- **File**: `alpha_testing/requests/tests/test_utils.py`
- **Severity**: Critical
- **Location**: Lines 50-50

**Code Snippet**:

```
class TestSuperLen:
    @pytest.mark.parametrize(
        "stream, value",
        (
            (StringIO.StringIO, "Test"),
            (BytesIO, b"Test"),
            pytest.param(
                cStringIO, "Test", marks=pytest.mark.skipif("cStringIO is None")
            ),
        ),
    )
    def test_io_streams(self, stream, value):
        """Ensures that we properly deal with different kinds of IO streams."""
        assert super_len(stream()) == 0
        assert super_len(stream(value)) == 4

    def test_super_len_correctly_calculates_len_of_partially_read_file(self):
        """Ensure that we handle partially consumed file like objects."""
        s = StringIO.StringIO()
        s.write("foobarbogus")
        assert super_len(s) == 0

    @pytest.mark.parametrize("error", [IOError, OSError])
    def test_super_len_handles_files_raising_weird_errors_in_tell(self, error):
        """If tell() raises errors, assume the cursor is at position zero."""

        class BoomFile:
            def __len__(self):
                return 5

            def tell(self):
                raise error()

        assert super_len(BoomFile()) == 0

    @pytest.mark.parametrize("error", [IOError, OSError])
    def test_super_len_tell_ioerror(self, error):
        """Ensure that if tell gives an IOError super_len doesn't fail"""

        class NoLenBoomFile:
            def tell(self):
                raise error()

            def seek(self, offset, whence):
                pass

        assert super_len(NoLenBoomFile()) == 0

    def test_string(self):
        assert super_len("Test") == 4

    @pytest.mark.parametrize(
        "mode, warnings_num",
        (
            ("r", 1),
            ("rb", 0),
        ),
    )
    def test_file(self, tmpdir, mode, warnings_num, recwarn):
        file_obj = tmpdir.join("test.txt")
        file_obj.write("Test")
        with file_obj.open(mode) as fd:
            assert super_len(fd) == 4
        assert len(recwarn) == warnings_num

    def test_tarfile_member(self, tmpdir):
        file_obj = tmpdir.join("test.txt")
        file_obj.write("Test")

        tar_obj = str(tmpdir.join("test.tar"))
        with tarfile.open(tar_obj, "w") as tar:
            tar.add(str(file_obj), arcname="test.txt")

        with tarfile.open(tar_obj) as tar:
            member = tar.extractfile("test.txt")
            assert super_len(member) == 4

    def test_super_len_with__len__(self):
        foo = [1, 2, 3, 4]
        len_foo = super_len(foo)
        assert len_foo == 4

    def test_super_len_with_no__len__(self):
        class LenFile:
            def __init__(self):
                self.len = 5

        assert super_len(LenFile()) == 5

    def test_super_len_with_tell(self):
        foo = StringIO.StringIO("12345")
        assert super_len(foo) == 5
        foo.read(2)
        assert super_len(foo) == 3

    def test_super_len_with_fileno(self):
        with open(__file__, "rb") as f:
            length = super_len(f)
            file_data = f.read()
        assert length == len(file_data)

    def test_super_len_with_no_matches(self):
        """Ensure that objects without any length methods default to 0"""
        assert super_len(object()) == 0
```


#### Issue #21: Component 'caller' has 157 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/test_utils.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #22: God Object detected: 'TestCaseInsensitiveDict' has 8 methods and 4 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 5 trivial, 3 complex

- **File**: `alpha_testing/requests/tests/test_structures.py`
- **Severity**: Medium
- **Location**: Lines 6-6

**Code Snippet**:

```
class TestCaseInsensitiveDict:
    @pytest.fixture(autouse=True)
    def setup(self):
        """CaseInsensitiveDict instance with "Accept" header."""
        self.case_insensitive_dict = CaseInsensitiveDict()
        self.case_insensitive_dict["Accept"] = "application/json"

    def test_list(self):
        assert list(self.case_insensitive_dict) == ["Accept"]

    possible_keys = pytest.mark.parametrize(
        "key", ("accept", "ACCEPT", "aCcEpT", "Accept")
    )

    @possible_keys
    def test_getitem(self, key):
        assert self.case_insensitive_dict[key] == "application/json"

    @possible_keys
    def test_delitem(self, key):
        del self.case_insensitive_dict[key]
        assert key not in self.case_insensitive_dict

    def test_lower_items(self):
        assert list(self.case_insensitive_dict.lower_items()) == [
            ("accept", "application/json")
        ]

    def test_repr(self):
        assert repr(self.case_insensitive_dict) == "{'Accept': 'application/json'}"

    def test_copy(self):
        copy = self.case_insensitive_dict.copy()
        assert copy is not self.case_insensitive_dict
        assert copy == self.case_insensitive_dict

    @pytest.mark.parametrize(
        "other, result",
        (
            ({"AccePT": "application/json"}, True),
            ({}, False),
            (None, False),
        ),
    )
    def test_instance_equality(self, other, result):
        assert (self.case_insensitive_dict == other) is result
```


#### Issue #23: God Object detected: 'TestRequests' has 213 methods and 566 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 149 trivial, 64 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Critical
- **Location**: Lines 82-82

**Code Snippet**:

```
class TestRequests:
    digest_auth_algo = ("MD5", "SHA-256", "SHA-512")

    def test_entry_points(self):
        requests.session
        requests.session().get
        requests.session().head
        requests.get
        requests.head
        requests.put
        requests.patch
        requests.post
        # Not really an entry point, but people rely on it.
        from requests.packages.urllib3.poolmanager import PoolManager  # noqa:F401

    @pytest.mark.parametrize(
        "exception, url",
        (
            (MissingSchema, "hiwpefhipowhefopw"),
            (InvalidSchema, "localhost:3128"),
            (InvalidSchema, "localhost.localdomain:3128/"),
            (InvalidSchema, "10.122.1.1:3128/"),
            (InvalidURL, "http://"),
            (InvalidURL, "http://*example.com"),
            (InvalidURL, "http://.example.com"),
        ),
    )
    def test_invalid_url(self, exception, url):
        with pytest.raises(exception):
            requests.get(url)

    def test_basic_building(self):
        req = requests.Request()
        req.url = "http://kennethreitz.org/"
        req.data = {"life": "42"}

        pr = req.prepare()
        assert pr.url == req.url
        assert pr.body == "life=42"

    @pytest.mark.parametrize("method", ("GET", "HEAD"))
    def test_no_content_length(self, httpbin, method):
        req = requests.Request(method, httpbin(method.lower())).prepare()
        assert "Content-Length" not in req.headers

    @pytest.mark.parametrize("method", ("POST", "PUT", "PATCH", "OPTIONS"))
    def test_no_body_content_length(self, httpbin, method):
        req = requests.Request(method, httpbin(method.lower())).prepare()
        assert req.headers["Content-Length"] == "0"

    @pytest.mark.parametrize("method", ("POST", "PUT", "PATCH", "OPTIONS"))
    def test_empty_content_length(self, httpbin, method):
        req = requests.Request(method, httpbin(method.lower()), data="").prepare()
        assert req.headers["Content-Length"] == "0"

    def test_override_content_length(self, httpbin):
        headers = {"Content-Length": "not zero"}
        r = requests.Request("POST", httpbin("post"), headers=headers).prepare()
        assert "Content-Length" in r.headers
        assert r.headers["Content-Length"] == "not zero"

    def test_path_is_not_double_encoded(self):
        request = requests.Request("GET", "http://0.0.0.0/get/test case").prepare()

        assert request.path_url == "/get/test%20case"

    @pytest.mark.parametrize(
        "url, expected",
        (
            (
                "http://example.com/path#fragment",
                "http://example.com/path?a=b#fragment",
            ),
            (
                "http://example.com/path?key=value#fragment",
                "http://example.com/path?key=value&a=b#fragment",
            ),
        ),
    )
    def test_params_are_added_before_fragment(self, url, expected):
        request = requests.Request("GET", url, params={"a": "b"}).prepare()
        assert request.url == expected

    def test_params_original_order_is_preserved_by_default(self):
        param_ordered_dict = collections.OrderedDict(
            (("z", 1), ("a", 1), ("k", 1), ("d", 1))
        )
        session = requests.Session()
        request = requests.Request(
            "GET", "http://example.com/", params=param_ordered_dict
        )
        prep = session.prepare_request(request)
        assert prep.url == "http://example.com/?z=1&a=1&k=1&d=1"

    def test_params_bytes_are_encoded(self):
        request = requests.Request(
            "GET", "http://example.com", params=b"test=foo"
        ).prepare()
        assert request.url == "http://example.com/?test=foo"

    def test_binary_put(self):
        request = requests.Request(
            "PUT", "http://example.com", data="ööö".encode()
        ).prepare()
        assert isinstance(request.body, bytes)

    def test_whitespaces_are_removed_from_url(self):
        # Test for issue #3696
        request = requests.Request("GET", " http://example.com").prepare()
        assert request.url == "http://example.com/"

    @pytest.mark.parametrize("scheme", ("http://", "HTTP://", "hTTp://", "HttP://"))
    def test_mixed_case_scheme_acceptable(self, httpbin, scheme):
        s = requests.Session()
        s.proxies = getproxies()
        parts = urlparse(httpbin("get"))
        url = scheme + parts.netloc + parts.path
        r = requests.Request("GET", url)
        r = s.send(r.prepare())
        assert r.status_code == 200, f"failed for scheme {scheme}"

    def test_HTTP_200_OK_GET_ALTERNATIVE(self, httpbin):
        r = requests.Request("GET", httpbin("get"))
        s = requests.Session()
        s.proxies = getproxies()

        r = s.send(r.prepare())

        assert r.status_code == 200

    def test_HTTP_302_ALLOW_REDIRECT_GET(self, httpbin):
        r = requests.get(httpbin("redirect", "1"))
        assert r.status_code == 200
        assert r.history[0].status_code == 302
        assert r.history[0].is_redirect

    def test_HTTP_307_ALLOW_REDIRECT_POST(self, httpbin):
        r = requests.post(
            httpbin("redirect-to"),
            data="test",
            params={"url": "post", "status_code": 307},
        )
        assert r.status_code == 200
        assert r.history[0].status_code == 307
        assert r.history[0].is_redirect
        assert r.json()["data"] == "test"

    def test_HTTP_307_ALLOW_REDIRECT_POST_WITH_SEEKABLE(self, httpbin):
        byte_str = b"test"
        r = requests.post(
            httpbin("redirect-to"),
            data=io.BytesIO(byte_str),
            params={"url": "post", "status_code": 307},
        )
        assert r.status_code == 200
        assert r.history[0].status_code == 307
        assert r.history[0].is_redirect
        assert r.json()["data"] == byte_str.decode("utf-8")

    def test_HTTP_302_TOO_MANY_REDIRECTS(self, httpbin):
        try:
            requests.get(httpbin("relative-redirect", "50"))
        except TooManyRedirects as e:
            url = httpbin("relative-redirect", "20")
            assert e.request.url == url
            assert e.response.url == url
            assert len(e.response.history) == 30
        else:
            pytest.fail("Expected redirect to raise TooManyRedirects but it did not")

    def test_HTTP_302_TOO_MANY_REDIRECTS_WITH_PARAMS(self, httpbin):
        s = requests.session()
        s.max_redirects = 5
        try:
            s.get(httpbin("relative-redirect", "50"))
        except TooManyRedirects as e:
            url = httpbin("relative-redirect", "45")
            assert e.request.url == url
            assert e.response.url == url
            assert len(e.response.history) == 5
        else:
            pytest.fail(
                "Expected custom max number of redirects to be respected but was not"
            )

    def test_http_301_changes_post_to_get(self, httpbin):
        r = requests.post(httpbin("status", "301"))
        assert r.status_code == 200
        assert r.request.method == "GET"
        assert r.history[0].status_code == 301
        assert r.history[0].is_redirect

    def test_http_301_doesnt_change_head_to_get(self, httpbin):
        r = requests.head(httpbin("status", "301"), allow_redirects=True)
        print(r.content)
        assert r.status_code == 200
        assert r.request.method == "HEAD"
        assert r.history[0].status_code == 301
        assert r.history[0].is_redirect

    def test_http_302_changes_post_to_get(self, httpbin):
        r = requests.post(httpbin("status", "302"))
        assert r.status_code == 200
        assert r.request.method == "GET"
        assert r.history[0].status_code == 302
        assert r.history[0].is_redirect

    def test_http_302_doesnt_change_head_to_get(self, httpbin):
        r = requests.head(httpbin("status", "302"), allow_redirects=True)
        assert r.status_code == 200
        assert r.request.method == "HEAD"
        assert r.history[0].status_code == 302
        assert r.history[0].is_redirect

    def test_http_303_changes_post_to_get(self, httpbin):
        r = requests.post(httpbin("status", "303"))
        assert r.status_code == 200
        assert r.request.method == "GET"
        assert r.history[0].status_code == 303
        assert r.history[0].is_redirect

    def test_http_303_doesnt_change_head_to_get(self, httpbin):
        r = requests.head(httpbin("status", "303"), allow_redirects=True)
        assert r.status_code == 200
        assert r.request.method == "HEAD"
        assert r.history[0].status_code == 303
        assert r.history[0].is_redirect

    def test_header_and_body_removal_on_redirect(self, httpbin):
        purged_headers = ("Content-Length", "Content-Type")
        ses = requests.Session()
        req = requests.Request("POST", httpbin("post"), data={"test": "data"})
        prep = ses.prepare_request(req)
        resp = ses.send(prep)

        # Mimic a redirect response
        resp.status_code = 302
        resp.headers["location"] = "get"

        # Run request through resolve_redirects
        next_resp = next(ses.resolve_redirects(resp, prep))
        assert next_resp.request.body is None
        for header in purged_headers:
            assert header not in next_resp.request.headers

    def test_transfer_enc_removal_on_redirect(self, httpbin):
        purged_headers = ("Transfer-Encoding", "Content-Type")
        ses = requests.Session()
        req = requests.Request("POST", httpbin("post"), data=(b"x" for x in range(1)))
        prep = ses.prepare_request(req)
        assert "Transfer-Encoding" in prep.headers

        # Create Response to avoid https://github.com/kevin1024/pytest-httpbin/issues/33
        resp = requests.Response()
        resp.raw = io.BytesIO(b"the content")
        resp.request = prep
        setattr(resp.raw, "release_conn", lambda *args: args)

        # Mimic a redirect response
        resp.status_code = 302
        resp.headers["location"] = httpbin("get")

        # Run request through resolve_redirect
        next_resp = next(ses.resolve_redirects(resp, prep))
        assert next_resp.request.body is None
        for header in purged_headers:
            assert header not in next_resp.request.headers

    def test_fragment_maintained_on_redirect(self, httpbin):
        fragment = "#view=edit&token=hunter2"
        r = requests.get(httpbin("redirect-to?url=get") + fragment)

        assert len(r.history) > 0
        assert r.history[0].request.url == httpbin("redirect-to?url=get") + fragment
        assert r.url == httpbin("get") + fragment

    def test_HTTP_200_OK_GET_WITH_PARAMS(self, httpbin):
        heads = {"User-agent": "Mozilla/5.0"}

        r = requests.get(httpbin("user-agent"), headers=heads)

        assert heads["User-agent"] in r.text
        assert r.status_code == 200

    def test_HTTP_200_OK_GET_WITH_MIXED_PARAMS(self, httpbin):
        heads = {"User-agent": "Mozilla/5.0"}

        r = requests.get(
            httpbin("get") + "?test=true", params={"q": "test"}, headers=heads
        )
        assert r.status_code == 200

    def test_set_cookie_on_301(self, httpbin):
        s = requests.session()
        url = httpbin("cookies/set?foo=bar")
        s.get(url)
        assert s.cookies["foo"] == "bar"

    def test_cookie_sent_on_redirect(self, httpbin):
        s = requests.session()
        s.get(httpbin("cookies/set?foo=bar"))
        r = s.get(httpbin("redirect/1"))  # redirects to httpbin('get')
        assert "Cookie" in r.json()["headers"]

    def test_cookie_removed_on_expire(self, httpbin):
        s = requests.session()
        s.get(httpbin("cookies/set?foo=bar"))
        assert s.cookies["foo"] == "bar"
        s.get(
            httpbin("response-headers"),
            params={"Set-Cookie": "foo=deleted; expires=Thu, 01-Jan-1970 00:00:01 GMT"},
        )
        assert "foo" not in s.cookies

    def test_cookie_quote_wrapped(self, httpbin):
        s = requests.session()
        s.get(httpbin('cookies/set?foo="bar:baz"'))
        assert s.cookies["foo"] == '"bar:baz"'

    def test_cookie_persists_via_api(self, httpbin):
        s = requests.session()
        r = s.get(httpbin("redirect/1"), cookies={"foo": "bar"})
        assert "foo" in r.request.headers["Cookie"]
        assert "foo" in r.history[0].request.headers["Cookie"]

    def test_request_cookie_overrides_session_cookie(self, httpbin):
        s = requests.session()
        s.cookies["foo"] = "bar"
        r = s.get(httpbin("cookies"), cookies={"foo": "baz"})
        assert r.json()["cookies"]["foo"] == "baz"
        # Session cookie should not be modified
        assert s.cookies["foo"] == "bar"

    def test_request_cookies_not_persisted(self, httpbin):
        s = requests.session()
        s.get(httpbin("cookies"), cookies={"foo": "baz"})
        # Sending a request with cookies should not add cookies to the session
        assert not s.cookies

    def test_generic_cookiejar_works(self, httpbin):
        cj = cookielib.CookieJar()
        cookiejar_from_dict({"foo": "bar"}, cj)
        s = requests.session()
        s.cookies = cj
        r = s.get(httpbin("cookies"))
        # Make sure the cookie was sent
        assert r.json()["cookies"]["foo"] == "bar"
        # Make sure the session cj is still the custom one
        assert s.cookies is cj

    def test_param_cookiejar_works(self, httpbin):
        cj = cookielib.CookieJar()
        cookiejar_from_dict({"foo": "bar"}, cj)
        s = requests.session()
        r = s.get(httpbin("cookies"), cookies=cj)
        # Make sure the cookie was sent
        assert r.json()["cookies"]["foo"] == "bar"

    def test_cookielib_cookiejar_on_redirect(self, httpbin):
        """Tests resolve_redirect doesn't fail when merging cookies
        with non-RequestsCookieJar cookiejar.

        See GH #3579
        """
        cj = cookiejar_from_dict({"foo": "bar"}, cookielib.CookieJar())
        s = requests.Session()
        s.cookies = cookiejar_from_dict({"cookie": "tasty"})

        # Prepare request without using Session
        req = requests.Request("GET", httpbin("headers"), cookies=cj)
        prep_req = req.prepare()

        # Send request and simulate redirect
        resp = s.send(prep_req)
        resp.status_code = 302
        resp.headers["location"] = httpbin("get")
        redirects = s.resolve_redirects(resp, prep_req)
        resp = next(redirects)

        # Verify CookieJar isn't being converted to RequestsCookieJar
        assert isinstance(prep_req._cookies, cookielib.CookieJar)
        assert isinstance(resp.request._cookies, cookielib.CookieJar)
        assert not isinstance(resp.request._cookies, requests.cookies.RequestsCookieJar)

        cookies = {}
        for c in resp.request._cookies:
            cookies[c.name] = c.value
        assert cookies["foo"] == "bar"
        assert cookies["cookie"] == "tasty"

    def test_requests_in_history_are_not_overridden(self, httpbin):
        resp = requests.get(httpbin("redirect/3"))
        urls = [r.url for r in resp.history]
        req_urls = [r.request.url for r in resp.history]
        assert urls == req_urls

    def test_history_is_always_a_list(self, httpbin):
        """Show that even with redirects, Response.history is always a list."""
        resp = requests.get(httpbin("get"))
        assert isinstance(resp.history, list)
        resp = requests.get(httpbin("redirect/1"))
        assert isinstance(resp.history, list)
        assert not isinstance(resp.history, tuple)

    def test_headers_on_session_with_None_are_not_sent(self, httpbin):
        """Do not send headers in Session.headers with None values."""
        ses = requests.Session()
        ses.headers["Accept-Encoding"] = None
        req = requests.Request("GET", httpbin("get"))
        prep = ses.prepare_request(req)
        assert "Accept-Encoding" not in prep.headers

    def test_headers_preserve_order(self, httpbin):
        """Preserve order when headers provided as OrderedDict."""
        ses = requests.Session()
        ses.headers = collections.OrderedDict()
        ses.headers["Accept-Encoding"] = "identity"
        ses.headers["First"] = "1"
        ses.headers["Second"] = "2"
        headers = collections.OrderedDict([("Third", "3"), ("Fourth", "4")])
        headers["Fifth"] = "5"
        headers["Second"] = "222"
        req = requests.Request("GET", httpbin("get"), headers=headers)
        prep = ses.prepare_request(req)
        items = list(prep.headers.items())
        assert items[0] == ("Accept-Encoding", "identity")
        assert items[1] == ("First", "1")
        assert items[2] == ("Second", "222")
        assert items[3] == ("Third", "3")
        assert items[4] == ("Fourth", "4")
        assert items[5] == ("Fifth", "5")

    @pytest.mark.parametrize("key", ("User-agent", "user-agent"))
    def test_user_agent_transfers(self, httpbin, key):
        heads = {key: "Mozilla/5.0 (github.com/psf/requests)"}

        r = requests.get(httpbin("user-agent"), headers=heads)
        assert heads[key] in r.text

    def test_HTTP_200_OK_HEAD(self, httpbin):
        r = requests.head(httpbin("get"))
        assert r.status_code == 200

    def test_HTTP_200_OK_PUT(self, httpbin):
        r = requests.put(httpbin("put"))
        assert r.status_code == 200

    def test_BASICAUTH_TUPLE_HTTP_200_OK_GET(self, httpbin):
        auth = ("user", "pass")
        url = httpbin("basic-auth", "user", "pass")

        r = requests.get(url, auth=auth)
        assert r.status_code == 200

        r = requests.get(url)
        assert r.status_code == 401

        s = requests.session()
        s.auth = auth
        r = s.get(url)
        assert r.status_code == 200

    @pytest.mark.parametrize(
        "username, password",
        (
            ("user", "pass"),
            ("имя".encode(), "пароль".encode()),
            (42, 42),
            (None, None),
        ),
    )
    def test_set_basicauth(self, httpbin, username, password):
        auth = (username, password)
        url = httpbin("get")

        r = requests.Request("GET", url, auth=auth)
        p = r.prepare()

        assert p.headers["Authorization"] == _basic_auth_str(username, password)

    def test_basicauth_encodes_byte_strings(self):
        """Ensure b'test' formats as the byte string "test" rather
        than the unicode string "b'test'" in Python 3.
        """
        auth = (b"\xc5\xafsername", b"test\xc6\xb6")
        r = requests.Request("GET", "http://localhost", auth=auth)
        p = r.prepare()

        assert p.headers["Authorization"] == "Basic xa9zZXJuYW1lOnRlc3TGtg=="

    @pytest.mark.parametrize(
        "url, exception",
        (
            # Connecting to an unknown domain should raise a ConnectionError
            ("http://doesnotexist.google.com", ConnectionError),
            # Connecting to an invalid port should raise a ConnectionError
            ("http://localhost:1", ConnectionError),
            # Inputing a URL that cannot be parsed should raise an InvalidURL error
            ("http://fe80::5054:ff:fe5a:fc0", InvalidURL),
        ),
    )
    def test_errors(self, url, exception):
        with pytest.raises(exception):
            requests.get(url, timeout=1)

    def test_proxy_error(self):
        # any proxy related error (address resolution, no route to host, etc) should result in a ProxyError
        with pytest.raises(ProxyError):
            requests.get(
                "http://localhost:1", proxies={"http": "non-resolvable-address"}
            )

    def test_proxy_error_on_bad_url(self, httpbin, httpbin_secure):
        with pytest.raises(InvalidProxyURL):
            requests.get(httpbin_secure(), proxies={"https": "http:/badproxyurl:3128"})

        with pytest.raises(InvalidProxyURL):
            requests.get(httpbin(), proxies={"http": "http://:8080"})

        with pytest.raises(InvalidProxyURL):
            requests.get(httpbin_secure(), proxies={"https": "https://"})

        with pytest.raises(InvalidProxyURL):
            requests.get(httpbin(), proxies={"http": "http:///example.com:8080"})

    def test_respect_proxy_env_on_send_self_prepared_request(self, httpbin):
        with override_environ(http_proxy=INVALID_PROXY):
            with pytest.raises(ProxyError):
                session = requests.Session()
                request = requests.Request("GET", httpbin())
                session.send(request.prepare())

    def test_respect_proxy_env_on_send_session_prepared_request(self, httpbin):
        with override_environ(http_proxy=INVALID_PROXY):
            with pytest.raises(ProxyError):
                session = requests.Session()
                request = requests.Request("GET", httpbin())
                prepared = session.prepare_request(request)
                session.send(prepared)

    def test_respect_proxy_env_on_send_with_redirects(self, httpbin):
        with override_environ(http_proxy=INVALID_PROXY):
            with pytest.raises(ProxyError):
                session = requests.Session()
                url = httpbin("redirect/1")
                print(url)
                request = requests.Request("GET", url)
                session.send(request.prepare())

    def test_respect_proxy_env_on_get(self, httpbin):
        with override_environ(http_proxy=INVALID_PROXY):
            with pytest.raises(ProxyError):
                session = requests.Session()
                session.get(httpbin())

    def test_respect_proxy_env_on_request(self, httpbin):
        with override_environ(http_proxy=INVALID_PROXY):
            with pytest.raises(ProxyError):
                session = requests.Session()
                session.request(method="GET", url=httpbin())

    def test_proxy_authorization_preserved_on_request(self, httpbin):
        proxy_auth_value = "Bearer XXX"
        session = requests.Session()
        session.headers.update({"Proxy-Authorization": proxy_auth_value})
        resp = session.request(method="GET", url=httpbin("get"))
        sent_headers = resp.json().get("headers", {})

        assert sent_headers.get("Proxy-Authorization") == proxy_auth_value

    @pytest.mark.parametrize(
        "url,has_proxy_auth",
        (
            ("http://example.com", True),
            ("https://example.com", False),
        ),
    )
    def test_proxy_authorization_not_appended_to_https_request(
        self, url, has_proxy_auth
    ):
        session = requests.Session()
        proxies = {
            "http": "http://test:pass@localhost:8080",
            "https": "http://test:pass@localhost:8090",
        }
        req = requests.Request("GET", url)
        prep = req.prepare()
        session.rebuild_proxies(prep, proxies)

        assert ("Proxy-Authorization" in prep.headers) is has_proxy_auth

    def test_basicauth_with_netrc(self, httpbin):
        auth = ("user", "pass")
        wrong_auth = ("wronguser", "wrongpass")
        url = httpbin("basic-auth", "user", "pass")

        old_auth = requests.sessions.get_netrc_auth

        try:

            def get_netrc_auth_mock(url):
                return auth

            requests.sessions.get_netrc_auth = get_netrc_auth_mock

            # Should use netrc and work.
            r = requests.get(url)
            assert r.status_code == 200

            # Given auth should override and fail.
            r = requests.get(url, auth=wrong_auth)
            assert r.status_code == 401

            s = requests.session()

            # Should use netrc and work.
            r = s.get(url)
            assert r.status_code == 200

            # Given auth should override and fail.
            s.auth = wrong_auth
            r = s.get(url)
            assert r.status_code == 401
        finally:
            requests.sessions.get_netrc_auth = old_auth

    def test_basicauth_with_netrc_leak(self, httpbin):
        url1 = httpbin("basic-auth", "user", "pass")
        url = url1[len("http://") :]
        domain = url.split(":")[0]
        url = f"http://example.com:@{url}"

        netrc_file = ""
        with tempfile.NamedTemporaryFile(mode="w", delete=False) as fp:
            fp.write("machine example.com\n")
            fp.write("login wronguser\n")
            fp.write("password wrongpass\n")
            fp.write(f"machine {domain}\n")
            fp.write("login user\n")
            fp.write("password pass\n")
            fp.close()
            netrc_file = fp.name

        old_netrc = os.environ.get("NETRC", "")
        os.environ["NETRC"] = netrc_file

        try:
            # Should use netrc
            # Make sure that we don't use the example.com credentails
            # for the request
            r = requests.get(url)
            assert r.status_code == 200
        finally:
            os.environ["NETRC"] = old_netrc
            os.unlink(netrc_file)

    def test_DIGEST_HTTP_200_OK_GET(self, httpbin):
        for authtype in self.digest_auth_algo:
            auth = HTTPDigestAuth("user", "pass")
            url = httpbin("digest-auth", "auth", "user", "pass", authtype, "never")

            r = requests.get(url, auth=auth)
            assert r.status_code == 200

            r = requests.get(url)
            assert r.status_code == 401
            print(r.headers["WWW-Authenticate"])

            s = requests.session()
            s.auth = HTTPDigestAuth("user", "pass")
            r = s.get(url)
            assert r.status_code == 200

    def test_DIGEST_AUTH_RETURNS_COOKIE(self, httpbin):
        for authtype in self.digest_auth_algo:
            url = httpbin("digest-auth", "auth", "user", "pass", authtype)
            auth = HTTPDigestAuth("user", "pass")
            r = requests.get(url)
            assert r.cookies["fake"] == "fake_value"

            r = requests.get(url, auth=auth)
            assert r.status_code == 200

    def test_DIGEST_AUTH_SETS_SESSION_COOKIES(self, httpbin):
        for authtype in self.digest_auth_algo:
            url = httpbin("digest-auth", "auth", "user", "pass", authtype)
            auth = HTTPDigestAuth("user", "pass")
            s = requests.Session()
            s.get(url, auth=auth)
            assert s.cookies["fake"] == "fake_value"

    def test_DIGEST_STREAM(self, httpbin):
        for authtype in self.digest_auth_algo:
            auth = HTTPDigestAuth("user", "pass")
            url = httpbin("digest-auth", "auth", "user", "pass", authtype)

            r = requests.get(url, auth=auth, stream=True)
            assert r.raw.read() != b""

            r = requests.get(url, auth=auth, stream=False)
            assert r.raw.read() == b""

    def test_DIGESTAUTH_WRONG_HTTP_401_GET(self, httpbin):
        for authtype in self.digest_auth_algo:
            auth = HTTPDigestAuth("user", "wrongpass")
            url = httpbin("digest-auth", "auth", "user", "pass", authtype)

            r = requests.get(url, auth=auth)
            assert r.status_code == 401

            r = requests.get(url)
            assert r.status_code == 401

            s = requests.session()
            s.auth = auth
            r = s.get(url)
            assert r.status_code == 401

    def test_DIGESTAUTH_QUOTES_QOP_VALUE(self, httpbin):
        for authtype in self.digest_auth_algo:
            auth = HTTPDigestAuth("user", "pass")
            url = httpbin("digest-auth", "auth", "user", "pass", authtype)

            r = requests.get(url, auth=auth)
            assert '"auth"' in r.request.headers["Authorization"]

    def test_POSTBIN_GET_POST_FILES(self, httpbin):
        url = httpbin("post")
        requests.post(url).raise_for_status()

        post1 = requests.post(url, data={"some": "data"})
        assert post1.status_code == 200

        with open("requirements-dev.txt") as f:
            post2 = requests.post(url, files={"some": f})
        assert post2.status_code == 200

        post4 = requests.post(url, data='[{"some": "json"}]')
        assert post4.status_code == 200

        with pytest.raises(ValueError):
            requests.post(url, files=["bad file data"])

    def test_invalid_files_input(self, httpbin):
        url = httpbin("post")
        post = requests.post(url, files={"random-file-1": None, "random-file-2": 1})
        assert b'name="random-file-1"' not in post.request.body
        assert b'name="random-file-2"' in post.request.body

    def test_POSTBIN_SEEKED_OBJECT_WITH_NO_ITER(self, httpbin):
        class TestStream:
            def __init__(self, data):
                self.data = data.encode()
                self.length = len(self.data)
                self.index = 0

            def __len__(self):
                return self.length

            def read(self, size=None):
                if size:
                    ret = self.data[self.index : self.index + size]
                    self.index += size
                else:
                    ret = self.data[self.index :]
                    self.index = self.length
                return ret

            def tell(self):
                return self.index

            def seek(self, offset, where=0):
                if where == 0:
                    self.index = offset
                elif where == 1:
                    self.index += offset
                elif where == 2:
                    self.index = self.length + offset

        test = TestStream("test")
        post1 = requests.post(httpbin("post"), data=test)
        assert post1.status_code == 200
        assert post1.json()["data"] == "test"

        test = TestStream("test")
        test.seek(2)
        post2 = requests.post(httpbin("post"), data=test)
        assert post2.status_code == 200
        assert post2.json()["data"] == "st"

    def test_POSTBIN_GET_POST_FILES_WITH_DATA(self, httpbin):
        url = httpbin("post")
        requests.post(url).raise_for_status()

        post1 = requests.post(url, data={"some": "data"})
        assert post1.status_code == 200

        with open("requirements-dev.txt") as f:
            post2 = requests.post(url, data={"some": "data"}, files={"some": f})
        assert post2.status_code == 200

        post4 = requests.post(url, data='[{"some": "json"}]')
        assert post4.status_code == 200

        with pytest.raises(ValueError):
            requests.post(url, files=["bad file data"])

    def test_post_with_custom_mapping(self, httpbin):
        class CustomMapping(MutableMapping):
            def __init__(self, *args, **kwargs):
                self.data = dict(*args, **kwargs)

            def __delitem__(self, key):
                del self.data[key]

            def __getitem__(self, key):
                return self.data[key]

            def __setitem__(self, key, value):
                self.data[key] = value

            def __iter__(self):
                return iter(self.data)

            def __len__(self):
                return len(self.data)

        data = CustomMapping({"some": "data"})
        url = httpbin("post")
        found_json = requests.post(url, data=data).json().get("form")
        assert found_json == {"some": "data"}

    def test_conflicting_post_params(self, httpbin):
        url = httpbin("post")
        with open("requirements-dev.txt") as f:
            with pytest.raises(ValueError):
                requests.post(url, data='[{"some": "data"}]', files={"some": f})

    def test_request_ok_set(self, httpbin):
        r = requests.get(httpbin("status", "404"))
        assert not r.ok

    def test_status_raising(self, httpbin):
        r = requests.get(httpbin("status", "404"))
        with pytest.raises(requests.exceptions.HTTPError):
            r.raise_for_status()

        r = requests.get(httpbin("status", "500"))
        assert not r.ok

    def test_decompress_gzip(self, httpbin):
        r = requests.get(httpbin("gzip"))
        r.content.decode("ascii")

    @pytest.mark.parametrize(
        "url, params",
        (
            ("/get", {"foo": "føø"}),
            ("/get", {"føø": "føø"}),
            ("/get", {"føø": "føø"}),
            ("/get", {"foo": "foo"}),
            ("ø", {"foo": "foo"}),
        ),
    )
    def test_unicode_get(self, httpbin, url, params):
        requests.get(httpbin(url), params=params)

    def test_unicode_header_name(self, httpbin):
        requests.put(
            httpbin("put"),
            headers={"Content-Type": "application/octet-stream"},
            data="\xff",
        )  # compat.str is unicode.

    def test_pyopenssl_redirect(self, httpbin_secure, httpbin_ca_bundle):
        requests.get(httpbin_secure("status", "301"), verify=httpbin_ca_bundle)

    def test_invalid_ca_certificate_path(self, httpbin_secure):
        INVALID_PATH = "/garbage"
        with pytest.raises(IOError) as e:
            requests.get(httpbin_secure(), verify=INVALID_PATH)
        assert str(
            e.value
        ) == "Could not find a suitable TLS CA certificate bundle, invalid path: {}".format(
            INVALID_PATH
        )

    def test_invalid_ssl_certificate_files(self, httpbin_secure):
        INVALID_PATH = "/garbage"
        with pytest.raises(IOError) as e:
            requests.get(httpbin_secure(), cert=INVALID_PATH)
        assert str(
            e.value
        ) == "Could not find the TLS certificate file, invalid path: {}".format(
            INVALID_PATH
        )

        with pytest.raises(IOError) as e:
            requests.get(httpbin_secure(), cert=(".", INVALID_PATH))
        assert str(e.value) == (
            f"Could not find the TLS key file, invalid path: {INVALID_PATH}"
        )

    @pytest.mark.parametrize(
        "env, expected",
        (
            ({}, True),
            ({"REQUESTS_CA_BUNDLE": "/some/path"}, "/some/path"),
            ({"REQUESTS_CA_BUNDLE": ""}, True),
            ({"CURL_CA_BUNDLE": "/some/path"}, "/some/path"),
            ({"CURL_CA_BUNDLE": ""}, True),
            ({"REQUESTS_CA_BUNDLE": "", "CURL_CA_BUNDLE": ""}, True),
            (
                {
                    "REQUESTS_CA_BUNDLE": "/some/path",
                    "CURL_CA_BUNDLE": "/curl/path",
                },
                "/some/path",
            ),
            (
                {
                    "REQUESTS_CA_BUNDLE": "",
                    "CURL_CA_BUNDLE": "/curl/path",
                },
                "/curl/path",
            ),
        ),
    )
    def test_env_cert_bundles(self, httpbin, env, expected):
        s = requests.Session()
        with mock.patch("os.environ", env):
            settings = s.merge_environment_settings(
                url=httpbin("get"), proxies={}, stream=False, verify=True, cert=None
            )
        assert settings["verify"] == expected

    def test_http_with_certificate(self, httpbin):
        r = requests.get(httpbin(), cert=".")
        assert r.status_code == 200

    @pytest.mark.skipif(
        SNIMissingWarning is None,
        reason="urllib3 2.0 removed that warning and errors out instead",
    )
    def test_https_warnings(self, nosan_server):
        """warnings are emitted with requests.get"""
        host, port, ca_bundle = nosan_server
        if HAS_MODERN_SSL or HAS_PYOPENSSL:
            warnings_expected = ("SubjectAltNameWarning",)
        else:
            warnings_expected = (
                "SNIMissingWarning",
                "InsecurePlatformWarning",
                "SubjectAltNameWarning",
            )

        with pytest.warns() as warning_records:
            warnings.simplefilter("always")
            requests.get(f"https://localhost:{port}/", verify=ca_bundle)

        warning_records = [
            item
            for item in warning_records
            if item.category.__name__ != "ResourceWarning"
        ]

        warnings_category = tuple(item.category.__name__ for item in warning_records)
        assert warnings_category == warnings_expected

    def test_certificate_failure(self, httpbin_secure):
        """
        When underlying SSL problems occur, an SSLError is raised.
        """
        with pytest.raises(RequestsSSLError):
            # Our local httpbin does not have a trusted CA, so this call will
            # fail if we use our default trust bundle.
            requests.get(httpbin_secure("status", "200"))

    def test_urlencoded_get_query_multivalued_param(self, httpbin):
        r = requests.get(httpbin("get"), params={"test": ["foo", "baz"]})
        assert r.status_code == 200
        assert r.url == httpbin("get?test=foo&test=baz")

    def test_form_encoded_post_query_multivalued_element(self, httpbin):
        r = requests.Request(
            method="POST", url=httpbin("post"), data=dict(test=["foo", "baz"])
        )
        prep = r.prepare()
        assert prep.body == "test=foo&test=baz"

    def test_different_encodings_dont_break_post(self, httpbin):
        with open(__file__, "rb") as f:
            r = requests.post(
                httpbin("post"),
                data={"stuff": json.dumps({"a": 123})},
                params={"blah": "asdf1234"},
                files={"file": ("test_requests.py", f)},
            )
        assert r.status_code == 200

    @pytest.mark.parametrize(
        "data",
        (
            {"stuff": "ëlïxr"},
            {"stuff": "ëlïxr".encode()},
            {"stuff": "elixr"},
            {"stuff": b"elixr"},
        ),
    )
    def test_unicode_multipart_post(self, httpbin, data):
        with open(__file__, "rb") as f:
            r = requests.post(
                httpbin("post"),
                data=data,
                files={"file": ("test_requests.py", f)},
            )
        assert r.status_code == 200

    def test_unicode_multipart_post_fieldnames(self, httpbin):
        filename = os.path.splitext(__file__)[0] + ".py"
        with open(filename, "rb") as f:
            r = requests.Request(
                method="POST",
                url=httpbin("post"),
                data={b"stuff": "elixr"},
                files={"file": ("test_requests.py", f)},
            )
            prep = r.prepare()

        assert b'name="stuff"' in prep.body
        assert b"name=\"b'stuff'\"" not in prep.body

    def test_unicode_method_name(self, httpbin):
        with open(__file__, "rb") as f:
            files = {"file": f}
            r = requests.request(
                method="POST",
                url=httpbin("post"),
                files=files,
            )
        assert r.status_code == 200

    def test_unicode_method_name_with_request_object(self, httpbin):
        s = requests.Session()
        with open(__file__, "rb") as f:
            files = {"file": f}
            req = requests.Request("POST", httpbin("post"), files=files)
            prep = s.prepare_request(req)
        assert isinstance(prep.method, builtin_str)
        assert prep.method == "POST"

        resp = s.send(prep)
        assert resp.status_code == 200

    def test_non_prepared_request_error(self):
        s = requests.Session()
        req = requests.Request("POST", "/")

        with pytest.raises(ValueError) as e:
            s.send(req)
        assert str(e.value) == "You can only send PreparedRequests."

    def test_custom_content_type(self, httpbin):
        with open(__file__, "rb") as f1:
            with open(__file__, "rb") as f2:
                data = {"stuff": json.dumps({"a": 123})}
                files = {
                    "file1": ("test_requests.py", f1),
                    "file2": ("test_requests", f2, "text/py-content-type"),
                }
                r = requests.post(httpbin("post"), data=data, files=files)
        assert r.status_code == 200
        assert b"text/py-content-type" in r.request.body

    def test_hook_receives_request_arguments(self, httpbin):
        def hook(resp, **kwargs):
            assert resp is not None
            assert kwargs != {}

        s = requests.Session()
        r = requests.Request("GET", httpbin(), hooks={"response": hook})
        prep = s.prepare_request(r)
        s.send(prep)

    def test_session_hooks_are_used_with_no_request_hooks(self, httpbin):
        def hook(*args, **kwargs):
            pass

        s = requests.Session()
        s.hooks["response"].append(hook)
        r = requests.Request("GET", httpbin())
        prep = s.prepare_request(r)
        assert prep.hooks["response"] != []
        assert prep.hooks["response"] == [hook]

    def test_session_hooks_are_overridden_by_request_hooks(self, httpbin):
        def hook1(*args, **kwargs):
            pass

        def hook2(*args, **kwargs):
            pass

        assert hook1 is not hook2
        s = requests.Session()
        s.hooks["response"].append(hook2)
        r = requests.Request("GET", httpbin(), hooks={"response": [hook1]})
        prep = s.prepare_request(r)
        assert prep.hooks["response"] == [hook1]

    def test_prepared_request_hook(self, httpbin):
        def hook(resp, **kwargs):
            resp.hook_working = True
            return resp

        req = requests.Request("GET", httpbin(), hooks={"response": hook})
        prep = req.prepare()

        s = requests.Session()
        s.proxies = getproxies()
        resp = s.send(prep)

        assert hasattr(resp, "hook_working")

    def test_prepared_from_session(self, httpbin):
        class DummyAuth(requests.auth.AuthBase):
            def __call__(self, r):
                r.headers["Dummy-Auth-Test"] = "dummy-auth-test-ok"
                return r

        req = requests.Request("GET", httpbin("headers"))
        assert not req.auth

        s = requests.Session()
        s.auth = DummyAuth()

        prep = s.prepare_request(req)
        resp = s.send(prep)

        assert resp.json()["headers"]["Dummy-Auth-Test"] == "dummy-auth-test-ok"

    def test_prepare_request_with_bytestring_url(self):
        req = requests.Request("GET", b"https://httpbin.org/")
        s = requests.Session()
        prep = s.prepare_request(req)
        assert prep.url == "https://httpbin.org/"

    def test_request_with_bytestring_host(self, httpbin):
        s = requests.Session()
        resp = s.request(
            "GET",
            httpbin("cookies/set?cookie=value"),
            allow_redirects=False,
            headers={"Host": b"httpbin.org"},
        )
        assert resp.cookies.get("cookie") == "value"

    def test_links(self):
        r = requests.Response()
        r.headers = {
            "cache-control": "public, max-age=60, s-maxage=60",
            "connection": "keep-alive",
            "content-encoding": "gzip",
            "content-type": "application/json; charset=utf-8",
            "date": "Sat, 26 Jan 2013 16:47:56 GMT",
            "etag": '"6ff6a73c0e446c1f61614769e3ceb778"',
            "last-modified": "Sat, 26 Jan 2013 16:22:39 GMT",
            "link": (
                "<https://api.github.com/users/kennethreitz/repos?"
                'page=2&per_page=10>; rel="next", <https://api.github.'
                "com/users/kennethreitz/repos?page=7&per_page=10>; "
                ' rel="last"'
            ),
            "server": "GitHub.com",
            "status": "200 OK",
            "vary": "Accept",
            "x-content-type-options": "nosniff",
            "x-github-media-type": "github.beta",
            "x-ratelimit-limit": "60",
            "x-ratelimit-remaining": "57",
        }
        assert r.links["next"]["rel"] == "next"

    def test_cookie_parameters(self):
        key = "some_cookie"
        value = "some_value"
        secure = True
        domain = "test.com"
        rest = {"HttpOnly": True}

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value, secure=secure, domain=domain, rest=rest)

        assert len(jar) == 1
        assert "some_cookie" in jar

        cookie = list(jar)[0]
        assert cookie.secure == secure
        assert cookie.domain == domain
        assert cookie._rest["HttpOnly"] == rest["HttpOnly"]

    def test_cookie_as_dict_keeps_len(self):
        key = "some_cookie"
        value = "some_value"

        key1 = "some_cookie1"
        value1 = "some_value1"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value)
        jar.set(key1, value1)

        d1 = dict(jar)
        d2 = dict(jar.iteritems())
        d3 = dict(jar.items())

        assert len(jar) == 2
        assert len(d1) == 2
        assert len(d2) == 2
        assert len(d3) == 2

    def test_cookie_as_dict_keeps_items(self):
        key = "some_cookie"
        value = "some_value"

        key1 = "some_cookie1"
        value1 = "some_value1"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value)
        jar.set(key1, value1)

        d1 = dict(jar)
        d2 = dict(jar.iteritems())
        d3 = dict(jar.items())

        assert d1["some_cookie"] == "some_value"
        assert d2["some_cookie"] == "some_value"
        assert d3["some_cookie1"] == "some_value1"

    def test_cookie_as_dict_keys(self):
        key = "some_cookie"
        value = "some_value"

        key1 = "some_cookie1"
        value1 = "some_value1"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value)
        jar.set(key1, value1)

        keys = jar.keys()
        assert keys == list(keys)
        # make sure one can use keys multiple times
        assert list(keys) == list(keys)

    def test_cookie_as_dict_values(self):
        key = "some_cookie"
        value = "some_value"

        key1 = "some_cookie1"
        value1 = "some_value1"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value)
        jar.set(key1, value1)

        values = jar.values()
        assert values == list(values)
        # make sure one can use values multiple times
        assert list(values) == list(values)

    def test_cookie_as_dict_items(self):
        key = "some_cookie"
        value = "some_value"

        key1 = "some_cookie1"
        value1 = "some_value1"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value)
        jar.set(key1, value1)

        items = jar.items()
        assert items == list(items)
        # make sure one can use items multiple times
        assert list(items) == list(items)

    def test_cookie_duplicate_names_different_domains(self):
        key = "some_cookie"
        value = "some_value"
        domain1 = "test1.com"
        domain2 = "test2.com"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value, domain=domain1)
        jar.set(key, value, domain=domain2)
        assert key in jar
        items = jar.items()
        assert len(items) == 2

        # Verify that CookieConflictError is raised if domain is not specified
        with pytest.raises(requests.cookies.CookieConflictError):
            jar.get(key)

        # Verify that CookieConflictError is not raised if domain is specified
        cookie = jar.get(key, domain=domain1)
        assert cookie == value

    def test_cookie_duplicate_names_raises_cookie_conflict_error(self):
        key = "some_cookie"
        value = "some_value"
        path = "some_path"

        jar = requests.cookies.RequestsCookieJar()
        jar.set(key, value, path=path)
        jar.set(key, value)
        with pytest.raises(requests.cookies.CookieConflictError):
            jar.get(key)

    def test_cookie_policy_copy(self):
        class MyCookiePolicy(cookielib.DefaultCookiePolicy):
            pass

        jar = requests.cookies.RequestsCookieJar()
        jar.set_policy(MyCookiePolicy())
        assert isinstance(jar.copy().get_policy(), MyCookiePolicy)

    def test_time_elapsed_blank(self, httpbin):
        r = requests.get(httpbin("get"))
        td = r.elapsed
        total_seconds = (
            td.microseconds + (td.seconds + td.days * 24 * 3600) * 10**6
        ) / 10**6
        assert total_seconds > 0.0

    def test_empty_response_has_content_none(self):
        r = requests.Response()
        assert r.content is None

    def test_response_is_iterable(self):
        r = requests.Response()
        io = StringIO.StringIO("abc")
        read_ = io.read

        def read_mock(amt, decode_content=None):
            return read_(amt)

        setattr(io, "read", read_mock)
        r.raw = io
        assert next(iter(r))
        io.close()

    def test_response_decode_unicode(self):
        """When called with decode_unicode, Response.iter_content should always
        return unicode.
        """
        r = requests.Response()
        r._content_consumed = True
        r._content = b"the content"
        r.encoding = "ascii"

        chunks = r.iter_content(decode_unicode=True)
        assert all(isinstance(chunk, str) for chunk in chunks)

        # also for streaming
        r = requests.Response()
        r.raw = io.BytesIO(b"the content")
        r.encoding = "ascii"
        chunks = r.iter_content(decode_unicode=True)
        assert all(isinstance(chunk, str) for chunk in chunks)

    def test_response_reason_unicode(self):
        # check for unicode HTTP status
        r = requests.Response()
        r.url = "unicode URL"
        r.reason = "Komponenttia ei löydy".encode()
        r.status_code = 404
        r.encoding = None
        assert not r.ok  # old behaviour - crashes here

    def test_response_reason_unicode_fallback(self):
        # check raise_status falls back to ISO-8859-1
        r = requests.Response()
        r.url = "some url"
        reason = "Komponenttia ei löydy"
        r.reason = reason.encode("latin-1")
        r.status_code = 500
        r.encoding = None
        with pytest.raises(requests.exceptions.HTTPError) as e:
            r.raise_for_status()
        assert reason in e.value.args[0]

    def test_response_chunk_size_type(self):
        """Ensure that chunk_size is passed as None or an integer, otherwise
        raise a TypeError.
        """
        r = requests.Response()
        r.raw = io.BytesIO(b"the content")
        chunks = r.iter_content(1)
        assert all(len(chunk) == 1 for chunk in chunks)

        r = requests.Response()
        r.raw = io.BytesIO(b"the content")
        chunks = r.iter_content(None)
        assert list(chunks) == [b"the content"]

        r = requests.Response()
        r.raw = io.BytesIO(b"the content")
        with pytest.raises(TypeError):
            chunks = r.iter_content("1024")

    @pytest.mark.parametrize(
        "exception, args, expected",
        (
            (urllib3.exceptions.ProtocolError, tuple(), ChunkedEncodingError),
            (urllib3.exceptions.DecodeError, tuple(), ContentDecodingError),
            (urllib3.exceptions.ReadTimeoutError, (None, "", ""), ConnectionError),
            (urllib3.exceptions.SSLError, tuple(), RequestsSSLError),
        ),
    )
    def test_iter_content_wraps_exceptions(self, httpbin, exception, args, expected):
        r = requests.Response()
        r.raw = mock.Mock()
        # ReadTimeoutError can't be initialized by mock
        # so we'll manually create the instance with args
        r.raw.stream.side_effect = exception(*args)

        with pytest.raises(expected):
            next(r.iter_content(1024))

    def test_request_and_response_are_pickleable(self, httpbin):
        r = requests.get(httpbin("get"))

        # verify we can pickle the original request
        assert pickle.loads(pickle.dumps(r.request))

        # verify we can pickle the response and that we have access to
        # the original request.
        pr = pickle.loads(pickle.dumps(r))
        assert r.request.url == pr.request.url
        assert r.request.headers == pr.request.headers

    def test_prepared_request_is_pickleable(self, httpbin):
        p = requests.Request("GET", httpbin("get")).prepare()

        # Verify PreparedRequest can be pickled and unpickled
        r = pickle.loads(pickle.dumps(p))
        assert r.url == p.url
        assert r.headers == p.headers
        assert r.body == p.body

        # Verify unpickled PreparedRequest sends properly
        s = requests.Session()
        resp = s.send(r)
        assert resp.status_code == 200

    def test_prepared_request_with_file_is_pickleable(self, httpbin):
        with open(__file__, "rb") as f:
            r = requests.Request("POST", httpbin("post"), files={"file": f})
            p = r.prepare()

        # Verify PreparedRequest can be pickled and unpickled
        r = pickle.loads(pickle.dumps(p))
        assert r.url == p.url
        assert r.headers == p.headers
        assert r.body == p.body

        # Verify unpickled PreparedRequest sends properly
        s = requests.Session()
        resp = s.send(r)
        assert resp.status_code == 200

    def test_prepared_request_with_hook_is_pickleable(self, httpbin):
        r = requests.Request("GET", httpbin("get"), hooks=default_hooks())
        p = r.prepare()

        # Verify PreparedRequest can be pickled
        r = pickle.loads(pickle.dumps(p))
        assert r.url == p.url
        assert r.headers == p.headers
        assert r.body == p.body
        assert r.hooks == p.hooks

        # Verify unpickled PreparedRequest sends properly
        s = requests.Session()
        resp = s.send(r)
        assert resp.status_code == 200

    def test_cannot_send_unprepared_requests(self, httpbin):
        r = requests.Request(url=httpbin())
        with pytest.raises(ValueError):
            requests.Session().send(r)

    def test_http_error(self):
        error = requests.exceptions.HTTPError()
        assert not error.response
        response = requests.Response()
        error = requests.exceptions.HTTPError(response=response)
        assert error.response == response
        error = requests.exceptions.HTTPError("message", response=response)
        assert str(error) == "message"
        assert error.response == response

    def test_session_pickling(self, httpbin):
        r = requests.Request("GET", httpbin("get"))
        s = requests.Session()

        s = pickle.loads(pickle.dumps(s))
        s.proxies = getproxies()

        r = s.send(r.prepare())
        assert r.status_code == 200

    def test_fixes_1329(self, httpbin):
        """Ensure that header updates are done case-insensitively."""
        s = requests.Session()
        s.headers.update({"ACCEPT": "BOGUS"})
        s.headers.update({"accept": "application/json"})
        r = s.get(httpbin("get"))
        headers = r.request.headers
        assert headers["accept"] == "application/json"
        assert headers["Accept"] == "application/json"
        assert headers["ACCEPT"] == "application/json"

    def test_uppercase_scheme_redirect(self, httpbin):
        parts = urlparse(httpbin("html"))
        url = "HTTP://" + parts.netloc + parts.path
        r = requests.get(httpbin("redirect-to"), params={"url": url})
        assert r.status_code == 200
        assert r.url.lower() == url.lower()

    def test_transport_adapter_ordering(self):
        s = requests.Session()
        order = ["https://", "http://"]
        assert order == list(s.adapters)
        s.mount("http://git", HTTPAdapter())
        s.mount("http://github", HTTPAdapter())
        s.mount("http://github.com", HTTPAdapter())
        s.mount("http://github.com/about/", HTTPAdapter())
        order = [
            "http://github.com/about/",
            "http://github.com",
            "http://github",
            "http://git",
            "https://",
            "http://",
        ]
        assert order == list(s.adapters)
        s.mount("http://gittip", HTTPAdapter())
        s.mount("http://gittip.com", HTTPAdapter())
        s.mount("http://gittip.com/about/", HTTPAdapter())
        order = [
            "http://github.com/about/",
            "http://gittip.com/about/",
            "http://github.com",
            "http://gittip.com",
            "http://github",
            "http://gittip",
            "http://git",
            "https://",
            "http://",
        ]
        assert order == list(s.adapters)
        s2 = requests.Session()
        s2.adapters = {"http://": HTTPAdapter()}
        s2.mount("https://", HTTPAdapter())
        assert "http://" in s2.adapters
        assert "https://" in s2.adapters

    def test_session_get_adapter_prefix_matching(self):
        prefix = "https://example.com"
        more_specific_prefix = prefix + "/some/path"

        url_matching_only_prefix = prefix + "/another/path"
        url_matching_more_specific_prefix = more_specific_prefix + "/longer/path"
        url_not_matching_prefix = "https://another.example.com/"

        s = requests.Session()
        prefix_adapter = HTTPAdapter()
        more_specific_prefix_adapter = HTTPAdapter()
        s.mount(prefix, prefix_adapter)
        s.mount(more_specific_prefix, more_specific_prefix_adapter)

        assert s.get_adapter(url_matching_only_prefix) is prefix_adapter
        assert (
            s.get_adapter(url_matching_more_specific_prefix)
            is more_specific_prefix_adapter
        )
        assert s.get_adapter(url_not_matching_prefix) not in (
            prefix_adapter,
            more_specific_prefix_adapter,
        )

    def test_session_get_adapter_prefix_matching_mixed_case(self):
        mixed_case_prefix = "hTtPs://eXamPle.CoM/MixEd_CAse_PREfix"
        url_matching_prefix = mixed_case_prefix + "/full_url"

        s = requests.Session()
        my_adapter = HTTPAdapter()
        s.mount(mixed_case_prefix, my_adapter)

        assert s.get_adapter(url_matching_prefix) is my_adapter

    def test_session_get_adapter_prefix_matching_is_case_insensitive(self):
        mixed_case_prefix = "hTtPs://eXamPle.CoM/MixEd_CAse_PREfix"
        url_matching_prefix_with_different_case = (
            "HtTpS://exaMPLe.cOm/MiXeD_caSE_preFIX/another_url"
        )

        s = requests.Session()
        my_adapter = HTTPAdapter()
        s.mount(mixed_case_prefix, my_adapter)

        assert s.get_adapter(url_matching_prefix_with_different_case) is my_adapter

    def test_session_get_adapter_prefix_with_trailing_slash(self):
        # from issue #6935
        prefix = "https://example.com/"  # trailing slash
        url_matching_prefix = "https://example.com/some/path"
        url_not_matching_prefix = "https://example.com.other.com/some/path"

        s = requests.Session()
        adapter = HTTPAdapter()
        s.mount(prefix, adapter)

        assert s.get_adapter(url_matching_prefix) is adapter
        assert s.get_adapter(url_not_matching_prefix) is not adapter

    def test_session_get_adapter_prefix_without_trailing_slash(self):
        # from issue #6935
        prefix = "https://example.com"  # no trailing slash
        url_matching_prefix = "https://example.com/some/path"
        url_extended_hostname = "https://example.com.other.com/some/path"

        s = requests.Session()
        adapter = HTTPAdapter()
        s.mount(prefix, adapter)

        assert s.get_adapter(url_matching_prefix) is adapter
        assert s.get_adapter(url_extended_hostname) is adapter

    def test_header_remove_is_case_insensitive(self, httpbin):
        # From issue #1321
        s = requests.Session()
        s.headers["foo"] = "bar"
        r = s.get(httpbin("get"), headers={"FOO": None})
        assert "foo" not in r.request.headers

    def test_params_are_merged_case_sensitive(self, httpbin):
        s = requests.Session()
        s.params["foo"] = "bar"
        r = s.get(httpbin("get"), params={"FOO": "bar"})
        assert r.json()["args"] == {"foo": "bar", "FOO": "bar"}

    def test_long_authinfo_in_url(self):
        url = "http://{}:{}@{}:9000/path?query#frag".format(
            "E8A3BE87-9E3F-4620-8858-95478E385B5B",
            "EA770032-DA4D-4D84-8CE9-29C6D910BF1E",
            "exactly-------------sixty-----------three------------characters",
        )
        r = requests.Request("GET", url).prepare()
        assert r.url == url

    def test_header_keys_are_native(self, httpbin):
        headers = {"unicode": "blah", b"byte": "blah"}
        r = requests.Request("GET", httpbin("get"), headers=headers)
        p = r.prepare()

        # This is testing that they are builtin strings. A bit weird, but there
        # we go.
        assert "unicode" in p.headers.keys()
        assert "byte" in p.headers.keys()

    def test_header_validation(self, httpbin):
        """Ensure prepare_headers regex isn't flagging valid header contents."""
        valid_headers = {
            "foo": "bar baz qux",
            "bar": b"fbbq",
            "baz": "",
            "qux": "1",
        }
        r = requests.get(httpbin("get"), headers=valid_headers)
        for key in valid_headers.keys():
            assert valid_headers[key] == r.request.headers[key]

    @pytest.mark.parametrize(
        "invalid_header, key",
        (
            ({"foo": 3}, "foo"),
            ({"bar": {"foo": "bar"}}, "bar"),
            ({"baz": ["foo", "bar"]}, "baz"),
        ),
    )
    def test_header_value_not_str(self, httpbin, invalid_header, key):
        """Ensure the header value is of type string or bytes as
        per discussion in GH issue #3386
        """
        with pytest.raises(InvalidHeader) as excinfo:
            requests.get(httpbin("get"), headers=invalid_header)
        assert key in str(excinfo.value)

    @pytest.mark.parametrize(
        "invalid_header",
        (
            {"foo": "bar\r\nbaz: qux"},
            {"foo": "bar\n\rbaz: qux"},
            {"foo": "bar\nbaz: qux"},
            {"foo": "bar\rbaz: qux"},
            {"fo\ro": "bar"},
            {"fo\r\no": "bar"},
            {"fo\n\ro": "bar"},
            {"fo\no": "bar"},
        ),
    )
    def test_header_no_return_chars(self, httpbin, invalid_header):
        """Ensure that a header containing return character sequences raise an
        exception. Otherwise, multiple headers are created from single string.
        """
        with pytest.raises(InvalidHeader):
            requests.get(httpbin("get"), headers=invalid_header)

    @pytest.mark.parametrize(
        "invalid_header",
        (
            {" foo": "bar"},
            {"\tfoo": "bar"},
            {"    foo": "bar"},
            {"foo": " bar"},
            {"foo": "    bar"},
            {"foo": "\tbar"},
            {" ": "bar"},
        ),
    )
    def test_header_no_leading_space(self, httpbin, invalid_header):
        """Ensure headers containing leading whitespace raise
        InvalidHeader Error before sending.
        """
        with pytest.raises(InvalidHeader):
            requests.get(httpbin("get"), headers=invalid_header)

    def test_header_with_subclass_types(self, httpbin):
        """If the subclasses does not behave *exactly* like
        the base bytes/str classes, this is not supported.
        This test is for backwards compatibility.
        """

        class MyString(str):
            pass

        class MyBytes(bytes):
            pass

        r_str = requests.get(httpbin("get"), headers={MyString("x-custom"): "myheader"})
        assert r_str.request.headers["x-custom"] == "myheader"

        r_bytes = requests.get(
            httpbin("get"), headers={MyBytes(b"x-custom"): b"myheader"}
        )
        assert r_bytes.request.headers["x-custom"] == b"myheader"

        r_mixed = requests.get(
            httpbin("get"), headers={MyString("x-custom"): MyBytes(b"myheader")}
        )
        assert r_mixed.request.headers["x-custom"] == b"myheader"

    @pytest.mark.parametrize("files", ("foo", b"foo", bytearray(b"foo")))
    def test_can_send_objects_with_files(self, httpbin, files):
        data = {"a": "this is a string"}
        files = {"b": files}
        r = requests.Request("POST", httpbin("post"), data=data, files=files)
        p = r.prepare()
        assert "multipart/form-data" in p.headers["Content-Type"]

    def test_can_send_file_object_with_non_string_filename(self, httpbin):
        f = io.BytesIO()
        f.name = 2
        r = requests.Request("POST", httpbin("post"), files={"f": f})
        p = r.prepare()

        assert "multipart/form-data" in p.headers["Content-Type"]

    def test_autoset_header_values_are_native(self, httpbin):
        data = "this is a string"
        length = "16"
        req = requests.Request("POST", httpbin("post"), data=data)
        p = req.prepare()

        assert p.headers["Content-Length"] == length

    def test_nonhttp_schemes_dont_check_URLs(self):
        test_urls = (
            "data:image/gif;base64,R0lGODlhAQABAHAAACH5BAUAAAAALAAAAAABAAEAAAICRAEAOw==",
            "file:///etc/passwd",
            "magnet:?xt=urn:btih:be08f00302bc2d1d3cfa3af02024fa647a271431",
        )
        for test_url in test_urls:
            req = requests.Request("GET", test_url)
            preq = req.prepare()
            assert test_url == preq.url

    def test_auth_is_stripped_on_http_downgrade(
        self, httpbin, httpbin_secure, httpbin_ca_bundle
    ):
        r = requests.get(
            httpbin_secure("redirect-to"),
            params={"url": httpbin("get")},
            auth=("user", "pass"),
            verify=httpbin_ca_bundle,
        )
        assert r.history[0].request.headers["Authorization"]
        assert "Authorization" not in r.request.headers

    def test_auth_is_retained_for_redirect_on_host(self, httpbin):
        r = requests.get(httpbin("redirect/1"), auth=("user", "pass"))
        h1 = r.history[0].request.headers["Authorization"]
        h2 = r.request.headers["Authorization"]

        assert h1 == h2

    def test_should_strip_auth_host_change(self):
        s = requests.Session()
        assert s.should_strip_auth(
            "http://example.com/foo", "http://another.example.com/"
        )

    def test_should_strip_auth_http_downgrade(self):
        s = requests.Session()
        assert s.should_strip_auth("https://example.com/foo", "http://example.com/bar")

    def test_should_strip_auth_https_upgrade(self):
        s = requests.Session()
        assert not s.should_strip_auth(
            "http://example.com/foo", "https://example.com/bar"
        )
        assert not s.should_strip_auth(
            "http://example.com:80/foo", "https://example.com/bar"
        )
        assert not s.should_strip_auth(
            "http://example.com/foo", "https://example.com:443/bar"
        )
        # Non-standard ports should trigger stripping
        assert s.should_strip_auth(
            "http://example.com:8080/foo", "https://example.com/bar"
        )
        assert s.should_strip_auth(
            "http://example.com/foo", "https://example.com:8443/bar"
        )

    def test_should_strip_auth_port_change(self):
        s = requests.Session()
        assert s.should_strip_auth(
            "http://example.com:1234/foo", "https://example.com:4321/bar"
        )

    @pytest.mark.parametrize(
        "old_uri, new_uri",
        (
            ("https://example.com:443/foo", "https://example.com/bar"),
            ("http://example.com:80/foo", "http://example.com/bar"),
            ("https://example.com/foo", "https://example.com:443/bar"),
            ("http://example.com/foo", "http://example.com:80/bar"),
        ),
    )
    def test_should_strip_auth_default_port(self, old_uri, new_uri):
        s = requests.Session()
        assert not s.should_strip_auth(old_uri, new_uri)

    def test_manual_redirect_with_partial_body_read(self, httpbin):
        s = requests.Session()
        r1 = s.get(httpbin("redirect/2"), allow_redirects=False, stream=True)
        assert r1.is_redirect
        rg = s.resolve_redirects(r1, r1.request, stream=True)

        # read only the first eight bytes of the response body,
        # then follow the redirect
        r1.iter_content(8)
        r2 = next(rg)
        assert r2.is_redirect

        # read all of the response via iter_content,
        # then follow the redirect
        for _ in r2.iter_content():
            pass
        r3 = next(rg)
        assert not r3.is_redirect

    def test_prepare_body_position_non_stream(self):
        data = b"the data"
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position is None

    def test_rewind_body(self):
        data = io.BytesIO(b"the data")
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position == 0
        assert prep.body.read() == b"the data"

        # the data has all been read
        assert prep.body.read() == b""

        # rewind it back
        requests.utils.rewind_body(prep)
        assert prep.body.read() == b"the data"

    def test_rewind_partially_read_body(self):
        data = io.BytesIO(b"the data")
        data.read(4)  # read some data
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position == 4
        assert prep.body.read() == b"data"

        # the data has all been read
        assert prep.body.read() == b""

        # rewind it back
        requests.utils.rewind_body(prep)
        assert prep.body.read() == b"data"

    def test_rewind_body_no_seek(self):
        class BadFileObj:
            def __init__(self, data):
                self.data = data

            def tell(self):
                return 0

            def __iter__(self):
                return

        data = BadFileObj("the data")
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position == 0

        with pytest.raises(UnrewindableBodyError) as e:
            requests.utils.rewind_body(prep)

        assert "Unable to rewind request body" in str(e)

    def test_rewind_body_failed_seek(self):
        class BadFileObj:
            def __init__(self, data):
                self.data = data

            def tell(self):
                return 0

            def seek(self, pos, whence=0):
                raise OSError()

            def __iter__(self):
                return

        data = BadFileObj("the data")
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position == 0

        with pytest.raises(UnrewindableBodyError) as e:
            requests.utils.rewind_body(prep)

        assert "error occurred when rewinding request body" in str(e)

    def test_rewind_body_failed_tell(self):
        class BadFileObj:
            def __init__(self, data):
                self.data = data

            def tell(self):
                raise OSError()

            def __iter__(self):
                return

        data = BadFileObj("the data")
        prep = requests.Request("GET", "http://example.com", data=data).prepare()
        assert prep._body_position is not None

        with pytest.raises(UnrewindableBodyError) as e:
            requests.utils.rewind_body(prep)

        assert "Unable to rewind request body" in str(e)

    def _patch_adapter_gzipped_redirect(self, session, url):
        adapter = session.get_adapter(url=url)
        org_build_response = adapter.build_response
        self._patched_response = False

        def build_response(*args, **kwargs):
            resp = org_build_response(*args, **kwargs)
            if not self._patched_response:
                resp.raw.headers["content-encoding"] = "gzip"
                self._patched_response = True
            return resp

        adapter.build_response = build_response

    def test_redirect_with_wrong_gzipped_header(self, httpbin):
        s = requests.Session()
        url = httpbin("redirect/1")
        self._patch_adapter_gzipped_redirect(s, url)
        s.get(url)

    @pytest.mark.parametrize(
        "username, password, auth_str",
        (
            ("test", "test", "Basic dGVzdDp0ZXN0"),
            (
                "имя".encode(),
                "пароль".encode(),
                "Basic 0LjQvNGPOtC/0LDRgNC+0LvRjA==",
            ),
        ),
    )
    def test_basic_auth_str_is_always_native(self, username, password, auth_str):
        s = _basic_auth_str(username, password)
        assert isinstance(s, builtin_str)
        assert s == auth_str

    def test_requests_history_is_saved(self, httpbin):
        r = requests.get(httpbin("redirect/5"))
        total = r.history[-1].history
        i = 0
        for item in r.history:
            assert item.history == total[0:i]
            i += 1

    def test_json_param_post_content_type_works(self, httpbin):
        r = requests.post(httpbin("post"), json={"life": 42})
        assert r.status_code == 200
        assert "application/json" in r.request.headers["Content-Type"]
        assert {"life": 42} == r.json()["json"]

    def test_json_param_post_should_not_override_data_param(self, httpbin):
        r = requests.Request(
            method="POST",
            url=httpbin("post"),
            data={"stuff": "elixr"},
            json={"music": "flute"},
        )
        prep = r.prepare()
        assert "stuff=elixr" == prep.body

    def test_response_iter_lines(self, httpbin):
        r = requests.get(httpbin("stream/4"), stream=True)
        assert r.status_code == 200

        it = r.iter_lines()
        next(it)
        assert len(list(it)) == 3

    def test_response_context_manager(self, httpbin):
        with requests.get(httpbin("stream/4"), stream=True) as response:
            assert isinstance(response, requests.Response)

        assert response.raw.closed

    def test_unconsumed_session_response_closes_connection(self, httpbin):
        s = requests.session()

        with contextlib.closing(s.get(httpbin("stream/4"), stream=True)) as response:
            pass

        assert response._content_consumed is False
        assert response.raw.closed

    @pytest.mark.xfail
    def test_response_iter_lines_reentrant(self, httpbin):
        """Response.iter_lines() is not reentrant safe"""
        r = requests.get(httpbin("stream/4"), stream=True)
        assert r.status_code == 200

        next(r.iter_lines())
        assert len(list(r.iter_lines())) == 3

    def test_session_close_proxy_clear(self):
        proxies = {
            "one": mock.Mock(),
            "two": mock.Mock(),
        }
        session = requests.Session()
        with mock.patch.dict(session.adapters["http://"].proxy_manager, proxies):
            session.close()
            proxies["one"].clear.assert_called_once_with()
            proxies["two"].clear.assert_called_once_with()

    def test_proxy_auth(self):
        adapter = HTTPAdapter()
        headers = adapter.proxy_headers("http://user:pass@httpbin.org")
        assert headers == {"Proxy-Authorization": "Basic dXNlcjpwYXNz"}

    def test_proxy_auth_empty_pass(self):
        adapter = HTTPAdapter()
        headers = adapter.proxy_headers("http://user:@httpbin.org")
        assert headers == {"Proxy-Authorization": "Basic dXNlcjo="}

    def test_response_json_when_content_is_None(self, httpbin):
        r = requests.get(httpbin("/status/204"))
        # Make sure r.content is None
        r.status_code = 0
        r._content = False
        r._content_consumed = False

        assert r.content is None
        with pytest.raises(ValueError):
            r.json()

    def test_response_without_release_conn(self):
        """Test `close` call for non-urllib3-like raw objects.
        Should work when `release_conn` attr doesn't exist on `response.raw`.
        """
        resp = requests.Response()
        resp.raw = StringIO.StringIO("test")
        assert not resp.raw.closed
        resp.close()
        assert resp.raw.closed

    def test_empty_stream_with_auth_does_not_set_content_length_header(self, httpbin):
        """Ensure that a byte stream with size 0 will not set both a Content-Length
        and Transfer-Encoding header.
        """
        auth = ("user", "pass")
        url = httpbin("post")
        file_obj = io.BytesIO(b"")
        r = requests.Request("POST", url, auth=auth, data=file_obj)
        prepared_request = r.prepare()
        assert "Transfer-Encoding" in prepared_request.headers
        assert "Content-Length" not in prepared_request.headers

    def test_stream_with_auth_does_not_set_transfer_encoding_header(self, httpbin):
        """Ensure that a byte stream with size > 0 will not set both a Content-Length
        and Transfer-Encoding header.
        """
        auth = ("user", "pass")
        url = httpbin("post")
        file_obj = io.BytesIO(b"test data")
        r = requests.Request("POST", url, auth=auth, data=file_obj)
        prepared_request = r.prepare()
        assert "Transfer-Encoding" not in prepared_request.headers
        assert "Content-Length" in prepared_request.headers

    def test_chunked_upload_does_not_set_content_length_header(self, httpbin):
        """Ensure that requests with a generator body stream using
        Transfer-Encoding: chunked, not a Content-Length header.
        """
        data = (i for i in [b"a", b"b", b"c"])
        url = httpbin("post")
        r = requests.Request("POST", url, data=data)
        prepared_request = r.prepare()
        assert "Transfer-Encoding" in prepared_request.headers
        assert "Content-Length" not in prepared_request.headers

    def test_custom_redirect_mixin(self, httpbin):
        """Tests a custom mixin to overwrite ``get_redirect_target``.

        Ensures a subclassed ``requests.Session`` can handle a certain type of
        malformed redirect responses.

        1. original request receives a proper response: 302 redirect
        2. following the redirect, a malformed response is given:
            status code = HTTP 200
            location = alternate url
        3. the custom session catches the edge case and follows the redirect
        """
        url_final = httpbin("html")
        querystring_malformed = urlencode({"location": url_final})
        url_redirect_malformed = httpbin("response-headers?%s" % querystring_malformed)
        querystring_redirect = urlencode({"url": url_redirect_malformed})
        url_redirect = httpbin("redirect-to?%s" % querystring_redirect)
        urls_test = [
            url_redirect,
            url_redirect_malformed,
            url_final,
        ]

        class CustomRedirectSession(requests.Session):
            def get_redirect_target(self, resp):
                # default behavior
                if resp.is_redirect:
                    return resp.headers["location"]
                # edge case - check to see if 'location' is in headers anyways
                location = resp.headers.get("location")
                if location and (location != resp.url):
                    return location
                return None

        session = CustomRedirectSession()
        r = session.get(urls_test[0])
        assert len(r.history) == 2
        assert r.status_code == 200
        assert r.history[0].status_code == 302
        assert r.history[0].is_redirect
        assert r.history[1].status_code == 200
        assert not r.history[1].is_redirect
        assert r.url == urls_test[2]
```


#### Issue #24: God Object detected: 'CustomMapping' has 6 methods and 2 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 2 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Medium
- **Location**: Lines 890-890

**Code Snippet**:

```
class CustomMapping(MutableMapping):
            def __init__(self, *args, **kwargs):
                self.data = dict(*args, **kwargs)

            def __delitem__(self, key):
                del self.data[key]

            def __getitem__(self, key):
                return self.data[key]

            def __setitem__(self, key, value):
                self.data[key] = value

            def __iter__(self):
                return iter(self.data)

            def __len__(self):
                return len(self.data)
```


#### Issue #25: God Object detected: 'TestCaseInsensitiveDict' has 17 methods and 37 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 11 trivial, 6 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Critical
- **Location**: Lines 2277-2277

**Code Snippet**:

```
class TestCaseInsensitiveDict:
    @pytest.mark.parametrize(
        "cid",
        (
            CaseInsensitiveDict({"Foo": "foo", "BAr": "bar"}),
            CaseInsensitiveDict([("Foo", "foo"), ("BAr", "bar")]),
            CaseInsensitiveDict(FOO="foo", BAr="bar"),
        ),
    )
    def test_init(self, cid):
        assert len(cid) == 2
        assert "foo" in cid
        assert "bar" in cid

    def test_docstring_example(self):
        cid = CaseInsensitiveDict()
        cid["Accept"] = "application/json"
        assert cid["aCCEPT"] == "application/json"
        assert list(cid) == ["Accept"]

    def test_len(self):
        cid = CaseInsensitiveDict({"a": "a", "b": "b"})
        cid["A"] = "a"
        assert len(cid) == 2

    def test_getitem(self):
        cid = CaseInsensitiveDict({"Spam": "blueval"})
        assert cid["spam"] == "blueval"
        assert cid["SPAM"] == "blueval"

    def test_fixes_649(self):
        """__setitem__ should behave case-insensitively."""
        cid = CaseInsensitiveDict()
        cid["spam"] = "oneval"
        cid["Spam"] = "twoval"
        cid["sPAM"] = "redval"
        cid["SPAM"] = "blueval"
        assert cid["spam"] == "blueval"
        assert cid["SPAM"] == "blueval"
        assert list(cid.keys()) == ["SPAM"]

    def test_delitem(self):
        cid = CaseInsensitiveDict()
        cid["Spam"] = "someval"
        del cid["sPam"]
        assert "spam" not in cid
        assert len(cid) == 0

    def test_contains(self):
        cid = CaseInsensitiveDict()
        cid["Spam"] = "someval"
        assert "Spam" in cid
        assert "spam" in cid
        assert "SPAM" in cid
        assert "sPam" in cid
        assert "notspam" not in cid

    def test_get(self):
        cid = CaseInsensitiveDict()
        cid["spam"] = "oneval"
        cid["SPAM"] = "blueval"
        assert cid.get("spam") == "blueval"
        assert cid.get("SPAM") == "blueval"
        assert cid.get("sPam") == "blueval"
        assert cid.get("notspam", "default") == "default"

    def test_update(self):
        cid = CaseInsensitiveDict()
        cid["spam"] = "blueval"
        cid.update({"sPam": "notblueval"})
        assert cid["spam"] == "notblueval"
        cid = CaseInsensitiveDict({"Foo": "foo", "BAr": "bar"})
        cid.update({"fOO": "anotherfoo", "bAR": "anotherbar"})
        assert len(cid) == 2
        assert cid["foo"] == "anotherfoo"
        assert cid["bar"] == "anotherbar"

    def test_update_retains_unchanged(self):
        cid = CaseInsensitiveDict({"foo": "foo", "bar": "bar"})
        cid.update({"foo": "newfoo"})
        assert cid["bar"] == "bar"

    def test_iter(self):
        cid = CaseInsensitiveDict({"Spam": "spam", "Eggs": "eggs"})
        keys = frozenset(["Spam", "Eggs"])
        assert frozenset(iter(cid)) == keys

    def test_equality(self):
        cid = CaseInsensitiveDict({"SPAM": "blueval", "Eggs": "redval"})
        othercid = CaseInsensitiveDict({"spam": "blueval", "eggs": "redval"})
        assert cid == othercid
        del othercid["spam"]
        assert cid != othercid
        assert cid == {"spam": "blueval", "eggs": "redval"}
        assert cid != object()

    def test_setdefault(self):
        cid = CaseInsensitiveDict({"Spam": "blueval"})
        assert cid.setdefault("spam", "notblueval") == "blueval"
        assert cid.setdefault("notspam", "notblueval") == "notblueval"

    def test_lower_items(self):
        cid = CaseInsensitiveDict(
            {
                "Accept": "application/json",
                "user-Agent": "requests",
            }
        )
        keyset = frozenset(lowerkey for lowerkey, v in cid.lower_items())
        lowerkeyset = frozenset(["accept", "user-agent"])
        assert keyset == lowerkeyset

    def test_preserve_key_case(self):
        cid = CaseInsensitiveDict(
            {
                "Accept": "application/json",
                "user-Agent": "requests",
            }
        )
        keyset = frozenset(["Accept", "user-Agent"])
        assert frozenset(i[0] for i in cid.items()) == keyset
        assert frozenset(cid.keys()) == keyset
        assert frozenset(cid) == keyset

    def test_preserve_last_key_case(self):
        cid = CaseInsensitiveDict(
            {
                "Accept": "application/json",
                "user-Agent": "requests",
            }
        )
        cid.update({"ACCEPT": "application/json"})
        cid["USER-AGENT"] = "requests"
        keyset = frozenset(["ACCEPT", "USER-AGENT"])
        assert frozenset(i[0] for i in cid.items()) == keyset
        assert frozenset(cid.keys()) == keyset
        assert frozenset(cid) == keyset

    def test_copy(self):
        cid = CaseInsensitiveDict(
            {
                "Accept": "application/json",
                "user-Agent": "requests",
            }
        )
        cid_copy = cid.copy()
        assert cid == cid_copy
        cid["changed"] = True
        assert cid != cid_copy
```


#### Issue #26: God Object detected: 'TestTimeout' has 7 methods and 2 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 4 trivial, 3 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Medium
- **Location**: Lines 2483-2483

**Code Snippet**:

```
class TestTimeout:
    def test_stream_timeout(self, httpbin):
        try:
            requests.get(httpbin("delay/10"), timeout=2.0)
        except requests.exceptions.Timeout as e:
            assert "Read timed out" in e.args[0].args[0]

    @pytest.mark.parametrize(
        "timeout, error_text",
        (
            ((3, 4, 5), "(connect, read)"),
            ("foo", "must be an int, float or None"),
        ),
    )
    def test_invalid_timeout(self, httpbin, timeout, error_text):
        with pytest.raises(ValueError) as e:
            requests.get(httpbin("get"), timeout=timeout)
        assert error_text in str(e)

    @pytest.mark.parametrize("timeout", (None, Urllib3Timeout(connect=None, read=None)))
    def test_none_timeout(self, httpbin, timeout):
        """Check that you can set None as a valid timeout value.

        To actually test this behavior, we'd want to check that setting the
        timeout to None actually lets the request block past the system default
        timeout. However, this would make the test suite unbearably slow.
        Instead we verify that setting the timeout to None does not prevent the
        request from succeeding.
        """
        r = requests.get(httpbin("get"), timeout=timeout)
        assert r.status_code == 200

    @pytest.mark.parametrize(
        "timeout", ((None, 0.1), Urllib3Timeout(connect=None, read=0.1))
    )
    def test_read_timeout(self, httpbin, timeout):
        try:
            requests.get(httpbin("delay/10"), timeout=timeout)
            pytest.fail("The recv() request should time out.")
        except ReadTimeout:
            pass

    @pytest.mark.parametrize(
        "timeout", ((0.1, None), Urllib3Timeout(connect=0.1, read=None))
    )
    def test_connect_timeout(self, timeout):
        try:
            requests.get(TARPIT, timeout=timeout)
            pytest.fail("The connect() request should time out.")
        except ConnectTimeout as e:
            assert isinstance(e, ConnectionError)
            assert isinstance(e, Timeout)

    @pytest.mark.parametrize(
        "timeout", ((0.1, 0.1), Urllib3Timeout(connect=0.1, read=0.1))
    )
    def test_total_timeout_connect(self, timeout):
        try:
            requests.get(TARPIT, timeout=timeout)
            pytest.fail("The connect() request should time out.")
        except ConnectTimeout:
            pass

    def test_encoded_methods(self, httpbin):
        """See: https://github.com/psf/requests/issues/2316"""
        r = requests.request(b"GET", httpbin("get"))
        assert r.ok
```


#### Issue #27: God Object detected: 'RedirectSession' has 4 methods and 13 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 1 (>1 indicates low cohesion) Methods: 2 trivial, 2 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: High
- **Location**: Lines 2555-2555

**Code Snippet**:

```
class RedirectSession(SessionRedirectMixin):
    def __init__(self, order_of_redirects):
        self.redirects = order_of_redirects
        self.calls = []
        self.max_redirects = 30
        self.cookies = {}
        self.trust_env = False

    def send(self, *args, **kwargs):
        self.calls.append(SendCall(args, kwargs))
        return self.build_response()

    def build_response(self):
        request = self.calls[-1].args[0]
        r = requests.Response()

        try:
            r.status_code = int(self.redirects.pop(0))
        except IndexError:
            r.status_code = 200

        r.headers = CaseInsensitiveDict({"Location": "/"})
        r.raw = self._build_raw()
        r.request = request
        return r

    def _build_raw(self):
        string = StringIO.StringIO("")
        setattr(string, "release_conn", lambda *args: args)
        return string
```


#### Issue #28: God Object detected: 'TestPreparingURLs' has 18 methods and 41 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 12 trivial, 6 complex

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Critical
- **Location**: Lines 2706-2706

**Code Snippet**:

```
class TestPreparingURLs:
    @pytest.mark.parametrize(
        "url,expected",
        (
            ("http://google.com", "http://google.com/"),
            ("http://ジェーピーニック.jp", "http://xn--hckqz9bzb1cyrb.jp/"),
            ("http://xn--n3h.net/", "http://xn--n3h.net/"),
            ("http://ジェーピーニック.jp".encode(), "http://xn--hckqz9bzb1cyrb.jp/"),
            ("http://straße.de/straße", "http://xn--strae-oqa.de/stra%C3%9Fe"),
            (
                "http://straße.de/straße".encode(),
                "http://xn--strae-oqa.de/stra%C3%9Fe",
            ),
            (
                "http://Königsgäßchen.de/straße",
                "http://xn--knigsgchen-b4a3dun.de/stra%C3%9Fe",
            ),
            (
                "http://Königsgäßchen.de/straße".encode(),
                "http://xn--knigsgchen-b4a3dun.de/stra%C3%9Fe",
            ),
            (b"http://xn--n3h.net/", "http://xn--n3h.net/"),
            (
                b"http://[1200:0000:ab00:1234:0000:2552:7777:1313]:12345/",
                "http://[1200:0000:ab00:1234:0000:2552:7777:1313]:12345/",
            ),
            (
                "http://[1200:0000:ab00:1234:0000:2552:7777:1313]:12345/",
                "http://[1200:0000:ab00:1234:0000:2552:7777:1313]:12345/",
            ),
        ),
    )
    def test_preparing_url(self, url, expected):
        def normalize_percent_encode(x):
            # Helper function that normalizes equivalent
            # percent-encoded bytes before comparisons
            for c in re.findall(r"%[a-fA-F0-9]{2}", x):
                x = x.replace(c, c.upper())
            return x

        r = requests.Request("GET", url=url)
        p = r.prepare()
        assert normalize_percent_encode(p.url) == expected

    @pytest.mark.parametrize(
        "url",
        (
            b"http://*.google.com",
            b"http://*",
            "http://*.google.com",
            "http://*",
            "http://☃.net/",
        ),
    )
    def test_preparing_bad_url(self, url):
        r = requests.Request("GET", url=url)
        with pytest.raises(requests.exceptions.InvalidURL):
            r.prepare()

    @pytest.mark.parametrize("url, exception", (("http://:1", InvalidURL),))
    def test_redirecting_to_bad_url(self, httpbin, url, exception):
        with pytest.raises(exception):
            requests.get(httpbin("redirect-to"), params={"url": url})

    @pytest.mark.parametrize(
        "input, expected",
        (
            (
                b"http+unix://%2Fvar%2Frun%2Fsocket/path%7E",
                "http+unix://%2Fvar%2Frun%2Fsocket/path~",
            ),
            (
                "http+unix://%2Fvar%2Frun%2Fsocket/path%7E",
                "http+unix://%2Fvar%2Frun%2Fsocket/path~",
            ),
            (
                b"mailto:user@example.org",
                "mailto:user@example.org",
            ),
            (
                "mailto:user@example.org",
                "mailto:user@example.org",
            ),
            (
                b"data:SSDimaUgUHl0aG9uIQ==",
                "data:SSDimaUgUHl0aG9uIQ==",
            ),
        ),
    )
    def test_url_mutation(self, input, expected):
        """
        This test validates that we correctly exclude some URLs from
        preparation, and that we handle others. Specifically, it tests that
        any URL whose scheme doesn't begin with "http" is left alone, and
        those whose scheme *does* begin with "http" are mutated.
        """
        r = requests.Request("GET", url=input)
        p = r.prepare()
        assert p.url == expected

    @pytest.mark.parametrize(
        "input, params, expected",
        (
            (
                b"http+unix://%2Fvar%2Frun%2Fsocket/path",
                {"key": "value"},
                "http+unix://%2Fvar%2Frun%2Fsocket/path?key=value",
            ),
            (
                "http+unix://%2Fvar%2Frun%2Fsocket/path",
                {"key": "value"},
                "http+unix://%2Fvar%2Frun%2Fsocket/path?key=value",
            ),
            (
                b"mailto:user@example.org",
                {"key": "value"},
                "mailto:user@example.org",
            ),
            (
                "mailto:user@example.org",
                {"key": "value"},
                "mailto:user@example.org",
            ),
        ),
    )
    def test_parameters_for_nonstandard_schemes(self, input, params, expected):
        """
        Setting parameters for nonstandard schemes is allowed if those schemes
        begin with "http", and is forbidden otherwise.
        """
        r = requests.Request("GET", url=input, params=params)
        p = r.prepare()
        assert p.url == expected

    def test_post_json_nan(self, httpbin):
        data = {"foo": float("nan")}
        with pytest.raises(requests.exceptions.InvalidJSONError):
            requests.post(httpbin("post"), json=data)

    def test_json_decode_compatibility(self, httpbin):
        r = requests.get(httpbin("bytes/20"))
        with pytest.raises(requests.exceptions.JSONDecodeError) as excinfo:
            r.json()
        assert isinstance(excinfo.value, RequestException)
        assert isinstance(excinfo.value, JSONDecodeError)
        assert r.text not in str(excinfo.value)

    def test_json_decode_persists_doc_attr(self, httpbin):
        r = requests.get(httpbin("bytes/20"))
        with pytest.raises(requests.exceptions.JSONDecodeError) as excinfo:
            r.json()
        assert excinfo.value.doc == r.text

    def test_status_code_425(self):
        r1 = requests.codes.get("TOO_EARLY")
        r2 = requests.codes.get("too_early")
        r3 = requests.codes.get("UNORDERED")
        r4 = requests.codes.get("unordered")
        r5 = requests.codes.get("UNORDERED_COLLECTION")
        r6 = requests.codes.get("unordered_collection")

        assert r1 == 425
        assert r2 == 425
        assert r3 == 425
        assert r4 == 425
        assert r5 == 425
        assert r6 == 425

    def test_different_connection_pool_for_tls_settings_verify_True(self):
        def response_handler(sock):
            consume_socket_content(sock, timeout=0.5)
            sock.send(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: 18\r\n\r\n"
                b'\xff\xfe{\x00"\x00K0"\x00=\x00"\x00\xab0"\x00\r\n'
            )

        s = requests.Session()
        close_server = threading.Event()
        server = TLSServer(
            handler=response_handler,
            wait_to_close_event=close_server,
            requests_to_handle=3,
            cert_chain="tests/certs/expired/server/server.pem",
            keyfile="tests/certs/expired/server/server.key",
        )

        with server as (host, port):
            url = f"https://{host}:{port}"
            r1 = s.get(url, verify=False)
            assert r1.status_code == 200

            # Cannot verify self-signed certificate
            with pytest.raises(requests.exceptions.SSLError):
                s.get(url)

            close_server.set()
        assert 2 == len(s.adapters["https://"].poolmanager.pools)

    def test_different_connection_pool_for_tls_settings_verify_bundle_expired_cert(
        self,
    ):
        def response_handler(sock):
            consume_socket_content(sock, timeout=0.5)
            sock.send(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: 18\r\n\r\n"
                b'\xff\xfe{\x00"\x00K0"\x00=\x00"\x00\xab0"\x00\r\n'
            )

        s = requests.Session()
        close_server = threading.Event()
        server = TLSServer(
            handler=response_handler,
            wait_to_close_event=close_server,
            requests_to_handle=3,
            cert_chain="tests/certs/expired/server/server.pem",
            keyfile="tests/certs/expired/server/server.key",
        )

        with server as (host, port):
            url = f"https://{host}:{port}"
            r1 = s.get(url, verify=False)
            assert r1.status_code == 200

            # Has right trust bundle, but certificate expired
            with pytest.raises(requests.exceptions.SSLError):
                s.get(url, verify="tests/certs/expired/ca/ca.crt")

            close_server.set()
        assert 2 == len(s.adapters["https://"].poolmanager.pools)

    def test_different_connection_pool_for_tls_settings_verify_bundle_unexpired_cert(
        self,
    ):
        def response_handler(sock):
            consume_socket_content(sock, timeout=0.5)
            sock.send(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: 18\r\n\r\n"
                b'\xff\xfe{\x00"\x00K0"\x00=\x00"\x00\xab0"\x00\r\n'
            )

        s = requests.Session()
        close_server = threading.Event()
        server = TLSServer(
            handler=response_handler,
            wait_to_close_event=close_server,
            requests_to_handle=3,
            cert_chain="tests/certs/valid/server/server.pem",
            keyfile="tests/certs/valid/server/server.key",
        )

        with server as (host, port):
            url = f"https://{host}:{port}"
            r1 = s.get(url, verify=False)
            assert r1.status_code == 200

            r2 = s.get(url, verify="tests/certs/valid/ca/ca.crt")
            assert r2.status_code == 200

            close_server.set()
        assert 2 == len(s.adapters["https://"].poolmanager.pools)

    def test_different_connection_pool_for_mtls_settings(self):
        client_cert = None

        def response_handler(sock):
            nonlocal client_cert
            client_cert = sock.getpeercert()
            consume_socket_content(sock, timeout=0.5)
            sock.send(
                b"HTTP/1.1 200 OK\r\n"
                b"Content-Length: 18\r\n\r\n"
                b'\xff\xfe{\x00"\x00K0"\x00=\x00"\x00\xab0"\x00\r\n'
            )

        s = requests.Session()
        close_server = threading.Event()
        server = TLSServer(
            handler=response_handler,
            wait_to_close_event=close_server,
            requests_to_handle=2,
            cert_chain="tests/certs/expired/server/server.pem",
            keyfile="tests/certs/expired/server/server.key",
            mutual_tls=True,
            cacert="tests/certs/expired/ca/ca.crt",
        )

        cert = (
            "tests/certs/mtls/client/client.pem",
            "tests/certs/mtls/client/client.key",
        )
        with server as (host, port):
            url = f"https://{host}:{port}"
            r1 = s.get(url, verify=False, cert=cert)
            assert r1.status_code == 200
            with pytest.raises(requests.exceptions.SSLError):
                s.get(url, cert=cert)
            close_server.set()

        assert client_cert is not None
```


#### Issue #29: Component 'caller' has 1039 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/test_requests.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #30: Component 'caller' has 15 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/conftest.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #31: God Object detected: 'Server' has 11 methods and 16 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 7 trivial, 4 complex

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: Critical
- **Location**: Lines 25-25

**Code Snippet**:

```
class Server(threading.Thread):
    """Dummy server using for unit testing"""

    WAIT_EVENT_TIMEOUT = 5

    def __init__(
        self,
        handler=None,
        host="localhost",
        port=0,
        requests_to_handle=1,
        wait_to_close_event=None,
    ):
        super().__init__()

        self.handler = handler or consume_socket_content
        self.handler_results = []

        self.host = host
        self.port = port
        self.requests_to_handle = requests_to_handle

        self.wait_to_close_event = wait_to_close_event
        self.ready_event = threading.Event()
        self.stop_event = threading.Event()

    @classmethod
    def text_response_server(cls, text, request_timeout=0.5, **kwargs):
        def text_response_handler(sock):
            request_content = consume_socket_content(sock, timeout=request_timeout)
            sock.send(text.encode("utf-8"))

            return request_content

        return Server(text_response_handler, **kwargs)

    @classmethod
    def basic_response_server(cls, **kwargs):
        return cls.text_response_server(
            "HTTP/1.1 200 OK\r\n" + "Content-Length: 0\r\n\r\n", **kwargs
        )

    def run(self):
        try:
            self.server_sock = self._create_socket_and_bind()
            # in case self.port = 0
            self.port = self.server_sock.getsockname()[1]
            self.ready_event.set()
            self._handle_requests()

            if self.wait_to_close_event:
                self.wait_to_close_event.wait(self.WAIT_EVENT_TIMEOUT)
        finally:
            self.ready_event.set()  # just in case of exception
            self._close_server_sock_ignore_errors()
            self.stop_event.set()

    def _create_socket_and_bind(self):
        sock = socket.socket()
        sock.bind((self.host, self.port))
        sock.listen()
        return sock

    def _close_server_sock_ignore_errors(self):
        try:
            self.server_sock.close()
        except OSError:
            pass

    def _handle_requests(self):
        for _ in range(self.requests_to_handle):
            sock = self._accept_connection()
            if not sock:
                break

            handler_result = self.handler(sock)

            self.handler_results.append(handler_result)
            sock.close()

    def _accept_connection(self):
        try:
            ready, _, _ = select.select(
                [self.server_sock], [], [], self.WAIT_EVENT_TIMEOUT
            )
            if not ready:
                return None

            return self.server_sock.accept()[0]
        except OSError:
            return None

    def __enter__(self):
        self.start()
        if not self.ready_event.wait(self.WAIT_EVENT_TIMEOUT):
            raise RuntimeError("Timeout waiting for server to be ready.")
        return self.host, self.port

    def __exit__(self, exc_type, exc_value, traceback):
        if exc_type is None:
            self.stop_event.wait(self.WAIT_EVENT_TIMEOUT)
        else:
            if self.wait_to_close_event:
                # avoid server from waiting for event timeouts
                # if an exception is found in the main thread
                self.wait_to_close_event.set()

        # ensure server thread doesn't get stuck waiting for connections
        self._close_server_sock_ignore_errors()
        self.join()
        return False  # allow exceptions to propagate
```


#### Issue #32: Component 'caller' has 30 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/testserver/server.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #33: God Object detected: 'TestTestServer' has 12 methods and 29 fields. (Thresholds: methods>5, fields>8) LCOM4 score: 2 (>1 indicates low cohesion) Methods: 8 trivial, 4 complex

- **File**: `alpha_testing/requests/tests/test_testserver.py`
- **Severity**: Critical
- **Location**: Lines 11-11

**Code Snippet**:

```
class TestTestServer:
    def test_basic(self):
        """messages are sent and received properly"""
        question = b"success?"
        answer = b"yeah, success"

        def handler(sock):
            text = sock.recv(1000)
            assert text == question
            sock.sendall(answer)

        with Server(handler) as (host, port):
            sock = socket.socket()
            sock.connect((host, port))
            sock.sendall(question)
            text = sock.recv(1000)
            assert text == answer
            sock.close()

    def test_server_closes(self):
        """the server closes when leaving the context manager"""
        with Server.basic_response_server() as (host, port):
            sock = socket.socket()
            sock.connect((host, port))

            sock.close()

        with pytest.raises(socket.error):
            new_sock = socket.socket()
            new_sock.connect((host, port))

    def test_text_response(self):
        """the text_response_server sends the given text"""
        server = Server.text_response_server(
            "HTTP/1.1 200 OK\r\n" "Content-Length: 6\r\n" "\r\nroflol"
        )

        with server as (host, port):
            r = requests.get(f"http://{host}:{port}")

            assert r.status_code == 200
            assert r.text == "roflol"
            assert r.headers["Content-Length"] == "6"

    def test_basic_response(self):
        """the basic response server returns an empty http response"""
        with Server.basic_response_server() as (host, port):
            r = requests.get(f"http://{host}:{port}")
            assert r.status_code == 200
            assert r.text == ""
            assert r.headers["Content-Length"] == "0"

    def test_basic_waiting_server(self):
        """the server waits for the block_server event to be set before closing"""
        block_server = threading.Event()

        with Server.basic_response_server(wait_to_close_event=block_server) as (
            host,
            port,
        ):
            sock = socket.socket()
            sock.connect((host, port))
            sock.sendall(b"send something")
            time.sleep(2.5)
            sock.sendall(b"still alive")
            block_server.set()  # release server block

    def test_multiple_requests(self):
        """multiple requests can be served"""
        requests_to_handle = 5

        server = Server.basic_response_server(requests_to_handle=requests_to_handle)

        with server as (host, port):
            server_url = f"http://{host}:{port}"
            for _ in range(requests_to_handle):
                r = requests.get(server_url)
                assert r.status_code == 200

            # the (n+1)th request fails
            with pytest.raises(requests.exceptions.ConnectionError):
                r = requests.get(server_url)

    @pytest.mark.skip(reason="this fails non-deterministically under pytest-xdist")
    def test_request_recovery(self):
        """can check the requests content"""
        # TODO: figure out why this sometimes fails when using pytest-xdist.
        server = Server.basic_response_server(requests_to_handle=2)
        first_request = b"put your hands up in the air"
        second_request = b"put your hand down in the floor"

        with server as address:
            sock1 = socket.socket()
            sock2 = socket.socket()

            sock1.connect(address)
            sock1.sendall(first_request)
            sock1.close()

            sock2.connect(address)
            sock2.sendall(second_request)
            sock2.close()

        assert server.handler_results[0] == first_request
        assert server.handler_results[1] == second_request

    def test_requests_after_timeout_are_not_received(self):
        """the basic response handler times out when receiving requests"""
        server = Server.basic_response_server(request_timeout=1)

        with server as address:
            sock = socket.socket()
            sock.connect(address)
            time.sleep(1.5)
            sock.sendall(b"hehehe, not received")
            sock.close()

        assert server.handler_results[0] == b""

    def test_request_recovery_with_bigger_timeout(self):
        """a biggest timeout can be specified"""
        server = Server.basic_response_server(request_timeout=3)
        data = b"bananadine"

        with server as address:
            sock = socket.socket()
            sock.connect(address)
            time.sleep(1.5)
            sock.sendall(data)
            sock.close()

        assert server.handler_results[0] == data

    def test_server_finishes_on_error(self):
        """the server thread exits even if an exception exits the context manager"""
        server = Server.basic_response_server()
        with pytest.raises(Exception):
            with server:
                raise Exception()

        assert len(server.handler_results) == 0

        # if the server thread fails to finish, the test suite will hang
        # and get killed by the jenkins timeout.

    def test_server_finishes_when_no_connections(self):
        """the server thread exits even if there are no connections"""
        server = Server.basic_response_server()
        with server:
            pass

        assert len(server.handler_results) == 0

        # if the server thread fails to finish, the test suite will hang
        # and get killed by the jenkins timeout.
```


#### Issue #34: Component 'caller' has 59 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/test_testserver.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


#### Issue #35: Component 'caller' has 113 dependencies, exceeding critical threshold of 15

- **File**: `alpha_testing/requests/tests/test_lowlevel.py`
- **Severity**: Critical

**Code Snippet**:

```
caller
```

**AI Analysis**:

Consider reducing dependencies through dependency injection or interface abstraction


## Architecture Diagrams

### God Objects

#### RequestEncodingMixin

```mermaid
classDiagram
    class RequestEncodingMixin {
        +guess_filename()
    }
```

#### Request

```mermaid
classDiagram
    class Request {
    }
```

#### PreparedRequest

```mermaid
classDiagram
    class PreparedRequest {
    }
```

#### Response

```mermaid
classDiagram
    class Response {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### SessionRedirectMixin

```mermaid
classDiagram
    class SessionRedirectMixin {
    }
```

#### Session

```mermaid
classDiagram
    class Session {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### CaseInsensitiveDict

```mermaid
classDiagram
    class CaseInsensitiveDict {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### MockRequest

```mermaid
classDiagram
    class MockRequest {
    }
```

#### RequestsCookieJar

```mermaid
classDiagram
    class RequestsCookieJar {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### HTTPDigestAuth

```mermaid
classDiagram
    class HTTPDigestAuth {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### HTTPAdapter

```mermaid
classDiagram
    class HTTPAdapter {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### TestSuperLen

```mermaid
classDiagram
    class TestSuperLen {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### TestCaseInsensitiveDict

```mermaid
classDiagram
    class TestCaseInsensitiveDict {
    }
```

#### TestRequests

```mermaid
classDiagram
    class TestRequests {
    }
```

#### CustomMapping

```mermaid
classDiagram
    class CustomMapping {
    }
```

#### TestCaseInsensitiveDict

```mermaid
classDiagram
    class TestCaseInsensitiveDict {
    }
```

#### TestTimeout

```mermaid
classDiagram
    class TestTimeout {
    }
```

#### RedirectSession

```mermaid
classDiagram
    class RedirectSession {
    }
```

#### TestPreparingURLs

```mermaid
classDiagram
    class TestPreparingURLs {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### Server

```mermaid
classDiagram
    class Server {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### TestTestServer

```mermaid
classDiagram
    class TestTestServer {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

#### caller

```mermaid
classDiagram
    class caller {
    }
```

