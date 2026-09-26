# Guidelines for AI Coding Agents

For docs how to use SpacetimeDSL and SpacetimeDB together, see [`DOCUMENTATION.md`](docs/DOCUMENTATION.md).

## Methodologies

### Self-Documenting Code

Code must be self-documenting through clear naming:

- **No abbreviations**: Use `InputOutput` not `Io`, `FileSystemWatcher` not `Watcher`, `DirectoryWalker` not `Walk`.
- **Descriptive identifiers**: Names should convey meaning without requiring comments.
- **No redundant comments**: Never document "how" — the code shows that. Only document "what" and "why" when not obvious from the code itself.
- **Remove comments that repeat the code**: A comment like `/// IO error.` above `Io(io::Error)` adds no value.
- **Don't document the past**: A comment states what the code does, never how it came to look that way. Drop "now fixed", "this used to panic", "the plan moved this here", etc. and every reference to the task, issue or review behind a change — git holds that history, and a comment repeating it goes stale the moment the next change lands. A leftover `todo!("Task 8 fills this in")` is worse than no comment at all: it names a step no reader of the code can look up.

### Test Driven Development

Red and green are **observations, not intentions**. A step is red once its failure has been read, and green once a gate has printed its success marker. Assuming either state is how a change lands broken.

#### Never invoke `cargo` directly

Building a workspace member on its own fails to link against **SpacetimeDB**. `x.ps1` is the only supported entry point. A linker error is a sign that a raw `cargo` command was used, not a problem to investigate.

#### The two gates

`.\x.ps1 unit-test` runs the snapshot harness (`derive`) and the diagnostics harness (`compile-tests`). It reports its own result honestly, so filter it rather than reading it whole:

```powershell
.\x.ps1 unit-test 2>&1 | Select-String -Pattern "test result:|FAILED|^error|^warning: " | Select-Object -First 20
```

`.\x.ps1 test` publishes the example modules to the local server and runs the `tester` reducer. **Its exit code is meaningless** — the script runs each `spacetime` command without checking the result and always exits 0. The reducer's success marker is the only signal:

```powershell
$output = .\x.ps1 test 2>&1 | Out-String
if ($output | Select-String -Pattern "Test executed successfully" -Quiet) {
    "MARKER FOUND"
} else {
    "MARKER ABSENT - relevant output:"
    $output -split "`n" | Select-String -Pattern "^error|-->|panic|should" | Select-Object -First 30
}
```

Finding the marker is enough. Only when it is absent does the output need reading, and then only the lines that carry a diagnostic.

#### Which of the three test kinds to write

Start from what the change does to the DSL's input:

| The change …                                     | Test kind                              |
| ------------------------------------------------ | -------------------------------------- |
| rejects an input                                 | Diagnostics — `compile-tests/tests/ui` |
| changes what is generated for an accepted input  | Snapshot — `derive/tests/fixtures`     |
| changes what the generated code does at run time | Runtime — `examples/test/src`          |

Most generator work needs two of them and a whole feature needs all three, because each kind is blind to what the next one sees. Write the cheapest kind that can fail for the reason under test, then add the kinds its blind spots require — never substitute a cheap one for a blind spot.

**Diagnostics — `compile-tests/tests/ui/<case>.rs` + `<case>.stderr`**

Every input the DSL rejects. The pair pins the message *and* the span it underlines, so a message that points at the wrong token is a failing test rather than a silent regression.

- `trybuild` globs the directory — a new file needs no registration.
- One rejection per file, named after the rejection.
- A fixture is valid except for the one rejection it pins: applying the fix its diagnostic suggests leaves a program that compiles. Write the companions that fix relies on — a `#[spacetimedsl::hook]` function, an `impl DefaultSingleton`, the module of the table on the other side of a foreign key — into the fixture up front.
- Leave out only what would be a rejection of its own before the fix, such as the marker column a table gains together with `method(soft_delete = true)`. That belongs to the fix; writing it up front would hide a second rejection behind the first.
- Anything beyond the fix's companions stays out. It pulls in unrelated rejections that fire first and mask the one being pinned.
- A rejected `#[dsl]` emits the error and nothing else, so every companion or neighbouring table naming its struct or a generated item fails as well. Those follow-on errors stay in the `.stderr`, and the fixture's `//!` comment names where they come from in one sentence.
- When one rule has several shapes, each shape gets its own file and each message names everything its fix removes. `soft_delete_method_on_singleton`, `soft_delete_method_with_marker_column_on_singleton` and `marker_column_on_singleton` are one rule in three shapes.
- Blind spot: says nothing whatsoever about input the DSL accepts.

