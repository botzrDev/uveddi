
An Architectural Blueprint for Integrating Rust and TypeScript Full-Stack Applications


Executive Summary

This report provides a comprehensive architectural blueprint for integrating a high-performance Rust backend with a modern Vite/React/TypeScript frontend, tailored for Uveddi's requirements. The analysis focuses on establishing a secure, robust, versioned, and maintainable communication layer that maximizes developer productivity and long-term scalability.
The core recommendations are to build the backend using the axum web framework, leveraging its deep integration with the tower middleware ecosystem. API contracts will be defined using a code-first approach with utoipa to generate an OpenAPI specification, which then enables the automatic generation of a type-safe TypeScript client for the frontend. This creates an end-to-end type-safe workflow, drastically reducing integration errors.
Security is addressed through a multi-layered strategy. Cross-Origin Resource Sharing (CORS) will be managed with a strict, environment-aware policy. Authentication will be implemented using a hybrid JSON Web Token (JWT) pattern, storing short-lived access tokens in frontend memory and long-lived, revocable refresh tokens in secure, HttpOnly cookies. Error handling will be standardized using the RFC 7807 Problem Details format to ensure consistent and predictable client-side behavior.
For long-term maintainability, the report advocates for URL-based API versioning due to its simplicity and clarity in a tightly coupled full-stack context. Strategies for non-breaking schema evolution are outlined to allow for iterative development without disrupting existing clients. Finally, the entire project should be structured as a Cargo workspace monorepo, which provides unified tooling, simplified dependency management, and atomic commits, fostering a cohesive and efficient development environment.
By adopting these patterns and technologies, Uveddi can build a system that is not only performant and secure but also a pleasure to develop and maintain.

Part 1: Foundational Architectural Choices: The Rust Backend

The foundation of any robust full-stack application is its backend architecture. The choices made at this stage—specifically the web framework and the method for defining the API contract—have profound and lasting implications for performance, security, maintainability, and developer experience. This section analyzes these critical decisions for the Rust backend.

1.1 Selecting the Optimal Web Framework: A 2025 Perspective

The choice of web framework is the most critical initial decision for the backend, as it dictates the middleware ecosystem, performance characteristics, and developer ergonomics. In the contemporary Rust landscape, three primary contenders stand out for building REST APIs: axum, actix-web, and warp.1

1.1.1 Analysis of axum

Developed and maintained by the Tokio team, axum is a modern web framework designed for ergonomics and modularity.1 Its core philosophy is to integrate seamlessly with the foundational libraries of the Rust asynchronous ecosystem, namely
hyper (the HTTP implementation) and tower (the service abstraction layer).4
This deep integration with tower is axum's defining feature. It provides access to the extensive tower-http crate, which contains a rich collection of production-ready middleware for common tasks like CORS, logging, compression, and request tracing.1 Because
tower::Service is a generic trait, middleware written for it is often applicable beyond just web contexts (e.g., for gRPC services), leading to a robust and well-maintained ecosystem.6
From a developer experience perspective, axum is praised for its macro-free API. It relies on Rust's powerful type system, using "extractors" to parse request components (like headers, path parameters, or JSON bodies) and the IntoResponse trait to convert return types into HTTP responses.4 This approach is often described as simple, composable, and easy to reason about.5 Performance is excellent, with benchmarks consistently placing it among the top-tier Rust frameworks, often just behind
actix-web in raw throughput.1

1.1.2 Analysis of actix-web

actix-web is one of the most mature and battle-tested frameworks in the Rust ecosystem, historically renowned for its exceptional performance, frequently leading web framework benchmarks.1 It was originally built on the actor model provided by the
actix crate, although its modern architecture has evolved to more closely resemble standard async/await patterns, with the actor model being less prominent in typical use cases.7
The most significant point of divergence between actix-web and axum lies in their middleware philosophy. actix-web maintains its own distinct ecosystem and its own Service trait, which is incompatible with tower's.7 This creates a "walled garden" effect: the ecosystem is consistent and its components are designed to work well together, but it lacks access to the broader
tower ecosystem.6 While
actix-web is considered feature-rich and "batteries-included" with excellent documentation, this separation is a crucial strategic consideration.2

1.1.3 Analysis of warp

Like axum, warp is also built on hyper and tokio and is compatible with tower middleware.7 Its unique characteristic is a highly composable, functional "filter" system for defining endpoints. A route is constructed by chaining together filters that extract parts of the request, reject it, or pass it on.
While this approach is powerful and expressive, it is often criticized for producing extremely complex and verbose type signatures, which can make debugging compiler errors challenging.7 The learning curve is considered steep, and in recent years, the framework has seen less active development compared to its peers, with many developers and projects migrating from
warp to axum in search of better ergonomics and maintainability.8

1.1.4 Framework Recommendation

While all three frameworks are capable of building high-performance APIs, the primary differentiator for a new, long-term project is not raw performance—which is excellent for both axum and actix-web—but the strategic choice of ecosystem.
actix-web offers a mature, self-contained ecosystem. In contrast, axum is a component within the larger, more standardized tokio/tower ecosystem. This ecosystem is maintained by the same team responsible for Rust's core async runtime, ensuring tight integration and a shared direction. For a project like Uveddi's, which values modularity, long-term maintainability, and access to a broad set of community-driven tools, aligning with the tower stack via axum is the most strategic and future-proof choice. It avoids the "walled garden" of actix-web and the significant ergonomic challenges associated with warp. This decision simplifies the implementation of subsequent patterns, as tower-http provides robust solutions for CORS, logging, and other essential middleware.

