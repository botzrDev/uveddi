
Architecting Enterprise-Grade Authorization: A Deep Dive into the Casbin Engine


Introduction

In the landscape of modern software architecture, particularly with the rise of distributed systems, microservices, and multi-tenant Software-as-a-Service (SaaS) platforms, the challenge of managing access control has escalated in complexity. Authorization—the process of determining what an authenticated user is allowed to do—is a critical security function that must be both robust and flexible. It is a concern distinct from authentication, which merely verifies a user's identity.1 A poorly designed authorization system can lead to critical security vulnerabilities, operational inefficiencies, and an inability to adapt to evolving business requirements.
To address this challenge, the Casbin authorization engine has emerged as a powerful, flexible, and language-agnostic open-source solution.1 Its core design principle is the decoupling of authorization logic from application code. By externalizing access control rules into a configurable model and a set of policies, Casbin allows developers to manage, modify, and even completely overhaul their authorization scheme without altering the application's source code. It natively supports a wide array of access control paradigms, including Access Control Lists (ACL), Role-Based Access Control (RBAC), and Attribute-Based Access Control (ABAC), as well as hybrid combinations thereof.5
Often, the journey into a sophisticated authorization framework like Casbin begins with a deceptively simple error. In this context, a series of failing authorization tests, all reporting a PolicyEvaluationError with an underlying UnmatchRequestDefinition(4, 3) error, serves as a critical signpost. This error is not merely a bug to be fixed; it is a symptom of a more profound condition—an architectural misalignment between the system's evolving authorization needs and the current definition of its access control model. The application is attempting to ask a more complex question (e.g., "Can this user access this resource within this specific tenant?") than its authorization model is configured to understand.
This report provides a comprehensive architectural guide to designing, implementing, and optimizing a resilient, enterprise-grade authorization service using Casbin. It begins by performing a root cause analysis of the UnmatchRequestDefinition(4, 3) error, using it as a gateway to explore the foundational principles of Casbin's design. From there, it delivers a corrected, production-ready model for multi-tenant RBAC, explores advanced patterns like role inheritance and hybrid RBAC/ABAC, outlines a rigorous testing strategy, and concludes with a detailed analysis of performance considerations for large-scale deployments. The objective is to empower architects and engineers not only to resolve the immediate issue but to build a secure, scalable, and maintainable authorization layer capable of meeting the demands of complex, modern applications.

Section 1: The Casbin PERM Metamodel: A Foundational Framework

At the heart of Casbin's power and flexibility lies the PERM metamodel, an abstraction that stands for Policy, Effect, Request, and Matchers. Every access control model in Casbin, regardless of its complexity, is an implementation of this metamodel, defined within a .conf configuration file.3 Understanding these four foundational components is the absolute prerequisite to effectively wielding the Casbin engine. They collectively describe the relationship between users and resources, forming the logical contract for every authorization decision.

[request_definition] - The Enforcement Contract

The [request_definition] section serves a singular, critical purpose: it defines the precise structure of an access request. It specifies the number, order, and conventional names of the parameters that must be passed to the enforcer's enforce(...) method.6
The most common and classic definition is a three-parameter tuple representing the core elements of access control 10:

[request_definition]
r = sub, obj, act
In this definition:
r is the conventional name for the request object itself.
sub represents the subject, the entity attempting to perform an action (e.g., a user ID, a service account).
obj represents the object, the resource being accessed (e.g., a file path, a database record ID, an API endpoint).
act represents the action, the operation the subject wishes to perform on the object (e.g., read, write, delete).
While these names are conventional, they are fully customizable. A system could, for instance, use user, resource, method if that better aligns with its domain language.12
The most crucial aspect of the [request_definition] is that it establishes a rigid API contract for the enforce() method. The number of arguments passed to an enforce() call in the application code must align perfectly with the number of tokens defined in this section. An enforce("alice", "data1", "read") call is valid for the definition above, but an enforce("alice", "tenant1", "data1", "read") call is not. This strict enforcement of the request shape is fundamental to the engine's operation and is the direct source of the error under investigation. The engine cannot proceed to the matching phase if the incoming request does not conform to the defined contract.

[policy_definition] - The Structure of Rules

The [policy_definition] section defines the structure and semantics of the individual policy rules. These are the rules that are typically stored in a policy file (like a .csv) or a database via a Casbin adapter.6 This definition dictates how Casbin should interpret each line of policy data.
For a standard ACL or RBAC model, the policy definition mirrors the request definition:
[policy_definition]
p = sub, obj, act
Here, p is the conventional name for a policy type. A corresponding policy rule in a .csv file would look like this 14:

p, admin, /api/users, POST
This rule grants the subject admin the action POST on the object /api/users.
Casbin also supports an optional eft (effect) token, which is essential for policies that need to explicitly allow or deny access.6

[policy_definition]
p = sub, obj, act, eft
A policy rule using this definition might be:
p, alice, /reports/confidential, read, deny
If eft is not included in the policy definition, its value is implicitly allow for any matching policy rule.6

[role_definition] - The Foundation of RBAC