**Snapshot — `derive/tests/fixtures/<fixture>.rs` → `derive/tests/snapshots/<fixture>/<Struct>/*.snap`**

What the macro emits for input it accepts. Register the fixture with a test in `derive/src/characterization_tests.rs`:

```rust
#[test]
fn soft_delete_flag() {
    snapshot_fixture("soft_delete_flag");
}
```

- Blind spot: the harness diffs token streams and never feeds them to a compiler. Generated code can be snapshot-green and not build.
- Blind spot: an accepted snapshot is only as correct as the reading that accepted it. Green afterwards means *unchanged*, not *right* — which is why the `git diff` is the real test and rubber-stamping it defeats the whole corpus.
- Name a fixture after the one shape it pins. Let it carry a second shape only when the subject needs both at once — a cascade fixture covering both marker shapes is honest, because the cascade needs a referenced table and a referencing one anyway.

**Runtime — `examples/test/src`**

The only gate that compiles, links and runs generated code against a real **SpacetimeDB**. Put tables which belong together in a file of their own, with their hooks and a `pub(crate) fn run_tests` holding their assertions, and call it from the `tester` reducer in `lib.rs`. Return `Err(String)` naming what should have happened.

- Use it for what no token stream can show: a value actually written, a cascade actually reaching a row, an operation actually being idempotent.
- Blind spot: it is one **SpacetimeDB** module, so table and accessor names are global across all of its files and collide.
- Blind spot: the assertions sit in the module of their tables, where private items are in reach, so they still compile when a generated item they call has lost its `pub`. Only the snapshots pin that visibility.
- Blind spot: a failure points at a reducer line, not at the generator that caused it.

When this gate catches something the other two could not, add the missing cheap test in the same commit. A `?` inside a generated cascade compiled fine as tokens and failed only here; the fixture corpus had no case pairing that marker shape with a cascade, so one was added rather than leaving the next regression to the slowest gate.

#### A test that cannot run is not a test

Before trusting a new test, confirm the harness executes it. A `#[cfg(test)] mod tests` in the root crate, for example, is never run by `.\x.ps1 unit-test` — it would pass by never executing. Prefer an observable the existing harnesses already watch: a snapshot, a `.stderr` file, or an assertion in the `tester` reducer.

#### Read the failure, not just the fact of it

Red has to fail for the reason under test. A new `compile-tests/tests/ui` case that fails because a keyword is unknown is red for the right reason; one that fails because the fixture is malformed is not, and it will go green for the wrong one. Read the failure text before writing the implementation, not just the word `FAILED`.

#### Regenerating the recorded output

Snapshots and diagnostics have separate switches, and neither affects the other. A change that moves both regenerates both:

```powershell
$env:INSTA_FORCE_UPDATE = "1"
.\x.ps1 unit-test
$env:INSTA_FORCE_UPDATE = $null
git diff derive/tests/snapshots
```

```powershell
$env:TRYBUILD = "overwrite"
.\x.ps1 unit-test
$env:TRYBUILD = $null
git diff compile-tests/tests/ui
```

Accepting `*.snap.new` files one batch at a time costs a whole harness run per moved snapshot, because `insta` reports only the first failing assertion per test function. `INSTA_FORCE_UPDATE` writes every snapshot in place in one run and drops `insta`'s scratch `assertion_line:` metadata by itself.

**Read the `git diff` before committing it.** A recorded output is the only record of what the generator emits, and the diff shows exactly what moved against the last commit. Revert anything unexpected with `git checkout -- <path>` rather than committing it.

#### Green includes the formatter

`.\x.ps1 format` runs `cargo fmt` and `clippy --fix`. Anything it rewrites is a finding to review and commit, not a pass. A task is done when a second run changes nothing.

## Programming Principles

### Principle Checklists

