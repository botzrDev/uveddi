
Principled Security for Production-Ready Rust Web Services: A Comprehensive Analysis


Introduction


Purpose and Scope

This report provides an exhaustive analysis of security best practices, common vulnerabilities, and recommended tooling for developing web backends in Rust. It is intended for senior engineers and security architects tasked with building and securing production-grade services. The analysis moves from the foundational security principles inherent in the Rust language to practical, implementation-level guidance for core security domains such as authentication, authorization, input validation, and denial-of-service mitigation.

The Rust Value Proposition for Security

Rust has emerged as a compelling choice for secure systems programming, promising the performance of C/C++ without the associated risks of memory unsafety.1 Its unique combination of zero-cost abstractions, memory safety, and fearless concurrency provides a robust foundation for building reliable and performant web services.3 The language's design philosophy, centered on the ownership model and a strong static type system, allows the compiler to eliminate entire classes of common vulnerabilities at compile time, a paradigm shift from traditional languages where such issues must be found through runtime testing and manual code review.3
However, Rust is not a panacea. While it provides powerful guards against memory-related exploits, it does not automatically prevent logical vulnerabilities, insecure design patterns, or flaws in dependency management.3 A disciplined, defense-in-depth approach to security remains paramount. This report aims to provide the principled guidance necessary to leverage Rust's strengths while diligently mitigating the risks that remain, enabling the development of truly secure and resilient web backends.

Part I: The Rust Security Proposition

This part establishes the foundational security context of Rust, moving from its core language features to the real-world threats that developers must still address.

Section 1: A Paradigm Shift in Secure Systems Programming

The security benefits of Rust are not incidental; they are a direct consequence of core language design decisions that prioritize correctness and safety. Understanding these principles is the first step toward building secure applications.

1.1 Memory and Concurrency Safety by Default

The most celebrated feature of Rust is its compile-time enforcement of memory safety without a garbage collector. This is achieved through a trio of interconnected concepts: ownership, borrowing, and lifetimes. The compiler statically tracks which part of the code owns a piece of data, who can borrow it (and whether that borrow is mutable or immutable), and for how long those borrows are valid. This system makes it impossible to compile code that contains common memory safety errors.2
Buffer Overflows: Accessing an array or vector out of bounds is prevented. Operations like indexing are bounds-checked at runtime, while iterators provide a compile-time safe way to access elements.
Use-After-Free: The ownership system ensures that once a value is deallocated (its owner goes out of scope), no references to it can possibly exist. The compiler will refuse to compile code that attempts to use a reference after its referent has been dropped.
Null Pointer Dereferencing: Rust does not have null pointers. Instead, it uses the Option<T> enum, which forces the programmer to explicitly handle the case where a value might be absent (None), making accidental dereferencing of a "null" value a compile-time error.
Furthermore, this safety model extends to concurrency. Rust's type system includes the Send and Sync marker traits, which are used to enforce thread safety at compile time. A type is Send if it can be safely transferred to another thread, and Sync if it can be safely shared (via reference) between multiple threads. The compiler uses these traits to prevent data races, a class of concurrency bugs that can lead to unpredictable behavior and security vulnerabilities, before the program is ever run.3
The power of this approach is recognized in professional security audits. An audit of a Rust TLS library highlighted the use of the type system to statically encode the TLS state transition function as a "great defense-in-depth design" decision. This pattern makes it a compile-time error to use the TLS connection in an incorrect state, effectively eliminating a whole category of potential bugs.5

1.2 The unsafe Keyword: A Deliberate Contract

Rust provides an escape hatch for situations where the compiler's safety checks are too restrictive or when interfacing with lower-level systems is necessary. This is the unsafe keyword. Crucially, unsafe does not turn off the borrow checker or other safety mechanisms; rather, it allows the programmer to perform a small set of "superpowers" that the compiler cannot prove are safe.7 These include:
Dereferencing a raw pointer.
Calling an unsafe function or method (often from FFI).
Accessing or modifying a mutable static variable.
Implementing an unsafe trait.
Accessing fields of a union.
The unsafe keyword is a feature, not a bug. Its true value lies in its explicitness. Unlike in C or C++, where any line of code could potentially perform an unsafe memory operation, Rust forces these operations to be clearly demarcated within an unsafe block.5 This creates a clear contract: the code inside the
unsafe block is the programmer's responsibility to manually verify for soundness. It dramatically narrows the scope of a security audit, allowing reviewers to focus their attention on these well-defined critical sections.5
Best practices for using unsafe are stringent and widely adopted by the community:
Forbid by Default: Crates should begin with #![forbid(unsafe_code)] at the top level (lib.rs or main.rs). This makes any use of unsafe a conscious, explicit decision that requires overriding this attribute.9
Encapsulate and Justify: Any necessary unsafe code should be wrapped in a safe abstraction. The function providing this safe API should include a detailed // SAFETY: comment explaining why the use of unsafe is sound and what invariants the unsafe block relies on, which must be upheld by the safe code around it.7
The evolution of the Actix Web framework serves as a powerful case study. In its early days, it made extensive use of unsafe for performance. Following community feedback and security reviews, a significant effort was undertaken to remove all unsound uses of unsafe, demonstrating both the community's commitment to correctness and the utility of the unsafe keyword as a tool for focusing audit efforts.8
This leads to a critical understanding of the security posture in Rust. Since the compiler provides strong guarantees for safe code, the primary focus of a manual security review shifts. The goal becomes minimizing and rigorously auditing the "audit surface"—the total volume and complexity of unsafe code within the application and its entire dependency tree. Tools that detect the use of unsafe in dependencies, as recommended by the ANSSI security guide, are not mere linters; they are fundamental instruments for risk management.10 The security of a Rust application is, therefore, inversely proportional to the size of its unverified
unsafe footprint.

Section 2: The Modern Threat Landscape in a Rust Context

While Rust's language features provide a formidable defense against certain attacks, they do not render a web service immune to all threats. Logical vulnerabilities and flaws in how the application interacts with its environment remain significant risks.

2.1 Re-evaluating the OWASP Top 10 for Rust

