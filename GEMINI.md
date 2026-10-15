# X-Route — Gemini Instructions

## 1. Project Overview

X-Route is an existing native desktop application built with:

- Tauri 2
- React 19
- TypeScript
- Vite
- Zustand
- TanStack Query
- Lucide React
- Prettier
- Oxlint
- Rust

The application interacts with Windows system networking and DNS configuration.

The project contains both:

- React/TypeScript frontend
- Tauri/Rust backend

Treat both parts as production code.

---

## 2. Most Important Rule

**Never change user-visible behavior unless explicitly requested.**

Before and after any refactoring, preserve:

- UI appearance
- UI states
- existing buttons and interactions
- DNS enable/disable behavior
- DNS provider selection
- DNS configuration behavior
- Windows system interactions
- error behavior
- existing settings
- existing user workflows

Refactoring must primarily improve:

- architecture
- maintainability
- readability
- separation of concerns
- testability
- type safety
- dependency boundaries

Do not use refactoring as an excuse to redesign the product.

---

## 3. Always Understand Existing Code First

Before modifying non-trivial code:

1. Inspect the relevant files.
2. Trace the actual execution flow.
3. Identify dependencies.
4. Understand where state originates and where it is consumed.
5. Understand frontend → Tauri → Rust → Windows interactions.
6. Only then decide how the code should be changed.

**Do not infer behavior from filenames, function names, or folder names alone.**

Read the implementation.

When working on an existing feature, prefer the smallest architectural change that solves the actual problem.

---

## 4. React Architecture

The React frontend follows Feature-Sliced Design.

Preferred layers:

```text
app
pages
widgets
features
entities
shared
```

Use FSD based on responsibility, not mechanically.

### Layer responsibilities

- `app` — application initialization, providers, global configuration
- `pages` — page composition
- `widgets` — large independent UI blocks
- `features` — user actions and application scenarios
- `entities` — domain entities and domain-related logic
- `shared` — reusable UI, utilities, infrastructure, configuration and types

Respect FSD dependency direction.

A lower layer must not depend on a higher layer.

Do not create a new FSD layer, slice, segment or directory merely to make the folder structure look "clean".

---

## 5. Separation of Concerns

Maintain clear boundaries between:

```text
UI
↓
Application / Feature Logic
↓
Domain
↓
Infrastructure
↓
Tauri / Rust
↓
Windows
```

React components should primarily handle:

- rendering
- user interaction
- composition

Application logic should handle:

- user scenarios
- orchestration
- state transitions

Domain code should contain:

- domain models
- domain types
- domain rules

Infrastructure should handle:

- Tauri API
- Rust commands
- Windows APIs
- system configuration
- persistence
- filesystem
- external APIs

React UI must not contain direct knowledge of Windows or Rust implementation details.

---

## 6. DNS and System Operations

DNS functionality is system-critical.

Treat changes to DNS behavior as high-risk.

Before modifying DNS-related code, determine:

- how the current DNS state is obtained
- where enabled/disabled state comes from
- how DNS providers are represented
- how DNS servers are selected
- how React communicates with Tauri
- how Tauri communicates with Rust
- how Rust modifies Windows configuration
- how automatic DNS is restored
- how errors propagate

Do not rewrite DNS/system logic without understanding the complete execution path.

Prefer explicit, typed boundaries between frontend and system infrastructure.

---

## 7. State Management

Do not automatically put state into Zustand.

For every piece of state, determine whether it is:

- local component state
- UI state
- domain state
- application state
- system state
- server/external state

Use the smallest appropriate scope.

Avoid duplicated sources of truth.

Do not introduce Zustand or TanStack Query into code where they provide no meaningful benefit.

---

## 8. TypeScript

Prefer clear, precise and maintainable TypeScript.

Use:

- discriminated unions where useful
- `unknown` instead of unsafe `any`
- meaningful domain types
- explicit API boundaries
- readonly types where they improve correctness

Avoid:

- unnecessary generic abstractions
- excessive type gymnastics
- duplicated types
- meaningless interfaces
- `any` used merely to silence TypeScript

Do not add type complexity that makes otherwise simple code harder to understand.

---