Use the following bullet points as checklist when planning, reviewing or implementing code changes.

#### Scope & Goal Discipline

- [ ] Identify essential outcome before touching code.
- [ ] Remove nonessential requirements, defer speculative work.
- [ ] Work limited to current user story.
- [ ] Future ideas captured outside codebase.
- [ ] Latest change solves current requirement only.

#### Simplicity & Right-Sized Solutions

- [ ] Prefer straightforward data flows over clever abstractions.
- [ ] Review changes for simpler alternatives prior merge.
- [ ] Defer abstractions until duplication appears.
- [ ] Prefer linear solution before optimizing structures.
- [ ] Explain why added complexity was unavoidable.
- [ ] Delay abstractions until duplication patterns stay consistent.

#### Documentation & Communication Clarity

- [ ] Document validation steps before implementation begins.
- [ ] Document concern boundaries and integration contracts.
- [ ] Document intent so future maintainers understand choices.
- [ ] Prefer clarity over cleverness in code and comments.
- [ ] Apply Principle of Least Astonishment in interfaces.
- [ ] Document extension points so the framework knows when to call custom logic.
- [ ] Log blockers to future cleanups for retrospectives.

#### Refactoring & Change Containment

- [ ] Unused scaffolding removed immediately.
- [ ] Refactoring kept change radius shallow.
- [ ] Refactor immediately once simplest version works.
- [ ] Leave touched files cleaner than before commit.
- [ ] Fix small clarity issues during adjacent work.
- [ ] Stop when improvements risk derailing feature.

#### Testing & Verification

- [ ] Added tests justify each new branch.
- [ ] Reference user-impacting behaviors with guardrails or tests.
- [ ] Add tests before refactoring or when missing.
- [ ] Add tests for coupling regressions immediately.
- [ ] Keep unit tests running in milliseconds.
- [ ] Ensure each test has no shared state.
- [ ] Stabilize tests across environments and runs.
- [ ] Make assertions binary without manual inspection.
- [ ] Write tests before production code.
- [ ] Arrange inputs and environment first.
- [ ] Act with one focused call.
- [ ] Assert clear outcomes and side effects.

#### Modular Boundaries & Separation

- [ ] Split work into minimally overlapping modules.
- [ ] User interface changes shouldn't touch domain logic.
- [ ] Shared utilities expose APIs, not internal details.
- [ ] Keep unrelated responsibilities decoupled across modules.
- [ ] One domain change updates just one system surface.
- [ ] Design APIs without cross-cutting side effects.
- [ ] Prefer composable primitives over special-case hooks.
- [ ] Localize domain changes to the smallest module set.
- [ ] Avoid edits that force neighboring modules to change.
- [ ] Keep persistence, formatting, business logic in separate classes.

#### Coupling Awareness & Dependency Constraints

- [ ] Evaluate coupling via strength, locality, degree axes.
- [ ] Prefer weaker connascence forms when refactoring dependencies.
- [ ] Keep high-degree dependencies co-located or abstracted.
- [ ] Discuss coupling issues using shared connascence vocabulary.
- [ ] Reduce remote connascence before crossing service boundaries.
- [ ] Limit cross-module dependencies before coding integrations.
- [ ] Remove shared global state unless absolutely justified.
- [ ] Keep method calls within Law of Demeter bounds.
- [ ] Review change blast radius for every edit.
- [ ] Anticipate maintenance; avoid hidden coupling or magic.

#### Duplication Control & Reuse

- [ ] One authoritative source for each business rule.
- [ ] Abstract repeated logic instead of copy/paste fixes.
- [ ] Sync related artifacts—code, docs, tests—whenever knowledge changes.
- [ ] Remove duplication without coupling unrelated responsibilities.

#### Dependency & Interface Management