Analyzing the OWASP Top 10 2021 list through the lens of Rust reveals a clear pattern: memory-related risks are heavily mitigated, while logic-based risks are largely unaffected by the choice of language.4
A01:2021-Broken Access Control: This is a purely logical vulnerability. Rust provides no special protection against a developer forgetting to check if a user is an administrator before allowing access to a privileged endpoint. Mitigation relies entirely on the correct implementation of authorization logic, for example, using RBAC middleware.4
A02:2021-Cryptographic Failures: This risk stems from weak algorithms, improper key management, or failure to encrypt sensitive data. Rust itself does not prevent these mistakes. Mitigation depends on using well-vetted cryptographic libraries like ring or rustls and following established best practices.4
A03:2021-Injection: Rust offers partial mitigation. Its strong type system and the standard library's design discourage the kind of dynamic string concatenation that often leads to command injection. However, SQL injection remains a threat if developers manually construct queries using string formatting. The primary mitigation is the disciplined use of parameterized queries, which is the default and idiomatic approach in libraries like sqlx.4
A04:2021-Insecure Design: This category addresses flaws at the architectural level, such as a missing rate limiter or an insecure password reset flow. It is entirely language-agnostic.4
A05:2021-Security Misconfiguration: This risk, which includes things like leaving default credentials or enabling verbose error messages in production, is a deployment and operational issue, not a language-level one.4
A07:2021-Identification and Authentication Failures: Similar to access control, these are logical flaws in how authentication flows are implemented. Rust does not prevent weak password policies or improper session management.4
A10:2021-Server-Side Request Forgery (SSRF): This vulnerability occurs when an attacker can induce the server to make requests to an unintended location. Mitigation relies on strict input validation of URLs and network egress filtering, which are application-level concerns.4
Memory Management Errors: The OWASP Top 10 for 2021 has a section on "Next Steps" which includes memory management errors. It explicitly notes that "In the case of Rust, memory safety is a crucial feature of the language," acknowledging its inherent resilience to this class of vulnerability compared to languages like C and C++.11

2.2 Real-World Vulnerabilities in the Rust Ecosystem

An analysis of historical security advisories from the RustSec database provides a practical view of where vulnerabilities tend to occur.12
Command Injection (CVE-2024-24576 / RUSTSEC-2024-0006): A critical vulnerability was discovered in the Rust standard library itself. On Windows, the std::process::Command API did not properly escape arguments when invoking batch files (.bat, .cmd). This allowed an attacker who could control the arguments to execute arbitrary shell commands.12 This is a stark reminder that even the standard library is not infallible and that platform-specific interactions are a potent source of risk.
Race Conditions: A TOCTOU (Time-of-check-to-time-of-use) vulnerability was found in std::fs::remove_dir_all. An attacker could exploit a race condition to replace a directory with a symbolic link after the program checked permissions but before it deleted the target, potentially tricking a privileged program into deleting files it shouldn't.12 This demonstrates that while Rust prevents data races, logical race conditions related to external state (like the filesystem) can still occur.
HTTP Protocol Parsing: A recurring source of vulnerabilities is the parsing of ambiguous or malformed HTTP requests. Advisories for hyper (the foundation of many Rust web frameworks), actix-http, and pingora-core have detailed issues that could lead to HTTP Request Smuggling or HTTP Desync attacks, where frontend and backend servers interpret the boundaries of a request differently, allowing an attacker to bypass security controls.14
Dependency Soundness and unsafe: A large number of advisories are classified as "Unsoundness." These occur when a crate exposes a safe API that can, through normal use, trigger undefined behavior because of an incorrect implementation of an internal unsafe block. This is the most common way for memory corruption vulnerabilities to appear in the ecosystem and underscores the importance of auditing unsafe code in dependencies.14
Denial of Service (DoS): DoS vulnerabilities are common. Examples include resource exhaustion bugs in the h2 crate (an HTTP/2 implementation) where a malicious client could cause a server to panic or consume excessive memory, and similar issues in GraphQL libraries like juniper and async-graphql where deeply nested queries could lead to resource exhaustion.14
These real-world examples reveal a subtle but critical vulnerability pattern. The most severe bugs, like the command injection and filesystem race condition, are not memory corruption issues within Rust's abstract machine. They are logical flaws that arise from a mismatch between the assumptions of a safe Rust abstraction and the complex, stateful, and often "unsafe" reality of the underlying system it interacts with—be it the Windows shell, the operating system's filesystem, or a complex network protocol. Rust's safety guarantees are strongest within its own ecosystem. The moment it interfaces with the outside world via FFI, OS-specific APIs, or protocol parsing, a new class of risk emerges. Therefore, security reviews must pay heightened attention to these bridge points, as they represent the seams where the guarantees of the language meet the unverified complexities of the real world.10

Part II: Core Security Pillars: Implementation and Best Practices

This part provides actionable guidance on implementing critical security controls, recommending specific crates and architectural patterns to build a layered defense for a Rust web API.

Section 3: Authentication: Verifying Identity

Authentication is the process of confirming a user's identity. In Rust, this is achieved using a combination of robust cryptographic libraries and well-defined authentication protocols.

3.1 Password Management

