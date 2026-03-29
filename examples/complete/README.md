# Plan: Create Sub-Agent Prompt for `complete` Architecture Design

## Context

We need to create a new example (`complete`) in the `examples/` directory that demonstrates ALL features of both SpacetimeDB Rust Server Modules and SpacetimeDSL. The example is a simple 2D tile-based game where players move on a grid and chat. Two fully compilable versions (vanilla SpacetimeDB and SpacetimeDB + SpacetimeDSL) will be produced side-by-side to showcase SpacetimeDSL's benefits.

Before writing any code, an architect sub-agent must first design the macro architecture in plain English. This plan describes how to craft the prompt for that architect agent.

## User Decisions Summary

| Decision                | Choice                                                                                |
| ----------------------- | ------------------------------------------------------------------------------------- |
| Project name            | `complete`                                                                            |
| Scope                   | Server-only (no client)                                                               |
| Two versions            | Both fully compilable                                                                 |
| Directory structure     | Sibling dirs: `examples/complete/vanilla/` and `examples/complete/dsl/`               |
| Module layout           | Multi-module: one table per file, reducers/procedures/views/helpers in separate files |
| Feature coverage        | ALL features of SpacetimeDB (except Direct indexes) and ALL features of SpacetimeDSL  |
| Game loop               | Request-based (players call reducers to move/act)                                     |
| Grid model              | Config-bounded implicit (Config table for bounds, no Tile table)                      |
| Entity types            | Players + items + NPCs + structures                                                   |
| Inventory               | Yes, with inventory table                                                             |
| Views                   | Architect decides                                                                     |
| Custom types            | Rich type system (enums + structs)                                                    |
| Vanilla rigor           | Full manual validation (implement all checks DSL does automatically)                  |
| Event tables            | Yes                                                                                   |
| Index types             | B-tree only (no Direct)                                                               |
| Auth patterns           | Full (OIDC claims, admin auth, service-to-service)                                    |
| HTTP calls (procedure)  | Placeholder URL                                                                       |
| Binary data             | Architect decides approach                                                            |
| Multi-table struct      | Yes (all features)                                                                    |
| Self-referencing tables | Yes                                                                                   |
| Schema migration        | No                                                                                    |
| FK strategies           | All 4 (Delete, Error, SetZero, Ignore) — architect maps to game concepts              |
| Hooks                   | All 6 types                                                                           |
| RNG                     | Yes, for gameplay (item drops, NPC behavior)                                          |
| Comparison script       | Just mention it exists (out of scope)                                                 |
| `Option<T>` specifics   | Implementation decides                                                                |
| Deliverable format      | Structured markdown document                                                          |
| Coverage checklist      | Mandatory — feature-to-game-concept mapping                                           |

## Deliverable

A single file: `examples/complete/ARCHITECTURE_PROMPT.md`

This file is a self-contained sub-agent prompt following the best practices in `docs/ai/SUB_AGENT_PROMPTS.md` and `docs/ai/WRITING_GUIDELINES.md`. It instructs an architect agent to produce a structured markdown document describing the macro architecture of `complete` in plain English.

## Prompt Structure (following SUB_AGENT_PROMPTS.md)

The prompt will have these sections:

### 1. Identity

- Role: Game Architecture Designer for SpacetimeDB + SpacetimeDSL example project

### 2. Task

- Design the macro architecture for a 2D tile-based multiplayer game example
- Produce a structured markdown document (not code)

### 3. Context

- Reference docs: `examples/complete/docs/spacetimedb/Summary.md`, `examples/complete/docs/spacetimedsl/Summary.md`
- Existing examples for patterns: `examples/blackholio/`, `examples/test/`
- Chat App tutorial: `docs/spacetimedb/05-tutorials-chat-app-.md`

### 4. Scope

In-scope and out-of-scope clearly defined per Section 4.4 of SUB_AGENT_PROMPTS.md

### 5. Constraints

- All user decisions from the table above encoded as hard constraints
- Feature coverage checklists embedded (from `examples/complete/docs/spacetimedb/Features.md` and `examples/complete/docs/spacetimedsl/Features.md`)