- [ ] Decide early which responsibilities belong to the framework versus custom modules.
- [ ] Route orchestration through containers, factories, or callbacks instead of ad-hoc callers.
- [ ] Depend on contracts/interfaces so modules stay decoupled from specific implementations.
- [ ] Provide dependencies via injection or lookup rather than instantiating collaborators internally.
- [ ] Decouple high-level modules to maximize reuse and maintainability.
- [ ] Inject abstractions so mocks enable isolated unit tests.
- [ ] Shield changes to reduce cascading failures.
- [ ] Introduce new implementations without touching existing clients.
- [ ] Prefer stable abstractions over volatile concretions.
- [ ] Define interfaces to represent dependencies.
- [ ] Wire classes against abstractions instead of concretions.
- [ ] Adopt DI patterns when selecting collaborators.
- [ ] Leverage IoC containers to manage lifecycle ownership.
- [ ] Keep systems decoupled to ease refactors, changes, redeployments.
- [ ] Split fat interfaces so clients receive only needed methods.
- [ ] Reject requirements that force SRP-violating interface methods.

#### Robustness & Reliability

- [ ] Enforce strict output formats before sending responses.
- [ ] Accept unknown inputs only when semantics remain clear.
- [ ] Log and surface malformed partner payloads immediately.
- [ ] Document tolerance rules plus removal plans for shims.
- [ ] Prefer protocol fixes over bug-for-bug compatibility.

#### Performance & Optimization Discipline

- [ ] Profile hotspots before considering any micro-optimization.
- [ ] Define performance target and acceptable baseline early.
- [ ] Optimize only after failing measurable acceptance criteria.
- [ ] Preserve readability; document every performance tradeoff.
- [ ] Rerun regression and performance tests after tuning.

#### Lifecycle & Deletion Strategy

- [ ] Keep modules small enough to rewrite in a week.
- [ ] Limit coupling so deletions don't require cascade edits.
- [ ] Isolate features so removal doesn't break shared contracts.
- [ ] Record explicit dependencies to spot safe deletion seams.
- [ ] Delete code, tests, and configuration in the same pass.

#### Cohesion & Responsibility Alignment

- [ ] Keep each module focused on one responsibility.
- [ ] Cut scope until module complexity stays low.
- [ ] Group related operations so components stay reusable.
- [ ] Localize each change request to one module.
- [ ] Split code by stakeholder or actor-specific responsibilities.
- [ ] Apply Curly's Law so classes do one job.
- [ ] Extract new classes whenever unrelated change reasons emerge.
- [ ] Define the single goal before touching a code path.
- [ ] Reject extra responsibilities that dilute that one outcome.
- [ ] Align module boundaries with a single user-visible outcome.
- [ ] Split methods/classes until each has one change driver.
- [ ] Explicitly state what the unit will not cover.

#### Encapsulation & Interface Hygiene

- [ ] Limit calls to immediate collaborators only.
- [ ] Avoid chaining through returned collaborators.
- [ ] Push delegation into owning object interfaces.
- [ ] Expose DTOs for views instead of domain graphs.
- [ ] Document justified exceptions when structure must stay public.
- [ ] Hide implementation details behind small, stable interfaces.
- [ ] Stabilize interfaces so client changes stay unnecessary.
- [ ] Minimize class and member accessibility.
- [ ] Keep member data private and encapsulated.
- [ ] Exclude private implementation details from public interfaces.
- [ ] Reduce coupling to conceal implementation details.

#### Composition & Object Design

- [ ] Validate relationship is has-a before inheriting.
- [ ] Split behaviors into small interfaces or components.
- [ ] Use delegation to avoid breaking LSP.
- [ ] Allow runtime swapping of composed collaborators.
- [ ] Document extra obligations when inheritance unavoidable.
- [ ] Subclasses honor every pre/postcondition promised by supertype.
- [ ] Never strengthen preconditions when overriding base behavior.
- [ ] Never weaken postconditions or invariants in derived classes.
- [ ] Subtypes only throw exceptions declared by the base.
- [ ] Remove hierarchies that force instanceof checks in clients.

#### Variation Isolation & Extensibility

- [ ] Minimize changes to existing code to preserve stability.
- [ ] Favor extension points over direct modification.
- [ ] Hide non-variant details and expose only adjustable seams.
- [ ] Minimize edits whenever change happens.
- [ ] Hide each varying term behind interface.
- [ ] Isolate varying term in standalone module.

#### Command/Query Interaction Design

Note: Apply this only to (low-level) internal methods. (High-level) public interfaces can be command-query hybrids:

- [ ] Separate queries (reading) from commands (writing) to boost confidence.
- [ ] Declare each method solely query or command.
- [ ] Name methods to signal query versus command behavior.