The foundational principle of password management is to never store raw, plaintext passwords. Instead, they must be processed with a modern, one-way cryptographic hash function before being stored.16
Recommended Algorithm and Crate: The argon2 crate is the recommended choice for password hashing in Rust.17 Argon2 was the winner of the Password Hashing Competition and is specifically designed to be resistant to both GPU-based cracking attempts and side-channel attacks. It is highly configurable in terms of memory cost, time cost, and parallelism.
Implementation: The process involves two steps: hashing and verification.
Hashing: When a user registers, generate a cryptographically secure random salt. Use the argon2 crate to hash the user's password with this salt. The library conveniently produces a single output string that contains the algorithm identifier, version, parameters, salt, and the final hash, ready for storage in a database column.17
Verification: When a user logs in, retrieve their stored hash string from the database. Use the argon2::PasswordVerifier::verify_password() function, providing the plaintext password from the login attempt and the stored hash. The library will automatically parse the parameters and salt from the stored hash string to perform the comparison in constant time, mitigating timing attacks.17
Rust
use argon2::{self, Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use rand::Rng;

// Hashing a new password
pub fn hash_password(password: &str) -> Result<String, argon2::Error> {
    let salt = rand::thread_rng().gen::<[u8; 16]>();
    let argon2 = Argon2::default();
    let password_hash = argon2.hash_password(password.as_bytes(), &salt)?;
    Ok(password_hash.to_string())
}

// Verifying a password against a stored hash
pub fn verify_password(hash: &str, password: &str) -> Result<bool, argon2::password_hash::Error> {
    let parsed_hash = PasswordHash::new(hash)?;
    Ok(Argon2::default().verify_password(password.as_bytes(), &parsed_hash).is_ok())
}



3.2 Token-Based Authentication (JWT)

JSON Web Tokens (JWTs) are an industry standard for creating access tokens that assert claims about a subject. They are ideal for stateless authentication in distributed systems and microservice architectures.19
Architecture and Best Practices: A key architectural recommendation is to differentiate between internal and external tokens. Use JWTs for internal, service-to-service communication where services can trust each other and benefit from the embedded claim information. For external clients, such as single-page applications or third-party integrators, it is more secure to issue opaque tokens. An API Gateway can then perform a translation, exchanging the opaque token for a JWT before forwarding the request to internal services. This pattern, known as the "Phantom Token" or "Split Token" approach, prevents the leakage of internal system details and avoids creating a rigid public contract based on the JWT's structure.20 Furthermore, token issuance should be centralized in a dedicated OAuth Authorization Server, which handles the complex logic of authenticating users and clients, rather than having individual APIs mint their own tokens.20
Recommended Crate: The jsonwebtoken crate is a mature and widely used library for creating and validating JWTs in Rust.4
Implementation:
Claims: Define a Rust struct that derives serde::Serialize and serde::Deserialize to represent the JWT claims. This should include standard claims like sub (subject, e.g., user ID) and exp (expiration time, as a Unix timestamp), along with any custom claims needed for authorization.17
Generation: Use jsonwebtoken::encode to create a token. This function takes a Header, a reference to the claims struct, and an EncodingKey. The secret key used for signing must be strong and loaded from a secure configuration source (e.g., environment variable or secret manager), never hardcoded in the source.17
Validation: Use jsonwebtoken::decode to validate an incoming token. This function takes the token string, a DecodingKey, and a Validation struct. The library automatically performs critical security checks, including verifying the cryptographic signature and ensuring the token has not expired.17
Rust
use jsonwebtoken::{encode, decode, Header, EncodingKey, DecodingKey, Validation, Algorithm};
use serde::{Deserialize, Serialize};
use chrono::{Utc, Duration};

#
struct Claims {
    sub: String, // Subject (user id)
    exp: usize,  // Expiration time
    role: String,
}

const SECRET_KEY: &[u8] = b"your-super-secret-key-that-is-at-least-32-bytes-long";

// Generating a JWT
fn generate_jwt(user_id: &str, role: &str) -> Result<String, jsonwebtoken::errors::Error> {
    let expiration = Utc::now()
       .checked_add_signed(Duration::hours(1))
       .expect("valid timestamp")
       .timestamp() as usize;
    let claims = Claims { sub: user_id.to_owned(), exp: expiration, role: role.to_owned() };
    encode(&Header::new(Algorithm::HS256), &claims, &EncodingKey::from_secret(SECRET_KEY))
}

// Verifying a JWT
fn verify_jwt(token: &str) -> Result<Claims, jsonwebtoken::errors::Error> {
    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(SECRET_KEY),
        &Validation::new(Algorithm::HS256),
    )?;
    Ok(token_data.claims)
}



3.3 Stateful Session Management

For traditional monolithic web applications, server-side sessions remain a viable and secure authentication method. In this model, the server creates a session, stores the session data (e.g., in Redis or a database), and provides the client with a cookie containing only a random session ID.23
Security Best Practices: The session cookie must be properly secured to prevent attacks.
HttpOnly: Prevents the cookie from being accessed by client-side JavaScript, mitigating Cross-Site Scripting (XSS) attacks aimed at stealing the session token.24
Secure: Ensures the cookie is only sent over HTTPS connections, preventing it from being intercepted on insecure networks.24
SameSite: Controls when the cookie is sent with cross-origin requests, providing strong protection against Cross-Site Request Forgery (CSRF). SameSite=Strict or SameSite=Lax are recommended settings.25
Crate Comparison: The Rust ecosystem provides robust session management libraries for each major web framework.
actix-session: The official middleware for Actix Web. It is highly configurable and supports multiple storage backends, including a pure cookie-based store (CookieSessionStore) for self-contained sessions and a Redis backend (RedisSessionStore) for distributed systems. It requires a secret Key for signing and encrypting the session cookie.26
axum-session: A comprehensive session library for Axum. It boasts a wide range of features, including support for numerous persistent backends (SQLx, Redis, MongoDB, SurrealDB), automatic cookie/header signing, session data encryption, and IP/User-Agent binding to deter cookie theft.29
rocket_session: A community-provided session crate for Rocket. It is designed for simplicity, primarily offering cookie-based sessions. It uses a per-session mutex accessed via a .tap() method to ensure thread-safe access to session data.32

3.4 Federated Identity (OAuth 2.0 / OIDC)

OAuth 2.0 is the industry-standard protocol for authorization, commonly used to implement "Log in with Google/GitHub" functionality, allowing users to grant a third-party application limited access to their data without sharing their credentials.
Recommended Crate: The oauth2 crate is the de facto standard for implementing OAuth 2.0 clients in Rust. It provides strongly-typed, extensible, and asynchronous implementations of the various grant flows defined in the RFC.18
Implementation Focus (Authorization Code Grant): The Authorization Code Grant is the most secure and recommended flow for web server applications. The process involves the following high-level steps 33:
Configuration: Instantiate an oauth2::BasicClient with the provider's details: Client ID, Client Secret, Authorization URL, and Token URL. These should be loaded from a secure configuration.
Redirect: Generate a unique, unguessable state parameter to prevent CSRF attacks. Construct the full authorization URL using client.authorize_url() and redirect the user's browser to this URL.
Callback Handling: The user authenticates with the provider and authorizes the application. The provider then redirects the user back to a pre-configured callback URL in your application. This redirect will include an authorization_code and the state parameter.
State Verification: Your callback handler must first verify that the returned state parameter matches the one generated in step 2. If they do not match, the request must be rejected as it may be a CSRF attempt.
Token Exchange: If the state is valid, exchange the received authorization_code for an access token (and optionally a refresh token) by making a POST request to the provider's token endpoint. The oauth2 crate handles this with the client.exchange_code() method.
API Access: Use the obtained access token to make authenticated requests to the provider's resource server (e.g., the Google People API) to fetch user information.

