## Spec documents

Project documentation lives in `spec/`, in this repository. Do not add the old numbered decision directories or the old log directories. `spec/` holds product requirements and behavior specifications that mirror the source tree: page and component UI/UX, API behavior, business rules, interface contracts, permission constraints, known limitations and non-goals.

A source change and the spec change it needs go in the same commit.

These are specifications, not plans. They describe, in the present tense, how the system behaves now. Intentions, roadmaps and options still being weighed don't belong here: a spec that describes something not yet built is itself a bug in that spec.

### Read the spec with the source

**Whenever you open a source file, open its spec too.** This is not limited to making changes: it applies just as much to reading code to answer a question, tracking down a bug, or deciding whether something should change. The source says what the code does; the spec says why, what was deliberately left out, and which constraints are load-bearing. Reading only one leads to confident, wrong conclusions: "simplifying away" a workaround as redundant, or "fixing" behavior that is intentional.

When a source file has no spec, say so explicitly; don't assume it has no context. A missing spec is a gap to fill, not evidence that the file explains itself.

### Layout

- The spec tree mirrors the source tree's relative paths: `crates/op-core/src/tile.rs` is specified in `spec/crates/op-core/src/tile.md`.
- A file's spec replaces the source extension with `.md`: `src/foo/bar.ts` → `spec/src/foo/bar.md`.
- Pages and API routes follow their source entry points: `page.tsx` → `page.md`, `route.ts` → `route.md`.
- Component specs mirror the component's source path too, e.g. `src/components/foo/bar.tsx` → `spec/src/components/foo/bar.md`.
- A capability, experience guideline or system rule shared by several files goes in the `README.md` of the nearest source directory.
- Specs for repository-wide or root files go in the root of `spec/`, mirroring the root path, e.g. `.prettierrc.mjs` → `spec/.prettierrc.md`.

### Writing rules

- Specs are written in English: titles, body, product wording, interaction descriptions, interface logic and limitations alike.
- Before changing a page, component, API, product wording, business rule or user flow, read the spec at the matching source path.
- Requirements, interactions, interface logic, business rules and security constraints settled in discussion (with an AI or the team) go into the most relevant mirrored spec.
- When a source change alters behavior, an interface contract or design intent, update the spec in the same working session. Source and spec must never contradict each other: if a change invalidates an existing statement, correct it in place instead of leaving stale content.
- When a source file is moved or split, its spec moves or splits in the same commit. A spec whose mirrored path no longer matches is worse than no spec: an agent looks it up by the path convention, doesn't find it, and concludes there is no context.
- A frontend `page.md` must cover: what the page is for, user flows, information architecture, main interactions, filtering/sorting/pagination rules, loading/empty/error states, permission states, responsive behavior, copy, accessibility and key UI/UX constraints.
- A frontend component `.md` must cover: the component's responsibility, where it's used, props/data inputs, visual states, interaction states, keyboard/hover/focus/disabled/loading behavior, edge cases, and how it composes with pages or other components.
- A backend `route.md` must cover: what the API is for, request/response, authentication and tenant boundaries, parameter validation, the core processing flow, database reads and writes, external service calls, side effects, error codes, expiry/idempotency/concurrency rules, security constraints and edge cases.
- When a feature spans frontend and backend, frontend interaction and page/component UI/UX go in the frontend spec; the backend API, data flow, detailed logic and security constraints go in the API or service spec.
- Don't use `Status / Decision / Alternatives / Consequences` as a spec's main template. Historical content goes in a "Previous approaches", "Deprecated wording" or "Change log" section, kept for what it explains about the current requirements.

## Git

- Commit messages are written in English: the subject line, the body and any notes alike, even when the conversation that produced the change was in another language. Product and UI names keep Photoshop's English wording (`Edit › Fade`, `Selection Brush`).
- The subject says what changed in the product or code, in a short line without a trailing period; the body, when needed, explains why and what was measured or verified.
- Branch names and tag names are English too.