## 9. React Components

Components should have clear responsibilities.

When a component becomes too complex, consider extracting:

- presentational components
- custom hooks
- feature logic
- domain logic
- infrastructure calls

However:

**Do not split components simply because they are large.**

Extraction should improve cohesion and readability.

Avoid creating dozens of tiny components with no meaningful responsibility.

---

## 10. Error Handling

Errors must be handled at the appropriate architectural level.

Do not:

- silently swallow errors
- use broad `try/catch` blocks without a reason
- mix infrastructure errors directly into UI code
- duplicate error transformation logic

Errors crossing architectural boundaries should be translated into appropriate typed representations.

User-facing errors should remain understandable.

---

## 11. Tauri / Rust

Keep the boundary between React and Rust explicit.

Preferred conceptual flow:

```text
React
  ↓
Application / Feature Logic
  ↓
Infrastructure Adapter
  ↓
Tauri Command
  ↓
Rust
  ↓
Windows
```

Do not allow UI components to become tightly coupled to Rust command implementation details.

When refactoring Rust:

- preserve command behavior
- preserve serialization contracts
- preserve error semantics
- preserve Windows behavior
- avoid unnecessary abstractions

Do not apply FSD to Rust.

Use an architecture appropriate for Rust instead.

---

## 12. Engineering Principles

Prefer:

- SOLID
- DRY
- KISS
- YAGNI
- separation of concerns
- dependency inversion
- composition over inheritance
- explicit dependencies
- high cohesion
- low coupling
- predictable state management

But do not apply principles mechanically.

A simple implementation is preferable to an elaborate abstraction when both solve the problem equally well.

---

## 13. Avoid Overengineering

Do NOT introduce:

- dependency injection frameworks
- unnecessary libraries
- service → service → service chains
- factories with a single implementation
- classes where functions are sufficient
- abstractions over one trivial function
- unnecessary design patterns
- excessive file splitting

Before introducing an abstraction, identify the concrete problem it solves.

If there is no concrete problem, do not introduce it.

---

## 14. Dependencies and Technology

Do not replace existing technologies merely because another technology is preferred.

Do not add dependencies unless there is a concrete reason.

Prefer existing project dependencies and established project patterns.

Before adding a library, verify whether the existing stack can solve the problem adequately.

---

## 15. Refactoring Strategy

For large refactors:

1. Understand the current implementation.
2. Identify architectural problems.
3. Define the target boundary.
4. Make small coherent changes.
5. Keep behavior unchanged.
6. Validate after significant changes.
7. Continue only after the previous step remains correct.

Avoid massive rewrites when incremental refactoring is possible.

Do not refactor unrelated code simply because it is nearby.

---

## 16. Validation

After significant changes, verify as much as the project tooling allows:

```text
TypeScript
Lint
Build
Tauri build/run
```

For system functionality, additionally verify:

```text
DNS enable
DNS disable
Automatic DNS restoration
DNS provider selection
Windows system interaction
Error handling
```

If a check cannot be performed, explicitly state that it could not be verified.

Never claim that something was tested if it was not actually tested.

---

## 17. Git Safety

Do not:

- reset user changes
- discard uncommitted work
- overwrite unrelated modifications
- modify files unrelated to the current task

Assume the working tree may contain intentional user changes.

Before large modifications, inspect the current Git state when relevant.

---

## 18. Communication

When analyzing a problem:

1. Explain what the existing code actually does.
2. Identify the concrete architectural problem.
3. Explain why it is a problem.
4. Propose the simplest appropriate solution.
5. Then implement it when instructed or when the current task explicitly authorizes implementation.

Do not invent architecture without inspecting the existing code.

Do not claim that a refactoring is correct merely because it matches a theoretical pattern.

The existing behavior is the source of truth.

---

## 19. Priority Order

When architectural goals conflict, use this priority:

1. Preserve existing user behavior.
2. Preserve correctness and system functionality.
3. Preserve existing public contracts.
4. Maintain clear architectural boundaries.
5. Improve maintainability.
6. Improve readability.
7. Improve testability.
8. Apply architectural patterns where they provide real value.

**Correct and understandable code is more important than formal architectural purity.**