Section 4: Authorization: Enforcing Permissions

Authorization is the process of determining if an authenticated user has the necessary permissions to perform a specific action or access a particular resource. This is distinct from authentication and is a common source of critical security vulnerabilities.

4.1 Principles of Access Control

Effective authorization systems often combine several concepts to provide layered, flexible control:
Scopes (Coarse-Grained): Typically used in OAuth 2.0, scopes represent broad categories of permissions (e.g., read:profile, write:articles). They are excellent for delegating general permissions and are often checked at the edge of the system, such as in an API Gateway.20
Claims (Fine-Grained): Custom claims embedded within a JWT can carry specific attributes about a user, such as their user_id, tenant_id, or subscription_tier. The API business logic can then use these claims to make highly granular, context-aware access decisions. For example, an API might check that the user_id in the JWT matches the owner of the resource being requested.20
Role-Based Access Control (RBAC): A powerful and scalable model where permissions are not assigned directly to users but to roles (e.g., Admin, Editor, Viewer). Users are then assigned one or more roles. This greatly simplifies permission management, as changing a role's permissions automatically updates access for all users with that role.4

4.2 Implementing RBAC with casbin-rs

For applications with complex or dynamic authorization requirements, using a dedicated policy engine is highly recommended. casbin-rs is a powerful, open-source authorization library that supports various access control models, including ACL, RBAC, and ABAC (Attribute-Based Access Control).18
Core Concepts: Casbin's key strength is its decoupling of the authorization logic from the application code. This is achieved through three main components 39:
Model: A configuration file (e.g., model.conf) that defines the abstract access control model using the PERM (Policy, Effect, Request, Matchers) metamodel. For RBAC, this would define subjects, objects, actions, and the role hierarchy.
Policy: The set of authorization rules, typically stored in a file (e.g., policy.csv) or a database via an adapter. A rule might look like p, admin_role, /api/users/*, POST, granting the admin_role permission to make POST requests to any path under /api/users.
Enforcer: The Casbin API object within your application that loads the model and policy. Your code uses the enforcer's enforce() method to check if a given request is permitted.
Framework Integration: The casbin-rs ecosystem provides middleware for seamless integration with major Rust web frameworks:
Actix Web: The actix-casbin and actix-casbin-auth crates provide middleware that can be attached to the Actix App. The middleware extracts the subject, object, and action from the request and uses the Casbin enforcer to permit or deny access.40
Axum: The axum-casbin crate offers a CasbinAxumLayer that functions as tower middleware. It can be applied to the Axum Router to protect routes.42 A complete example repository is available at
casbin-rs/axum-middleware-example.43
Rocket: The rocket_casbin_auth crate provides a Rocket Fairing for initialization and a CasbinGuard for route protection. By adding the guard to a route's signature, authorization is automatically enforced before the handler is executed.44

4.3 Advanced Authorization Patterns

For even more complex scenarios, developers can build upon these foundations. One advanced pattern involves creating a permission system that supports not only roles but also direct permissions assigned to users and wildcard matching for hierarchical resources. For instance, a permission like /organizations/acme/departments/* could grant access to all resources within any department of the "acme" organization. This can be implemented efficiently using a custom Trie data structure for permission matching.37
The ecosystem also offers alternative, framework-specific authorization crates. For Axum, protect-axum provides an attribute-based macro (#[protect_axum::protect(...)]) for declaring permissions directly on handlers.46
axum-login is another powerful option that integrates authentication and authorization, supporting user and group permissions through an AuthzBackend trait.47
This variety presents a key architectural decision. The first path is to use a generic, externalized policy engine like casbin-rs. This approach offers maximum flexibility, allowing for complex and dynamically changing authorization rules without altering application code. It is particularly well-suited for polyglot microservice environments that require a single, consistent authorization model. The trade-off is the added complexity of managing the model, policy, and adapter.
The second path is to use a more tightly integrated, framework-idiomatic solution like axum-login or protect-axum. These crates are often simpler to set up for common use cases (e.g., checking for an "ADMIN" role) and feel more native to the framework's programming model. The authorization logic, however, becomes embedded within the application code, making it less portable and harder to change without a redeployment. The choice between these two approaches depends on the project's specific needs: maximum flexibility and interoperability versus simplicity and idiomatic development.

Section 5: Securing the Perimeter: Input, Traffic, and Dependencies

Beyond authentication and authorization, a secure API must rigorously control its inputs, manage incoming traffic to prevent abuse, and ensure the integrity of its software supply chain.

5.1 Input Validation and Sanitization

All external input is untrusted and must be validated before being processed. A failure to do so can lead to a wide range of vulnerabilities, from data corruption to injection attacks.
Philosophy: "Parse, Don't Validate": The most robust and idiomatic approach in Rust is to leverage the type system to make invalid states unrepresentable.48 Instead of passing a
String through the application and validating it as an email at multiple points, the goal should be to parse the input String into a strongly-typed EmailAddress at the application boundary. The constructor for EmailAddress would contain the validation logic, ensuring that any instance of this type is guaranteed to be valid. This shifts validation errors from runtime logic bugs to type-checking and parsing errors at the edge.
Recommended Crates:
nutype: This crate is an excellent implementation of the "Parse, Don't Validate" philosophy. It provides a procedural macro to create newtypes with built-in sanitization (e.g., trim, lowercase) and validation (e.g., not_empty, len_char_max) rules. This makes it impossible to construct an invalid value, even during serde deserialization.49
validator: A mature and popular crate for validating the fields of a struct, typically after deserialization from JSON or another format. It uses a convenient derive macro and field attributes (#[validate(email)], #[validate(range(min=0))]) to define validation rules. It integrates well with web frameworks to provide detailed error responses.50
sanitizer: This crate focuses specifically on data sanitization—the process of modifying input to ensure it is safe (e.g., removing non-numeric characters, trimming whitespace). It can be used in conjunction with a validation library.52
An example using validator with serde demonstrates a common pattern:

Rust


use serde::Deserialize;
use validator::{Validate, ValidationError};

#
pub struct SignupRequest {
    #[validate(email(message = "Must be a valid email address"))]
    pub email: String,
    #[validate(length(min = 8, message = "Password must be at least 8 characters long"))]
    pub password: String,
    #[validate(range(min = 18, message = "You must be at least 18 years old"))]
    pub age: u32,
}

// In a web handler:
// let signup_data: SignupRequest = json_payload.into_inner();
// match signup_data.validate() {
//     Ok(_) => { /* process valid data */ },
//     Err(e) => { /* return a 400 Bad Request with validation errors */ },
// }