### 6. Output Format

Structured markdown with these sections:

1. **Game Overview** — 2-3 paragraph description of the game
2. **Custom Types** — all SpacetimeType enums and structs with variant/field descriptions
3. **Tables** — for each table: name, purpose, columns (name + type + constraints), FK relationships, hooks, DSL config
4. **Reducers** — for each: name, purpose, parameters, logic summary
5. **Lifecycle Reducers** — init, client_connected, client_disconnected
6. **Procedures** — for each: name, purpose, HTTP call details
7. **Views** — for each: name, ViewContext vs AnonymousViewContext, return type, query logic
8. **Scheduled Tables** — for each: name, interval/time, reducer, purpose
9. **Event Tables** — for each: name, purpose, columns
10. **File Layout** — exact file-per-table/reducer/view/procedure/helper mapping for both vanilla/ and dsl/
11. **FK Relationship Map** — table of all FK links with on_delete strategy and justification
12. **Feature Coverage Checklist** — the full SpacetimeDB + SpacetimeDSL checklists with game concept filled in for each item

### 7. Decision Criteria (per Section 2.4)

Explicit criteria for the architect's design choices:

- Every SpacetimeDB feature must map to at least one game concept
- Every SpacetimeDSL feature must map to at least one game concept
- FK strategies must feel justified (not arbitrary)
- Game mechanics should be simple enough to understand but rich enough to cover all features

### 8. Error Handling (per Section 3.3)

Instructions for what to do if a feature cannot be naturally covered

### 9. Validation Criteria (per Section 3.4)

Self-check before returning: all checklist items filled, no orphan tables, no missing FK pairs

## Critical Reference Files

| File                                              | Purpose                                            |
| ------------------------------------------------- | -------------------------------------------------- |
| `examples/complete/docs/spacetimedb/Summary.md`   | All SpacetimeDB features the architect must cover  |
| `examples/complete/docs/spacetimedsl/Summary.md`  | All SpacetimeDSL features the architect must cover |
| `docs/ai/SUB_AGENT_PROMPTS.md`                    | Best practices for structuring the prompt          |
| `docs/ai/WRITING_GUIDELINES.md`                   | Writing guidelines for instruction quality         |
| `examples/complete/docs/spacetimedb/Features.md`  | Extracted feature checklist                        |
| `examples/complete/docs/spacetimedsl/Features.md` | Extracted feature checklist                        |
| `examples/blackholio/src/lib.rs`                  | Existing game example for patterns                 |
| `examples/test/src/lib.rs`                        | Existing DSL feature test for patterns             |
| `docs/spacetimedb/05-tutorials-chat-app-.md`      | Chat App tutorial to build upon                    |

## Implementation Steps

1. Read the two feature checklist files (`examples/complete/docs/spacetimedb/Features.md`, `examples/complete/docs/spacetimedsl/Features.md`)
2. Read `docs/ai/SUB_AGENT_PROMPTS.md` and `docs/ai/WRITING_GUIDELINES.md` for prompt structure patterns
3. Compose the prompt following the structure above
4. Embed both feature checklists in full as mandatory coverage requirements
5. Write the result to `examples/complete/ARCHITECTURE_PROMPT.md`

## Verification

After writing the prompt, verify:

- [ ] All 137 SpacetimeDB features from the checklist are included
- [ ] All 71 SpacetimeDSL features from the checklist are included
- [ ] All user decisions from the summary table are encoded as constraints
- [ ] The prompt follows SUB_AGENT_PROMPTS.md structure (headers, delimiters, instructions before data, decision criteria, error handling, validation)
- [ ] The prompt follows WRITING_GUIDELINES.md pillars (single-tasked, specific, short, surrounded with context, validated upfront)
- [ ] Output format is fully specified with section structure
- [ ] The comparison script is mentioned as an out-of-scope future tool
- [ ] No Direct indexes are requested
- [ ] No schema migration patterns are requested