Feature
axum
actix-web
warp
Performance
Excellent, top-tier performance.1
Exceptional, often leads benchmarks.1
Good, but often trails axum and actix-web.
Middleware Ecosystem
Integrates with the broad tower ecosystem (tower-http).6
Self-contained actix ecosystem; incompatible with tower.7
Integrates with the tower ecosystem.7
Developer Ergonomics
Macro-free, type-driven, simple and composable API.6
Mature, feature-rich, uses macros. Some legacy actor model concepts remain.7
Functional "filter" style; can lead to very complex type signatures and a steep learning curve.8
Learning Curve
Moderate. Familiarity with async/await is key.
Moderate to High. Documentation is excellent but the ecosystem is distinct.7
High. The filter system is a unique paradigm.8
Recommendation
Highly Recommended
Viable Alternative
Not Recommended


1.2 Establishing a Type-Safe Bridge: OpenAPI and utoipa

To achieve true end-to-end type safety between a Rust backend and a TypeScript frontend, the API must be described by a machine-readable contract. This contract serves as the single source of truth for data structures and endpoints. The industry standard for this is the OpenAPI Specification.11 The
utoipa crate is a leading solution in the Rust ecosystem for generating OpenAPI v3 specifications directly from Rust code using a "code-first" approach.13
This workflow leverages Rust's strong type system, turning it into a powerful tool for API documentation and contract generation. The process is as follows:
Annotate Data Structures: Business logic data models (e.g., User, Product, Order) are defined as standard Rust structs. By adding # from utoipa and # from serde, a single struct definition serves three purposes: it defines the shape of the data for the application's logic, it dictates how the data is serialized to and from JSON, and it generates the corresponding OpenAPI schema component.14
Annotate API Handlers: Each axum handler function is annotated with the #[utoipa::path] macro. This macro declaratively describes the HTTP method, path, expected request parameters, and possible responses, including their HTTP status codes and body schemas. utoipa provides an axum_extras feature that can automatically infer information like path parameters directly from the handler's signature, reducing boilerplate.13
Aggregate into a Single Document: A top-level struct, conventionally named ApiDoc, is created. This struct derives #[derive(OpenApi)] and uses the #[openapi(...)] macro to list all the annotated handlers and schemas that should be included in the final specification. It also defines top-level API metadata like the title, version, and description.14
Serve Interactive Documentation: The utoipa-swagger-ui crate can be used to serve an interactive Swagger UI or Redoc page from the generated ApiDoc. This provides developers with live, browsable API documentation that is guaranteed to be in sync with the implementation, as it is generated from the code itself.16
This "code-first" methodology offers a profound architectural advantage. It establishes the Rust source code as the undisputed single source of truth. A change to a field in a Rust struct automatically propagates through the entire system: the database mapping (if using sqlx), the JSON serialization format, and the public API contract exposed via the generated openapi.json file. This dramatically reduces the risk of documentation drift, a common problem where the implementation and its documentation diverge over time.18 Furthermore, this generated contract is the essential prerequisite for creating a type-safe TypeScript client, completing the bridge between the backend and frontend. For larger projects, the
utoipauto crate can be used to automatically discover annotated handlers, further reducing the manual effort required to maintain the ApiDoc struct.15

Part 2: Core API Integration Patterns

With the backend framework and contract generation mechanism in place, the next step is to define the fundamental protocols and conventions that govern communication. These patterns ensure that interactions between the Vite/React frontend and the Rust backend are secure, predictable, and robust.

2.1 Cross-Origin Resource Sharing (CORS) Configuration