5.2 Rate Limiting and DoS Protection

Rate limiting is a critical defense against automated abuse, brute-force attacks, and Denial-of-Service (DoS) attacks. It works by controlling the frequency of requests a client can make within a given time window.53
Strategy: A robust strategy involves multiple layers. For anonymous traffic, rate limiting is typically based on the client's IP address. For authenticated users, it should be based on the user ID or API key. This prevents a malicious user from easily bypassing IP-based limits by using a proxy or VPN.55 It is also important to distinguish between anti-abuse limiting, which needs to be extremely fast and can tolerate some imprecision, and resource-billing limiting, which must be precise but can handle higher latency.53
Algorithms: Common algorithms include the Fixed Window, the more accurate Sliding Window, and the Token Bucket or Leaky Bucket algorithms, which provide smoother traffic shaping. The Generic Cell Rate Algorithm (GCRA) is a sophisticated variant of the leaky bucket algorithm known for its efficiency and low memory overhead.54
Recommended Crate: For applications built on Axum or other tower-based frameworks, tower-governor is the idiomatic choice.55 It is a
tower middleware built on top of the governor crate, which implements GCRA.
Implementation: tower-governor is added as a Layer to the Axum Router. It can be configured with a quota (e.g., per_second) and a burst size. By default, it uses a PeerIpKeyExtractor to limit by IP address. For more advanced use cases, a custom KeyExtractor can be implemented to rate limit based on other request properties, such as an API key in an HTTP header.55

Rust


// In an Axum main function
use axum::{routing::get, Router};
use tower_governor::{governor::GovernorConfigBuilder, GovernorLayer};
use std::time::Duration;

//...

// Allow 5 requests per 10-second period.
let governor_conf = Box::new(
    GovernorConfigBuilder::default()
       .per_second(Duration::from_secs(2).as_nanos() as u64) // This is equivalent to 1 request every 2 seconds
       .burst_size(5)
       .finish()
       .unwrap(),
);

let app = Router::new()
   .route("/", get(handler))
   .layer(GovernorLayer {
        config: Box::leak(governor_conf),
    });



5.3 Secrets Management

Securely managing sensitive data like API keys, database credentials, and signing secrets is fundamental to application security.
Best Practices:
Never Hardcode Secrets: Secrets must never be committed to version control.59
Use Secure Storage: Use environment variables for local development. In production, use a dedicated secrets management system like AWS Secrets Manager, Google Cloud Secret Manager, or HashiCorp Vault.59
Least Privilege: Configure IAM roles or service accounts to ensure that an application has access only to the specific secrets it requires to function.60
Rotation: Implement a strategy for regularly rotating secrets to limit the window of exposure if a secret is compromised.60
Recommended Crate: secret-vault provides a robust, unified API for managing secrets from multiple sources.60 It supports environment variables, files (ideal for Kubernetes secrets), and direct integration with AWS and GCP secret managers. It also provides valuable features like in-memory caching, automatic refreshing, and optional in-memory encryption of secrets using AEAD cryptography or cloud KMS envelope encryption.
Security Note: It is important to recognize the limitations of such tools. Even with secret-vault, secrets must be decrypted into memory to be used, creating a window where they could potentially be exposed. These tools are a critical mitigation layer, not a complete solution against all possible attacks.60 The most secure configuration offered by the crate involves using GCP Secret Manager with KMS envelope encryption.60

5.4 Supply Chain Security

A modern application's code is composed largely of its third-party dependencies. A vulnerability in any crate in the dependency tree becomes a vulnerability in the application itself. This makes supply chain security a critical concern.13 Using unmaintained crates is a particularly high-risk practice, as security issues will not be patched.14
Essential Tooling: The Rust ecosystem provides excellent tooling for managing this risk. Integrating these tools into the CI/CD pipeline is not optional; it is a mandatory practice for secure development.
cargo-audit: This tool scans the project's Cargo.lock file and checks all dependencies against the RustSec Advisory Database, a curated repository of known vulnerabilities in Rust crates. It will fail the build if any vulnerable dependencies are found.9
cargo-deny: This is a more comprehensive linting tool. In addition to checking for security vulnerabilities (like cargo-audit), it can enforce policies on dependency licenses, detect duplicate versions of the same crate, and block dependencies from disallowed sources.13
Dependabot: A service (commonly integrated with GitHub) that automatically monitors dependencies for updates and security advisories. It can be configured to automatically create pull requests to update vulnerable or outdated dependencies, streamlining the patching process.9

Part III: The Ecosystem: Frameworks and Recommendations

This final part synthesizes the preceding analysis into concrete recommendations, focusing on the choice of web framework and providing actionable checklists to guide secure development.

Section 6: A Comparative Analysis of Web Framework Security

The choice of web framework is a significant architectural decision that influences an application's security posture through its design philosophy, default behaviors, and the maturity of its security-related ecosystem. While all major Rust frameworks benefit from the language's core safety guarantees, their approaches to building web services and integrating security features differ.1

6.1 Actix Web

Philosophy and Features: Actix Web is renowned for its exceptional performance, often topping third-party benchmarks. It follows a more "batteries-included" philosophy, providing a rich set of features out of the box, including routing, middleware, multipart streams, and WebSocket support, all built upon its own actor model.1
Security Ecosystem: As one of the most mature frameworks, Actix Web has a well-established ecosystem. It offers official or well-supported crates for critical security functions, such as actix-session for robust session management 26 and
actix-casbin-auth for integrating the Casbin authorization engine.40
unsafe History and Posture: Actix Web has a notable history regarding the unsafe keyword. Early versions used unsafe extensively to achieve maximum performance, which led to community concerns and a major, successful effort to refactor the codebase and remove all unsound uses of unsafe.8 Today, the framework still contains carefully audited
unsafe blocks in performance-critical areas, such as its internal buffer management and linked-list implementations. This history reflects a strong commitment to security and correctness, but also a design philosophy that is willing to leverage unsafe for performance gains after careful consideration.