The [role_definition] section is what elevates a simple ACL model to a Role-Based Access Control (RBAC) model. This section is optional and should only be included if the model utilizes roles.7 It defines the structure of role inheritance relationships, which Casbin refers to as "grouping policies."
The standard definition for a single RBAC system is:
[role_definition]
g = _, _
In this syntax:
g is the conventional name for the grouping function.
_, _ signifies that the relationship involves two parties.10 This is typically used to map a user to a role (e.g.,
alice is a member of editor) or a role to another role for inheritance (e.g., admin inherits all permissions from editor).
Casbin can even support multiple, independent RBAC systems within the same model by defining additional grouping functions like g2.15 This is useful for advanced scenarios, such as having roles for users (
g) and separate groups for resources (g2).16
The actual user-to-role mappings are not defined in the model but in the policy source.15 For example:

g, alice, data_admin
This line, prefixed with g, declares that the user alice is a member of the data_admin role.

[policy_effect] - The Decision Aggregator

When an incoming request matches multiple policy rules, the [policy_effect] section determines how to aggregate those matches into a single, final decision: allow or deny.12
The most common effect is allow-override, where if at least one matching policy rule permits access, the final result is allow.
[policy_effect]
e = some(where (p.eft == allow))
This means that the existence of a single allow rule is sufficient to grant access. This is the default behavior in many access control systems.13
A more secure and often preferred pattern is deny-override, where a single deny rule can veto any number of allow rules.
[policy_effect]
e =!some(where (p.eft == deny))
This means access is granted only if there are no matching policies that explicitly deny it.13
These can be combined to create an allow-and-deny model, where access is granted only if there is at least one allow rule and zero deny rules:
[policy_effect]
e = some(where (p.eft == allow)) &&!some(where (p.eft == deny))

[matchers] - The Logical Core

The [matchers] section is the logical heart of the Casbin engine. It contains the boolean expression that the enforcer evaluates for each policy rule against the incoming request.6 The result of this evaluation determines if a policy rule is considered a "match" for the request.
For a simple ACL model, the matcher performs a direct comparison of the request's sub, obj, and act with the policy's corresponding fields:
[matchers]
m = r.sub == p.sub && r.obj == p.obj && r.act == p.act
When RBAC is introduced, the matcher is modified to incorporate the role definition. Instead of checking if the request's subject is identical to the policy's subject, it checks if the request's subject is a member of the policy's subject (which is now interpreted as a role). This is done by calling the grouping function g defined in the [role_definition] section.10

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
Here, g(r.sub, p.sub) is a function call that returns true if r.sub has been assigned the role p.sub directly, or if it has inherited that role through a chain of other roles. This single function call encapsulates the entire logic of role hierarchy traversal.
Table 1: Casbin PERM Metamodel Components

Section
Purpose
Example Syntax
Description
[request_definition]
Defines the API contract for enforce() calls.
r = sub, obj, act
Specifies the number, order, and names of parameters in an access request. Must match enforce() arguments exactly. 6
[policy_definition]
Defines the structure of policy rules.
p = sub, obj, act
Specifies the number, order, and names of fields in a policy rule stored in a file or database. 13
[role_definition]
Defines the structure of role inheritance relationships.
g = _, _
Enables RBAC by defining grouping policies. g is a function used in the matcher to check role membership. 15
[policy_effect]
Aggregates results from multiple matching policies.
e = some(where (p.eft == allow))
Determines the final allow/deny decision. Common patterns include allow-override and deny-override. 13
[matchers]
Defines the boolean logic for comparing a request to a policy.
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act
The core evaluation engine. It uses tokens from the request (r) and policy (p) to determine if a rule applies. 10


Section 2: Root Cause Analysis: Decoding the UnmatchRequestDefinition(4, 3) Error

An error message, when properly understood, is not an obstacle but a guide. The specific error signature at the heart of the failing tests—PolicyEvaluationError { policy: "Admin:projects:read", error: "Casbin Request Error: UnmatchRequestDefinition(4, 3)" }—is a precise diagnostic tool that points directly to the root of the problem.

The Error Signature

The error consists of two parts. The outer PolicyEvaluationError indicates that the failure occurred during the policy enforcement process. The inner error, UnmatchRequestDefinition(4, 3), is the key. To understand its meaning, one can look at the Casbin source code itself. The Rust implementation of Casbin (casbin-rs), for example, defines this error as UnmatchRequestDefinition(usize, usize), where the first parameter is the expected length and the second is the found length.17
Therefore, the message UnmatchRequestDefinition(4, 3) indicates that the Casbin enforcer found 3 parameters in the request definition but expected 4. The numbers appear reversed from the source code definition, which is a common variation in how different language bindings might report the error string. The critical takeaway is the mismatch: the model is defined for 3 parameters, but the enforce() call is being made with 4. This is a structural failure that occurs before any policy matching can even be attempted. The engine rejects the request outright because its shape violates the contract established by the [request_definition] section of the model file.

Reconstructing the Scenario