### Principles

#### Keep It Simple, Stupid (KISS)

*Checklist coverage:* Scope & Goal Discipline; Simplicity & Right-Sized Solutions; Documentation & Communication Clarity.

#### You Aren't Gonna Need It (YAGNI)

*Checklist coverage:* Scope & Goal Discipline; Refactoring & Change Containment; Testing & Verification.

#### Do The Simplest Thing That Could Possibly Work

*Checklist coverage:* Scope & Goal Discipline; Simplicity & Right-Sized Solutions; Refactoring & Change Containment.

#### Separation of Concerns

*Checklist coverage:* Documentation & Communication Clarity; Modular Boundaries & Separation.

#### Code For The Maintainer

*Checklist coverage:* Documentation & Communication Clarity; Testing & Verification; Coupling Awareness & Dependency Constraints.

#### Avoid Premature Optimization

*Checklist coverage:* Performance & Optimization Discipline.

#### Optimize for Deletion

*Checklist coverage:* Lifecycle & Deletion Strategy.

#### Don't Repeat Yourself (DRY)

*Checklist coverage:* Simplicity & Right-Sized Solutions; Duplication Control & Reuse.

#### Boy Scout Rule

*Checklist coverage:* Documentation & Communication Clarity; Refactoring & Change Containment; Testing & Verification.

#### Connascence

*Checklist coverage:* Coupling Awareness & Dependency Constraints.

#### Minimize Coupling

*Checklist coverage:* Coupling Awareness & Dependency Constraints; Encapsulation & Interface Hygiene.

#### Law of Demeter

*Checklist coverage:* Encapsulation & Interface Hygiene.

#### Composition Over Inheritance

*Checklist coverage:* Composition & Object Design.

#### Orthogonality

*Checklist coverage:* Modular Boundaries & Separation; Testing & Verification.

#### Robustness Principle

*Checklist coverage:* Robustness & Reliability.

#### Inversion of Control

*Checklist coverage:* Documentation & Communication Clarity; Dependency & Interface Management.

#### Maximize Cohesion

*Checklist coverage:* Modular Boundaries & Separation; Cohesion & Responsibility Alignment.

#### Liskov Substitution Principle (LSP)

*Checklist coverage:* Composition & Object Design.

#### Open/Closed

*Checklist coverage:* Variation Isolation & Extensibility.

#### Single Responsibility Principle (SRP)

*Checklist coverage:* Modular Boundaries & Separation; Cohesion & Responsibility Alignment.

#### Hide Implementation Details

*Checklist coverage:* Encapsulation & Interface Hygiene.

#### Curly's Law

*Checklist coverage:* Cohesion & Responsibility Alignment.

#### Encapsulate What Changes

*Checklist coverage:* Variation Isolation & Extensibility.

#### Interface Segregation Principle (ISP)

*Checklist coverage:* Dependency & Interface Management.

#### Command Query Separation (CQS)

*Checklist coverage:* Command/Query Interaction Design.

#### Dependency Inversion Principle (DIP)

*Checklist coverage:* Dependency & Interface Management.

#### F.I.R.S.T Principles of Testing

*Checklist coverage:* Testing & Verification.

#### Arrange, Act, Assert (3A)

*Checklist coverage:* Testing & Verification.

### Conflicts between Programming Principles

- **YAGNI vs Boy Scout Rule** — Cleaning adjacent code risks scope creep YAGNI prevents, while only delivering current story blocks opportunistic cleanups Boy Scout Rule encourages.
  - **Important Action**: Favor Boy Scout Rule over YAGNI.
- **Separation of Concerns vs Maximize Cohesion** — Strict technical concern splitting scatters workflows cohesion keeps together, while grouping by cohesive domain blurs technical boundaries Separation of Concerns enforces.
  - **Important Action**: Favor Maximize Cohesion without violating Separation of Concerns — split in domain modules wiring together technical modules.
- **Code For The Maintainer vs Optimize for Deletion** — Disposable code may trade away documentation maintainers need, while extensive context makes code harder to throw away.
  - **Important Action**: Favor Optimize for Deletion without violating Code For The Maintainer — write self-documenting code.