6.2 Axum

Philosophy and Features: Developed and maintained by the Tokio team, Axum's core philosophy is modularity, ergonomics, and deep, seamless integration with the tower and tower-http ecosystem.66 It is not a monolithic framework but rather a set of composable components (routers, extractors, responses). Its primary strength is its ability to leverage the vast library of
tower middleware for concerns like tracing, compression, timeouts, and security.68
Security Ecosystem: Axum's security story is one of composition. Instead of providing its own solutions, it relies on the tower ecosystem. tower-governor is the natural and idiomatic choice for rate limiting.55 Session management is handled by powerful community crates like
axum-session or the more foundational tower-sessions.29 Authorization is similarly flexible, with options ranging from the integrated
axum-login 47 and
protect-axum 46 to the generic
axum-casbin middleware.42
unsafe Posture: The core axum crate is written with #![forbid(unsafe_code)], providing a strong guarantee that the framework's own logic is implemented in 100% safe Rust. All unsafe operations are pushed down into its foundational dependencies like hyper and tokio, which are themselves rigorously maintained and audited.68

6.3 Rocket

Philosophy and Features: Rocket's primary focus is on developer experience, ergonomics, and ease of use, making it a popular choice for developers new to Rust web development.1 Its most lauded feature is its type-safe routing system, which uses procedural macros to ensure at compile time that route handlers have the correct parameters, reducing the potential for runtime errors.1
Security Ecosystem: The ecosystem for Rocket is generally perceived as smaller and more focused than that of Actix Web or Axum.1 Community-driven crates provide key security functionalities, such as
rocket_session for cookie-based sessions 32 and
rocket_casbin_auth for authorization.44 A significant historical drawback was its reliance on the nightly Rust compiler, which was a barrier for many production environments; however, as of version 0.5, Rocket is fully compatible with the stable Rust toolchain.2
unsafe Posture: Rocket prioritizes safe abstractions and developer-friendly APIs. While the framework itself may have internal unsafe usage, its public-facing design strongly encourages writing safe, straightforward application code. (Note: Marketing materials for unrelated commercial products named "Rocket" 69 are not relevant to the security posture of the Rocket web framework and have been disregarded in this analysis).
The choice between these frameworks is not merely technical; it reflects an alignment with a specific development and security philosophy. A team prioritizing raw, benchmark-topping performance might lean towards Actix Web, accepting that its core contains highly optimized, audited unsafe code. A team that values flexibility, composability, and the ability to leverage a vast ecosystem of shared middleware would find Axum to be a natural fit, building their security layer from well-defined, independent components. Finally, a team that prioritizes development velocity and minimizing the cognitive load on developers might choose Rocket for its ergonomic and type-safe approach. This decision shapes how security features are integrated (monolithic vs. compositional), the nature of the trust boundary (trusting the framework vs. trusting the composed parts), and the overall development lifecycle.
Feature
Actix Web
Axum
Rocket
Core Philosophy
Performance & Feature-Rich
Modularity & Ecosystem
Ergonomics & Safety
unsafe Code Policy
Contains audited unsafe for performance
forbid(unsafe_code) (relies on dependencies like hyper)
Focus on safe abstractions
Session Management
actix-session (official)
axum-session / tower-sessions (community)
rocket_session (community)
Authorization (RBAC)
actix-casbin-auth (community)
axum-casbin, axum-login, protect-axum (community)
rocket_casbin_auth (community)
Rate Limiting
Manual / Community Crates
tower-governor (idiomatic ecosystem choice)
Manual / Community Crates
Ecosystem Maturity
Mature & Broad
Rapidly Growing & Composable
Smaller & Focused


Section 7: Concluding Recommendations and Security Checklists

Building secure web services in Rust requires combining the language's inherent safety with disciplined application-level security practices and a robust understanding of the ecosystem. The following table and checklists distill the key findings of this report into actionable guidance.

Category
Recommended Crate(s)
Key Features & Rationale
Password Hashing
argon2
Modern, configurable, and resistant to GPU-based attacks. The industry-standard choice.17
JWT
jsonwebtoken
Actively maintained, supports most standard algorithms, and provides clear APIs for encoding and validation.17
OAuth2 Client
oauth2
Strongly typed, asynchronous, and provides feature-complete implementations of standard grant flows.18
Authorization Engine
casbin-rs
Highly flexible, model-based engine supporting ACL, RBAC, and ABAC. Good integration with all major frameworks.38
Input Validation
validator & nutype
validator is mature and feature-rich for struct validation.50
nutype provides superior type safety via the "Parse, Don't Validate" pattern.49
Rate Limiting
tower-governor
The idiomatic choice for axum and tower-based applications. Implements the efficient GCRA algorithm.55
Secrets Management
secret-vault
Provides a unified API for cloud-native secret management (AWS, GCP) with caching and in-memory encryption.60
Dependency Auditing
cargo-audit & cargo-deny
cargo-audit is essential for checking against the RustSec DB.62
cargo-deny adds license and other policy checks.13


7.1 Consolidated Recommendations

Embrace the Type System: Make "Parse, Don't Validate" your primary strategy for handling untrusted input. Use strong, specific types (EmailAddress, UserID) instead of generic primitives (String, i64) to make invalid states unrepresentable in your application's domain logic.
Audit Your Dependencies Relentlessly: The security of your application is the security of its weakest dependency. Make cargo audit and cargo deny mandatory, blocking steps in your CI/CD pipeline. Use Dependabot to stay on top of security patches.9
Isolate and Justify unsafe: Treat every unsafe block as a critical security boundary. It must be minimized, encapsulated within a safe API, and accompanied by a rigorous // SAFETY: comment explaining its invariants. Use #![forbid(unsafe_code)] as a default project-wide policy.9
Layer Your Defenses: Security should not rely on a single mechanism. Combine architectural patterns (e.g., API Gateway with an opaque-to-JWT token exchange), tower or framework middleware (rate limiting, authentication), and fine-grained, application-level logic (authorization checks) for defense-in-depth.
Centralize Authentication: Do not mint tokens in every service. Use a dedicated, centralized OAuth 2.0 Authorization Server to handle user and client authentication and issue tokens. This promotes consistency and isolates complex security logic.20
Stay Informed: The Rust ecosystem evolves rapidly. Regularly monitor the RustSec Advisory Database for new vulnerabilities and keep your Rust toolchain and dependencies up-to-date to receive the latest security fixes.13