Given the context, a clear hypothesis emerges. The failing tests—test_role_permission_check, test_role_management, test_complete_authorization_flow, and test_permission_inheritance—are all hallmarks of an enterprise-grade RBAC implementation. In such systems, particularly those that are multi-tenant or have complex organizational structures, a simple (subject, object, action) triplet is often insufficient. A fourth parameter is typically required to represent scope or tenancy. In Casbin, this concept is formally known as a domain.18
The most probable scenario is as follows:
The Need: The application requires multi-tenancy. Authorization checks must be scoped to a specific tenant or domain to prevent data leakage and ensure proper isolation.
The Application Code: The developers have correctly updated their application code to reflect this need. Their enforce() calls now include the fourth domain parameter, leading to calls like enforcer.enforce("alice", "tenant_A", "/projects/123", "read"). This accounts for the "4" in the error message.
The Model Configuration: However, the corresponding model.conf file was not updated in tandem. It likely still contains a standard, three-parameter request definition from an earlier, simpler version of the system, such as r = sub, obj, act. This accounts for the "3" in the error message.
The result is a fundamental disconnect. The application is sending a four-parameter request, but the Casbin engine, configured by the outdated model file, is only prepared to accept three.
This analysis reveals that the UnmatchRequestDefinition error should not be viewed as a simple bug but as a crucial indicator of an architectural mismatch. It is a signal that the application's authorization requirements have evolved beyond the capabilities of its currently defined authorization model. The system's needs (a four-parameter, domain-aware request) have outgrown its formal definition (a three-parameter, domain-unaware model). This reframing elevates the task from merely "fixing an error" to "performing a necessary architectural upgrade." The solution, therefore, is not to change the application's enforce() calls back to three parameters—which would abandon the critical requirement of tenancy—but to evolve the Casbin model to correctly handle the more sophisticated, four-parameter request.

Section 3: Designing the Correct RBAC Model and Policy Structure

The root cause analysis clearly indicates that the existing authorization model is insufficient for the system's requirements. The solution is to formally adopt and correctly implement the "RBAC with Domains" pattern, a standard and powerful feature of Casbin designed specifically for multi-tenant and scoped access control scenarios.18 This involves a coordinated update to the model definition (
model.conf), the policy data structure, and the application's enforce() calls.

Corrected Model Definition (model.conf)

To resolve the UnmatchRequestDefinition(4, 3) error and properly support tenancy, the model.conf file must be updated to expect four parameters in its definitions and to use them correctly in its logic. The conventional name for the domain or tenant token in Casbin is dom.18
A corrected and complete model file for RBAC with Domains should be structured as follows:

Ini, TOML


# Request Definition
[request_definition]
r = sub, dom, obj, act

# Policy Definition
[policy_definition]
p = sub, dom, obj, act

# Role Definition
[role_definition]
g = _, _, _

# Policy Effect
[policy_effect]
e = some(where (p.eft == allow))

# Matchers
[matchers]
m = g(r.sub, p.sub, r.dom) && r.dom == p.dom && r.obj == p.obj && r.act == p.act


Let's deconstruct the critical changes in this model:
[request_definition] r = sub, dom, obj, act: This is the most direct fix for the error. The model now explicitly declares that every enforce() call will contain four arguments, which will be bound to r.sub, r.dom, r.obj, and r.act respectively.
[policy_definition] p = sub, dom, obj, act: The policy rules themselves must now be extended to include the domain. This scopes every permission to a specific domain, preventing a permission granted in domain1 from being valid in domain2.
[role_definition] g = _, _, _: This is a subtle but profoundly important change. The standard RBAC definition g = _, _ is replaced with a three-parameter version. The third parameter is the domain.18 This means a user-role assignment is no longer global; it is now a tuple of
(user, role, domain). A user can have the admin role in one domain and a viewer role in another, or no role at all.
[matchers]: The matcher expression becomes more sophisticated to enforce domain isolation:
g(r.sub, p.sub, r.dom): This is the domain-aware role check. It asks, "Does the request's subject (r.sub) have the policy's role (p.sub) within the request's domain (r.dom)?"
r.dom == p.dom: This is a crucial security check. It ensures that the policy rule being evaluated applies to the same domain that is being requested. This check explicitly prevents cross-tenant authorization bleed.
r.obj == p.obj && r.act == p.act: These remain the standard checks for the resource and action.

Corrected Policy Format

With the model updated, the structure of the policy data—whether in a .csv file or a database table—must also align. The policy source will now contain four columns for p rules and three for g rules (in addition to the ptype column).14
Policy (p) lines grant permissions to roles within a specific domain:
Example: p, admin, tenant_A, /api/v1/projects, GET
Meaning: This rule grants the admin role the permission to perform a GET action on the /api/v1/projects object, but only within the context of tenant_A.
Grouping (g) lines assign users to roles within a specific domain:
Example: g, user_alice, admin, tenant_A
Meaning: This rule assigns the user user_alice to the admin role, but only within tenant_A. If user_alice were to make a request in tenant_B, she would not have the admin role.
The domain parameter effectively acts as a namespace for roles and permissions. The role "admin" in tenant_A is a completely separate and distinct entity from the role "admin" in tenant_B. This namespacing is the core mechanism that enables secure multi-tenancy and prevents role and permission pollution between tenants, a critical requirement for any enterprise SaaS application.