- **Code For The Maintainer vs Hide Implementation Details** — Strict encapsulation obscures intent future maintainers need, yet surfacing intent can pressure exposing structure that should stay hidden.
  - **Important Action**: Favor Hide Implementation Details without violating Code For The Maintainer — maintain clear conventions for data flow from entry points to utilities.
- **Avoid Premature Optimization vs Optimize for Deletion** — Engineering for easy removal ahead of evidence is speculative optimization, while deferring speculative work pushes back on deletion seams before justified.
  - **Important Action**: Favor Optimize for Deletion — Avoid Premature Optimization targets performance (CPU, memory), more measurable than code quality.
- **DRY vs ISP** — Splitting interfaces per client reintroduces similar signatures undermining single authoritative source, but centralizing for reuse bloats interfaces with methods some clients don't need.
  - **Important Action**: Favor ISP without violating DRY — carefully manage shared abstractions used by different client interfaces.
- **Connascence vs Orthogonality** — Total module independence makes it harder to surface relationships connascence tracks, while allowing related elements to co-evolve acknowledges coupling orthogonality eliminates.
  - **Important Action**: Favor Connascence without violating Orthogonality — split in domain modules wiring together technical modules.
- **Minimize Coupling vs CQS** — Splitting reads/writes across distinct interfaces increases collaborator count, yet reducing dependencies discourages extra command/query partitions CQS demands.
  - **Important Action**: Favor CQS over Minimize Coupling — Note: Apply this only to (low-level) internal methods. (High-level) public interfaces can be command-query hybrids.
- **Law of Demeter vs Inversion of Control** — Global containers or callbacks force knowledge beyond immediate neighbor, and IoC plumbing may require reaching into containers or nested delegates.
  - **Important Action**: Favor Law of Demeter over Inversion of Control.
- **Composition Over Inheritance vs LSP** — Favoring composition rejects subtype hierarchies even when safe substitution simplifies clients, while insisting on composition prevents leveraging polymorphic substitution when it preserves contracts.
  - **Important Action**: Favor Composition Over Inheritance over LSP.
- **Maximize Cohesion vs SRP** — Keeping all related actions together may introduce multiple change reasons, and keeping every related behavior together can mix multiple change reasons inside single module.
  - **Important Action**: Favor Maximize Cohesion without violating SRP — split in domain modules wiring together technical modules.
- **Curly's Law vs Encapsulate What Changes** — Isolating volatility can slice one user-facing outcome across many modules violating single-goal guidance, while keeping all logic for one goal together makes extracting unstable parts harder.
  - **Important Action**: Favor Curly's Law without violating Encapsulate What Changes — split in domain modules wiring together technical modules.

- **KISS vs Open/Closed** — Extension seams up front add abstraction layers moving away from simplest implementation, while stripping to bare minimum removes extension points Open/Closed expects.
  - **Action**: Favor KISS over Open/Closed.
- **YAGNI vs Robustness Principle** — Hardening for malformed inputs may build capabilities no stakeholder requested, but skipping defensive code contradicts robustness expectation for unknown inputs.
  - **Action**: Favor Robustness Principle over YAGNI.
- **YAGNI vs F.I.R.S.T Principles of Testing** — Comprehensive pre-emptive tests feel like upfront work outside immediate requirement, yet limiting work to what requested undercuts comprehensive coverage F.I.R.S.T expects.
  - **Action**: Favor F.I.R.S.T over YAGNI.
- **Do The Simplest Thing vs DIP** — Abstractions and injection layers add ceremony beyond quickest working solution, while pursuing most direct implementation skips abstraction seams dependency inversion requires.
  - **Action**: Favor DIP over Do The Simplest Thing.
- **Do The Simplest Thing vs Inversion of Control** — Delegating orchestration to containers introduces indirection contradicting most straightforward implementation, yet wiring through IoC containers rarely most straightforward path for small change.
  - **Action**: Favor Inversion of Control over Do The Simplest Thing.
- **Separation of Concerns vs Arrange, Act, Assert (3A)** — Keeping setup/action/verification co-located in one test clashes with isolating each concern into shared helpers, but forcing steps into separate fixtures undermines single-test narrative 3A emphasizes.
  - **Action**: Favor Separation of Concerns over 3A.