7.2 Comprehensive Security Checklist

This checklist synthesizes recommendations from the ANSSI Rust Security Guide and Mozilla's security guidelines for a practical review process.3
Development Environment & Tooling
[ ] Use a stable Rust toolchain for production builds.10
[ ] Regularly run linters (cargo clippy) and formatters (cargo fmt) to maintain code quality and catch common mistakes.9
[ ] cargo audit is integrated into the CI pipeline and blocks on vulnerabilities.9
[ ] cargo deny is used to enforce policies on licenses and duplicate dependencies.13
[ ] A Cargo.lock file is committed for all binary projects to ensure reproducible builds.9
[ ] Dependabot or a similar tool is configured to automate dependency updates.9
Language, Memory, and Code Quality
[ ] #![forbid(unsafe_code)] is used at the crate level.9
[ ] All uses of unsafe blocks are minimized, encapsulated in safe APIs, and include a detailed // SAFETY: justification comment.9
[ ] Code avoids functions that can panic!, especially in library code. Use Result and Option for error handling.10
[ ] .unwrap() and .expect() are not used on Result or Option types in production code paths; errors are handled or propagated gracefully.59
[ ] Integer arithmetic is checked for potential overflows (e.g., using checked_add).10
[ ] Sensitive data (e.g., keys, passwords) is explicitly zeroed out from memory after use where possible (e.g., using the zeroize crate).10
[ ] FFI boundaries are handled with extreme care, using only C-compatible types and ensuring clear data ownership.10
API and Web Service Security
[ ] Authentication:
[ ] Strong, modern password hashing (Argon2) is used for stored credentials.17
[ ] JWTs have a short expiration (exp claim) and use a strong signing algorithm (e.g., HS256 with a strong secret, or an RS/ES algorithm with proper key management).17
[ ] Session cookies are configured with Secure, HttpOnly, and SameSite=Strict or SameSite=Lax attributes.24
[ ] Authorization:
[ ] All sensitive endpoints are protected by an authorization check. There is no "fail-open" logic.
[ ] The principle of least privilege is applied to roles and permissions.
[ ] Input Validation:
[ ] All untrusted input from any source (request body, path parameters, query strings, headers) is rigorously validated and/or parsed into strong types at the application boundary.4
[ ] Configuration & Secrets:
[ ] No secrets are hardcoded in source code or configuration files.59
[ ] Secrets are loaded from environment variables or a dedicated secret management service.60
[ ] Logging & Monitoring:
[ ] Sufficient logging is in place to monitor for security events (e.g., failed logins, access denied errors).4
[ ] Logs are scrubbed of all sensitive data (passwords, tokens, PII) before being written.
[ ] Log levels are configurable and set appropriately for production to avoid excessive logging that could become a DoS vector.9
[ ] Transport Security:
[ ] TLS is enforced for all external communication.
[ ] Vetted libraries like rustls are preferred for TLS implementation.4
[ ] Denial of Service:
[ ] Rate limiting is implemented on all public-facing endpoints, especially authentication and computationally expensive ones.55
[ ] Request payload size limits are enforced to prevent resource exhaustion.51
Works cited
Rust Web Frameworks: A Comprehensive Comparison | by Rahul Sharma - Medium, accessed July 7, 2025, https://medium.com/@rs4528090/rust-web-frameworks-a-comprehensive-comparison-58f94113f864
The Best Rust Web Frameworks for Modern Development - Yalantis, accessed July 7, 2025, https://yalantis.com/blog/rust-web-frameworks/
Introduction - Secure Rust Guidelines - GitHub Pages, accessed July 7, 2025, https://anssi-fr.github.io/rust-guide/
Addressing the OWASP Top 10 in Rust apps | by Basillica | Medium, accessed July 7, 2025, https://basillica.medium.com/securing-your-rust-web-application-against-the-owasp-top-10-in-rust-95564c68aa6c
Is Rust any good for building RESTful apis? - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/sgfvn0/is_rust_any_good_for_building_restful_apis/
ANSSI-FR/rust-guide: Recommendations for secure applications development with Rust, accessed July 7, 2025, https://github.com/ANSSI-FR/rust-guide
Is there a checklist for review of unsafe code? - help - Rust Users Forum, accessed July 7, 2025, https://users.rust-lang.org/t/is-there-a-checklist-for-review-of-unsafe-code/28414
actix-web has removed all unsound use of unsafe in its codebase. It's down to less than 15 occurences of unsafe from 100+. : r/rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/8wlkbe/actixweb_has_removed_all_unsound_use_of_unsafe_in/
rust.md - mozilla-services/websec-check - GitHub, accessed July 7, 2025, https://github.com/mozilla-services/websec-check/blob/master/rust.md
Checklist - Secure Rust Guidelines, accessed July 7, 2025, https://anssi-fr.github.io/rust-guide/checklist.html
A11:2021 – Next Steps - OWASP Foundation, accessed July 7, 2025, https://owasp.org/Top10/A11_2021-Next_Steps/
Rust CVEs and Security Vulnerabilities - OpenCVE, accessed July 7, 2025, https://www.opencve.io/cve?vendor=rust-lang&product=rust
rustsec/advisory-db: Security advisory database for Rust crates published through crates.io - GitHub, accessed July 7, 2025, https://github.com/rustsec/advisory-db
Advisories › RustSec Advisory Database, accessed July 7, 2025, https://rustsec.org/advisories/
Critical Vulnerability in Rust on Windows - CERT-EU, accessed July 7, 2025, https://cert.europa.eu/publications/security-advisories/2024-035/
Password auth in Rust, from scratch - Attacks and best practices | Luca Palmieri, accessed July 7, 2025, https://lpalmieri.com/posts/password-authentication-in-rust/
Securing Web Applications with Rust: Building a Safe Authentication System 🛡️ - Medium, accessed July 7, 2025, https://medium.com/solo-devs/securing-web-applications-with-rust-building-a-safe-authentication-system-%EF%B8%8F-2063e327b2a7
Authentication — list of Rust libraries/crates // Lib.rs, accessed July 7, 2025, https://lib.rs/authentication
JWT authentication in Rust - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/jwt-authentication-in-rust/
API Security Best Practices | Curity, accessed July 7, 2025, https://curity.io/resources/learn/api-security-best-practices/
Authenticate Service Account in Rust with JWT(JSON Web Tokens) | by Itsuki - Medium, accessed July 7, 2025, https://medium.com/@itsuki.enjoy/authenticate-service-account-in-rust-with-jwt-json-web-tokens-cfa5056251be
Rust | Axum , JsonWebToken | Jwt |Protected Route | Part-2 | by Mike Code | Medium, accessed July 7, 2025, https://medium.com/@mikecode/rust-axum-jsonwebtoken-jwt-protected-route-part-2-d63ee4952787
Sessions? · Issue #196 · rwf2/Rocket - GitHub, accessed July 7, 2025, https://github.com/SergioBenitez/Rocket/issues/196
Web Apps with Rust: Implementing Secure Authentication and Authorization - Bits Kingdom, accessed July 7, 2025, https://bitskingdom.com/blog/web-apps-rust-authentication-authorization/
Authentication with Axum - mattrighetti, accessed July 7, 2025, https://mattrighetti.com/2025/05/03/authentication-with-axum
actix_session - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/actix-session
SessionMiddleware in actix_session - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/actix-session/latest/actix_session/struct.SessionMiddleware.html
actix-session - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/actix-session
axum_session - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/axum_session
AscendingCreations/AxumSession: Axum Session Management Libraries that use Sqlx - GitHub, accessed July 7, 2025, https://github.com/AscendingCreations/AxumSession
Axum | 33 , Session | Rust. We can use session to store and get… | by Mike Code - Medium, accessed July 7, 2025, https://medium.com/@mikecode/axum-33-session-rust-2e7870607ab2
rocket_session - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/rocket_session
oauth2 - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/async-oauth2
oauth2 - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/oauth2/latest/oauth2/
OAuth Libraries for Rust, accessed July 7, 2025, https://oauth.net/code/rust/
How to Implement OAuth in Rust - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2023/08/30/using-oauth-with-axum
Building a Scalable RBAC System in Rust with Permission Scopes | by Basillica - Medium, accessed July 7, 2025, https://basillica.medium.com/building-a-scalable-rbac-system-in-rust-with-permission-scopes-0355f72fb491
casbin/casbin-rs: An authorization library that supports ... - GitHub, accessed July 7, 2025, https://github.com/casbin/casbin-rs
Basic Role-Based HTTP Authorization in Rust with Casbin - zupzup, accessed July 7, 2025, https://www.zupzup.org/rust-casbin-example/
actix-casbin - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/actix-casbin
actix-casbin - Lib.rs, accessed July 7, 2025, https://lib.rs/crates/actix-casbin
CasbinAxumLayer in axum_casbin::middleware - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/axum-casbin/latest/axum_casbin/middleware/struct.CasbinAxumLayer.html
casbin-rs/axum-middleware-example - GitHub, accessed July 7, 2025, https://github.com/casbin-rs/axum-middleware-example
rocket_casbin_auth - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/rocket_casbin_auth
casbin-rs/rocket-authz: Casbin Rocket access control middleware - GitHub, accessed July 7, 2025, https://github.com/casbin-rs/rocket-authz
protect-axum - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/protect-axum
axum_login - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/axum-login
What crate(s) or method do you use for api Request validation ? : r/rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1g29q1t/what_crates_or_method_do_you_use_for_api_request/
nutype - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/nutype
validator - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/validator
Validating JSON input in Rust web services - Vinted Engineering, accessed July 7, 2025, https://vinted.engineering/2021/02/15/validating-json-input-in-rust-web-services/
sanitizer - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/sanitizer
Rate limiting | Fastly Documentation, accessed July 7, 2025, https://www.fastly.com/documentation/guides/concepts/rate-limiting/
How does rate limiting work to protect APIs from DOS attacks? - miniOrange, accessed July 7, 2025, https://www.miniorange.com/blog/rate-limiting-to-protect-apis-from-dos-attack/
Implementing API Rate Limiting in Rust - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2024/02/22/api-rate-limiting-rust
How to Prevent DDoS Attacks with Rate Limiting Techniques - Appknox, accessed July 7, 2025, https://www.appknox.com/blog/preventing-denial-of-service-attacks-with-rate-limiting-techniques
Looking for Better Rate Limiting / DDoS Protection Strategies on ICP - Rust - DFINITY Forum, accessed July 7, 2025, https://forum.dfinity.org/t/looking-for-better-rate-limiting-ddos-protection-strategies-on-icp/43888
tower_governor - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/tower_governor
iAnonymous3000/awesome-rust-security-guide ... - GitHub, accessed July 7, 2025, https://github.com/iAnonymous3000/awesome-rust-security-guide
secret-vault - crates.io: Rust Package Registry, accessed July 7, 2025, https://crates.io/crates/secret-vault
secret_vault - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/secret-vault
About RustSec › RustSec Advisory Database, accessed July 7, 2025, https://rustsec.org/
Exploring the top Rust web frameworks - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/top-rust-web-frameworks/
Best Rust Web Frameworks (2024) - Rustfinity, accessed July 7, 2025, https://www.rustfinity.com/blog/best-rust-web-frameworks
Actix Web, accessed July 7, 2025, https://actix.rs/
Best RUST web framework? : r/rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1ff38nb/best_rust_web_framework/
axum - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/axum/latest/axum/
tokio-rs/axum: Ergonomic and modular web framework built with Tokio, Tower, and Hyper, accessed July 7, 2025, https://github.com/tokio-rs/axum
Secure WordPress Hosting - Rocket.Net, accessed July 7, 2025, https://rocket.net/features/secure-wordpress-hosting/
Security Practices - Rocket Software, accessed July 7, 2025, https://www.rocketsoftware.com/en-us/legal/security-practices
We're committed to your security | Rocket Money, accessed July 7, 2025, https://www.rocketmoney.com/security