Request Parameter Mapping

Finally, the enforce() call in the application code now maps directly and correctly to the new model definition. A call such as:
enforcer.enforce("user_alice", "tenant_A", "/api/v1/projects", "GET")
is mapped by the Casbin engine as follows:
"user_alice" -> r.sub
"tenant_A" -> r.dom
"/api/v1/projects" -> r.obj
"GET" -> r.act
With this alignment across the enforce() call, the model definition, and the policy structure, the UnmatchRequestDefinition error will be resolved, and the authorization logic will correctly evaluate requests based on subject, object, action, and, crucially, domain.
Table 2: Comparison of RBAC Model Definitions

Section
Standard RBAC (Incorrect for this use case)
RBAC with Domains (Corrected)
Key Difference
[request_definition]
r = sub, obj, act
r = sub, dom, obj, act
Adds the dom token to accept a fourth parameter for the domain/tenant. 18
[policy_definition]
p = sub, obj, act
p = sub, dom, obj, act
Adds the dom field to scope permissions to a specific domain. 18
[role_definition]
g = _, _
g = _, _, _
The grouping policy now takes three arguments, binding a user-role assignment to a domain. 18
[matchers]
m = g(r.sub, p.sub) &&...
m = g(r.sub, p.sub, r.dom) && r.dom == p.dom &&...
The matcher now performs a domain-aware role check and explicitly verifies that the request domain matches the policy domain. 18

Table 3: Policy and Grouping Rules for RBAC with Domains

Rule Type
V0 (ptype)
V1 (user/role)
V2 (role/domain)
V3 (obj)
V4 (act)
Description
Policy
p
project_manager
tenant_A
/projects/123
write
Grants the project_manager role write access to /projects/123 within tenant_A. 14
Grouping
g
user_bob
project_manager
tenant_A
(N/A)
Assigns user_bob the project_manager role within tenant_A. 14


Section 4: Advanced Enterprise Authorization Patterns

Resolving the immediate error by adopting the "RBAC with Domains" model establishes a solid foundation. However, enterprise authorization systems often require more nuance and flexibility than this pattern alone can provide. Casbin's true power lies in its ability to compose these foundational models into more sophisticated, hybrid patterns that can address complex, real-world requirements.

Implementing True Role Inheritance

Role inheritance is a cornerstone of RBAC, allowing for the creation of logical permission hierarchies that reduce policy management overhead. In Casbin, role inheritance is transitive by default.15 This means if a role
A inherits from B, and B inherits from C, then A implicitly inherits all of C's permissions.
This is implemented through chained g rules in the policy source. Consider a scenario with admin, editor, and viewer roles within a domain:
p, viewer, domain1, /reports, read
p, editor, domain1, /reports, write
g, editor, viewer, domain1 // The 'editor' role inherits from 'viewer' in domain1
g, admin, editor, domain1 // The 'admin' role inherits from 'editor' in domain1
g, alice, admin, domain1 // Alice is an admin in domain1
In this configuration, a user alice who is an admin also implicitly holds the editor and viewer roles within domain1. Consequently, she is granted both read and write permissions on /reports.
For security and performance reasons, Casbin's default Role Manager implementation includes a maxHierarchyLevel setting, which defaults to 10.15 This prevents runaway recursion in the case of misconfigured circular role dependencies and limits the depth of the inheritance chain that the engine will traverse. This value can be configured when instantiating the role manager.
When querying a user's roles, it is important to distinguish between explicitly assigned roles and implicitly inherited ones. The GetRolesForUser() API will only return directly assigned roles, while GetImplicitRolesForUser() will traverse the hierarchy and return all roles, both direct and inherited.15

Architecting a Hybrid RBAC/ABAC Model

