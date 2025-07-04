
A Systematic Analysis of Leaky Abstractions in JavaScript and TypeScript for Automated Detection


I. Introduction: The Nature of Abstraction in a Dynamic Ecosystem


1.1 The Law of Leaky Abstractions in Modern Software

In software engineering, an abstraction is a mechanism for hiding the complex implementation details of a system, exposing only a simplified interface to its consumers. The goal is to manage complexity, allowing developers to build sophisticated systems without needing to understand every underlying component in its entirety.1 However, this ideal is rarely achieved perfectly. A "leaky abstraction" is a design flaw where the underlying complexity is not fully concealed, forcing developers to become aware of the very details the abstraction was meant to hide.1 This phenomenon was famously codified by Joel Spolsky in what he termed the "Law of Leaky Abstractions," which posits that all non-trivial abstractions, to some degree, are leaky.1
This law is not an indictment of abstraction itself, but a pragmatic acknowledgment of its inherent limitations. As systems grow in complexity, the probability of such leaks increases.1 These leaks manifest in various ways, including unexpected performance degradation, subtle bugs, and an increased cognitive load on developers who must mentally model both the abstraction and its underlying mechanics to use it effectively or troubleshoot it.1 For instance, an Object-Relational Mapper (ORM) abstracts away the need to write raw SQL. This is a powerful simplification. However, when a query performs poorly, the developer is forced to "leak" down a level, analyzing the generated SQL and understanding database indexing strategies—details the ORM was supposed to hide.3 Similarly, the TCP protocol provides the abstraction of a reliable data stream, but when network packets are lost, this implementation detail leaks through as increased latency, a detail the consumer of the abstraction must often handle.1
The existence of this law suggests that any tool designed to detect leaky abstractions cannot function as a simple binary validator, flagging code as merely "correct" or "incorrect." Many foundational technologies we rely on, from network protocols to database query languages, have inherent, well-understood leaks that are accepted as part of a trade-off for their utility.1 An effective detection tool must therefore be diagnostic in nature. It needs to differentiate between an unavoidable, inherent leak in a technology (the performance characteristics of an ORM) and an architectural violation introduced by a developer (using that ORM's query builder directly within a UI component). The goal of such a tool should be to illuminate these boundary crossings, providing developers with the context needed to make informed architectural decisions rather than blindly "fixing" every reported leak.2

1.2 The JavaScript/TypeScript Context

The JavaScript and TypeScript ecosystem is a particularly fertile ground for leaky abstractions. JavaScript's journey from a simple scripting language for web pages to a platform for complex, full-stack applications has been characterized by the continuous layering of new abstractions to manage its evolving complexity.7 Many modern web development tools and languages, for instance, compile
to JavaScript, making JavaScript itself the target of an abstraction layer, which can introduce its own set of leaks.7
TypeScript adds another layer of abstraction over JavaScript by introducing a static type system. A key feature of this system is its structural nature, often referred to as "duck typing".9 In a structural type system, compatibility between types is determined by their shape—the properties and methods they contain—rather than by explicit inheritance or declaration. If an object has the same shape as an interface, TypeScript considers it compatible, regardless of whether the developer explicitly stated that the object
implements that interface.9 This provides immense flexibility but can also be a source of leaks. An object can satisfy a contract by accident, and the abstraction of "type" becomes more about coincidental structure than explicit design intent, a nuance that developers must understand.

1.3 A Framework for Analysis: Architectural Layers

To systematically identify and analyze leaky abstractions, it is essential to establish a clear architectural model. This report will adopt a framework inspired by Clean Architecture, which organizes a system into concentric layers of concern.10 This model provides a set of boundaries, and a leak can be defined as an inappropriate dependency that crosses these boundaries in the wrong direction. The primary layers for our analysis are:
Presentation Layer: Responsible for displaying information to the user and handling user interaction. This includes UI components (e.g., React, Vue), templates, and DOM manipulation logic.
Application Layer: Orchestrates use cases and coordinates the flow of data. This layer contains application-specific business rules and is often implemented as services or interactors. It directs the domain layer to fulfill tasks.
Domain Layer: Contains the core business logic and entities of the application. This layer represents the fundamental rules and data structures of the business, independent of any application or framework. It should be composed of Plain Old JavaScript Objects (POJOs) or pure classes with no external dependencies.
Infrastructure Layer: Contains all the external concerns and implementation details. This includes frameworks (e.g., Express), database clients (e.g., ORMs, drivers), third-party API clients, and other low-level tools.
The fundamental principle governing these layers is the Dependency Rule: source code dependencies can only point inwards. For example, the Application layer can depend on the Domain layer, but the Domain layer must not depend on the Application layer or any other outer layer.11 A leaky abstraction, in this context, is a violation of this rule. When a detail from an outer layer (e.g., an Express
request object from the Infrastructure layer) is passed into an inner layer (e.g., a Domain service), the abstraction has leaked. This framework transforms the abstract concept of a "leak" into a concrete, detectable violation of the Separation of Concerns (SoC) principle, providing a robust foundation for automated analysis.1

II. Leaks in the Foundation: Module Systems and Global Scope

The very foundation of code organization in JavaScript—the module system—is an abstraction designed to manage scope, dependencies, and complexity. The two dominant systems, CommonJS (CJS) and ECMAScript Modules (ESM), provide different abstractions with distinct characteristics and potential for leaks.

2.1 Comparative Analysis: CommonJS (CJS) vs. ECMAScript Modules (ESM)

CJS emerged to solve critical problems in early server-side JavaScript with Node.js, such as global namespace pollution and unmanageable dependencies.8 ESM was later introduced in the ECMAScript 2015 specification as the official, standardized module system for all JavaScript environments.14 Their differing design philosophies result in different abstraction models.
The historical progression from unstructured scripts to manually managed scopes with Immediately Invoked Function Expressions (IIFEs), then to environment-specific modules like CJS, and finally to a standardized system with ESM, illustrates the Law of Leaky Abstractions in action.16 Each step was an attempt to create a better abstraction over JavaScript's execution model. IIFEs were a disciplined but verbose and manual solution to scope pollution.16 CJS automated this for Node.js but created an ecosystem fragmented from the browser.14 ESM aimed to unify this but introduced its own complexities, particularly around interoperability with the vast existing CJS ecosystem.14 This led to the rise of module bundlers, another layer of abstraction designed to hide these inconsistencies from the developer, but which introduce their own configuration and performance leaks.19 An automated detection tool must be aware of this entire history, as the context in which code is written (e.g., raw ESM for a modern browser vs. CJS transpiled and bundled by Webpack) dictates which leaks are possible.
The following table provides a consolidated comparison of their core features and the implications for abstraction.

Feature
CommonJS (CJS)
ECMAScript Modules (ESM)
Abstraction Implication
Loading
Synchronous (require()) 14
Asynchronous (import) 14
CJS's synchronous nature abstracts away the complexity of async operations, simplifying server-side code where I/O is local. However, it can leak performance issues by blocking the event loop. ESM's async-first model is better for non-blocking environments like the browser, but the complexity of managing the async loading is a detail hidden by the runtime/bundler.
Resolution
Dynamic (Runtime) 18
Static (Compile-time) 14
ESM's static structure is a powerful abstraction that enables tools to perform optimizations like tree-shaking (removing unused code), effectively hiding dead code from the final bundle. CJS's dynamic nature allows for flexible patterns like conditional require(), but this means the module's "shape" is unpredictable until runtime, a leak that prevents static analysis.
Syntax
require() / module.exports 14
import / export 12
ESM's declarative import/export syntax provides a cleaner, less error-prone abstraction boundary. The module.exports object in CJS can be completely replaced, leading to confusing and leaky module APIs if not handled carefully. The exports shortcut variable is another source of confusion, as it's just a reference to the initial module.exports.13
Binding
Value copy (for primitives), reference copy (for objects) 12
Live binding 12
CJS's behavior can be a subtle leak. If an exporting module changes a primitive value after it has been required, the importing module will not see the change, as it received a copy. ESM's live bindings create a more transparent abstraction where imported values are read-only views into the original module's variables, always reflecting their current state.
Interoperability
Can use ESM via dynamic import()
Can use CJS via import (often with bundler help) or createRequire() in Node.js 14
The very need for these complex interoperability mechanisms is a significant leak. Developers are forced to understand the underlying module format of their dependencies and use special syntax or helpers to bridge the gap, breaking the abstraction that they can simply "import a module."


2.2 Export Patterns and Interface Design

The choice of export pattern fundamentally shapes a module's public interface and the degree to which its internal structure is exposed.
export default: This pattern presents the module as a single, primary entity. It abstracts away the internal naming and structure, offering one main "thing" to the consumer. This can lead to cleaner imports but also obscures the contents of the module.
Named Exports (export const...): This pattern exposes specific, named parts of the module's implementation.12 It is more explicit and allows for better static analysis and tree-shaking, as consumers import only what they need.8 However, it can lead to tighter coupling if consumers depend on a large number of a module's internal pieces.
A well-designed module interface uses these patterns deliberately. For example, a module might use named exports for a set of utility functions but a default export for a primary class or factory function. A leaky abstraction occurs when these choices are haphazard, forcing consumers to guess the module's intended public API or to depend on implementation details that should have been kept private.

2.3 Namespace Pollution and Global Leaks

One of the earliest and most significant problems that JavaScript modules were designed to solve is global namespace pollution.12 In classic, pre-module JavaScript, any variable declared in the top-level scope of a
<script> tag becomes a property of the global object (window in browsers).17 This creates a high risk of naming collisions between different scripts and libraries, leading to unpredictable behavior and hard-to-debug errors.21
To combat this, developers created patterns that served as manual abstractions for scope. The most common was the Immediately Invoked Function Expression (IIFE), also known as the Module Pattern.8 By wrapping code in a function that is executed immediately, developers could create a private scope, preventing variables from leaking into the global namespace. Only the values explicitly returned from the function or attached to a global object would become public.17
Modern module systems like CJS and ESM provide this scoping abstraction automatically. Each file is its own module with its own top-level scope, effectively eliminating the problem of global namespace pollution for declared variables.8 However, leaks can still occur. The most common way is through accidental global variable creation. In non-strict mode, assigning a value to an undeclared variable (e.g.,
myVar = 'leaky') does not throw an error; instead, it creates a property on the global object.17 While modern development environments almost always use strict mode, which prevents this, it remains a potential source of leaks in legacy code or misconfigured projects.

2.4 Module Bundler Implications

For frontend development, module bundlers like Webpack, Vite, or Parcel are an essential layer of abstraction.19 They take a project's many individual JavaScript files and dependencies and combine them into one or a few optimized files that a browser can efficiently load.24 This hides the complexity of managing hundreds of HTTP requests and the differences between module formats.20 However, this powerful abstraction is itself non-trivial and prone to leaks.
Configuration Complexity: A bundler like Webpack is highly configurable, which is both a strength and a source of leaks. Developers are supposed to simply write import statements, but when a build fails or performs poorly, they are forced to dive into the bundler's configuration file and understand complex concepts like loaders, plugins, and code splitting strategies. The abstraction of a seamless build process leaks the reality of its complex machinery.24
Dependency Graph Leaks: Bundlers work by creating a dependency graph, starting from an entry point and mapping all import and require statements.24 The abstraction is that this "just works." The leak occurs when circular dependencies cause runtime errors or when the bundler's resolution logic for CJS vs. ESM modules produces unexpected results, forcing the developer to debug the graph itself.14
Hot Module Replacement (HMR): HMR is an abstraction that allows developers to see changes in their code without a full page reload. This is a powerful productivity feature, but it can leak. For example, it may not correctly handle state in certain situations, leading to a UI that is out of sync with the application's actual state, forcing a manual refresh—a clear sign the abstraction has failed.24
Naming Conflicts: When bundling modules, there is a potential for naming conflicts, especially with CSS classes or other global identifiers. While many bundlers and associated tools provide solutions like CSS Modules to create scoped names, a naive setup can lead to styles from one component leaking and affecting another.19

III. Common Abstraction Violations in JavaScript/TypeScript Applications

Beyond the foundational level of modules, a set of common anti-patterns consistently appear in JavaScript and TypeScript applications. These patterns represent violations of architectural boundaries, where implementation details from one layer leak into another. Fundamentally, all these patterns can be understood as specific instances of a single, overarching anti-pattern: violating the Dependency Rule of Clean Architecture. This principle states that high-level policies (business logic) should not depend on low-level details (UI, frameworks, databases); rather, dependencies should always point inwards toward the core, abstract business rules. Recognizing this unifying principle is key to building a robust detection tool, as it provides a single, powerful heuristic for identifying a wide range of seemingly disparate issues. An automated tool could be configured with knowledge of a project's architectural layers (e.g., via folder paths) and then flag any import that violates this inward-pointing dependency flow.

3.1 Pattern: DOM Manipulation in Business Logic

Description: This pattern occurs when a module intended to contain pure business logic directly interacts with the Document Object Model (DOM). For example, a function responsible for calculating a shopping cart's total price also updates an HTML element to display that price.26
Why it's a Leak: This tightly couples the business logic to a specific presentation technology (the web browser) and a specific HTML structure. The abstraction of "business logic" is broken because it now contains knowledge of "presentation details." This has several negative consequences:
Portability: The logic cannot be reused in a different environment, such as a Node.js backend for an API or a native mobile application, without significant refactoring.
Testability: Unit testing the logic becomes difficult. Instead of simply providing inputs and asserting outputs, the test environment must now include a simulated DOM (like JSDOM), adding complexity and slowing down tests.28
Maintainability: If the HTML structure changes (e.g., an id is renamed), the business logic code must also be changed, violating the principle of Separation of Concerns.
Example & Fix:
Leaky Code:
TypeScript
// src/services/cartService.ts
import { Product } from '../models/product';

// This function mixes calculation with DOM manipulation.
function calculateAndDisplayTotal(products: Product): void {
  const total = products.reduce((sum, p) => sum + p.price, 0);

  // LEAK: Direct dependency on the DOM and a specific element ID.
  const totalElement = document.getElementById('cart-total-display');
  if (totalElement) {
    totalElement.textContent = `$${total.toFixed(2)}`;
  }
}

Refactored Code:
TypeScript
// src/services/cartService.ts (Pure Business Logic)
import { Product } from '../models/product';

export function calculateTotal(products: Product): number {
  return products.reduce((sum, p) => sum + p.price, 0);
}

// src/components/Cart.tsx (Presentation Logic)
import React from 'react';
import { calculateTotal } from '../services/cartService';

function Cart({ products }) {
  const total = calculateTotal(products);

  // The UI layer is responsible for rendering the value.
  return (
    <div>
      {/*... list of products... */}
      <p>Total: <span id="cart-total-display">${total.toFixed(2)}</span></p>
    </div>
  );
}



3.2 Pattern: Framework-Specific Components in Data Models

Description: This leak occurs when core data models or domain entities are defined using constructs from a specific UI framework. For example, a User class might extend React.Component or use Vue's reactive() function internally.29
Why it's a Leak: The application's core data structures, which should represent pure, framework-agnostic business concepts, become entangled with the implementation details of the view layer. The abstraction of a "User" is polluted with the concerns of "how to render a User." This makes the data model non-portable and tightly coupled to a single framework, preventing its reuse in other contexts (e.g., sharing between a React web app and a React Native mobile app that might have different base components).31
Example & Fix:
Leaky Code:
TypeScript
// src/models/Poll.ts (Leaking Vue's reactivity system)
import { reactive } from 'vue';

// This is not a pure data model; it's a Vue-specific reactive object.
export function createPoll(question: string) {
  return reactive({
    question,
    votes: 0,
    addVote() {
      this.votes++;
    }
  });
}

Refactored Code:
TypeScript
// src/models/Poll.ts (Pure Domain Model)
export class Poll {
  public votes: number = 0;

  constructor(public readonly question: string) {}

  addVote(): void {
    this.votes++;
  }
}

// src/components/PollComponent.vue (UI Layer)
<script setup>
import { reactive } from 'vue';
import { Poll } from '../models/Poll';

// The framework concern (reactivity) is applied in the UI layer,
// wrapping the pure domain model.
const poll = reactive(new Poll('Is this a good abstraction?'));
</script>

<template>
  <p>{{ poll.question }}</p>
  <p>Votes: {{ poll.votes }}</p>
  <button @click="poll.addVote()">Vote</button>
</template>



3.3 Pattern: Raw HTTP Objects in Domain Logic

Description: Business logic in the domain or application layer directly consumes or operates on raw HTTP request or response objects, such as those from Node's built-in http module, Express, or a library like axios.32
Why it's a Leak: This practice leaks the details of the transport protocol (HTTP) into the core logic of the application. The business logic should be concerned with what to do (e.g., "create a user"), not how the request was delivered (e.g., via a POST request with a JSON body). This coupling makes the logic difficult to trigger from other sources, such as a command-line interface, a message queue consumer, or automated tests, without constructing mock HTTP objects.34
Example & Fix:
Leaky Code:
TypeScript
// src/services/userService.ts
import { Response as ExpressResponse } from 'express';
import axios, { AxiosResponse } from 'axios';

interface UserData {
  name: string;
  email: string;
}

// This service is coupled to the structure of an AxiosResponse.
async function processUserCreationResponse(response: AxiosResponse): Promise<UserData> {
  if (response.status!== 201) {
    throw new Error('Failed to create user');
  }
  // LEAK: Depends on the specific structure of the axios response object.
  return response.data;
}

Refactored Code:
TypeScript
// src/services/userService.ts (Pure Application Logic)
interface UserData {
  name: string;
  email: string;
}

export async function createUser(userData: UserData, userApi: IUserApi): Promise<UserData> {
  // Logic is now independent of HTTP details.
  const createdUser = await userApi.createUser(userData);
  return createdUser;
}

// src/infrastructure/UserApi.ts (Adapter Layer)
import axios from 'axios';

export interface IUserApi {
  createUser(userData: UserData): Promise<UserData>;
}

export class AxiosUserApi implements IUserApi {
  async createUser(userData: UserData): Promise<UserData> {
    const response = await axios.post('/api/users', userData);
    if (response.status!== 201) {
      throw new Error(`API Error: ${response.status}`);
    }
    // The adapter is responsible for mapping the infrastructure-specific
    // response to a clean data object.
    return response.data;
  }
}



3.4 Pattern: Infrastructure Details in High-Level Components

Description: This is a severe leak where high-level application components, such as UI components or application services, contain explicit knowledge of low-level infrastructure details. This includes embedding database connection strings, credentials, or raw SQL queries directly within the component.36
Why it's a Leak: This represents a fundamental breakdown of architectural layering and Separation of Concerns.
Security: Exposing credentials like database passwords or API keys in client-side code is a critical security vulnerability.36
Brittleness: The application becomes extremely brittle. A change in the database schema (e.g., renaming a column) requires changes in the UI code.3
Reusability: The component is completely non-reusable and tied to a specific database technology and schema.
Example & Fix:
Leaky Code:
TypeScript
// src/components/UserProfile.tsx (Leaking DB details into the UI)
import React, { useEffect, useState } from 'react';
import { createConnection } from 'mysql'; // LEAK: Direct DB driver import

function UserProfile({ userId }) {
  const [user, setUser] = useState(null);

  useEffect(() => {
    // LEAK: Connection details in the UI component.
    const connection = createConnection({
      host: 'db.example.com',
      user: 'readonly_user',
      password: 'insecure_password',
      database: 'prod_db'
    });

    // LEAK: Raw SQL query in the UI component.
    const query = `SELECT name, email FROM users WHERE id = ${userId}`;
    connection.query(query, (error, results) => {
      if (results) {
        setUser(results);
      }
      connection.end();
    });
  }, [userId]);

  //... render user
}

Refactored Code:
TypeScript
// src/repositories/UserRepository.ts (Data Access Layer)
import { dbPool } from '../infrastructure/database'; // Abstracted DB connection

export async function findUserById(userId: number): Promise<{ name: string; email: string } | null> {
  const [rows] = await dbPool.query('SELECT name, email FROM users WHERE id =?', [userId]);
  return rows |



| null;
}



// src/components/UserProfile.tsx (Clean UI Component)
import React, { useEffect, useState } from 'react';
// The component depends on an abstract function, not a specific implementation.
import { findUserById } from '../repositories/UserRepository';

function UserProfile({ userId }) {
  const [user, setUser] = useState(null);

  useEffect(() => {
    // The component calls a high-level function.
    // It has no knowledge of SQL, databases, or connections.
    findUserById(userId).then(setUser);
  }, [userId]);

  //... render user
}
```



IV. Frontend-Specific Leaky Patterns

The frontend environment, with its rich ecosystem of frameworks, state management libraries, and direct interaction with browser APIs, presents a unique set of challenges for maintaining clean abstractions.

4.1 React Component Internals in Business Logic

Description: This pattern involves embedding significant business logic directly within React components, often inside event handlers like onClick or lifecycle hooks like useEffect. The logic becomes intertwined with React-specific primitives such as useState, useMemo, and dependency arrays.38
Why it's a Leak: The abstraction of a "React component" is primarily for rendering UI based on state and props. When it becomes a host for complex business rules, the concerns of presentation and logic are conflated. This leads to several problems:
Entanglement with Rendering: The developer is forced to solve business logic problems while simultaneously managing React's rendering behavior. For example, a pure calculation might need to be wrapped in useMemo to prevent performance issues, or state-dependent logic inside useEffect might suffer from stale closures, forcing the developer to use useRef or functional state updates. These are rendering concerns, not business concerns.41
Poor Testability: To test the business logic, one must render the entire component, often with a mocked DOM, and then trigger events or state changes to execute the logic. This is slow and brittle compared to testing a pure function in isolation.38
Lack of Reusability: The business logic cannot be reused outside of the React component, for instance, in a separate utility or on a backend server.
Example & Fix: A common approach to fixing this is to extract logic into custom hooks. While this improves reusability within React, a more robust solution separates the pure, framework-agnostic logic from the hook that manages the state.
Leaky Code:
TypeScript
// src/components/RegistrationForm.tsx
import React, { useState, useCallback } from 'react';

function RegistrationForm() {
  const [email, setEmail] = useState('');
  const [password, setPassword] = useState('');
  const [error, setError] = useState<string | null>(null);

  const handleSubmit = useCallback(async (e: React.FormEvent) => {
    e.preventDefault();
    setError(null);

    // LEAK: Complex business logic (validation, API interaction) inside the component.
    if (!email.includes('@')) {
      setError('Invalid email address.');
      return;
    }
    if (password.length < 8) {
      setError('Password must be at least 8 characters long.');
      return;
    }

    try {
      const response = await fetch('/api/register', {
        method: 'POST',
        body: JSON.stringify({ email, password }),
        headers: { 'Content-Type': 'application/json' },
      });
      if (!response.ok) throw new Error('Registration failed');
      // Handle success...
    } catch (err) {
      setError(err.message);
    }
  }, [email, password]);

  //... render form
}

Refactored Code:
TypeScript
// src/services/authService.ts (Pure, testable business logic)
export function validateCredentials(email, password) {
  if (!email.includes('@')) return 'Invalid email address.';
  if (password.length < 8) return 'Password must be at least 8 characters long.';
  return null;
}

// src/hooks/useRegistration.ts (Custom hook for state management)
import { useState, useCallback } from 'react';
import { validateCredentials } from '../services/authService';

export function useRegistration() {
  const [isLoading, setIsLoading] = useState(false);
  const [error, setError] = useState<string | null>(null);

  const register = useCallback(async (email, password) => {
    const validationError = validateCredentials(email, password);
    if (validationError) {
      setError(validationError);
      return;
    }

    setIsLoading(true);
    setError(null);
    try {
      //... API call logic...
    } catch (err) {
      setError(err.message);
    } finally {
      setIsLoading(false);
    }
  },);

  return { register, isLoading, error };
}

// src/components/RegistrationForm.tsx (Clean, presentation-focused component)
import React from 'react';
import { useRegistration } from '../hooks/useRegistration';

function RegistrationForm() {
  const { register, isLoading, error } = useRegistration();

  const handleSubmit = (e: React.FormEvent<HTMLFormElement>) => {
    e.preventDefault();
    const formData = new FormData(e.currentTarget);
    const email = formData.get('email') as string;
    const password = formData.get('password') as string;
    register(email, password);
  };

  //... render form, using isLoading and error state
}



4.2 Browser APIs in Reusable Modules

Description: This leak occurs when a module intended for general-purpose use contains direct calls to browser-specific global APIs, such as window, document, localStorage, or navigator.42
Why it's a Leak: The module's abstraction of being "reusable" is broken. It implicitly depends on a browser environment. This causes crashes or unpredictable behavior when the module is used in a non-browser context, most notably during Server-Side Rendering (SSR) with frameworks like Next.js, or in a standard Node.js unit testing environment.44 The consumer of the module is forced to know about this hidden environmental dependency and add checks (e.g.,
if (typeof window!== 'undefined')) to prevent errors, defeating the purpose of the abstraction.
Example & Fix:
Leaky Code:
TypeScript
// src/utils/themeStore.ts
// This module is not truly reusable because it depends on localStorage.
export function saveThemePreference(theme: 'dark' | 'light'): void {
  // LEAK: Direct call to a browser-only API.
  // This will crash during SSR or in a Node.js test.
  localStorage.setItem('theme', theme);
}

export function getThemePreference(): string | null {
  return localStorage.getItem('theme');
}

Refactored Code (using Dependency Injection):
TypeScript
// src/utils/themeStore.ts (Now framework-agnostic)
export interface IStorage {
  setItem(key: string, value: string): void;
  getItem(key: string): string | null;
}

export function saveThemePreference(storage: IStorage, theme: 'dark' | 'light'): void {
  storage.setItem('theme', theme);
}

export function getThemePreference(storage: IStorage): string | null {
  return storage.getItem('theme');
}

// Usage in a browser context (e.g., a React component)
import { saveThemePreference } from './utils/themeStore';
// We inject the concrete implementation (localStorage).
saveThemePreference(localStorage, 'dark');

// Usage in a test context (e.g., Jest)
const mockStorage = {
  setItem: jest.fn(),
  getItem: jest.fn(),
};
saveThemePreference(mockStorage, 'dark');
expect(mockStorage.setItem).toHaveBeenCalledWith('theme', 'dark');



4.3 CSS-in-JS Leaking Styling Concerns

Description: CSS-in-JS libraries (like Styled Components or Emotion) provide a powerful abstraction that solves CSS's global scope problem by allowing developers to write component-scoped styles directly in JavaScript.45 However, this abstraction can leak in several ways, primarily related to performance and complexity.47
Why it's a Leak: The abstraction is "write styles naturally alongside your components." The leaks force the developer to understand the underlying implementation to avoid performance pitfalls:
Runtime Overhead: Styles defined inside a component's render function are re-serialized into CSS strings and injected into the DOM on every render. This JavaScript execution adds runtime overhead that traditional CSS does not have.47 In React's concurrent rendering, this can be particularly slow, as each render-part can trigger style recalculations across the entire DOM.48
Bundle Size: The library itself adds to the application's JavaScript bundle size. Furthermore, if not managed carefully, dynamic style generation can lead to duplicated CSS rules, bloating the final output.47
Blurred Concerns: The line between declarative styling (CSS) and imperative logic (JavaScript) becomes blurred. Developers may start putting complex logic inside their style definitions, making the code harder to reason about and maintain.47
Example & Fix:
Leaky Code (Performance Leak):
JavaScript
// src/components/DynamicButton.tsx
import styled from 'styled-components';

function DynamicButton({ isActive }) {
  // LEAK: This 'Button' component is redefined on every render of DynamicButton.
  // The style object is recreated and re-serialized every time.
  const Button = styled.button`
    background-color: ${isActive? 'blue' : 'gray'};
    color: white;
    padding: 10px;
  `;

  return <Button>Click Me</Button>;
}

Refactored Code:
JavaScript
// src/components/DynamicButton.tsx
import styled from 'styled-components';

// Define the styled component outside the render function.
// It is created only once when the module is loaded.
const Button = styled.button`
  background-color: ${(props) => (props.active? 'blue' : 'gray')};
  color: white;
  padding: 10px;
`;

function DynamicButton({ isActive }) {
  // Pass the dynamic value as a prop.
  // The style logic runs, but the component itself is not redefined.
  return <Button active={isActive}>Click Me</Button>;
}



4.4 State Management Library Details in Components

Description: This leak occurs when UI components are directly coupled to the specific API and concepts of a global state management library like Redux, Zustand, or MobX. For example, a component might directly call dispatch({ type: 'ADD_TO_CART', payload: item }).49
Why it's a Leak: The component's responsibility is to render UI and report user intentions (e.g., "the user wants to add an item to the cart"). It should not need to know how that state is managed globally (e.g., via a Redux store, actions, and reducers). This coupling makes the component:
Hard to Refactor: If the application decides to switch from Redux to Zustand, every component that dispatches actions or selects state must be rewritten.
Less Reusable: The component cannot be easily reused in another application that uses a different state management solution or none at all.
More Complex: The component's code is cluttered with library-specific boilerplate, obscuring its primary presentation logic.51
Example & Fix: The best practice is to create a mediating abstraction, often in the form of custom hooks, that encapsulates the interaction with the state management library.
Leaky Code (Redux):
JavaScript
// src/components/AddToCartButton.tsx
import React from 'react';
import { useDispatch } from 'react-redux';

function AddToCartButton({ product }) {
  // LEAK: Direct dependency on Redux's `useDispatch` and action object structure.
  const dispatch = useDispatch();

  const handleClick = () => {
    dispatch({ type: 'cart/addProduct', payload: product });
  };

  return <button onClick={handleClick}>Add to Cart</button>;
}

Refactored Code:
JavaScript
// src/hooks/useCart.ts (Custom hook as an abstraction layer)
import { useDispatch, useSelector } from 'react-redux';
import { addProduct } from '../store/cartSlice'; // Assuming Redux Toolkit slice

export function useCart() {
  const dispatch = useDispatch();
  const items = useSelector((state) => state.cart.items);

  const addToCart = (product) => {
    // The Redux-specific logic is hidden here.
    dispatch(addProduct(product));
  };

  return { items, addToCart };
}

// src/components/AddToCartButton.tsx (Clean, decoupled component)
import React from 'react';
import { useCart } from '../hooks/useCart';

function AddToCartButton({ product }) {
  // The component interacts with a simple, domain-oriented API.
  // It doesn't know Redux exists.
  const { addToCart } = useCart();

  const handleClick = () => {
    addToCart(product);
  };

  return <button onClick={handleClick}>Add to Cart</button>;
}



V. Node.js Backend-Specific Leaky Patterns

In the backend environment, particularly with Node.js and frameworks like Express, leaky abstractions often manifest as violations of architectural boundaries between request handling, business logic, and data persistence.

5.1 Express req/res Objects in Business Logic

Description: This is one of the most common leaks in Express applications. It occurs when the request (req) and response (res) objects, which are specific to the Express framework, are passed from the controller/route handler layer into the service/business logic layer.52
Why it's a Leak: The service layer's purpose is to encapsulate framework-agnostic business logic. By accepting req and res, the service becomes tightly coupled to Express.54 This breaks the abstraction of a reusable business service for several reasons:
Framework Lock-in: The service logic cannot be used with another web framework (e.g., Fastify) or in a non-HTTP context (e.g., a background job worker, a CLI script) without significant modification.
Testing Complexity: To unit test the service, one must now create complex mock objects that accurately mimic the Express req and res APIs, including methods like status(), json(), send(), and properties like body, params, and headers.54 This is far more difficult than testing a pure function that accepts and returns simple data objects.
Violation of Concerns: The service layer becomes responsible for HTTP-related tasks, such as setting status codes or sending responses, which are the proper concern of the controller layer.
Example & Fix:
Leaky Code:
TypeScript
// src/services/UserService.ts
import { Request, Response } from 'express';
import { UserModel } from '../models/User';

// LEAK: This service is coupled to Express.
export async function createUser(req: Request, res: Response) {
  try {
    const { name, email } = req.body;
    if (!name ||!email) {
      // LEAK: Service is handling HTTP responses.
      return res.status(400).json({ error: 'Name and email are required.' });
    }
    const newUser = await UserModel.create({ name, email });
    return res.status(201).json(newUser);
  } catch (error) {
    return res.status(500).json({ error: 'Internal server error' });
  }
}

Refactored Code:
TypeScript
// src/services/UserService.ts (Framework-agnostic business logic)
import { UserModel } from '../models/User';

interface UserInput {
  name: string;
  email: string;
}

export async function createUser(userData: UserInput) {
  const { name, email } = userData;
  if (!name ||!email) {
    // Throws a domain-specific error, not an HTTP response.
    throw new Error('Name and email are required for user creation.');
  }
  const newUser = await UserModel.create({ name, email });
  return newUser;
}

// src/controllers/UserController.ts (Controller handles HTTP concerns)
import { Request, Response } from 'express';
import * as UserService from '../services/UserService';

export async function handleCreateUser(req: Request, res: Response) {
  try {
    // The controller extracts data from `req` and passes a simple object to the service.
    const user = await UserService.createUser(req.body);
    res.status(201).json(user);
  } catch (error) {
    // The controller translates domain errors into HTTP status codes.
    if (error.message.includes('required')) {
      res.status(400).json({ error: error.message });
    } else {
      res.status(500).json({ error: 'Internal server error' });
    }
  }
}



5.2 Database Driver Specifics in Service Layers

Description: This leak occurs when the service layer directly uses the API of a specific database driver (e.g., node-oracledb, pg, mongodb) or a specific ORM's query language (e.g., Sequelize's or Prisma's query builder syntax).56
Why it's a Leak: This violates the Dependency Inversion Principle, a cornerstone of clean architecture. High-level modules (services) should not depend on low-level modules (database drivers); both should depend on abstractions.10 By depending on a concrete data access implementation, the service layer:
Loses Portability: Migrating from one database (e.g., PostgreSQL) to another (e.g., MongoDB) becomes a massive undertaking, as all service logic that interacts with the database must be rewritten.
Becomes Hard to Test: Unit testing the service requires a live database connection or complex mocking of the entire database driver/ORM API. A better approach is to mock a simple repository interface.10
Mixes Concerns: The service, which should focus on business rules, becomes polluted with data access logic, such as connection management or query construction.3
Example & Fix: The solution is to introduce a Repository layer that abstracts data access.
Leaky Code:
TypeScript
// src/services/ProductService.ts
import { PrismaClient } from '@prisma/client'; // LEAK: Direct dependency on Prisma client.

const prisma = new PrismaClient();

export async function getProductDetails(productId: string) {
  // LEAK: Business logic is tied to Prisma's specific query syntax.
  const product = await prisma.product.findUnique({
    where: { id: productId },
    include: { reviews: true },
  });
  if (!product) {
    throw new Error('Product not found');
  }
  // some business logic...
  const averageRating = product.reviews.reduce((acc, r) => acc + r.rating, 0) / product.reviews.length;
  return {...product, averageRating };
}

Refactored Code:
TypeScript
// src/repositories/IProductRepository.ts (The Abstraction)
export interface IProductRepository {
  findById(productId: string): Promise<ProductWithReviews | null>;
}

// src/repositories/PrismaProductRepository.ts (The Concrete Implementation)
import { PrismaClient } from '@prisma/client';
import { IProductRepository } from './IProductRepository';

export class PrismaProductRepository implements IProductRepository {
  private prisma = new PrismaClient();

  async findById(productId: string) {
    return this.prisma.product.findUnique({
      where: { id: productId },
      include: { reviews: true },
    });
  }
}

// src/services/ProductService.ts (Clean, decoupled service)
import { IProductRepository } from '../repositories/IProductRepository';

export class ProductService {
  // Depends on the abstraction, not the implementation.
  constructor(private productRepo: IProductRepository) {}

  async getProductDetails(productId: string) {
    const product = await this.productRepo.findById(productId);
    if (!product) {
      throw new Error('Product not found');
    }
    // Pure business logic.
    const averageRating = product.reviews.reduce((acc, r) => acc + r.rating, 0) / product.reviews.length;
    return {...product, averageRating };
  }
}



5.3 File System Operations in Domain Models

Description: This pattern involves placing file system operations (using Node's fs module) directly within domain model classes.59 For example, a
UserConfiguration class might have a .save() method that calls fs.writeFileSync() to persist its state to a JSON file.
Why it's a Leak: A domain model should be a pure, in-memory representation of a business concept or entity. Its responsibility is to enforce business rules and manage its own state in memory. Persistence (how it's saved and loaded) is an external, infrastructure concern. Mixing these concerns leaks the details of the storage mechanism (the file system) into the domain model itself.61 This makes the model difficult to test without actual file I/O and impossible to persist using a different mechanism (e.g., a database) without changing the model's code.
Example & Fix:
Leaky Code:
TypeScript
// src/models/Report.ts
import * as fs from 'fs/promises';
import * as path from 'path';

export class Report {
  constructor(public title: string, public content: string) {}

  // LEAK: The domain model knows how to save itself to the file system.
  async save(): Promise<void> {
    const filePath = path.join(__dirname, '..', '..', 'reports', `${this.title}.txt`);
    await fs.writeFile(filePath, this.content);
  }
}

Refactored Code:
TypeScript
// src/models/Report.ts (Pure domain model)
export class Report {
  constructor(public title: string, public content: string) {}
  // No persistence logic here.
}

// src/repositories/FileReportRepository.ts (Repository handles persistence)
import * as fs from 'fs/promises';
import * as path from 'path';
import { Report } from '../models/Report';

export class FileReportRepository {
  private basePath = path.join(__dirname, '..', '..', 'reports');

  async save(report: Report): Promise<void> {
    const filePath = path.join(this.basePath, `${report.title}.txt`);
    await fs.writeFile(filePath, report.content);
  }
}



5.4 Third-Party API Client Details in Interfaces

Description: This leak occurs when application or service layers directly use a third-party API client (e.g., the official stripe Node.js library, or a configured axios instance for a specific service) and operate on the data structures defined by that third party.62
Why it's a Leak: The application becomes tightly coupled to the external service's API contract and the specific client library used to access it. The abstraction of "charging a credit card" is implemented as "calling the stripe.charges.create method with a Stripe.ChargeCreateParams object." This creates several problems:
Vendor Lock-in: Switching to a different payment provider (e.g., from Stripe to Braintree) requires finding and refactoring every place the Stripe client is used.
Brittleness: If the third-party API introduces a breaking change, the impact ripples throughout the application's codebase.
Testing Difficulty: Tests must mock the entire third-party client library, which can be complex and unstable.
Example & Fix: The solution is the Adapter pattern. Create an interface that defines the required operations in terms of your application's domain, and then write an adapter that implements this interface by calling the third-party client.
Leaky Code:
TypeScript
// src/services/OrderService.ts
import Stripe from 'stripe'; // LEAK: Direct dependency on the Stripe library.

const stripe = new Stripe(process.env.STRIPE_SECRET_KEY!);

export async function placeOrder(order, paymentToken) {
  //... order logic...

  // LEAK: Business logic is coupled to Stripe's specific API and data structures.
  await stripe.charges.create({
    amount: order.totalPriceInCents,
    currency: 'usd',
    source: paymentToken,
    description: `Charge for order ${order.id}`,
  });

  //... more order logic...
}

Refactored Code:
TypeScript
// src/services/IPaymentGateway.ts (The Abstraction)
export interface PaymentRequest {
  amountInCents: number;
  currency: string;
  token: string;
  description: string;
}

export interface IPaymentGateway {
  createCharge(request: PaymentRequest): Promise<{ success: boolean; transactionId: string }>;
}

// src/adapters/StripePaymentGateway.ts (The Adapter)
import Stripe from 'stripe';
import { IPaymentGateway, PaymentRequest } from '../services/IPaymentGateway';

export class StripePaymentGateway implements IPaymentGateway {
  private stripe: Stripe;

  constructor(apiKey: string) {
    this.stripe = new Stripe(apiKey);
  }

  async createCharge(request: PaymentRequest) {
    const charge = await this.stripe.charges.create({
      amount: request.amountInCents,
      currency: request.currency,
      source: request.token,
      description: request.description,
    });
    return { success: charge.status === 'succeeded', transactionId: charge.id };
  }
}

// src/services/OrderService.ts (Clean, decoupled service)
import { IPaymentGateway } from './IPaymentGateway';

export class OrderService {
  // Depends on the abstraction.
  constructor(private paymentGateway: IPaymentGateway) {}

  async placeOrder(order, paymentToken) {
    //... order logic...

    await this.paymentGateway.createCharge({
      amountInCents: order.totalPriceInCents,
      currency: 'usd',
      token: paymentToken,
      description: `Charge for order ${order.id}`,
    });

    //... more order logic...
  }
}



VI. TypeScript-Specific Abstraction Leaks

TypeScript's static type system is itself a powerful abstraction over the dynamic nature of JavaScript. It allows developers to define contracts and catch errors at compile time. However, the features of this type system—interfaces, generics, and utility types—can be misused, leading to their own unique forms of leaky abstractions.

6.1 Type Definition Leaks

Description: This category covers leaks that arise from how types are defined and exposed. The most egregious is the overuse of any, but more subtle leaks involve exposing implementation-detail types from third-party libraries in a module's public API.
Why it's a Leak:
any: Using any is a deliberate choice to opt out of TypeScript's primary abstraction: type safety. It tells the compiler to trust that the developer knows what they are doing, effectively disabling all type checks for that value. This allows errors to propagate to runtime that TypeScript was designed to prevent.64 The type
unknown is a much safer alternative; it also accepts any value, but forces the developer to perform type checks or assertions before using the value, preserving type safety.64
Exposing Library Types: When a module's public API uses types imported directly from a dependency (e.g., exporting a function that returns a Prisma.User object), it leaks the implementation detail that it is using Prisma. Consumers of this module now also have a dependency on Prisma's types. If the underlying library were ever changed, it would be a breaking change for all consumers, even if the data shape remained identical.
Example & Fix:
Leaky Code:
TypeScript
// src/repositories/UserRepository.ts
import { PrismaClient, User as PrismaUser } from '@prisma/client';

const prisma = new PrismaClient();

// LEAK: The public API of this module exposes a type from an
// infrastructure dependency (Prisma).
export async function getUser(id: string): Promise<PrismaUser | null> {
  return prisma.user.findUnique({ where: { id } });
}

Refactored Code:
TypeScript
// src/models/User.ts (Agnostic domain model)
export interface User {
  id: string;
  email: string;
  name: string | null;
}

// src/repositories/UserRepository.ts
import { PrismaClient, User as PrismaUser } from '@prisma/client';
import { User } from '../models/User'; // Depends on the clean domain model

const prisma = new PrismaClient();

// The mapping from the infrastructure type to the domain type happens inside the repository.
function toDomainUser(prismaUser: PrismaUser): User {
  return {
    id: prismaUser.id,
    email: prismaUser.email,
    name: prismaUser.name,
  };
}

// The public API is now clean and dependency-free.
export async function getUser(id: string): Promise<User | null> {
  const prismaUser = await prisma.user.findUnique({ where: { id } });
  return prismaUser? toDomainUser(prismaUser) : null;
}



6.2 Interface vs. Implementation Exposure

Description: This subtle leak relates to the choice between depending on a concrete class versus an abstract interface. In TypeScript, a class declaration creates both a value (the constructor function) at runtime and a type (the shape of an instance) at compile time.65
Why it's a Leak: When a service or component depends on a concrete class from another module, it becomes coupled to that class's entire implementation, not just its public contract. The implements keyword in TypeScript is only a compile-time check; it does not change the type of the class.66 Depending on an
interface instead enforces a dependency on the abstraction alone. This promotes the Dependency Inversion Principle and makes the system more modular and testable, as any class that satisfies the interface can be provided (e.g., a real implementation in production, a mock in tests).9
Example & Fix:
Leaky Code (Dependency on Concrete Class):
TypeScript
// src/services/NotificationService.ts
import { EmailSender } from '../infrastructure/EmailSender'; // LEAK: Depends on a concrete class.

export class NotificationService {
  private emailSender: EmailSender;

  constructor() {
    // Tightly coupled, cannot be easily swapped or mocked.
    this.emailSender = new EmailSender();
  }

  sendWelcomeEmail(email: string) {
    this.emailSender.send(email, 'Welcome!', '...');
  }
}

Refactored Code (Dependency on Interface):
TypeScript
// src/services/IEmailSender.ts (The Abstraction)
export interface IEmailSender {
  send(to: string, subject: string, body: string): Promise<void>;
}

// src/infrastructure/EmailSender.ts (The Implementation)
import { IEmailSender } from '../services/IEmailSender';
export class EmailSender implements IEmailSender {
  async send(to: string, subject: string, body: string) { /*... */ }
}

// src/services/NotificationService.ts (Decoupled Service)
import { IEmailSender } from './IEmailSender';

export class NotificationService {
  // Depends on the interface, not the concrete class.
  constructor(private emailSender: IEmailSender) {}

  sendWelcomeEmail(email: string) {
    this.emailSender.send(email, 'Welcome!', '...');
  }
}



6.3 Generic Type Parameter Pollution

Description: Generics are a tool for relating the types of multiple values, typically the inputs and outputs of a function or the members of a class.68 Generic type parameter pollution occurs when generics are used unnecessarily, adding complexity to a type signature without providing any real benefit. A key heuristic, the "Golden Rule of Generics," states that a type parameter should appear at least twice in a function's signature (excluding its declaration).69 If it appears only once, it is not relating anything and can likely be replaced with a more specific type or
unknown.
Why it's a Leak: An unnecessary generic parameter makes the function's abstract contract harder to understand.
Single-Use Parameters: A function like function log<T>(value: T): void is needlessly generic. Since T is not used to constrain another parameter or the return type, it provides no more information than function log(value: unknown): void but is more verbose.69
Return-Only Generics: A function like function parseJson<T>(json: string): T is a particularly dangerous leak. It appears to offer type safety, but it's a disguised type assertion. The function has no way of ensuring the parsed object actually conforms to T. This creates a false sense of security. The more honest abstraction is function parseJson(json: string): unknown, which forces the caller to perform an explicit and visible type assertion.69
Example & Fix:
Leaky Code:
TypeScript
// LEAK: T is only used once. It's not relating any types.
function configure<T>(options: T): void {
  console.log(options);
}

// LEAK: This is a hidden `as any`. The function can't guarantee the return type.
function getFromCache<T>(key: string): T {
  const value = cache.get(key);
  return value; // Unsafe assertion
}

Refactored Code:
TypeScript
// This is a clearer and more honest contract.
function configure(options: object): void {
  console.log(options);
}

// This forces the caller to be responsible for the type assertion.
function getFromCache(key: string): unknown {
  return cache.get(key);
}

// Caller must now perform a safe check or an explicit assertion.
const user = getFromCache('user-1') as User;



6.4 Utility Type Misuse

Description: TypeScript provides powerful utility types like Pick<T, K>, Omit<T, K>, and Partial<T> for transforming existing types.70 Misuse occurs when these are used to create anonymous, ad-hoc types directly in function signatures or component props, instead of defining explicit, named types that convey intent.71
Why it's a Leak: While convenient, this practice creates a leaky abstraction by establishing a tight, implicit coupling to the original, larger type.
Loss of Intent: A function signature like function sendEmail(user: Pick<User, 'email' | 'name'>) tells us what properties are needed, but not why. The conceptual abstraction is lost. Is this a "user summary" or a "recipient info" object? The name is missing.
Brittleness: This creates a fragile dependency. If the base User type is refactored, these anonymous Pick and Omit types can break in non-obvious ways across the codebase.
Poor Readability: Complex chains of utility types (Partial<Omit<User, 'id'>>) are hard to read and understand compared to a single, well-named type.
Example & Fix:
Leaky Code:
TypeScript
interface User {
  id: string;
  name: string;
  email: string;
  lastLogin: Date;
  isActive: boolean;
}

// LEAK: The function's contract is implicitly coupled to the full User type.
// The intent of this subset of properties is unclear.
function displayUserHeader(user: Pick<User, 'name' | 'isActive'>): JSX.Element {
  //...
}

Refactored Code:
TypeScript
interface User {
  id: string;
  name: string;
  email: string;
  lastLogin: Date;
  isActive: boolean;
}

// A new, explicit type that describes the specific data needed.
// Its name conveys the intent.
interface UserHeaderViewModel {
  name: string;
  isActive: boolean;
}

// The function now has a clear, decoupled, and self-documenting contract.
function displayUserHeader(user: UserHeaderViewModel): JSX.Element {
  //...
}



VII. Automated Detection via Static Analysis

Implementing a tool to automatically detect these leaky abstractions requires programmatic analysis of the source code. The most robust method for this is static analysis of the code's Abstract Syntax Tree (AST).

7.1 Introduction to ASTs for Code Analysis

Static analysis is the process of analyzing code without executing it.73 The core of modern static analysis tools involves a two-step process:
Lexical Analysis (Tokenization): The raw source code string is broken down into a sequence of "tokens"—the smallest meaningful units of the language, such as keywords (const), identifiers (myVar), operators (=), and literals (123).74
Syntactic Analysis (Parsing): A parser consumes this stream of tokens and constructs an Abstract Syntax Tree (AST). An AST is a hierarchical, tree-like data structure that represents the syntactic structure of the code. Each node in the tree corresponds to a construct in the code, like a function declaration, a variable assignment, or a method call.74
Once the code is represented as an AST, tools can "walk" or "traverse" this tree, visiting each node to inspect its properties and relationships, thereby analyzing the code's structure and patterns.73

7.2 Core Tooling: Babel vs. TypeScript Compiler API

While several tools can parse JavaScript into an AST, two are predominant in the ecosystem:
Babel: A highly popular and versatile JavaScript compiler. Its parser, @babel/parser, is excellent at handling the latest JavaScript syntax and experimental features. Its traversal library, @babel/traverse, provides a powerful and easy-to-use "visitor" pattern for inspecting the AST.73 Babel is ideal for syntactic analysis and code transformations.
TypeScript Compiler API: For the task of detecting leaky abstractions in a TypeScript codebase, the TypeScript Compiler API is unequivocally the superior choice. Like Babel, it can parse code into an AST. However, its crucial advantage is the Type Checker.78 The Type Checker is a component of the compiler that performs semantic analysis. After parsing, it can be queried to determine the resolved, semantic type of any node in the AST.80
This capability is the key to moving beyond simple syntactic pattern matching. For example, a syntactic tool can find all function calls named createUser. A semantic tool using the Type Checker can determine if a variable named req is actually of type Express.Request or just a coincidentally named local variable. This semantic understanding is essential for accurately detecting the majority of the leaky patterns discussed in this report.

7.3 AST Analysis Techniques for Leaky Abstraction Detection

The following are high-level algorithms for detecting several key leaky patterns using the TypeScript Compiler API and its Type Checker.

Detecting Infrastructure Dependencies in Domain/Application Logic

This technique can be used to find leaks like req/res objects in services, database driver usage, or direct third-party API client use.
Configuration: Define architectural layers, for example, by mapping folder paths to layer names (e.g., src/services/ is 'Application', src/controllers/ is 'Presentation', src/repositories/ is 'Infrastructure'). Also, define a list of known infrastructure module names (e.g., express, mongodb, @prisma/client, axios, stripe).
Initialization: Use ts.createProgram to parse the entire project. This builds the ASTs and initializes the TypeChecker.
Traversal: Traverse the AST of all files within the "Application" and "Domain" layers using ts.forEachChild.
Analysis: For each FunctionDeclaration, MethodDeclaration, or ArrowFunction that is exported or is a public class method, inspect its parameters and return type.
Type Resolution: For each parameter or return type annotation, use checker.getTypeAtLocation(node) to get its resolved ts.Type.
Symbol Inspection: From the type, get its symbol (type.getSymbol()) and from the symbol, find its declarations (symbol.getDeclarations()).
Source Detection: For each declaration, find its source file. Check if the source file's path corresponds to a module name on the predefined infrastructure list.
Reporting: If a type used in the public API of a service/domain function originates from a known infrastructure module, flag it as a leaky abstraction. For instance, if a parameter's type is resolved to the Request interface from the express module, it's a leak.

Detecting DOM Manipulation in Business Logic

Configuration: Define "business logic" locations (e.g., files matching src/services/**/*.ts or src/domain/**/*.ts).
Initialization: Create the ts.Program and get the TypeChecker.
Traversal: Traverse the ASTs of the configured business logic files.
Analysis: Look for PropertyAccessExpression nodes (e.g., document.querySelector).
Type Resolution: For the object part of the expression (the document in document.querySelector), use the TypeChecker to get its type.
Identification: Check if the resolved type is a known global DOM type, such as Document or Window. The TypeScript compiler includes default library definitions (lib.dom.d.ts) that define these types. If the symbol originates from one of these library files, it's a DOM API call.
Reporting: If a call to a DOM API is found within a business logic file, flag it as a leak.

Detecting Generic Type Parameter Pollution

Initialization: Create the ts.Program. The Type Checker is needed to resolve inferred return types.
Traversal: Traverse the AST, visiting every FunctionDeclaration, ArrowFunction, and MethodDeclaration.
Analysis: If a node has a typeParameters property, iterate through each declared type parameter (e.g., T in <T>).
Usage Counting: For each type parameter, scan the function's parameters and its type (return type annotation). Count how many times the identifier for that type parameter appears.
Crucially, if there is no explicit return type annotation, use the TypeChecker to get the inferred signature of the function call and check if the type parameter appears in the inferred return type. This is how a function like function getProperty<T, K extends keyof T>(obj: T, key: K) is correctly identified as a valid use, because its inferred return type is T[K], meaning both T and K are used again.
Reporting: If the final count for any type parameter is less than two, report it as a violation of the "Golden Rule of Generics".69

7.4 Implementation via Custom ESLint Rules

The most practical and widely adopted way to implement such automated checks is by writing custom ESLint rules.82 The
@typescript-eslint project provides the necessary infrastructure to build type-aware linting rules.
The process involves:
Setup: Configure your ESLint project to use @typescript-eslint/parser, which provides the AST and, crucially, the parser services that expose the TypeScript Program and TypeChecker to rules.
Rule Creation: A custom rule is a JavaScript module that exports a meta object (containing documentation) and a create function.82
Visitor Logic: The create function returns a "visitor" object. This object has methods whose keys are AST node types or selectors (e.g., 'CallExpression', 'FunctionDeclaration'). As ESLint traverses the AST, it calls the corresponding method for each node it encounters.83
Type-Aware Analysis: Inside a visitor method, the rule can access the context object. From this, it can get the parser services, which provide access to the TypeChecker. The rule can then implement the analysis algorithms described above.
Reporting: When a leaky pattern is detected, the rule calls context.report(), passing the AST node where the leak occurred and a descriptive message. ESLint then handles displaying this error to the user.
By packaging these detection strategies as a set of custom ESLint rules, they can be easily integrated into any standard JavaScript/TypeScript development workflow, providing real-time feedback in code editors and CI/CD pipelines.
Works cited
Leaky abstraction - Wikipedia, accessed July 3, 2025, https://en.wikipedia.org/wiki/Leaky_abstraction
Leaky Abstractions | Alex Kondov - Software Engineer, accessed July 3, 2025, https://alexkondov.com/leaky-abstractions/
Leaky Abstraction — What Is It?. Spot leakness in your code and see how… | by Bartosz Salwiczek | Better Programming - Medium, accessed July 3, 2025, https://medium.com/better-programming/leaky-abstraction-what-is-it-ed0bc84000fd
Leaky Abstraction - Khalil Stemmler, accessed July 3, 2025, https://khalilstemmler.com/wiki/leaky-abstraction/
“Leaky” Abstractions and Conway's Law: The Reality of Development - alex_ber - Medium, accessed July 3, 2025, https://alex-ber.medium.com/leaky-abstractions-and-conways-law-the-reality-of-development-59dbdfac2220
The Law of Leaky Abstractions : r/programming - Reddit, accessed July 3, 2025, https://www.reddit.com/r/programming/comments/5qzc79/the_law_of_leaky_abstractions/
abstraction isn't inherently a bad thing, it's what gives us the tools to reason... - Hacker News, accessed July 3, 2025, https://news.ycombinator.com/item?id=12477846
CommonJS vs. ES6 Modules for Beginners - Full Stack Foundations, accessed July 3, 2025, https://www.fullstackfoundations.com/blog/commonjs-vs-es6
What Are Typescript Interfaces and How Do They Work? - Strapi, accessed July 3, 2025, https://strapi.io/blog/typescript-interfaces
Breaking Free from MVC Hell: Why Your Node.js Code Needs the Service-Repository-Controller Pattern | by Mohammed Abdul Basit | Medium, accessed July 3, 2025, https://medium.com/@mohammedbasit362/breaking-free-from-mvc-hell-why-your-node-js-code-needs-the-service-repository-controller-pattern-c080725ab910
Clean Architecture in Node.js. Introduction | by Ben Mishali | Medium, accessed July 3, 2025, https://medium.com/@ben.dev.io/clean-architecture-in-node-js-39c3358d46f3
CommonJS vs. ES Modules: The Ultimate Guide to JavaScript Modularity - Alexander Parks, accessed July 3, 2025, https://byte-explorer.medium.com/lets-talk-about-what-commonjs-and-es-module-are-and-their-differences-a4de3cdb8fc6
JavaScript Module Systems Showdown: CommonJS vs AMD vs ES2015 - Auth0, accessed July 3, 2025, https://auth0.com/blog/javascript-module-systems-showdown/
CommonJS vs. ES Modules | Better Stack Community, accessed July 3, 2025, https://betterstack.com/community/guides/scaling-nodejs/commonjs-vs-esm/
What is the difference between CommonJS and ES6 modules in Node.js? - Quora, accessed July 3, 2025, https://www.quora.com/What-is-the-difference-between-CommonJS-and-ES6-modules-in-Node-js
What is the best JavaScript module bundler ? | by Rukshan Dangalla - Medium, accessed July 3, 2025, https://medium.com/@rukshandangalla/what-is-best-javascript-module-bundler-316b73049660
What does it mean global namespace would be polluted? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/8862665/what-does-it-mean-global-namespace-would-be-polluted
An Update on ES6 Modules in Node.js | by James M Snell - Medium, accessed July 3, 2025, https://medium.com/the-node-js-collection/an-update-on-es6-modules-in-node-js-42c958b890c
Using Module Bundlers in JavaScript - Web Reference, accessed July 3, 2025, https://webreference.com/javascript/advanced/module-bundlers/
Why do we use bundlers if most modern modules are ES modules? - Reddit, accessed July 3, 2025, https://www.reddit.com/r/node/comments/12pnyp6/why_do_we_use_bundlers_if_most_modern_modules_are/
JavaScript Global Namespace Pollution, accessed July 3, 2025, https://www.gnucitizen.org/blog/javascript-global-namespace-pollution/
Measuring pollution of global namespace - javascript - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/12039217/measuring-pollution-of-global-namespace
How IIFE cuts link to Global environment? - The freeCodeCamp Forum, accessed July 3, 2025, https://forum.freecodecamp.org/t/how-iife-cuts-link-to-global-environment/151851
The Complete JavaScript Module Bundlers Guide - Snipcart, accessed July 3, 2025, https://snipcart.com/blog/javascript-module-bundler
Why are css settings leaking into other components? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/68720141/why-are-css-settings-leaking-into-other-components
DOM scripting introduction - Learn web development | MDN, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Scripting/DOM_scripting
JavaScript DOM Manipulation | Blog | CodeCoda, accessed July 3, 2025, https://codecoda.com/en/blog/entry/javascript-dom-manipulation
Learn to manipulate DOM - The freeCodeCamp Forum, accessed July 3, 2025, https://forum.freecodecamp.org/t/learn-to-manipulate-dom/368901
JavaScript frameworks and libraries - Learn web development | MDN, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Learn_web_development/Core/Frameworks_libraries
JavaScript Frameworks - Statamic Docs, accessed July 3, 2025, https://statamic.dev/javascript-frameworks
2. Modeling your data, accessed July 3, 2025, https://www.js-data.io/docs/modeling-your-data
http-request - Akamai TechDocs, accessed July 3, 2025, https://techdocs.akamai.com/edgeworkers/docs/http-request
Node.js Request Object - Tutorialspoint, accessed July 3, 2025, https://www.tutorialspoint.com/nodejs/nodejs_request_object.htm
Making HTTP Requests from JavaScript (AJAX) - GitHub Pages, accessed July 3, 2025, https://drstearns.github.io/tutorials/ajax/
A Guide to JavaScript HTTP Requests - Kinsta, accessed July 3, 2025, https://kinsta.com/knowledgebase/javascript-http-request/
Connect a data source - UI Bakery Docs, accessed July 3, 2025, https://docs.uibakery.io/getting-started/connect-a-data-source
Fetching Data - App Router - Next.js, accessed July 3, 2025, https://nextjs.org/learn/dashboard-app/fetching-data
Isolating business logic from render : r/reactjs - Reddit, accessed July 3, 2025, https://www.reddit.com/r/reactjs/comments/1equ753/isolating_business_logic_from_render/
How to divide business logic in one react component? - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/64608624/how-to-divide-business-logic-in-one-react-component
Path To A Clean(er) React Architecture (Part 6) - Business Logic Separation, accessed July 3, 2025, https://profy.dev/article/react-architecture-business-logic-and-dependency-injection
What are some anti-patterns even senior developers sometimes use? : r/react - Reddit, accessed July 3, 2025, https://www.reddit.com/r/react/comments/1iq6c6k/what_are_some_antipatterns_even_senior_developers/
How to make Reusable React Components ? - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/reactjs/how-to-make-reusable-react-components/
Designing Reusable React Components | by Cory House - Medium, accessed July 3, 2025, https://medium.com/@housecor/designing-reusable-react-components-1cbeb897b048
How to build a component library with React and TypeScript - LogRocket Blog, accessed July 3, 2025, https://blog.logrocket.com/how-to-build-component-library-react-typescript/
CSS: The bad bits (and how to avoid them) - Joe Forshaw, accessed July 3, 2025, https://www.joeforshaw.com/blog/css-the-bad-bits-and-how-to-avoid-them
The Problems and Benefits of CSS-in-JS - Sparkbox, accessed July 3, 2025, https://sparkbox.com/foundry/css_in_js_overview_css_in_js_pros_and_cons
Pitfalls of CSS-in-JS: What You Need to Know - PixelFreeStudio Blog, accessed July 3, 2025, https://blog.pixelfreestudio.com/pitfalls-of-css-in-js-what-you-need-to-know/
Why We're Breaking Up with CSS-in-JS - DEV Community, accessed July 3, 2025, https://dev.to/srmagura/why-were-breaking-up-wiht-css-in-js-4g9b
Understanding State Management in React.js | by Dharshi Balasubramaniyam - Medium, accessed July 3, 2025, https://medium.com/@dharshib.8/understanding-state-management-in-react-js-e19252c6fc12
State Management in React – Hooks, Context API and Redux - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/reactjs/state-management-in-react-hooks-context-api-and-redux/
Managing State - React, accessed July 3, 2025, https://react.dev/learn/managing-state
How to extend the Express Request object in TypeScript - LogRocket Blog, accessed July 3, 2025, https://blog.logrocket.com/extend-express-request-object-typescript/
Taking A Dive Into Express' Request and Response Objects - DEV Community, accessed July 3, 2025, https://dev.to/svper563/taking-a-dive-into-express-request-and-response-objects-4988
Node Service-oriented Architecture | Codementor, accessed July 3, 2025, https://www.codementor.io/@evanbechtol/node-service-oriented-architecture-12vjt9zs9i
Understanding Middlewares in Express.js and Their Internal Working | by nishanthan-k, accessed July 3, 2025, https://nishanthan-k.medium.com/understanding-middlewares-in-express-js-and-their-internal-working-49f8752d3f47
1. Introduction to the node-oracledb Driver for Oracle Database, accessed July 3, 2025, https://node-oracledb.readthedocs.io/en/v6.4.0/user_guide/introduction.html
The NPM Libraries That Will Make Your Node.js Micro Services Faster - Medium, accessed July 3, 2025, https://medium.com/@tamartwena/improve-node-js-api-performance-in-various-application-layers-5438ce4879ad
NodeJs Layered Architecture - DEV Community, accessed July 3, 2025, https://dev.to/yaariii3/nodejs-layered-architecture-4gk7
Node.js File System - GeeksforGeeks, accessed July 3, 2025, https://www.geeksforgeeks.org/node-js/node-js-file-system/
File system | Node.js v24.3.0 Documentation, accessed July 3, 2025, https://nodejs.org/api/fs.html
Domain Driven Design for Node.js - javascript - Stack Overflow, accessed July 3, 2025, https://stackoverflow.com/questions/41788679/domain-driven-design-for-node-js
Third-party APIs - Learn web development | MDN, accessed July 3, 2025, https://developer.mozilla.org/en-US/docs/Learn_web_development/Extensions/Client-side_APIs/Third_party_APIs
Integrating Third-Party Services in Your Node.js and React Application - Techify Solutions, accessed July 3, 2025, https://techifysolutions.com/blog/integrating-third-party-services/
Handbook - Basic Types - TypeScript, accessed July 3, 2025, https://www.typescriptlang.org/docs/handbook/basic-types.html
Can you give me an example of when an interface is a better implementation than a class? : r/typescript - Reddit, accessed July 3, 2025, https://www.reddit.com/r/typescript/comments/urpj3v/can_you_give_me_an_example_of_when_an_interface/
Documentation - Classes - TypeScript, accessed July 3, 2025, https://www.typescriptlang.org/docs/handbook/2/classes.html
Typescript classes exposed as interfaces - Reddit, accessed July 3, 2025, https://www.reddit.com/r/typescript/comments/1l4q1co/typescript_classes_exposed_as_interfaces/
Generics - TypeScript: Documentation, accessed July 3, 2025, https://www.typescriptlang.org/docs/handbook/2/generics.html
The Golden Rule of Generics - Effective TypeScript, accessed July 3, 2025, https://effectivetypescript.com/2020/08/12/generics-golden-rule/
Documentation - Utility Types - TypeScript, accessed July 3, 2025, https://www.typescriptlang.org/docs/handbook/utility-types.html
What TypeScript practices are actually causing you pain on a day to day basis? What should people do differently? - Reddit, accessed July 3, 2025, https://www.reddit.com/r/typescript/comments/1e56jrj/what_typescript_practices_are_actually_causing/
Typescript utility types that you must know - DEV Community, accessed July 3, 2025, https://dev.to/arafat4693/typescript-utility-types-that-you-must-know-4m6k
babel-plugin-handbook/README.md at master - GitHub, accessed July 3, 2025, https://github.com/kentcdodds/babel-plugin-handbook/blob/master/README.md
Abstract Syntax Tree (AST) - Explained in Plain English - DEV Community, accessed July 3, 2025, https://dev.to/balapriya/abstract-syntax-tree-ast-explained-in-plain-english-1h38
Abstract Syntax Tree In TypeScript - DEV Community, accessed July 3, 2025, https://dev.to/bilelsalemdev/abstract-syntax-tree-in-typescript-25ap
abstract-syntax-tree - NPM, accessed July 3, 2025, https://www.npmjs.com/package/abstract-syntax-tree
babel/traverse, accessed July 3, 2025, https://babeljs.io/docs/babel-traverse
TypeScript Compiler API Book, accessed July 3, 2025, https://typescriptcompilerapi.com/
Enhanced AST Static Analysis with Typescript Language Server - GitNation, accessed July 3, 2025, https://gitnation.com/contents/enhanced-ast-static-analysis-with-typescript-language-server
Detecting UI components with TypeScript Compiler API - François Wouts, accessed July 3, 2025, https://fwouts.com/articles/previewjs-detecting-components
no-unnecessary-type-parameters - typescript-eslint, accessed July 3, 2025, https://typescript-eslint.io/rules/no-unnecessary-type-parameters/
Custom Rules - ESLint - Pluggable JavaScript Linter, accessed July 3, 2025, https://eslint.org/docs/latest/extend/custom-rules
Selectors - ESLint - Pluggable JavaScript Linter, accessed July 3, 2025, https://eslint.org/docs/latest/extend/selectors