Cross-Origin Resource Sharing (CORS) is a browser security feature that restricts web pages from making requests to a different domain than the one that served the page. For the Uveddi system, this is immediately relevant in development, where the Vite dev server (e.g., http://localhost:5173) and the Rust backend (http://localhost:3000) are different origins. A correctly configured CORS policy on the backend is mandatory to allow the frontend to communicate with the API.19 Even in production, where the frontend and backend might be served from the same domain, a robust CORS policy acts as a critical defense-in-depth security layer.
The recommended approach for axum is to use the CorsLayer middleware from the tower-http crate.21 The following best practices are essential for a secure configuration:
Restrict Origins: The most critical rule of CORS is to never use a wildcard (*) for Access-Control-Allow-Origin in a production environment. This would allow any website on the internet to make requests to your API on behalf of your users, which is a major security vulnerability. The backend should maintain an explicit allow-list of trusted origins (e.g., https://app.uveddi.com). This list can be loaded from environment variables to differ between development, staging, and production environments.19
Handle Preflight Requests: When the frontend makes a "non-simple" request—such as a PUT, DELETE, or any request including an Authorization header—the browser first sends a preflight OPTIONS request to the server. This preflight checks for permission to send the actual request. The server must respond to this OPTIONS request with the appropriate Access-Control-Allow-Methods and Access-Control-Allow-Headers headers. The CorsLayer middleware handles this preflight negotiation automatically, provided it is configured correctly.19
Manage Credentials: For authentication, the frontend will need to send credentials (either cookies or Authorization headers). To permit this, a two-part configuration is required:
Backend: The CorsLayer must be configured with .allow_credentials(true).
Frontend: The fetch or axios request must be made with the credentials: 'include' option.
It is important to note that when Access-Control-Allow-Credentials is true, the Access-Control-Allow-Origin header cannot be a wildcard (*); it must be a specific origin.19
Specify Allowed Headers and Methods: Be explicit about which HTTP methods (e.g., GET, POST, PUT, DELETE, OPTIONS) and headers (e.g., Content-Type, Authorization, X-Api-Key) are allowed. Avoid wildcards for headers, as they can be insecure and are not always interpreted consistently by browsers. The Authorization header, in particular, must be explicitly listed.19
The deliberate, builder-pattern style of tower-http encourages a secure-by-default mindset. Unlike some frameworks where a permissive default is easy to set, CorsLayer requires developers to explicitly define their policy, reducing the likelihood of common misconfigurations that can lead to security vulnerabilities.

2.2 API Versioning Strategies

As an application evolves, its API will inevitably change. A clear versioning strategy is essential to introduce these changes—especially breaking ones—without disrupting existing clients.23 The two most prevalent strategies for REST APIs are URL-based versioning and header-based versioning.23
URL-based Versioning: This approach includes the version number directly in the URL path, for example, /api/v1/users. It is the most popular and widely understood method.23
Pros: It is simple, explicit, and easy for developers to see which version of an endpoint they are calling. It is also straightforward to test in a browser or with simple tools like curl. Because each versioned endpoint has a unique URL, they are easily and effectively cached by standard HTTP caches and CDNs.24
Cons: A common critique is that it is not purely "RESTful," as it "pollutes" the resource URI with metadata that is not part of the resource's identity. It can also lead to significant code branching within the application's router.24
Header-based Versioning: This approach places the version information in a custom request header (e.g., Api-Version: 1.0) or within the Accept header using a custom media type (e.g., Accept: application/vnd.company.v1+json).24
Pros: It keeps resource URLs clean and stable over time, which is considered more compliant with REST principles. It can allow for more granular versioning control.24
Cons: It is more complex for clients to implement, as they must remember to add the correct header to every request. It is also more difficult to test and debug directly in a browser. Caching is more complicated, as it requires proxies and caches to inspect the Vary response header to cache different versions of the same URL correctly.24
For Uveddi's full-stack application, where the Vite/React frontend is the primary and tightly coupled consumer of the Rust backend, URL-based versioning is the recommended strategy. The choice of versioning strategy depends heavily on the API's audience.28 While header-based versioning offers benefits for public APIs with diverse, third-party clients, the pragmatic advantages of URL versioning—its simplicity, explicit-ness, and ease of caching and debugging—outweigh the concerns of RESTful purity in this context. It is simpler to implement in
axum's router and more intuitive for frontend developers to consume and manage.

Aspect
URL Path Versioning (/api/v1)
Header Versioning (Accept-Version: 1)
REST Compliance
Lower (pollutes URI) 25
Higher (clean URI) 25
Ease of Use & Testing
High (can use browser directly) 24
Lower (requires custom headers/tools) 24
Cacheability
High (distinct URLs are cached separately) 25
Lower (requires Vary header and complex proxy configuration) 24
Tooling Support
Excellent (universally supported)
Good (supported by most modern tools)
Best For
Tightly-coupled clients (e.g., SPAs), public APIs where simplicity is key.24
Public APIs with diverse third-party consumers where URL stability is paramount.24


2.3 Robust Error Handling Conventions

A well-designed API must handle errors in a consistent, predictable, and informative manner. A standardized error format is crucial for building robust client-side logic and providing a good developer experience.29
The following best practices should be adopted:
Use Correct HTTP Status Codes: The foundation of RESTful error handling is the correct use of standard HTTP status codes. They provide an immediate, language-agnostic signal about the outcome of a request (e.g., 400 Bad Request for invalid input, 401 Unauthorized for missing authentication, 403 Forbidden for insufficient permissions, 404 Not Found for a missing resource, and 500 Internal Server Error for unexpected server-side failures).29
Standardize the Response Body with RFC 7807: For the error response body, the modern best practice is to adopt the IETF standard RFC 7807: Problem Details for HTTP APIs. This standard defines a JSON object with a specific structure and the application/problem+json media type, ensuring machine-readable and consistent error responses across the entire API.31
The core fields of a Problem Details object are 31:
type (string): A URI that uniquely identifies the problem type. This can and should link to human-readable documentation about that specific error.
title (string): A short, human-readable summary of the problem type. This should not change between occurrences of the same error.
status (number): The HTTP status code generated by the server for this occurrence.
detail (string): A human-readable, specific explanation of this particular error occurrence.
instance (string): A URI that identifies the specific occurrence of the problem.
A powerful feature of RFC 7807 is its allowance for custom extension members. A highly recommended extension is a traceId field. This unique identifier can be generated for each request and included in both the error response and the corresponding backend logs. When a user reports an error, they can provide the traceId, allowing developers to instantly locate the exact logs associated with that failed request, dramatically speeding up debugging.29
Adopting a standard like RFC 7807 transforms error handling from an ad-hoc process into a systematic one. On the frontend, a single, generic error handler (e.g., an axios or fetch interceptor) can be written to process all API errors. This handler can parse the application/problem+json response, log the traceId to a monitoring service, display a user-friendly message based on the title or detail, and potentially use the type URI to provide a "Learn More" link. This approach greatly simplifies client-side error logic and enhances maintainability. The recommended pattern in Rust is to create a custom ApiError enum that implements axum::response::IntoResponse. Each variant of the enum can map to a specific error condition and be responsible for generating the correct RFC 7807 response, centralizing all error formatting logic in one place.

Part 3: Secure Authentication and Session Management

A secure authentication system is the bedrock of a trusted application. This section details a modern, robust authentication architecture for the Uveddi platform, covering the full lifecycle of tokens from generation and validation to secure storage and revocation.

3.1 The Modern Authentication Flow: Access and Refresh Tokens

For modern, stateless APIs, JSON Web Tokens (JWTs) are the de facto standard for authentication.32 A robust and secure implementation goes beyond a single token, employing a dual-token strategy with short-lived access tokens and long-lived refresh tokens to balance security with user experience.34
Access Token: This is a JWT with a short lifespan (e.g., 5-15 minutes). It contains claims about the user, such as their user ID (sub), the token's expiration time (exp), and any relevant authorization data like roles or permissions.32 The access token is sent with every request to protected API endpoints, typically in the
Authorization header using the Bearer scheme (e.g., Authorization: Bearer <token>).32 The server validates the token's signature and expiration on each request. Because all necessary information is in the token, the server remains stateless for these operations.
Refresh Token: This is a token with a long lifespan (e.g., 7-30 days). Its sole purpose is to obtain a new access token after the current one has expired.34 It is sent to a single, dedicated endpoint (e.g.,
/api/v1/auth/refresh). The refresh token itself should be an opaque, high-entropy string, not a JWT containing sensitive data. It must be securely stored by the client and must be revocable on the server side.
The authentication flow proceeds as follows:
The user submits credentials (e.g., email/password) to a /login endpoint.
The server validates the credentials.
Upon success, the server generates both a new access token and a new refresh token.
The server sends the access token to the client in the JSON response body and the refresh token in a secure cookie.
The client uses the access token to make requests to protected API resources.
When the access token expires, the client's API layer detects the failure (typically a 401 Unauthorized response).
The client then automatically and silently calls the /auth/refresh endpoint, which sends the refresh token (via its cookie).
The server validates the refresh token, and if valid, issues a new access token (and potentially a new refresh token).
The client retries the original failed request with the new access token.
This entire refresh process is transparent to the end-user, who experiences a continuous session. In Rust, the jsonwebtoken crate is the standard for creating and validating JWT access tokens. A Claims struct can be defined with serde macros to represent the token's payload, and the encode and decode functions handle the cryptographic operations.32

3.2 Secure Token Storage on the Frontend: The Hybrid Pattern

The method used to store tokens on the frontend is one of the most critical security decisions for a Single-Page Application (SPA). The choice involves a direct trade-off between mitigating Cross-Site Scripting (XSS) and Cross-Site Request Forgery (CSRF) attacks.41
localStorage: Storing tokens in localStorage or sessionStorage is highly insecure and strongly discouraged. These storage mechanisms are accessible via JavaScript, meaning any XSS vulnerability on the page could allow an attacker to steal the token and impersonate the user.42
HttpOnly Cookies: Storing tokens in cookies with the HttpOnly flag prevents JavaScript from accessing them, providing excellent protection against XSS-based token theft.42 However, because browsers automatically attach cookies to every request to the issuing domain, this method makes the application vulnerable to CSRF attacks. An attacker could trick a logged-in user into visiting a malicious site, which could then make forged requests to the application's API, and the browser would dutifully include the authentication cookie.34
To resolve this dilemma, the industry best practice is the Hybrid Token Pattern, which combines the strengths of different storage mechanisms to mitigate both threats 35:
Access Token Storage: The short-lived access token is stored in-memory only. This means it is held in a JavaScript variable within the application's state (e.g., using React Context, Redux, or Zustand). It is never written to persistent storage like localStorage or cookies. When the user closes the tab or refreshes the page, the access token is lost, but this is acceptable because it can be quickly re-acquired using the refresh token.
Refresh Token Storage: The long-lived, high-value refresh token is stored in a secure cookie set by the server. This cookie must be configured with the following flags:
HttpOnly: Prevents JavaScript access, mitigating XSS.
Secure: Ensures the cookie is only sent over HTTPS connections, preventing interception.
SameSite=Strict: Provides the strongest protection against CSRF by ensuring the cookie is only sent for same-site requests.
Path=/api/v1/auth: Scopes the cookie so it is only sent to the authentication-related endpoints, not every API call.
This hybrid pattern elegantly decouples API authentication from browser session management. The API itself remains stateless, designed to accept bearer tokens in the Authorization header. This is a clean, standard design. The refresh token, protected in its HttpOnly cookie, handles the stateful "session" persistence. Because the access token must be explicitly attached to API requests by the frontend's JavaScript code, it is not vulnerable to CSRF. This approach provides robust, defense-in-depth protection against the most common web authentication attacks.

Storage Method
Vulnerability to XSS
Vulnerability to CSRF
Recommendation
localStorage
High (JS can read the token) 42
Low (Token must be manually attached) 42
Not Recommended
HttpOnly Cookie (for all tokens)
Low (JS cannot read the token) 43
High (Cookie is sent automatically) 34
Not Recommended
Hybrid Pattern (Access in Memory, Refresh in HttpOnly Cookie)
Low (Access token is short-lived; refresh token is protected) 42
Low (API expects Authorization header, not auth cookie) 42
Highly Recommended


3.3 Implementing Refresh Token Rotation and Revocation

For a truly secure system, refresh tokens cannot be permanent credentials. They must have a managed lifecycle that includes rotation and the ability to be revoked by the server.44
Refresh Token Rotation: This is a security enhancement where, upon successful use of a refresh token, the server invalidates that token and issues a new refresh token to the client along with the new access token.35 This practice helps detect token theft. If an attacker steals and uses a refresh token, the legitimate user's client will later attempt to use the same, now-invalidated token. This failed attempt can be logged as a high-priority security event, indicating a potential account compromise.35
Server-Side Revocation: A purely stateless JWT system has a significant drawback: a valid token cannot be revoked before it expires. This is a problem if a user's account is compromised, they change their password, or they explicitly log out. To address this, the server must maintain a small amount of state to track the validity of refresh tokens.
Implementation: The server should not store the raw refresh token. Instead, it should store a unique identifier for the token (e.g., a UUID, a family ID, or a secure hash of the token) in a persistent data store. This store can be a dedicated table in the primary PostgreSQL database or a high-performance key-value store like Redis.35
Validation: When the /auth/refresh endpoint receives a refresh token, it verifies the token's identifier against the allow-list in the database.
Revocation: When a user logs out or a security event occurs (like a password change), the server simply deletes the corresponding token identifier(s) from the database. Any subsequent attempt to use that refresh token will fail the validation check.
This mechanism provides a critical control point, bridging the gap between the efficiency of stateless JWTs and the security necessity of stateful session control. It allows for the immediate termination of a user's session when required, without compromising the stateless design of the majority of the API endpoints, which continue to operate only with short-lived, verifiable access tokens. The Uveddi system must include this server-side store for refresh token identifiers to implement a secure logout and session invalidation capability.

Part 4: Ensuring Long-Term Maintainability and Developer Experience

Beyond initial implementation, the success of a software project is determined by its long-term maintainability and the productivity of its developers. This section details the patterns and tools that create a seamless, low-friction workflow for the Rust and TypeScript stack, ensuring the application can scale gracefully.

4.1 Generating a Type-Safe TypeScript Client

The final and most impactful step in creating an end-to-end type-safe architecture is the automatic generation of a TypeScript client from the backend's OpenAPI specification. This practice eliminates a vast category of common integration bugs and dramatically accelerates frontend development.46
The workflow is straightforward and powerful:
Input: The process begins with the openapi.json file generated by utoipa on the Rust backend. This file serves as the definitive contract for the API.
Tooling: A code generation tool is used to parse the OpenAPI spec and output TypeScript code. While the venerable openapi-generator-cli is a powerful option with a typescript-axios generator, more modern, TypeScript-focused tools like @hey-api/openapi-ts are often preferred for their cleaner output and simpler configuration.46
Automation: This generation step is automated by adding a script to the frontend's package.json file. For example: "generate-client": "openapi-ts --input http://localhost:3000/api-docs/openapi.json --output./src/client". This script can be run manually during development or, ideally, integrated into the CI/CD pipeline to run automatically whenever the backend API changes.
Output: The generator creates a complete TypeScript client library. This includes TypeScript interface or type definitions for all API data models (e.g., Person, UserResponse), as well as a client class containing fully-typed methods for every API endpoint. For example, a GET /users/{id} endpoint would become a method like client.users.getUserById({ id: 1 }). The parameters, request body, and return values are all strongly typed, providing full static analysis and IDE intellisense.22
This automated workflow fundamentally transforms the relationship between the frontend and backend. It replaces error-prone manual communication and reliance on human-readable documentation with a machine-enforced contract. When a backend developer adds a field, renames an endpoint, or changes a data type, they simply regenerate the spec. The frontend developer then re-runs the client generator, and the TypeScript compiler will immediately highlight every part of the frontend code that needs to be updated. This tightens the development feedback loop from potentially hours or days down to minutes, making API refactoring a safe, predictable, and low-stress activity. It is arguably the single greatest productivity advantage of the Rust/TypeScript/OpenAPI stack.

4.2 Strategies for API Schema Evolution

As the Uveddi application evolves, its API schema—the structure of its data models—will need to change. Managing this evolution gracefully is key to maintaining a stable service for clients.11 The approach depends on whether a change is "backward-compatible."
Backward-Compatible (Non-Breaking) Changes: These are changes that can be introduced to the API without breaking existing clients. They do not require an increment of the major API version (e.g., from v1 to v2).
Adding a new, optional field to a response object. In Rust, this is modeled with Option<T>. For example, adding pub new_field: Option<String> to a response struct is a non-breaking change. The generated TypeScript type will be new_field?: string. Older clients that are unaware of this field will simply ignore it during deserialization.11
Adding a new API endpoint. This is purely additive and has no impact on clients that do not use it.
Adding a new, optional query parameter to an existing endpoint.
Backward-Incompatible (Breaking) Changes: These are changes that will break existing clients that have not been updated. These changes must be introduced under a new major API version (e.g., moving from /api/v1/users to /api/v2/users).
Removing a field from a response object.
Renaming an existing field.
Changing the data type of an existing field (e.g., from a number to a string).
Making a previously optional field required.
Adding a new required field to a request body.
A robust deprecation strategy should also be in place. Before a field or endpoint is removed in a future major version, it should be marked as deprecated in the current version. The #[deprecated] attribute can be used in Rust code, and the OpenAPI specification supports a deprecated: true flag on fields and operations. This provides a clear signal to clients that they need to migrate their code, giving them ample time to adapt before the feature is removed.54
The discipline required for managing schema evolution is similar to the discipline Rust itself applies to its own API evolution.57 By establishing a clear policy for what constitutes a breaking change and adhering to it, the development team can evolve the API with confidence. To enforce this, automated
contract testing can be integrated into the CI pipeline. This involves comparing the newly generated openapi.json against the version from the main branch to automatically detect and flag any unintended breaking changes.28

4.3 Full-Stack Integration Case Studies & Monorepo Architecture

Examining successful open-source projects and tutorials reveals a consistent and effective pattern for structuring full-stack Rust/TypeScript applications: the Cargo Workspace Monorepo.22
A monorepo is a single version control repository that contains the code for multiple projects. Rust's built-in support for "workspaces" makes this an especially natural and powerful fit for full-stack development.60
A typical monorepo layout for a Rust/TypeScript project looks like this:



/uveddi-project
├── Cargo.toml          # Defines the Cargo workspace
├── apps/
│   ├── backend/        # The axum server crate
│   │   ├── Cargo.toml
│   │   └── src/
│   └── frontend/       # The Vite/React/TypeScript app
│       ├── package.json
│       └── src/
├── libs/
│   ├── shared-types/   # Optional: Shared Rust types used by the backend
│   │   ├── Cargo.toml
│   │   └── src/
│   └── type-generator/ # Optional: A custom tool crate
├──.gitignore
└── README.md


This architecture offers several key benefits for developer experience and maintainability:
Unified Tooling and Version Control: All code lives in a single git repository, simplifying source control. High-level scripts, often in a root Makefile or package.json, can orchestrate common tasks like building, testing, and linting for both the frontend and backend simultaneously.60
Simplified Dependency Management: The root Cargo.toml can define workspace-level dependencies, ensuring that all Rust crates in the repository use consistent versions of key libraries like tokio, serde, and axum.60
Atomic Commits: Changes that span both the frontend and backend (e.g., adding a new API endpoint and the corresponding frontend component that calls it) can be made in a single, atomic commit. This makes the project history much easier to understand and navigate.
Code Sharing: It becomes trivial for crates within the workspace to depend on each other. For example, the backend crate can directly depend on the shared-types crate using a path dependency in its Cargo.toml.
Adopting a Cargo workspace monorepo from the project's inception is the recommended approach for Uveddi. It provides a level of tooling integration and cohesion that is difficult to achieve with separate repositories, establishing a solid foundation for scalable and maintainable development.

Part 5: Actionable Recommendations for Uveddi

This final section synthesizes the preceding analysis into a concise set of direct, actionable recommendations, providing a clear and opinionated path forward for the Uveddi application architecture.

5.1 Concise Summary of Findings

The analysis concludes that a modern, full-stack application leveraging Rust and TypeScript can achieve exceptional levels of performance, security, and developer productivity by adopting a specific set of integrated technologies and patterns. The recommended architecture prioritizes end-to-end type safety, a robust and modern authentication scheme, and clear conventions for API evolution and error handling. The key is to leverage the strengths of each ecosystem—Rust's performance and type system, TypeScript's frontend safety, and OpenAPI's role as a machine-readable contract—to create a tightly-knit, maintainable, and scalable system.

5.2 Recommended Technology Stack and Patterns

Based on the detailed analysis, the following stack and patterns are recommended for the Uveddi project:
Backend Framework: axum. Its seamless integration with the tokio and tower ecosystems provides access to a wealth of robust, standardized middleware and makes it the most strategic choice for a new, long-term project.1
API Specification: utoipa with utoipa-swagger-ui. Use its code-first approach to generate an OpenAPI 3.x specification directly from your annotated Rust structs and handlers. This is the cornerstone of the type-safe architecture.13
CORS Policy: Implement using tower_http::cors::CorsLayer. Configure it with a strict, environment-specific allow-list for origins, and explicitly define allowed methods and headers, including Authorization.19
API Versioning: Use URL-based versioning (e.g., /api/v1/). This approach is pragmatic, simple to implement and consume, and well-suited for a full-stack application where the frontend is the primary client.23
Error Handling: Standardize on RFC 7807 Problem Details. Implement a custom ApiError enum in Rust that implements IntoResponse and serializes to the application/problem+json format. Critically, include a traceId in all error responses to correlate with backend logs.29
Authentication Flow: Implement the hybrid token pattern.
Access Tokens: Short-lived JWTs, stored in-memory on the frontend (e.g., React Context or a state manager).
Refresh Tokens: Long-lived, opaque tokens stored in a secure, HttpOnly, SameSite=Strict cookie set by the server.
This pattern provides the best-in-class defense against both XSS and CSRF attacks.35
Token Management: Implement refresh token rotation for enhanced security. Maintain a server-side allow-list of valid refresh token identifiers (in PostgreSQL or Redis) to enable immediate token revocation for logout and security events.35
TypeScript Client Generation: Use @hey-api/openapi-ts to generate a fully-typed TypeScript client from the openapi.json specification provided by utoipa. Automate this generation step in your package.json scripts and CI pipeline.50
Project Structure: Structure the project as a Cargo workspace monorepo. This will house both the backend (axum) and frontend (Vite/React) applications, providing unified tooling, dependency management, and atomic commits across the stack.60

5.3 Curated List of Resources and Guides

For further implementation details and deeper understanding, the following resources are highly recommended:
Rust Web Frameworks & Middleware:
axum Official Repository & Examples: https://github.com/tokio-rs/axum
tower-http Crate Documentation (for CORS, etc.): https://docs.rs/tower-http/latest/tower_http/
Shuttle's Rust Web Framework Comparison (2023): https://www.shuttle.dev/blog/2023/08/23/rust-web-framework-comparison 7
API Specification & Documentation:
utoipa Crate Documentation & Examples: https://docs.rs/utoipa/latest/utoipa/ 14
utoipa GitHub Repository (with framework examples): https://github.com/juhaku/utoipa 13
OpenAPI Specification v3.1.0: https://spec.openapis.org/oas/v3.1.0
Authentication & Security:
jsonwebtoken Crate Documentation: https://docs.rs/jsonwebtoken/latest/jsonwebtoken/
Shuttle's JWT Auth with Axum Tutorial: https://www.shuttle.dev/blog/2024/02/21/using-jwt-auth-rust 32
Understanding Token Storage (Local Storage vs. HttpOnly Cookies): https://www.wisp.blog/blog/understanding-token-storage-local-storage-vs-httponly-cookies 42
RFC 7807: Problem Details for HTTP APIs: https://datatracker.ietf.org/doc/html/rfc7807
TypeScript Client Generation & Full-Stack Integration:
@hey-api/openapi-ts GitHub Repository: https://github.com/hey-api/openapi-ts 50
Full-Stack Rust/React Example with OpenAPI Generation (Poem): https://medium.com/@etiennedx00/creating-a-full-stack-app-with-rust-and-react-61783d6afd80 49
Rust Monorepo Guide with Cargo Workspaces: https://earthly.dev/blog/rust-monorepo/ 61
Works cited
Best Rust Web Frameworks (2024) - Rustfinity, accessed July 7, 2025, https://www.rustfinity.com/blog/best-rust-web-frameworks
Top Rust frameworks for 2024 — Part 1 | by Davide Ferrero | Level Up Coding, accessed July 7, 2025, https://levelup.gitconnected.com/top-rust-frameworks-for-2024-part-1-77f96131d91e
Top 5 Rust Frameworks (2025) - Mastering Backend, accessed July 7, 2025, https://masteringbackend.com/posts/top-5-rust-frameworks
The Ultimate Guide to Axum: From Hello World to Production in Rust (2025) - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2023/12/06/using-axum-rust
Which Web Framework do people recommend for Rust in 2023? - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/12jhxi2/which_web_framework_do_people_recommend_for_rust/
Axum or Actix in 2024 : r/rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1bj9rc3/axum_or_actix_in_2024/
Best Rust Web Frameworks to Use in 2023 - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2023/08/23/rust-web-framework-comparison
WARP or AXUM for microservices in production? Tell me your experience. : r/rust - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/ux6vth/warp_or_axum_for_microservices_in_production_tell/
Exploring the top Rust web frameworks - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/top-rust-web-frameworks/
Rust Web Frameworks: A Comprehensive Comparison | by Rahul Sharma - Medium, accessed July 7, 2025, https://medium.com/@rs4528090/rust-web-frameworks-a-comprehensive-comparison-58f94113f864
API design | rust-api.dev, accessed July 7, 2025, https://rust-api.dev/docs/part-1/api-design/
Best Practices for Versioning REST and GraphQL APIs | Moesif Blog, accessed July 7, 2025, https://www.moesif.com/blog/technical/api-design/Best-Practices-for-Versioning-REST-and-GraphQL-APIs/
juhaku/utoipa: Simple, Fast, Code first and Compile time generated OpenAPI documentation for Rust - GitHub, accessed July 7, 2025, https://github.com/juhaku/utoipa
utoipa - Rust - Docs.rs, accessed July 7, 2025, https://docs.rs/utoipa/latest/utoipa/
ProbablyClem/utoipauto: Rust Macros to automate the addition of Paths/Schemas to Utoipa crate, simulating Reflection during the compilation phase - GitHub, accessed July 7, 2025, https://github.com/ProbablyClem/utoipauto
Axum (Rust), OpenAPI + SwaggerUI integration - YouTube, accessed July 7, 2025, https://www.youtube.com/watch?v=wlNQvJ6i2nA
Axum (Rust), OpenAPI + SwaggerUI integration - YouTube, accessed July 7, 2025, https://m.youtube.com/watch?v=wlNQvJ6i2nA
Generating server (axum) based on the OpenAPI specification | by Grzegorz Bylica | Medium, accessed July 7, 2025, https://grzesiekb.medium.com/generating-server-axum-based-on-the-openapi-specification-704b59d9ca39
7 Tips for Managing CORS in Your Backend Applications | by ..., accessed July 7, 2025, https://medium.com/@arunangshudas/7-tips-for-managing-cors-in-your-backend-applications-a4341385110c
Implementing CORS in Rust: Examples and Best Practices - StackHawk, accessed July 7, 2025, https://www.stackhawk.com/blog/rust-cors-guide-what-it-is-and-how-to-enable-it/
Rust cors warning - help - The Rust Programming Language Forum, accessed July 7, 2025, https://users.rust-lang.org/t/rust-cors-warning/123917
alexeagleson/rust-fullstack-app-template: A simple ... - GitHub, accessed July 7, 2025, https://github.com/alexeagleson/rust-fullstack-app-template
What is API versioning? Benefits, types & best practices | Postmann, accessed July 7, 2025, https://www.postman.com/api-platform/api-versioning/
API versioning: URL vs Header vs Media Type versioning - Lonti, accessed July 7, 2025, https://www.lonti.com/blog/api-versioning-url-vs-header-vs-media-type-versioning
API Versioning Strategies for B2B SaaS - Userlens by Wudpecker, accessed July 7, 2025, https://www.wudpecker.io/blog/api-versioning-strategies-for-b2b-saas
API Versioning: Strategies & Best Practices - xMatters, accessed July 7, 2025, https://www.xmatters.com/blog/api-versioning-strategies
API Versioning Strategies: Best Practices Guide - Daily.dev, accessed July 7, 2025, https://daily.dev/blog/api-versioning-strategies-best-practices-guide
API Versioning Best Practices: How to Manage Changes Effectively, accessed July 7, 2025, https://www.getambassador.io/blog/api-versioning-best-practices
Best Practices for API Error Handling | Postman Blog, accessed July 7, 2025, https://blog.postman.com/best-practices-for-api-error-handling/
Exception Handling in REST API - Medium, accessed July 7, 2025, https://medium.com/@pratik.941/exception-handling-in-rest-api-be74cd8c62b8
Error Handling | Handbook - Technical Direction, accessed July 7, 2025, https://docs.devland.is/technical-overview/api-design-guide/errors
Implementing JWT Authentication in Rust - shuttle.dev, accessed July 7, 2025, https://www.shuttle.dev/blog/2024/02/21/using-jwt-auth-rust
JWT authentication in Rust - LogRocket Blog, accessed July 7, 2025, https://blog.logrocket.com/jwt-authentication-in-rust/
Best Practices for Securing JWT Tokens in React Applications | by Anuj Sharma | Medium, accessed July 7, 2025, https://medium.com/@myfacesproduction/best-practices-for-securing-jwt-tokens-in-react-applications-cc9f63b4dbc0
JWT refresh token flow - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/27726066/jwt-refresh-token-flow
Build an API in Rust with JWT Authentication using actix-web - Auth0, accessed July 7, 2025, https://auth0.com/blog/build-an-api-in-rust-with-jwt-authentication-using-actix-web/
Rust | Axum , JsonWebToken | Jwt |Protected Route | Part-2 | by Mike Code | Medium, accessed July 7, 2025, https://medium.com/@mikecode/rust-axum-jsonwebtoken-jwt-protected-route-part-2-d63ee4952787
JWT Authentication in Rust | A Step-by-Step Guide - YouTube, accessed July 7, 2025, https://www.youtube.com/watch?v=p2ljQrRl0Mg
What is the best approach for JWT Refresh Token? | by Vahit Bayri | KoçSistem | Medium, accessed July 7, 2025, https://medium.com/kocsistem/what-is-the-best-approach-for-jwt-refresh-token-682de2f5c43c
Authenticate Service Account in Rust with JWT(JSON Web Tokens) | by Itsuki - Medium, accessed July 7, 2025, https://medium.com/@itsuki.enjoy/authenticate-service-account-in-rust-with-jwt-json-web-tokens-cfa5056251be
Should JWT be stored in localStorage or cookie? [duplicate] - Stack Overflow, accessed July 7, 2025, https://stackoverflow.com/questions/34817617/should-jwt-be-stored-in-localstorage-or-cookie
Understanding Token Storage: Local Storage vs HttpOnly Cookies - Wisp CMS, accessed July 7, 2025, https://www.wisp.blog/blog/understanding-token-storage-local-storage-vs-httponly-cookies
How to Secure JWT in a Single-Page Application - DEV Community, accessed July 7, 2025, https://dev.to/nilanth/how-to-secure-jwt-in-a-single-page-application-cko
Getting started with REST API Web Services in Rust using Axum ..., accessed July 7, 2025, https://sheroz.com/pages/blog/rust-axum-rest-api-postgres-redis-jwt-docker.html
wpcodevo/rust-jwt-rs256: This guide will walk you through ... - GitHub, accessed July 7, 2025, https://github.com/wpcodevo/rust-jwt-rs256
Generating TypeScript Types with OpenAPI for REST API Consumption | PullRequest Blog, accessed July 7, 2025, https://www.pullrequest.com/blog/generating-typescript-types-with-openapi-for-rest-api-consumption/
A Rust-TypeScript integration - Hacker News, accessed July 7, 2025, https://news.ycombinator.com/item?id=44463654
EtienneDx/example-calculator-app: An example fullstack ... - GitHub, accessed July 7, 2025, https://github.com/EtienneDx/example-calculator-app
Creating a full-stack app with Rust and React | by EtienneDx | Medium, accessed July 7, 2025, https://medium.com/@etiennedx00/creating-a-full-stack-app-with-rust-and-react-61783d6afd80
hey-api/openapi-ts: The OpenAPI to TypeScript codegen. Generate clients, SDKs, validators, and more. Support - GitHub, accessed July 7, 2025, https://github.com/hey-api/openapi-ts
Generate Clients - FastAPI, accessed July 7, 2025, https://fastapi.tiangolo.com/advanced/generate-clients/
Documentation for the typescript-axios Generator, accessed July 7, 2025, https://openapi-generator.tech/docs/generators/typescript-axios/
Best Ways to Generate a TypeScript API Client from an OpenAPI File - API-Fiddle Blog, accessed July 7, 2025, https://blog.api-fiddle.com/posts/best-ways-to-generate-ts-client
API Backwards Compatibility Best Practices | Zuplo Blog, accessed July 7, 2025, https://zuplo.com/blog/2025/04/11/api-versioning-backward-compatibility-best-practices
How do you handle schema evolution in data pipelines and ensure backward compatibility?, accessed July 7, 2025, https://leonidasgorgo.medium.com/how-do-you-handle-schema-evolution-in-data-pipelines-and-ensure-backward-compatibility-48c01efebf71
Schema Evolution Patterns with Backward/Forward Compatibility - Dev3lop, accessed July 7, 2025, https://dev3lop.com/schema-evolution-patterns-with-backward-forward-compatibility/
1105-api-evolution - The Rust RFC Book, accessed July 7, 2025, https://rust-lang.github.io/rfcs/1105-api-evolution.html
4 best practices for your API versioning strategy in 2024 - liblab, accessed July 7, 2025, https://liblab.com/blog/api-versioning-best-practices
Rust Web App "Hello World" Setup (Rust+React+TypeScript+ ..., accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1gfs3iw/rust_web_app_hello_world_setup/
Structuring a Rust mono repo - Reddit, accessed July 7, 2025, https://www.reddit.com/r/rust/comments/1lra6h4/structuring_a_rust_mono_repo/
Building a Monorepo with Rust - Earthly Blog, accessed July 7, 2025, https://earthly.dev/blog/rust-monorepo/