While RBAC is excellent for managing permissions based on a user's job function, it can be too static for rules that depend on the properties of the request itself. For example, "a doctor can only view the medical records of patients assigned to them," or "financial transactions can only be approved by managers whose approval limit is greater than the transaction amount." These are attribute-based conditions, the domain of ABAC.
Casbin supports hybrid RBAC/ABAC models, and the most scalable and maintainable way to implement this is by using the eval() function within the matcher.21 This approach allows the RBAC component to handle the coarse-grained "who" (the user's role) while the ABAC component handles the fine-grained "under what circumstances" (the attributes of the subject, object, or environment).
This pattern avoids the limitations of Casbin's simpler ABAC implementation, which involves passing complex objects to the enforce() function but does not allow for defining attribute-based rules within the policy itself.21 The
eval() pattern, in contrast, moves the attribute logic into the policy, making the authorization system far more dynamic.
Here is a concrete model for a hybrid RBAC/ABAC system:

Ini, TOML


[request_definition]
r = sub, obj, act

[policy_definition]
p = sub, obj, act, condition

[role_definition]
g = _, _

[policy_effect]
e = some(where (p.eft == allow))

[matchers]
m = g(r.sub, p.sub) && r.obj == p.obj && r.act == p.act && eval(p.condition)


In this model, the policy_definition includes a new token, condition. The matcher uses eval(p.condition) to execute the string in the condition field of the policy as a boolean expression. The sub and obj passed to enforce() should be objects or structs with attributes that can be accessed in the expression.
Consider the following policy rules for this model:
p, doctor, medical_record, read, r.sub.id == r.obj.assigned_doctor_id
p, manager, expense_report, approve, r.sub.approval_limit >= r.obj.amount
g, dr_alice, doctor
g, bob_manager, manager
When dr_alice (who is a doctor) attempts to read a medical_record, the enforce() call would look like this: enforce(alice_object, record_object, "read"). The matcher first confirms her role via g(r.sub, p.sub). Then, eval() executes the condition r.sub.id == r.obj.assigned_doctor_id, dynamically comparing the id attribute of the alice_object with the assigned_doctor_id attribute of the record_object. Access is granted only if the RBAC check passes and the ABAC check passes.
This eval() pattern is the architectural bridge between RBAC and ABAC. It creates a powerful separation of concerns. The RBAC rules (g lines) manage stable, role-based permissions, while the ABAC rules (p lines with conditions) manage dynamic, context-sensitive permissions. Crucially, new attribute checks can be added to policies without requiring any changes to the application code or the core model definition, a significant advantage for maintainability in evolving enterprise systems.22

Flexible Resource-Action Mapping with Patterns

A common challenge in enterprise systems is policy explosion, where a large number of resources leads to an unmanageable number of policy rules. For example, granting a user access to every file in a directory would require one policy rule per file.
Casbin solves this with built-in pattern-matching functions that can be used in the matcher.23 Functions like
keyMatch2 (for RESTful path matching like /users/:id) and regexMatch (for regular expression matching) allow a single policy rule to cover a wide range of resources or actions.
Consider this matcher:
[matchers]
m = g(r.sub, p.sub) && keyMatch2(r.obj, p.obj) && regexMatch(r.act, p.act)
With this matcher, a single policy rule can express a complex permission:
p, doc_editor, /projects/:id/docs/*, (read|write)
This rule grants any user with the doc_editor role the ability to either read or write any resource that matches the path /projects/:id/docs/*. This is vastly more efficient than defining individual rules for every document and every action.
This pattern can also be applied to roles themselves in an "RBAC with Pattern" model.25 For example, a rule like
g, /book/:id, book_group could automatically assign any resource whose path starts with /book/ to the book_group, which can then have permissions assigned to it. These patterns are essential tools for keeping policy sets concise and manageable at an enterprise scale.

Section 5: A Comprehensive Validation and Testing Strategy

Authorization logic constitutes a critical security boundary of any application. A flaw in this logic can lead to unauthorized data exposure, privilege escalation, or complete system compromise. Therefore, it cannot be an afterthought; it must be subject to a rigorous and multi-layered validation and testing strategy.

Prototyping with the Casbin Online Editor

Before writing any code, the Casbin Online Editor serves as an invaluable tool for rapid prototyping and interactive validation.3 This web-based IDE allows developers to input their
model.conf, define a set of policy.csv rules, and simulate enforce() requests. It provides immediate feedback on whether a request is allowed or denied, helping to confirm that the model logic and policy structure are correct before they are integrated into the application. This is the ideal first step for validating the "RBAC with Domains" model and experimenting with advanced patterns.

Static Test Cases (Unit Tests)

The core of the validation strategy should be a comprehensive suite of automated tests that are part of the application's continuous integration pipeline. These tests should treat the authorization logic as a black box, providing a set of inputs (policies, roles, and a request) and asserting the expected boolean outcome. For the corrected "RBAC with Domains" model, these tests must cover a wide range of scenarios to ensure correctness and security.
A robust test suite must focus disproportionately on "deny" scenarios and security-critical edge cases. While it is important to verify that a user can perform an authorized action (the "happy path"), it is far more critical from a security perspective to prove that a user cannot perform an action they are not authorized for. A bug that incorrectly grants access is a high-severity vulnerability, whereas a bug that incorrectly denies access is a functional issue. Therefore, the test suite must prioritize scenarios that attempt to breach security boundaries.

Dynamic Testing with the Casbin API (Integration Tests)

While static tests with a fixed policy set are essential, real-world systems often involve policies and roles that change at runtime. Casbin provides a rich set of Management and RBAC APIs that allow for the programmatic manipulation of the policy state during tests, enabling more realistic integration testing.19
Key APIs for dynamic testing include:
Policy Management: AddPolicy, RemovePolicy, UpdatePolicy
Role Management: AddRoleForUser, DeleteRoleForUser
Domain-Aware Role Management: AddRoleForUserInDomain, DeleteRoleForUserInDomain
Permission Management: AddPermissionForUser, DeletePermissionForUser
An integration test can use these APIs to simulate a complete user lifecycle or administrative workflow. For example, a test could:
Setup: Initialize the enforcer with the domain model and an empty policy set.
Act:
Programmatically add a domain-specific role for a new user: enforcer.AddRoleForUserInDomain("new_user", "viewer", "domain_C").
Add a corresponding permission: enforcer.AddPolicy("viewer", "domain_C", "resource_X", "read").
Assert:
Verify the user can now access the resource: Assert(enforcer.enforce("new_user", "domain_C", "resource_X", "read") == true).
Verify the user cannot access a resource in another domain: Assert(enforcer.enforce("new_user", "domain_D", "resource_X", "read") == false).
Teardown:
Remove the role: enforcer.DeleteRoleForUserInDomain("new_user", "viewer", "domain_C").
Verify access is now revoked: Assert(enforcer.enforce("new_user", "domain_C", "resource_X", "read") == false).
Clean up the policy store for the next test: enforcer.ClearPolicy().
This dynamic approach allows for testing the full behavior of the authorization system, including the effects of administrative actions on user permissions.
Table 4: Comprehensive Test Scenarios for RBAC with Domains
Scenario Description
Policy/Role Setup
enforce() Call
Expected Result
Rationale
Basic Allow
p, admin, d1, res1, read g, alice, admin, d1
("alice", "d1", "res1", "read")
true
Validates that a user with the correct role in the correct domain can access a resource.
Basic Deny (Wrong Action)
p, admin, d1, res1, read g, alice, admin, d1
("alice", "d1", "res1", "write")
false
Ensures permissions are action-specific.
Basic Deny (Wrong Role)
p, admin, d1, res1, read g, alice, viewer, d1
("alice", "d1", "res1", "read")
false
Confirms that a user without the required role is denied access.
Critical: Cross-Domain Deny
p, admin, d1, res1, read g, alice, admin, d1
("alice", "d2", "res1", "read")
false
The most important test. Verifies that a user's role in one domain does not grant them any permissions in another domain.
Critical: Policy Scope Deny
p, admin, d1, res1, read g, alice, admin, d2
("alice", "d2", "res1", "read")
false
Verifies that a permission scoped to one domain does not apply even if the user has a role in another domain.
Role Inheritance Allow
p, viewer, d1, res1, read g, editor, viewer, d1 g, alice, editor, d1
("alice", "d1", "res1", "read")
true
Validates that permissions are correctly inherited through the role hierarchy.
Role Inheritance Deny
p, admin, d1, res1, write g, editor, viewer, d1 g, alice, editor, d1
("alice", "d1", "res1", "write")
false
Ensures a user does not inherit permissions from roles "above" them in the hierarchy.
Non-Existent User
p, admin, d1, res1, read
("bob", "d1", "res1", "read")
false
Confirms that an unknown user without any roles has no access.
Non-Existent Resource
p, admin, d1, res1, read g, alice, admin, d1
("alice", "d1", "res2", "read")
false
Validates that permissions are object-specific.


Section 6: Performance Analysis and Optimization for Scale

Authorization decisions often lie on the critical path of application requests, making the performance of the policy engine a paramount concern, especially in high-throughput, enterprise-scale environments.29 While Casbin is designed to be highly performant, naive implementations can encounter significant bottlenecks when dealing with a large number of policies or a high volume of traffic. Understanding the performance characteristics of different models and employing architectural optimization strategies is essential for building a scalable system.

Benchmark Analysis

Official benchmarks provided by the Casbin project offer valuable quantitative insight into the performance overhead of various models.30 Analyzing these benchmarks reveals clear patterns.
For the Go implementation, which is often used in high-performance backends, the overhead of an enforce() call demonstrates the cost of increasing complexity 30:
ACL (2 rules): 0.015 ms/op
RBAC (5 rules): 0.021 ms/op
RBAC with Domains (6 rules): 0.032 ms/op
This data shows that for a small, comparable number of rules, introducing basic RBAC adds a minor overhead compared to a simple ACL. Adopting the more complex "RBAC with Domains" model increases the latency by approximately 50% over basic RBAC. This is the expected trade-off for the added functionality of domain-scoping.
More importantly, the benchmarks show that performance degrades significantly as the number of policy rules grows into the thousands. For a large RBAC model with 110,000 rules, the enforcement time in Go jumps to over 23 ms. This highlights that loading the entire policy set into memory for every enforcer is not a viable strategy for large-scale, multi-tenant applications.

Architectural Best Practices for Enterprise Scale

To mitigate performance issues at scale, several architectural patterns and best practices are crucial.
Policy Subset Loading (Sharding)
This is the single most critical optimization for multi-tenant systems.29 Instead of a single, monolithic enforcer that loads all policies for all tenants, this pattern involves partitioning the policies by domain (or tenant ID). Each enforcer instance then loads only the subset of policies relevant to the domain(s) it serves. This is achieved using a
FilteredAdapter.31
For example, in a Kubernetes environment, a request for tenant_A could be routed to a pod running a Casbin enforcer that was initialized with a filter to load only policies where the domain is tenant_A. This keeps the in-memory policy set for that enforcer small and enforcement times low, regardless of whether there are 10 or 10,000 other tenants in the system. This pattern effectively makes performance independent of the total number of tenants.
Efficient Role Management
As a rule of thumb, permissions should be granted to roles, not directly to individual users.29 The reason lies in how Casbin's RBAC engine works. The role manager builds an efficient in-memory graph (a role-inheritance tree) to represent
g rules. When g(user, role) is checked, it performs a fast traversal of this graph. If permissions are assigned directly to users via p rules, the number of policies can explode, and for each enforce() call, the engine must iterate over a much larger set of p rules to find a match. Keeping the number of p rules small by centering them on roles is a key performance lever.
Matcher Optimization
The order of conditions within the [matchers] section can have a dramatic and often non-obvious impact on performance. Because boolean expressions are evaluated with short-circuiting logic, placing the cheapest and most selective conditions first can prevent the engine from executing more expensive operations.
A benchmark test in the Casbin repository provides a stunning example of this principle.13 An enforcement check with the matcher
m = g(r.sub, p.sub) && r.obj == p.obj &&... took over 6 seconds to complete under a heavy load. By simply reordering the conditions to m = r.obj == p.obj && g(r.sub, p.sub) &&..., the same check completed in just 7 milliseconds. The reason is that r.obj == p.obj is a very fast and highly selective string comparison that can immediately discard the vast majority of policy rules. The g(...) function, which may involve graph traversal, is more expensive and is only executed for the small subset of policies that have already passed the object check. This optimization is critical for systems with large numbers of policies.
Concurrency and Infrastructure
For handling high request volume, Casbin can be scaled both vertically and horizontally.
Multi-threading: A single application can instantiate multiple enforcers (or use a thread-safe enforcer) to take full advantage of all available CPU cores.29
Clustering: When deploying across multiple machines, a Watcher component should be used. The Watcher connects to a message bus (like Redis or etcd) and notifies all enforcer instances in the cluster whenever a policy is updated, ensuring that all nodes remain consistent.5 Discussions within the community also highlight patterns like enforcer pooling to manage resources efficiently in web server environments.32
Table 5: Performance Benchmark Analysis: Model Complexity vs. Overhead (Go)
Model Type
Rule Size
Time Overhead (ms/op)
Memory Overhead (KB)
ACL
2 rules (2 users)
0.015
5.6
RBAC (small)
1100 rules (1000 users, 100 roles)
0.164
80.6
RBAC with Domains (small)
6 rules (2 users, 1 role, 2 domains)
0.032
10.7
RBAC (large)
110,000 rules (100,000 users, 10,000 roles)
23.916
7,606.0

Source: Casbin official benchmarks 30
Table 6: Enterprise Performance Optimization Strategies

Strategy
Description
When to Use
Impact Level
Policy Subset Loading
Using a FilteredAdapter to load only a specific domain's/tenant's policies into an enforcer instance. 31
In any multi-tenant or large-scale application with partitioned data.
High
Role-Centric Permissions
Granting permissions to roles (p rules) and assigning users to roles (g rules), rather than assigning permissions directly to users. 29
Always, but especially when the number of users is much larger than the number of roles.
High
Matcher Ordering
Placing the cheapest and most selective conditions (e.g., string comparisons) before more expensive ones (e.g., g() role checks) in the matcher expression. 13
In all models, but the effect is most pronounced with large policy sets.
High
Concurrency/Clustering
Using multi-threaded enforcers and a Watcher to synchronize policy updates across a cluster of application instances. 29
For applications with high request throughput that require horizontal scaling.
Medium


Conclusion and Strategic Recommendations

The investigation into the UnmatchRequestDefinition(4, 3) error has served as a gateway to a much broader architectural discussion. The analysis confirms that the error was not a simple bug but a clear symptom of an architectural mismatch: the application's authorization needs had evolved to require multi-tenancy, while its underlying Casbin model had not. This disconnect between the four-parameter enforce() calls in the application and the three-parameter [request_definition] in the model was the direct cause of the failing tests.
The resolution involves a holistic upgrade of the authorization system. The foundational solution is the adoption of Casbin's "RBAC with Domains" pattern. This requires correcting the model.conf file to define four-parameter request, policy, and role definitions, and ensuring the policy data structure aligns with this new model. This change not only fixes the error but also establishes a secure, scalable foundation for multi-tenant access control by namespacing roles and permissions within specific domains. Beyond this immediate fix, this report has detailed advanced patterns for implementing true role inheritance, architecting flexible hybrid RBAC/ABAC systems using the eval() pattern, and managing policies efficiently with pattern matching. Furthermore, it has provided a comprehensive testing strategy focused on security and a set of critical performance optimization techniques, such as policy subset loading and matcher ordering, which are essential for enterprise-grade deployments.
To move forward successfully, the following strategic checklist is recommended:
Model First: Finalize the model.conf file, adopting the "RBAC with Domains" structure as the baseline. Critically evaluate whether future requirements will necessitate attribute-based rules and, if so, incorporate the eval() pattern into the matcher from the outset.
Structure Policies Correctly: Ensure the chosen policy adapter (e.g., database schema) is designed to store p and g rules with the domain as a distinct and indexable field.
Implement Comprehensive Tests: Build an automated test suite based on the scenarios outlined in this report. Place special emphasis on negative test cases that probe for security weaknesses, particularly those attempting to bypass domain isolation.
Plan for Scale: If the application is or will become multi-tenant, architect the system with Policy Subset Loading (sharding) from day one. Retrofitting this pattern later is significantly more difficult.
Optimize Matchers: Always order the conditions in the [matchers] section strategically, placing the cheapest and most selective checks first to maximize performance through short-circuiting.
Maintain Decoupling: Continue to leverage Casbin's core strength by treating it as a pure authorization engine. Keep concerns like authentication, user identity management, and session control within separate, dedicated services.
By following these recommendations, an organization can move beyond reactive bug-fixing and proactively architect an authorization service that is not only correct but also secure, maintainable, and prepared to scale with the demands of the business.
Works cited
Overview - Casbin, accessed July 18, 2025, https://casbin.org/docs/overview/
Casbin-ruby - an authorization library supporting distributed role-based access control (RBAC) by Evrone, accessed July 18, 2025, https://evrone.com/blog/casbin-ruby
casbin/casbin: An authorization library that supports access control models like ACL, RBAC, ABAC in Golang: https://discord.gg/S5UjpzGZjN - GitHub, accessed July 18, 2025, https://github.com/casbin/casbin
Casbin · An authorization library that supports access control models like ACL, RBAC, ABAC for Golang, Java, C/C++, Node.js, Javascript, PHP, Laravel, Python, .NET (C#), Delphi, Rust, Ruby, Swift (Objective-C), Lua (OpenResty), Dart, accessed July 18, 2025, https://casbin.org/
Casbin · An authorization library that supports access control models like ACL, RBAC, ABAC for Golang, Java, C/C++, Node.js, Javascript, PHP, Laravel, Python, .NET (C#), Delphi, Rust, Ruby, Swift (Objective-C), Lua (OpenResty), Dart, accessed July 18, 2025, https://v1.casbin.org/
How It Works - Casbin, accessed July 18, 2025, https://casbin.org/docs/how-it-works/
Syntax for Models - Casbin, accessed July 18, 2025, https://v1.casbin.org/docs/en/syntax-for-models
casbin.org, accessed July 18, 2025, https://casbin.org/docs/how-it-works/#:~:text=Request%E2%80%8B,%3D%7Bsub%2Cobj%2Cact%7D
Implementing Authorization using Casbin. Introduction to Casbin RBAC | Mano Sriram, accessed July 18, 2025, https://manosriram.com/posts/casbin-rbac/
casbin/examples/rbac_model.conf at master - GitHub, accessed July 18, 2025, https://github.com/casbin/casbin/blob/master/examples/rbac_model.conf
Using Casbin for user authorization in a system, accessed July 18, 2025, https://2coffee.dev/en/articles/using-casbin-for-user-authorization-in-a-system
Web authorization with Casbin | Andrew Klotz, accessed July 18, 2025, https://klotzandrew.com/blog/authorization-with-casbin/
Syntax for Models - Casbin, accessed July 18, 2025, https://casbin.org/docs/syntax-for-models/
Policy Storage | Casbin, accessed July 18, 2025, https://casbin.org/docs/policy-storage/
RBAC - Casbin, accessed July 18, 2025, https://v1.casbin.org/docs/en/rbac
RBAC - Casbin, accessed July 18, 2025, https://casbin.org/docs/rbac/
error.rs - source - Casbin, accessed July 18, 2025, https://v1.casbin.org/casbin-rs/src/casbin/error.rs.html
RBAC with Domains - Casbin, accessed July 18, 2025, https://casbin.org/docs/rbac-with-domains/
RBAC with Domains API - Casbin, accessed July 18, 2025, https://casbin.org/docs/rbac-with-domains-api/
How to list inherited permissions by father role? · Issue #137 · casbin/casbin - GitHub, accessed July 18, 2025, https://github.com/casbin/casbin/issues/137
ABAC - Casbin, accessed July 18, 2025, https://casbin.org/docs/abac/
nav/rbac-abac: An example implementation of RBAC and ... - GitHub, accessed July 18, 2025, https://github.com/nav/rbac-abac
casbin-tutorials/tutorials/RBAC-with-Casbin.md at master · php-casbin/casbin-tutorials - GitHub, accessed July 18, 2025, https://github.com/php-casbin/casbin-tutorials/blob/master/tutorials/RBAC-with-Casbin.md
Role Based Access Control By Example - Mechanical Rock, accessed July 18, 2025, https://www.mechanicalrock.io/blog/role-based-access-control-by-example
RBAC with Pattern - Casbin, accessed July 18, 2025, https://casbin.org/docs/rbac-with-pattern/
casbin/pycasbin: An authorization library that supports access control models like ACL, RBAC, ABAC in Python - GitHub, accessed July 18, 2025, https://github.com/casbin/pycasbin
API Overview | Casbin, accessed July 18, 2025, https://casbin.org/docs/api-overview/
RBAC API - Casbin, accessed July 18, 2025, https://casbin.org/docs/rbac-api
Performance Optimization - Casbin, accessed July 18, 2025, https://casbin.org/docs/performance/
Benchmarks | Casbin, accessed July 18, 2025, https://casbin.org/docs/benchmark/
Policy Subset Loading - Casbin, accessed July 18, 2025, https://casbin.org/docs/policy-subset-loading/
Casbin best practices · Issue #239 - GitHub, accessed July 18, 2025, https://github.com/casbin/Casbin.NET/issues/239