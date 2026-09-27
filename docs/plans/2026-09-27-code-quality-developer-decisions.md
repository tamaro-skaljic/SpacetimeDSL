# Plan: Code Quality Report — Developer Decisions

This plan carries out every entry of [`CODE_QUALITY_REPORT.md`](../../CODE_QUALITY_REPORT.md) that has a *Developer's decision*, except the entries whose decision says they must be done independently of every other fix. Each entry this plan addresses was moved out of the report and into the task that addresses it, quoted under **Moved from the report**, so the report only lists what is still open. When an entry was addressed only in part, the part the plan covers was moved and the rest stayed in the report.

**Goal:** Resolve the decided findings in one breaking release (`0.24.0`) without changing what the generator emits for an accepted input, except where a task says the recorded output moves and why.

**Approach:** Twelve phases in dependency order, all on the branch `improve-code-quality` and in **one pull request**, with **one commit per task**. The early phases make the gates trustworthy (scripts, CI, harnesses) before the later phases refactor code those gates guard.

**Reading a task.** *Moved from the report* is the original finding. *Test first* names the test kind (per AGENTS.md: diagnostics, snapshot, runtime), its file, and the red that has to be observed before the change. *Change* is the implementation outline. *Recorded output* says whether `git diff derive/tests/snapshots compile-tests/tests/ui` must stay empty (a *pure refactoring*) or which files move and why. *Docs* lists AGENTS.md, `docs/DOCUMENTATION.md` and `docs/MIGRATION.md` changes.

---

## Decisions taken while planning

These answers were given by the developer when this plan was written. They fill gaps the report left open, and every task below follows them.

| Topic | Decision |
| --- | --- |
| Versioning | One breaking release: every crate goes to `0.24.0` in the last phase. User-visible changes go into a new `docs/MIGRATION.md`. |
| `docs/MIGRATION.md` scope | Everything user-visible: breaking API changes (before/after code), newly rejected inputs (diagnostic and fix), changed runtime messages or `Display` text, `SetZero` on `Uuid`, and a note to `derive-input` consumers about the documented data-transfer contract. |
| Delivery | A single pull request, one commit per task. |
| Entries without a decision | Only two are resolved in addition: the dead `no_table_attribute_found` (Task 31), and moving `Display for OnDeleteStrategy` plus snapshot coverage of every emitted `Action` / `OneOrMultiple` variant (Task 24). |
| `unique_index` names (Task 32) | Reject names that match **no declared index** and repeated names. A name of a hash or single-column index stays accepted without effect; that entry stays in the report. |
| `singleton.rs` | Changed only where decided entries name it: type check via the classification (Task 36) and `rendered_primary_key` via the message module (Task 52). The "one home for the injected key" part stays in the report. |
| `derive/src/output.rs::build` | Only the parts the hook matrix (Task 28) and the entry-point iterators (Task 58) require. The rest stays in the report. |
| Table selector (Task 31) | `#[dsl(table = <accessor>)]` is **required iff the struct has ≥ 2 `#[table]` attributes**. With exactly one `#[table]` it is optional but must match if given. Singletons keep "exactly one `#[table]`". `plural_name` no longer selects anything. |
| `SetZero` types (Task 50) | Unsigned integers (`u8`–`u128`, set `0`) and `Uuid` (set `Uuid::NIL`). Every other type is rejected. |
| `Uuid::NIL` semantics (Task 50) | Mirror what `0` does today: the reference-integrity guards of create, update and upsert treat it as "no reference". The delete and soft-delete cascades stay unchanged; their missing `0` / `NIL` handling was added to the report as a new entry to be fixed independently. |
| Runtime paths (Task 20) | Every emitted runtime path is `crate::spacetimedsl::…`. |
| SpacetimeDB paths (Task 20) | Every emitted SpacetimeDB path is `::spacetimedb::…`, produced by a new counterpart of `api/runtime.rs`. |
| Prelude (Task 22) | `spacetimedsl::prelude` holds the runtime items only. The crate root, the macro's flat re-exports and the generated prelude glob it. The generated prelude also lists the generated items and the SpacetimeDB convenience re-exports, once. |
| `x lint` (Task 3) | `cargo fmt --all -- --check`, then `cargo clippy --workspace --all-targets --all-features -- -D warnings`. CI runs `./x.sh lint` instead of its own fmt and clippy steps. |
| `x test` (Task 1) | Checks every `spacetime` exit code, requires the `tester` call to succeed and the log to contain `Test executed successfully`, waits for the local server first, and cleans up in `finally` / `trap`. |
| `read_context_compatible` (Task 14) | Confirmed rule: a read-only table handle (views, `LocalReadOnly`) offers `count()` and index lookups but no full-table `iter()`. |
| Cascade panics (Task 63) | Only the named hook-fragment struct and the invariant-stating `expect` messages. The panic of SpacetimeDB's `update` stays in the report with the independent `update.rs` entry, since SpacetimeDB has no `try_update`. |
| Hook matrix shape (Task 28) | `HookKind { timing, operation }` with `HookKind::ALL`, and `SpacetimeDSLMethodHooks { pub declared: BTreeMap<HookKind, SpacetimeDSLMethodHook> }` with `get` / `iter`. |
| Unsupported column types (Task 39) | Every type except a plain `Type::Path` without `qself` is rejected, after unwrapping invisible groups and parentheses. One diagnostic per kind; compile-tests for array, tuple and reference. |
| Fixture registration check (Task 5) | Checks all four: every fixture is registered, no snapshot directory is orphaned, the test function is named after its fixture, and every registered fixture exists. |
| FK path equality (Task 51) | Compare after stripping a leading `::`; the diagnostic tells the user to spell both paths the same way. |
| Primitive spellings (Task 36) | `core::primitive::uN`, `std::primitive::uN` and a leading `::` count as `uN`. |

---

## Conventions for every task

- **Gates.** Run everything through `x.ps1` (never `cargo` directly):
  - `.\x.ps1 unit-test 2>&1 | Select-String -Pattern "test result:|FAILED|^error|^warning: " | Select-Object -First 20`
  - `.\x.ps1 test` — after Task 1 its exit code is meaningful: `.\x.ps1 test; if ($LASTEXITCODE -ne 0) { "FAILED" }`.
  - `.\x.ps1 lint` (after Task 3) and `.\x.ps1 format` until a second run changes nothing.
- **Red before green.** Observe the red and read its failure text before changing production code. When a task says the red "cannot be observed" (a guard for behaviour that is already correct), briefly break the code under test to watch the guard fail, then revert that break.
- **Recorded output.** For a pure refactoring, `git diff derive/tests/snapshots compile-tests/tests/ui` must be empty. Otherwise regenerate with `INSTA_FORCE_UPDATE=1` or `TRYBUILD=overwrite` as AGENTS.md describes, read the whole diff, and name every kind of change in the commit message.
- **API surface.** Every change to a public type of `derive-input` updates the data-transfer contract test from Task 13 in the same commit, and adds a line to `docs/MIGRATION.md`.
- **Future work** removed from the code (TODO, FIXME, commented-out code) goes to the issue tracker; the commit message names the issue.
- **Knowledge in docs** (AGENTS.md, `docs/DOCUMENTATION.md`) changes in the same commit as the code.

---

## Phase 0 — Baseline

### Task 0: Record the baseline and create `docs/MIGRATION.md`

- Run `.\x.ps1 unit-test`, `.\x.ps1 test` (check for the marker by hand, as AGENTS.md still describes at this point) and `.\x.ps1 format`, and note the results in the pull request description. A red baseline must be fixed first or reported; it must not be carried into Phase 1.
- Create `docs/MIGRATION.md` with the heading `## 0.23 → 0.24` and the empty sections *Breaking API changes*, *Newly rejected inputs*, *Changed messages and generated code* and *For crates building on `spacetimedsl_derive-input`*. Link it from `README.md` and from the top of `docs/DOCUMENTATION.md`.
- Commit: `Add docs/MIGRATION.md for the 0.24 release`.

---

## Phase 1 — Gates that report failure

### Task 1: Fail-fast `x test` and a side-effect-free `x debug`

*Moved from the report* — `gen-x.sh` › `gen-x.sh`: `generate_test` and `generate_debug`

> **Violates:** Robustness Principle — "Log and surface malformed partner payloads immediately"; F.I.R.S.T Principles of Testing — self-validating; Modular Boundaries & Separation — "Design APIs without cross-cutting side effects"
>
> The generated `test` command runs each `spacetime` command without checking its result and ends with a change of directory, so it reports success even when publishing fails or the `tester` reducer returns an error. AGENTS.md documents searching the output for the success marker as the workaround, and every tool built on top of the command inherits the problem (CI, `compare-performance.ps1`). The PowerShell `debug` command sets `$env:RUSTFLAGS` and never restores it, so every later build in the same session runs with `-Zmacro-backtrace`. `compare-performance.ps1` already shows the robust pattern in this repository: `$ErrorActionPreference = "Stop"`, explicit `$LASTEXITCODE` checks and waiting for the server.
>
> Recommendation: Generate fail-fast scripts (`set -euo pipefail`; `$ErrorActionPreference = 'Stop'` with `$LASTEXITCODE` checks), let `test` fail when the success marker is missing, and restore location and environment (`Push-Location` / `Pop-Location`, `try` / `finally`). Regenerate `x.sh` and `x.ps1`, and update AGENTS.md's description of the gate in the same change.
>
> Developer's decision: The recommendation should be applied.

- **Test first:** Make `tester` return `Err` temporarily (for example `return Err("red".into());` at its top), run `.\x.ps1 test` and observe `$LASTEXITCODE` = 0. That is the red. Revert the break after green.
- **Change (`gen-x.sh`):**
  - Bash header: `set -euo pipefail`. PowerShell header: `$ErrorActionPreference = 'Stop'`, plus a `$LASTEXITCODE` check (`throw` naming the command) after every native command.
  - `test`: wait for `spacetime server ping local` to succeed, with a timeout and a message that says to run `spacetime start` (the pattern of `compare-performance.ps1`). Then run `publish`, `call tester` and `logs` for `spacetimedsl`, each checked. Print the logs, and fail unless they contain `Test executed successfully`. `delete` the module in a `trap` / `finally`, so a failed run does not leave it published. Handle `blackholio` the same way: `publish` checked, `delete` in cleanup.
  - Replace `cd` / `Set-Location` with `pushd`/`popd` in bash and `Push-Location`/`Pop-Location` inside `try`/`finally` in PowerShell.
  - `debug`: PowerShell saves `$env:RUSTFLAGS` and restores it in `finally`. Bash already scopes it to the command.
  - Regenerate with `bash gen-x.sh`, and commit `x.sh`, `x.ps1` and `gen-x.sh` together.
- **Green:** With the temporary break, `.\x.ps1 test` exits non-zero and names the missing marker. Without it, it exits 0. Stop the server and check the command fails with the timeout message.
- **Docs:** AGENTS.md *The two gates*: `.\x.ps1 test` now fails on its own. Replace the marker-search snippet with an exit-code check that prints only diagnostic lines on failure. Remove "Its exit code is meaningless". `compare-performance.ps1` already relies on the exit code, so it needs no change; say so in the commit message.
- **Recorded output:** unchanged.

### Task 2: Correct the texts of `gen-x.sh`

*Moved from the report* — `gen-x.sh` › `gen-x.sh`: `generate_usage`, the final message and the embedded `loc` programs

> **Violates:** Code For The Maintainer; Self-Documenting Code — no redundant comments
>
> The usage text describes `format` as running a formatter check, although it rewrites files and checks nothing, and its description of `test` does not mention that `test` also publishes `blackholio`. The final message claims to have generated `x` instead of `x.sh`. The embedded `loc` programs carry comments that repeat the statement below them.
>
> Recommendation: Correct the texts and delete the comments that repeat the code.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `format` is described as "Format the code and apply clippy fixes"; `test` as "Publish and test the `test` module, then publish `blackholio`" (after Task 1, with the success-marker check). The final message says `x.sh`. Delete the comments in the embedded `loc` programs that repeat their statement. Regenerate `x.sh` and `x.ps1`.
- **Test first:** Not applicable (text only). Check `.\x.ps1` without arguments prints the new usage.
- **Recorded output:** unchanged.

### Task 3: `x lint` and CI that fails

*Moved from the report* — `.github/workflows/test.yml` › `test.yml`: job `test`

> **Violates:** Robustness Principle; F.I.R.S.T Principles of Testing — self-validating; DRY; Code For The Maintainer
>
> The step "Build & test SpacetimeDSL" relies on the exit status of `./x.sh test`, which is always success (see `gen-x.sh`), so this workflow — and the release workflow, which waits for it — can pass while the runtime tests fail. The clippy step appends `|| echo …` to every invocation, so it can never fail, and it lints directory by directory, which the comment in `gen-x.sh` explains leaves crates unlinted: the examples, `compile-tests` and `debug-helper` are never linted in CI. The comment above the installation retry loop states a different number of attempts than `MAX_ATTEMPTS`.
>
> Recommendation: Gate on the fixed script. Add a lint command to `gen-x.sh` that runs clippy over the workspace in check mode with warnings denied, and let CI and developers use the same command.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - `gen-x.sh`: add `generate_lint`, which runs `cargo fmt --all -- --check` and then `cargo clippy --workspace --all-targets --all-features -- -D warnings`. Add `lint` to the PowerShell `ArgumentCompleter` list and to the usage text. Regenerate.
  - `.github/workflows/test.yml`: the step "Build & test" keeps `spacetime start &` and runs `./x.sh unit-test` and `./x.sh test` (which now waits for the server and fails on its own). Replace "Check code formatting" and "Run clippy on derive libraries" with one step `./x.sh lint`. Make the comment above the installation retry loop agree with `MAX_ATTEMPTS` (4).
- **Test first:** Run `.\x.ps1 lint` before fixing anything and read the warnings it reports. The examples, `compile-tests` and `debug-helper` have never been linted, so expect findings. That list is the red. Fix every warning (a fix that changes generated code regenerates the snapshots and is described in the commit message), until `.\x.ps1 lint` exits 0.
- **Docs:** AGENTS.md *Green includes the formatter*: `.\x.ps1 lint` must exit 0 before a task counts as done. It is the command CI runs.
- **Recorded output:** unchanged, unless a lint fix touches the generator; then read and describe the diff.

### Task 4: Release workflow takes the pinned toolchain

*Moved from the report* — `.github/workflows/release.yml` › `release.yml`: step "Install Rust toolchain"

> **Violates:** DRY; Code For The Maintainer
>
> The test workflow states that the toolchain comes from `rust-toolchain.toml`, while the release workflow asks for `toolchain: stable`, which the pinned toolchain file overrides inside the repository — the configuration says something the build does not do.
>
> Recommendation: Drop the explicit `toolchain` input, so both workflows take the toolchain from `rust-toolchain.toml`.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Remove the `toolchain: stable` input from "Install Rust toolchain" in `.github/workflows/release.yml`, and add the same comment the test workflow has: the toolchain comes from `rust-toolchain.toml`.
- **Test first:** Not applicable. Check the step against `test.yml`'s.

---

## Phase 2 — Tests that validate themselves

### Task 5: Registration check for the snapshot fixtures

*Moved from the report* — `derive/src/characterization_tests.rs` › `characterization_tests.rs`: the hand-registered `#[test]` functions and the snapshot directories

> **Violates:** Connascence — of Name; Testing & Verification — *A test that cannot run is not a test*; Lifecycle & Deletion Strategy — "Delete code, tests, and configuration in the same pass"
>
> Each fixture needs a hand-written test whose name repeats the fixture's file name as a string; a fixture without such a test never runs, and nothing reports it. Snapshots whose fixture is gone stay behind unnoticed: `derive/tests/snapshots/update_hook_with_updated_at/` has neither a fixture nor a test (it predates the rename of `updated_at` to `set_on_update`). The fixture `hooks_all_six` claims in its name and doc to cover all hooks, while the soft-delete hooks added since are pinned by `soft_delete_hooks`. `is_dsl_attribute` repeats the string comparison of attribute paths from `derive/src/lib.rs`.
>
> Recommendation: Delete the orphaned snapshot directory after confirming that `update_hook_with_set_on_update` covers its content. To keep fixtures and registrations in step, the options are: (a) enumerate `tests/fixtures` in one test, the way `trybuild` globs its directory; (b) keep the explicit registration AGENTS.md prescribes and add a test that fails for an unregistered fixture or for a snapshot directory without a fixture; (c) make `x.ps1 unit-test` reject unreferenced snapshots. Options (a) and (b) change the rule written in AGENTS.md, which then has to change with them. Rename `hooks_all_six` after the shape it pins.
>
> Developer's decision: The recommendation (b) should be applied.

- **Test first (snapshot harness, `derive/src/characterization_tests.rs`):** Add `#[test] fn every_fixture_is_registered_under_its_own_name()`. It parses `include_str!("characterization_tests.rs")` with `syn::parse_file` and collects every `#[test]` function whose body is a single `snapshot_fixture("<literal>")` call. It then asserts, each with a message naming the offending names:
  1. every `tests/fixtures/*.rs` is registered;
  2. every directory under `tests/snapshots/` has a fixture of the same name;
  3. each such function is named exactly like its literal;
  4. every literal names an existing fixture file.

  The red is real: `update_hook_with_updated_at/` is an orphaned snapshot directory. Read that failure.
- **Change:**
  - Compare `derive/tests/snapshots/update_hook_with_updated_at/` with `update_hook_with_set_on_update/`. If the latter covers the same shapes, `git rm -r` the orphan; if not, first add the missing shape to `update_hook_with_set_on_update.rs`.
  - Rename the fixture `hooks_all_six` to `insert_update_and_delete_hooks` (fixture file, test function, snapshot directory via `git mv`) and correct its doc so it no longer claims to cover all hooks. The soft-delete hooks stay pinned by `soft_delete_hooks`.
  - `is_dsl_attribute` gets the shared helper in Task 25; leave it until then.
- **Recorded output:** only the deleted orphan and the renamed directory move. Check that `git diff -M --stat` shows the rename as 100 % similar.
- **Docs:** AGENTS.md *Snapshot*: "Register the fixture with a test named after it; `every_fixture_is_registered_under_its_own_name` fails for a fixture without a test, a test without a fixture, and a snapshot directory without a fixture."

### Task 6: `tester` runs every group and reports all failures

*Moved from the report* — `examples/test/src/lib.rs` › `lib.rs`: `fn tester(ctx: &ReducerContext) -> Result<(), String>`

> **Violates:** F.I.R.S.T Principles of Testing — independent, repeatable; Self-Documenting Code — don't document the past
>
> Every group runs in one reducer on one database. Later groups see the rows earlier groups left behind, and absolute assertions such as `count_of_all_entity_relationships().ne(&3)` depend on that; the first failing group stops all later ones and hides their results. The module `spacetimedsl_cascade_delete_hook_repro` is named after the bug report it reproduced rather than the behaviour it pins.
>
> Recommendation: Options: (a) one reducer per group, each called and checked by `x.ps1 test`; (b) run every group and report all failures together before failing. In both cases make count assertions relative to a count taken before the act, as `update_and_soft_delete_hook_test.rs` already does with `logged_before`. Rename the module after the behaviour it covers. AGENTS.md describes this harness and has to follow any change to it.
>
> Developer's decision: The recommendation (b) should be applied.

- **Test first (runtime):** Temporarily make an early group (for example `entity::run_tests`) return `Err`. Observe that no later group runs and that only the first message appears. That is the red.
- **Change:**
  - `examples/test/src/lib.rs`: a list of `(group name, fn(&DSL<'_, ReducerContext>) -> Result<(), String>)`. Run every group, collect `"<group>: <message>"` for each failure, and return one `Err` listing all of them. Log `Test executed successfully!` only when the list is empty. A panic still aborts the reducer, and the doc comment of `tester` says so.
  - Make every absolute count assertion in the groups relative: take a count before the act and compare the difference, as `update_and_soft_delete_hook_test.rs` does with `logged_before`. Find them with `grep -n "count_of_all_" examples/test/src`.
  - Rename `spacetimedsl_cascade_delete_hook_repro` to `primary_key_foreign_key_cascade_test` (file, `mod` line, and every `path = crate::spacetimedsl_cascade_delete_hook_repro`).
- **Green:** With the temporary break, the error lists the broken group and the other groups still run (their log lines appear). Without the break, the marker appears.
- **Docs:** AGENTS.md *Runtime*: `tester` runs every group and reports all failures together. A group's absolute counts must be relative, because groups share one database.

### Task 7: The primary-key foreign-key cascade asserts its behaviour

*Moved from the report* — `examples/test/src/spacetimedsl_cascade_delete_hook_repro.rs` › `spacetimedsl_cascade_delete_hook_repro.rs`: the tables `ParentRecord` and `ChildMarker` and their hooks

> **Violates:** Testing & Verification — *A test that cannot run is not a test*, "Make assertions binary without manual inspection"
>
> The module declares tables and hooks but has no `run_tests`, and `tester` does not call it. It only proves that the expansion compiles, which the runtime gate checks as a side effect rather than by an assertion.
>
> Recommendation: Add a `run_tests` that deletes a `ParentRecord` and asserts the cascade and the hook calls.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** In `primary_key_foreign_key_cascade_test.rs`, add a log table (accessor `primary_key_foreign_key_cascade_hook_call`; table names are global in the module) that the existing hooks append their name to. Add `pub(crate) fn run_tests`: create a `ParentRecord` and a `ChildMarker`, delete the `ParentRecord`, and assert (1) that the `ChildMarker` is gone and (2) that the log holds exactly `before_child_marker_delete`, `after_child_marker_delete` in auto-increment order. Register the group in `tester`. First write the expected sequence wrong (for example without the after-hook) and read the failure; then correct it.
- **Recorded output:** unchanged (examples only).

### Task 8: `soft_delete_skips_update_hooks_test` reads the stored row

*Moved from the report* — `examples/test/src/update_and_soft_delete_hook_test.rs` › `update_and_soft_delete_hook_test.rs`: `fn soft_delete_skips_update_hooks_test<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String>`

> **Violates:** F.I.R.S.T Principles of Testing — self-validating; Test Driven Development — *Read the failure, not just the fact of it*
>
> The last check reads `modified_at` from `member`, the value `create_guild_member` returned before any soft deletion happened, instead of the stored row afterwards, so it passes whatever the soft deletion does. Its message speaks of `deleted_at`, but the table's marker column is `deleted: bool`.
>
> Recommendation: Re-read the row with `get_guild_member_by_id` after the soft deletion and correct the message. Observe the check fail before trusting it, for example by temporarily asserting the opposite.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** Replace the read of `member` with `dsl.get_guild_member_by_id(...)` after the soft deletion. Temporarily invert the assertion and observe the failure, then restore it. Change the message from `deleted_at` to the table's marker column `deleted`.
- **Recorded output:** unchanged.

### Task 9: `entity.rs` — one function per behaviour

*Moved from the report* — `examples/test/src/entity.rs` › `entity.rs`: `pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String>`

> **Violates:** Arrange, Act, Assert (3A); F.I.R.S.T Principles of Testing; Code For The Maintainer; Self-Documenting Code — no abbreviations
>
> One function arranges, acts and asserts many unrelated behaviours in sequence. The failure message after deleting `player2` says that `player` should be deletable. Absolute counts depend on no other group having created relationships. `er4_1` and `er4_2` are abbreviations, and `match` blocks that return an error alternate with `?` without a reason.
>
> Recommendation: One function per behaviour with its own rows, a corrected message, relative counts, spelled-out names and one error-handling style.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Split `run_tests` into one function per behaviour. Each creates its own rows, acts with one call and asserts its outcome. `run_tests` calls them in sequence. Correct the message after deleting `player2`. Rename `er4_1` / `er4_2` after what they hold (`entity_relationship4_first`, `…_second`). Use one error style throughout: `?` with `map_err` producing a message that names what should have happened. Make the counts relative (if Task 6 did not already).
- **Test first:** For each new function, observe one assertion fail by temporarily changing its expected value.
- **Recorded output:** unchanged.

### Task 10: `component/test.rs` asserts what its calls return

*Moved from the report* — `examples/test/src/component/test.rs` › `test.rs`: `pub fn run_tests(dsl: &DSL<'_, ReducerContext>) -> Result<(), String>`

> **Violates:** F.I.R.S.T Principles of Testing — self-validating; Arrange, Act, Assert (3A); Testing & Verification — "Make assertions binary without manual inspection"
>
> Many calls discard their results (`let _ = dsl.get_tests_by_wrapped_index(…)`, `let _ = dsl.delete_tests_by_wrapped_index(…)`): they only prove that the argument types are accepted, which is a compile-time property, and they ignore run-time failures. `let _ = dsl.create_ship_object(…)` is the arrange step of the following assertion, so if it fails, the assertion checks something else.
>
> Recommendation: Assert what each call should return. Treat a failing arrange step as a failure (`?`).
>
> Developer's decision: The recommendation should be applied.

- **Change:** Replace every `let _ = dsl.…` with an assertion on the returned value: a count, the rows found, `Ok`/`Err` of a delete. Make `dsl.create_ship_object(…)` an arrange step with `?`.
- **Test first:** Observe each new assertion fail once, by temporarily expecting a wrong value.
- **Recorded output:** unchanged.

### Task 11: Delete the outdated FIXMEs in `blackholio`

*Moved from the report* — `examples/blackholio/src/lib.rs` › `lib.rs`: the FIXME and TODO comments

> **Violates:** Self-Documenting Code — don't document the past
>
> The FIXMEs on `Entity::position` and `Entity::login_status` report that `DbVector2` and `LoginStatus` lack `PartialEq`, but both derive it now. The FIXME above `Player` says `update = true` should not have been valid because all fields were private, but the struct has public fields now.
>
> Recommendation: Delete the outdated FIXMEs.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Delete the three FIXMEs. `.\x.ps1 test` publishes `blackholio` and must still pass.

### Task 12: `debug-helper` names itself

*Moved from the report* — `debug-helper/src/main.rs` › `main.rs`: `impl Display for Error` and `fn render_location(formatter, err: &syn::Error, filepath: &Path, code: &str) -> fmt::Result`

> **Violates:** Code For The Maintainer — Principle of Least Astonishment; Self-Documenting Code — no abbreviations
>
> The usage message calls the program `dump-syntax`, the name of the `syn` example it was taken from, while it is the `spacetimedsl-debug` crate started through `x debug`. `err` and `n` are abbreviations.
>
> Recommendation: Name the actual program in the usage text and spell the identifiers out.
>
> Developer's decision: The recommendation should be applied.

- **Change:** The usage text names `x debug` (the `spacetimedsl-debug` crate) instead of `dump-syntax`. Rename `err` to `error` and `n` after what it counts.
- **Test first:** Not applicable (tool text). Run `.\x.ps1 debug` once.

---

## Phase 3 — The public surface of `derive-input`

### Task 13: Document `Table` and `Column` as a data-transfer contract and pin it

*Moved from the report* — `derive-input/src/lib.rs` › `lib.rs`: `pub struct Table` and `pub struct Column`

> **Violates:** Hide Implementation Details — "Keep member data private and encapsulated"; Minimize Coupling; YAGNI; DRY; Documentation & Communication Clarity — "Document concern boundaries and integration contracts"
>
> The crate advertises itself as a base for other procedural macro crates and exposes its whole analysis through public fields, including generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`) and bookkeeping (`compile_error_checks`). Every internal restructuring is therefore a breaking change for such crates, and the `derive` crate reads deep into the graph. The doc of `Table::try_parse` uses `/** */` and speaks of a derive macro, although it is called from an attribute macro.
>
> Recommendation: The developers decide what external crates may rely on. Options: (a) keep public fields, document every one of them as a data-transfer contract and pin that contract with a consumer-side test; (b) make the fields private and expose accessors for the parts meant to be stable; (c) withdraw the promise to external crates and reduce the model to crate visibility.
>
> Developer's decision: The recommendation (a) should be applied.

- **Test first (derive crate, run by `.\x.ps1 unit-test`):** Add `derive/src/data_transfer_contract_tests.rs` (`#[cfg(test)] mod` in `derive/src/lib.rs`). It parses a fixture struct through `Table::try_parse`, and destructures `Table`, `Column` and every public struct reachable from them **without `..`**. It matches every public enum **without a wildcard arm**. It asserts a few values per struct, so the test also checks something at run time. Adding, removing or renaming a field or a variant then fails to compile. Observe that red by temporarily adding a field to `Column`.
- **Change:**
  - `derive-input/src/lib.rs`: add a crate-level `//!` stating the contract. The fields of `api` are a public data-transfer structure for procedural macro crates; every field is part of the semver contract; generated token streams (`method_impl`, `wrapper_impl`, `struct_impl`) are opaque output to splice, not to inspect.
  - Document every public field of every `api` type with `///`, in DSL terms: when it is `Some`, and what it holds.
  - Replace the `/** */` blocks with `///`, and describe `Table::try_parse` as called from an attribute macro.
- **Docs:** `docs/MIGRATION.md` → *For crates building on `spacetimedsl_derive-input`*: the contract, and that the test pins it.
- **Recorded output:** unchanged.

### Task 14: Document the column and method fields in rustdoc

*Moved from the report* — `derive-input/src/api/dsl/column.rs` › `column.rs`: `pub struct SpacetimeDSLColumn` and `pub struct SpacetimeDSLColumnMethodsForUniqueIndex` / `ForIndex`

> **Violates:** Documentation & Communication Clarity — "Document concern boundaries and integration contracts"; Self-Documenting Code
>
> The conditions under which the public fields are `Some` are written as `//` comments, so the rustdoc of this published crate shows none of them, and "Only `Some(T)` if mutable" does not say what makes a column mutable (a non-private field). The types in `method.rs`, `hook.rs`, `getter.rs` and `setter.rs` document no fields at all, and `table.rs` documents only some; `SpacetimeDSLMethod::read_context_compatible`, for instance, does not say why `get_all_<tables>` is unavailable in a read-only context while `count_of_all_<tables>` is available.
>
> Recommendation: Turn the comments into `///` documentation stated in DSL terms, and document the SpacetimeDB rule behind each `read_context_compatible` value, which the developers need to confirm.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Turn the `//` conditions in `api/dsl/column.rs` into `///` in DSL terms: "`Some` when the field is not private". Do the same for the fields of `method.rs`, `hook.rs`, `getter.rs`, `setter.rs`, `mut_getter.rs` and `table.rs` (if Task 13 left any). Document `SpacetimeDSLMethod::read_context_compatible` with the confirmed rule: a read-only table handle (views, `LocalReadOnly`) offers `count()` and index lookups but no full-table `iter()`, so `get_all_<tables>` is `false` and `count_of_all_<tables>` and index getters are `true`. Add a one-line reason next to each generator's value.
- **Recorded output:** unchanged.

### Task 15: `internal` constructors become `pub(crate)`

*Moved from the report* — `derive-input/src/lib.rs` › `lib.rs`: `mod internal` and the `pub` inherent functions its modules add to `api` types

> **Violates:** Hide Implementation Details — "Minimize class and member accessibility", "Exclude private implementation details from public interfaces"
>
> The modules below the private `internal` module add `pub` inherent functions to public `api` types — for example `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `RustField::map`, `RustVisibility::map`, `SpacetimeDSLColumn::try_parse`, `WrapperType::try_parse`, `Getter::map`, `Setter::map` — and `impl Display for RustVisibility`. An inherent method's visibility follows its own `pub`, not the module of its `impl` block, so these internal constructors are callable from every dependent crate. `#[doc(hidden)]` on a private module has no effect, which suggests the intent was to hide them.
>
> Recommendation: Reduce these functions to `pub(crate)`, keeping `Table::try_parse` as the only entry point; replace the `Display` implementation as described in `internal/rust.rs`; remove the ineffective `#[doc(hidden)]`.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Every inherent function that a module below `internal` adds to an `api` type becomes `pub(crate)`. Find them with `grep -rn "^\s*pub fn" derive-input/src/internal` inside `impl <ApiType>` blocks (among others `SpacetimeDBColumn::map`, `SpacetimeDBTable::map`, `RustField::map`, `RustVisibility::map`, `SpacetimeDSLColumn::try_parse`, `SpacetimeDSLTable::try_parse`, `WrapperType::try_parse`, `ForeignKey::try_parse`, `ReferencingTable::try_parse`, `UUIDVersion::try_parse`, `Getter::map`, `MutGetter::map`, `Setter::map`, `SpacetimeDSLColumnMethods::map`, the `WrapperType` helpers). `Table::try_parse` stays the only entry point. Remove `#[doc(hidden)]` from `mod internal`. `Display for RustVisibility` goes in Task 16.
- **Test first:** Temporarily call `RustField::map` from `derive`, observe that it compiles, and observe that it no longer compiles after the change. There is no permanent test; the Task 13 contract test pins the fields.
- **Docs:** `docs/MIGRATION.md` (derive-input consumers): the constructors are no longer public.
- **Recorded output:** unchanged.

### Task 16: `RustVisibility` implements `ToTokens`

*Moved from the report* — `derive-input/src/internal/rust.rs` › `rust.rs`: `impl fmt::Display for RustVisibility`

> **Violates:** Connascence — of Meaning; KISS; Hide Implementation Details
>
> A visibility is rendered to text so that the `derive` crate can parse it back (`derive/src/output/accessor.rs`), and it is compared as text in `internal/dsl/method/reference_integrity.rs` and `internal/dsl/method/upsert.rs` (`rust_field_visibility.to_string()` against `RustVisibility::Private.to_string()`) where `matches!` would do. As an implementation on a public type, the rendering is public API.
>
> Recommendation: Implement `ToTokens`, compare with `matches!`, and remove `Display` once nothing needs it.
>
> Developer's decision: The recommendation should be applied.

*Moved from the report* — `derive/src/output/accessor.rs` › `accessor.rs`: `fn definition(&self) -> syn::Result<AccessorDefinition<'a>>` and `fn method_tokens(&self) -> TokenStream`

> **Violates:** Connascence — of Meaning; DRY; KISS
>
> The visibility of mutable getters and setters travels between the crates as text: `derive-input` renders `RustVisibility` with `Display` (`pub (crate)`), and this function parses it back with `parse_str`. Every accessor body imports `spacetimedsl::Wrapper` through a path written here by hand, although `derive-input/src/api/runtime.rs` declares itself the single place for runtime paths.
>
> Recommendation: Let `RustVisibility` implement `ToTokens` and drop the parse. Emit the import through `runtime`, after the spelling decision described in `derive-input/src/api/runtime.rs`.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - `impl ToTokens for RustVisibility`: `pub`, `pub(crate)`, `pub(super)`, `pub(in path)`, nothing for `Private`.
  - `derive/src/output/accessor.rs`: `AccessorDefinition::method_visibility` holds tokens from `ToTokens`, `parse_str` goes, and `definition` becomes infallible.
  - Replace the string comparisons in `reference_integrity.rs` and `upsert.rs` with `matches!(…, RustVisibility::Private)`.
  - Remove `impl Display for RustVisibility`.
  - The second half of the accessor entry, the hand-written `use spacetimedsl::Wrapper`, is Task 20.
- **Docs:** `docs/MIGRATION.md` (derive-input consumers): `Display` removed, use `ToTokens`.
- **Recorded output:** pure refactoring (the snapshots are pretty-printed, so the spacing of `pub (crate)` does not show).

### Task 17: `SpacetimeDSLArgType::actual_type()`

*Moved from the report* — `derive-input/src/api/dsl/method.rs` › `method.rs`: `pub struct SpacetimeDSLArg` and `pub enum SpacetimeDSLArgType`

> **Violates:** YAGNI; Law of Demeter — "Push delegation into owning object interfaces"
>
> `is_option` and `Wrapped::wrapped_type` only consumers (`derive/src/output.rs::map_args`, `internal/dsl/method/create.rs`) match on the enum just to take `actual_type` from either variant.
>
> Recommendation: Add `SpacetimeDSLArgType::actual_type()`.
>
> Developer's decision: The recommendation should be applied.

*Moved from the report* — `derive/src/output.rs` › `output.rs`: `fn map_args(args: &Vec<SpacetimeDSLArg>) -> Vec<TokenStream>` and `pub fn malformed_code_generation_result(result: String) -> String`

> **Violates:** Law of Demeter — "Push delegation into owning object interfaces"; KISS; Scope & Goal Discipline — "Future ideas captured outside codebase"
>
> `map_args` destructures `SpacetimeDSLArgType` through fully qualified paths only to take `actual_type` from either variant; `derive-input/src/internal/dsl/method/create.rs` does the same. A FIXME above `malformed_code_generation_result` records an idea.
>
> Recommendation: Add an `actual_type()` method to `SpacetimeDSLArgType` in `derive-input` and use it in both places, and move the FIXME to the issue tracker.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Add `pub fn actual_type(&self) -> &TokenStream` to `SpacetimeDSLArgType`, and use it in `derive/src/output.rs::map_args` and `internal/dsl/method/create.rs`. `map_args` takes `&[SpacetimeDSLArg]` and uses the imported type name. Move the FIXME above `malformed_code_generation_result` to the issue tracker.
- **Recorded output:** pure refactoring.

### Task 18: `output/hook.rs::build` takes `Option<&SpacetimeDSLMethodHook>`

*Moved from the report* — `derive/src/output/hook.rs` › `hook.rs`: `pub fn build(hook: &Option<SpacetimeDSLMethodHook>) -> syn::Result<TokenStream>`

> **Violates:** KISS
>
> An `is_none()` check followed by `as_ref().unwrap()` spells out what `let Some(hook) = hook else` expresses, the parameter is `&Option<T>` instead of `Option<&T>`, and the function cannot fail although it returns `syn::Result`.
>
> Recommendation: `fn build(hook: Option<&SpacetimeDSLMethodHook>) -> TokenStream` with a `let … else`.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `fn build(hook: Option<&SpacetimeDSLMethodHook>) -> TokenStream` with `let Some(hook) = hook else { return TokenStream::default() };`. Adapt the call sites (Task 28 turns them into a loop).
- **Recorded output:** pure refactoring.

### Task 19: `IdentExt::unraw` replaces `rm_rsharp`

*Moved from the report* — `derive-input/src/internal/table.rs` › `table.rs`: `pub fn rm_rsharp(ident: syn::Ident) -> syn::Ident`

> **Violates:** Self-Documenting Code — no abbreviations; KISS; Maximize Cohesion
>
> The name abbreviates "remove the `r#` prefix", the function re-implements `syn::ext::IdentExt::unraw`, and it lives in the table module although `internal.rs`, `rust/table.rs` and `db/table.rs` use it for arbitrary identifiers.
>
> Recommendation: Use `IdentExt::unraw()`.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Delete `rm_rsharp`. Callers use `syn::ext::IdentExt::unraw(&ident)`.
- **Recorded output:** pure refactoring. `unraw` keeps the span that `format_ident!` replaced with the call site. If a `.stderr` moves, read it; a span now pointing at the user's identifier is an improvement to describe in the commit message.

---

## Phase 4 — The contract with the runtime

### Task 20: Every emitted path goes through `api::runtime` or `api::spacetimedb`

*Moved from the report* — `derive-input/src/api/runtime.rs` › `runtime.rs`: the module's promise that every runtime path is written here exactly once

> **Violates:** DRY; Encapsulate What Changes; Code For The Maintainer
>
> The module states that every `crate::spacetimedsl::…` path any generator emits is written here exactly once. The `Wrapper` trait nevertheless appears as `crate::spacetimedsl::Wrapper` (`wrapper_trait`), as `::spacetimedsl::Wrapper` (`wrapper_trait_path`) and as `spacetimedsl::Wrapper` (typed by hand in `derive/src/output/accessor.rs` and `internal/dsl/wrapper.rs`); `itertools_import` and `derive/src/lib.rs` use `::spacetimedsl::…`. Paths into `spacetimedb` (`spacetimedb::TryInsertError`, `spacetimedb::SpacetimeType`, `spacetimedb::{CtxDbRead, CtxDbWrite, Table}`) have no such home at all, so a SpacetimeDB rename is exactly the text hunt through `quote!` bodies the module was written to prevent. `deletion_result_entry` takes its last field with the trailing comma included, to accommodate one call site's shorthand, which bends the API to a stylistic difference.
>
> Recommendation: Route every emitted path through this module and add a counterpart for `spacetimedb` paths. The `Wrapper` spellings resolve differently in a user's crate (crate-local re-export, extern crate, relative path): understand why each one was chosen before unifying them, and pin the result with snapshots. Let `deletion_result_entry` take the child-entries expression and write the field itself.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** Declare a small table with `#[create_wrapper]` in `examples/test/src/lib.rs`, next to `::spacetimedsl::spacetimedsl!();`. Today `use spacetimedsl::Wrapper;` is ambiguous there (the local `mod spacetimedsl` versus the extern crate), so `.\x.ps1 test` fails to build. Read that error; it is the red. Give the table a group asserting its wrapper round trip.
- **Change:**
  - `api/runtime.rs`: `wrapper_trait` emits `crate::spacetimedsl::Wrapper`, and a new `wrapper_trait_import()` emits `use crate::spacetimedsl::Wrapper;`; the old `::spacetimedsl::Wrapper` goes. `itertools_import` emits `use crate::spacetimedsl::itertools::Itertools;`. Add `spacetimedsl_derive()` for `crate::spacetimedsl::SpacetimeDSL`, used by `derive/src/lib.rs::derive_table_helper_attr`; the macro re-exports the derive (Task 22 lists it in one place).
  - New public module `api/spacetimedb.rs`, the counterpart for `::spacetimedb::…`: `try_insert_error(variant)`, `spacetimetype_derive()`, `table_traits_import()` (`use ::spacetimedb::{CtxDbRead, CtxDbWrite, Table as _};`), and `uuid_nil()` (used in Task 50). Its module doc makes the same promise `runtime.rs` makes.
  - Route every hand-written path through them: `derive/src/output/accessor.rs`, `derive/src/output/function.rs`, `internal/dsl/wrapper.rs` (the `From` impls and the `SpacetimeType` derive), `internal/dsl/method/create.rs` and `upsert.rs` (`TryInsertError`). Check with `grep -rn "spacetimedsl::\|spacetimedb::" derive-input/src derive/src` that only the two modules (and diagnostics text) still spell paths.
  - `deletion_result_entry` takes the child-entries expression and writes `child_entries: #child_entries` itself. The shorthand call site passes `child_entries`.
- **Recorded output:** snapshots move in every fixture: `Wrapper` import paths, `Itertools` import, `::spacetimedb::` prefixes. Read the diff: only path spellings may change.
- **Docs:** `docs/DOCUMENTATION.md`: any hand-written example `impl spacetimedsl::Wrapper<…>` stays valid (the extern path still exists). Add a note that generated code needs `spacetimedsl!()` at the crate root, which was already required. `docs/MIGRATION.md`: "Generated code now reaches the runtime only through `crate::spacetimedsl`."

### Task 21: `Wrapper<WrappedType>` drops its unused parameter

*Moved from the report* — `src/lib.rs` › `lib.rs`: `pub trait Wrapper<WrappedType: Clone, WrapperType>`

> **Violates:** KISS; YAGNI; Code For The Maintainer — Principle of Least Astonishment
>
> No method uses the second type parameter, and every generated implementation passes the implementing type itself (`impl crate::spacetimedsl::Wrapper<u64, ThingId> for ThingId`). A reader has to work out that the parameter carries no information.
>
> Recommendation: Reduce the trait to `Wrapper<WrappedType>` and adapt `derive-input/src/api/runtime.rs::wrapper_trait`. This breaks code that spells out the second parameter, so it belongs to the next breaking release; the snapshots change accordingly.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `pub trait Wrapper<WrappedType: Clone>: …` in `src/lib.rs`. `api/runtime.rs::wrapper_trait(wrapped_type)` loses the second argument. Update `docs/DOCUMENTATION.md` (the trait listing near "Wrapper Types", and the `impl spacetimedsl::Wrapper<i32, ConfigId> for ConfigId` example) and every hand-written implementation in `examples/`.
- **Test first:** The runtime gate: a hand-written `impl Wrapper<u64, X> for X` in `examples/test` stops compiling. Update it, and assert its round trip.
- **Recorded output:** every `impl crate::spacetimedsl::Wrapper<T, Name> for Name` becomes `Wrapper<T>`. Read the diff: nothing else may change.
- **Docs:** `docs/MIGRATION.md` *Breaking API changes*: before/after of a hand-written wrapper.

### Task 22: One prelude, defined in the runtime crate

*Moved from the report* — `src/lib.rs` › `lib.rs`: `macro_rules! spacetimedsl` — re-export lists, `DSL` and `ReadOnlyDSL`

> **Violates:** DRY; YAGNI; KISS
>
> The public surface of the runtime is listed repeatedly — in the `pub use` at the crate root, in the module-level re-exports of the generated `crate::spacetimedsl`, in its "flat re-exports" and in its `prelude` — and the lists have already drifted: `OnDeleteStrategyFailure` is re-exported by the macro but not by the crate root.
>
> Recommendation: Define the prelude once in the runtime crate and let the macro re-export it, so a new runtime item is one edit.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** Use `spacetimedsl::OnDeleteStrategyFailure` from the crate root in `examples/test`, for example as a type annotation in a cascade test. It fails today because the crate root does not re-export it (red).
- **Change:**
  - `src/lib.rs`: `pub mod prelude` lists every runtime item once: the context traits, `Wrapper`, the capability traits, the delete types including `OnDeleteStrategyFailure`, the error types, `itertools::Itertools`, `NewUUID`. The crate root uses `pub use prelude::*;` next to its modules and the derive macros.
  - `macro_rules! spacetimedsl`: the flat re-exports become `pub use ::spacetimedsl::prelude::*;` plus the modules `delete`, `error`, `itertools` and the derive `SpacetimeDSL` (Task 20). The generated `prelude` is `pub use ::spacetimedsl::prelude::*;` plus the generated items (`DSL`, `DSLMethodHooks`, `DefaultSingleton`, `ReadOnlyDSL`, `dsl`, `read_only_dsl`) plus the one list of SpacetimeDB convenience re-exports.
- **Recorded output:** unchanged (the macro is not snapshotted). The runtime gate is the check.
- **Docs:** `docs/DOCUMENTATION.md` where it lists what the prelude contains. `docs/MIGRATION.md`: the crate root now also re-exports the capability traits and `OnDeleteStrategyFailure`, which is additive.

### Task 23: `ReferenceIntegrityViolationError::OnCreateOrUpdate` cannot hold a wrong action

*Moved from the report* — `src/error.rs` › `error.rs`: `impl Display for SpacetimeDSLError`

> **Violates:** Robustness Principle; Code For The Maintainer — Principle of Least Astonishment; KISS; Self-Documenting Code — descriptive identifiers
>
> Formatting an error can panic: `ReferenceIntegrityViolationError::OnCreateOrUpdate` stores a general `Action`, and `Display` panics for `Get`, `Delete` and `SoftDelete`. The panic exists only because the field can hold states the variant forbids. The message is built into an intermediate `String` before it is written, and the local `dig_spacetimedb` does not say what it holds.
>
> Recommendation: Make the illegal states unrepresentable with a dedicated type for `create_or_update` that has only the create and update cases; that removes the panic. Write to the formatter directly and name the local after its content. Changing the field type breaks the public API, so it belongs to the next breaking release.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** In a reference-integrity test group, match the returned error on `create_or_update: CreateOrUpdate::Create` (and `::Update` for the update path), and assert its `Display` text. It does not compile yet (red).
- **Change:** `src/error.rs`: `pub enum CreateOrUpdate { Create, Update }` with `Display` (`create` / `update`), used as the field type, so the `panic!` arm goes. Every arm of `Display for SpacetimeDSLError` writes to the formatter directly (`write!(f, …)`), without the intermediate `String`. Rename `dig_spacetimedb` to `spacetimedb_gives_no_details`. `api/runtime.rs::reference_integrity_violation_on_create_or_update` emits `crate::spacetimedsl::error::CreateOrUpdate::#variant`.
- **Recorded output:** snapshots of every method with a reference-integrity check change `error::Action::Create` / `::Update` to `error::CreateOrUpdate::…` in that one field. Nothing else may move.
- **Docs:** `docs/MIGRATION.md` *Breaking API changes*: the field type.

### Task 24: `Display for OnDeleteStrategy` next to its enum; every emitted variant is snapshotted

*Moved from the report (partially)* — `src/error.rs` › `error.rs`: `pub enum Action`, `pub enum OneOrMultiple` and `impl Display for OnDeleteStrategy`. The part moved here:

> `Display for OnDeleteStrategy` lives here, away from its enum in `delete.rs`.
>
> Recommendation: Move `Display for OnDeleteStrategy` next to its enum. […] at the very least make sure every emitted variant appears in a snapshot fixture, so a rename shows up as a snapshot diff.
>
> Developer's decision (taken while planning): resolve this part; the mirrors of `Action` and `OneOrMultiple` stay in the report.

- **Change:** Move the `impl Display for OnDeleteStrategy` to `src/delete.rs`. List every `error::Action::…`, `error::ErrorFrom::…`, `error::OneOrMultiple::…` and `OnDeleteStrategy::…` variant the generator can emit (`grep` the generators for the `runtime::` constructors that take a variant). For each variant, find a snapshot containing it (`grep -rl "OneOrMultiple::Multiple" derive/tests/snapshots`). Add a fixture for any variant without one.
- **Test first:** For every added fixture, the red is the missing snapshot (the new test fails with "new snapshot"). Read the new snapshot and accept it.
- **Recorded output:** new snapshot directories only.

---

## Phase 5 — Parsing the `#[dsl]` attribute

### Task 25: One helper recognises attributes by path

*Moved from the report* — `derive-input/src/internal/integration.rs` › `integration.rs`: `fn get_all_table_attributes(input: &DeriveInput) -> syn::Result<Vec<(TableArgs, ColumnArgs)>>`

> **Violates:** Robustness Principle; DRY; Self-Documenting Code — no redundant comments
>
> Attributes are recognised by comparing their stringified path with `"table"` and `"spacetimedb :: table"`, which misses `::spacetimedb::table`; `derive/src/lib.rs` and `derive/src/characterization_tests.rs` recognise `#[dsl]` the same way. Comments such as "Find all table attributes" repeat the code.
>
> Recommendation: One helper that recognises an attribute by the last segment of its path (with an optional crate prefix), shared by all these places; delete the comments that repeat the code.
>
> Developer's decision: The recommendation should be applied.

*Also moved (partially) — `derive/src/lib.rs` › `is_last_dsl_attribute`:* "The dead function also recognises the attribute by comparing its stringified path with `"dsl"` and `"spacetimedsl :: dsl"`, which misses spellings such as `::spacetimedsl::dsl`." Only this sentence; whether the dead function stays is undecided and stays in the report.

- **Test first (snapshot):** New fixture `absolute_attribute_paths.rs` with `#[::spacetimedsl::dsl(…)]` over `#[::spacetimedb::table(…)]`. Red: the harness counts no `#[dsl]` ("should contain at least one struct") and the parser finds no table. Register it (Task 5 enforces that).
- **Change:** New public module `derive-input/src/api/attribute.rs` with `pub fn is_dsl_attribute(&Attribute) -> bool` and a crate-private `is_table_attribute`, both built on one function that accepts a path whose last segment is the name and which is either bare or prefixed by the crate name, with or without a leading `::`. Use it in `integration.rs`, `derive/src/lib.rs::is_last_dsl_attribute` and `derive/src/characterization_tests.rs::is_dsl_attribute`. Delete the comments that repeat the code ("Find all table attributes", …).
- **Recorded output:** only the new fixture's snapshots.

### Task 26: A singleton has no plural name to fake

*Moved from the report* — `derive-input/src/internal.rs` › `internal.rs`: the placeholder plural name `__singleton_placeholder`

> **Violates:** Connascence — of Execution; Coupling Awareness — "Anticipate maintenance; avoid hidden coupling or magic"
>
> For a singleton, `try_parse_dsl` returns a fabricated plural name, and `try_parse` overwrites it with the table accessor afterwards. In between, `DSLData::plural_name` holds a magic value, and that value is handed to the table selection in `integration.rs`.
>
> Recommendation: Model the difference in the type — an enum that distinguishes a singleton from a table with a plural name — so no code can read a placeholder.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `DSLData` holds `kind: DSLTableKind`, with `enum DSLTableKind { Singleton(SingletonKind), Table { plural_name: Ident } }`, replacing `singleton` and `plural_name`. `try_parse_dsl` no longer fabricates `__singleton_placeholder`. `internal::try_parse` derives `SpacetimeDSLTable::plural_name` for a singleton from the selected table's accessor, once, where the public field is filled. The integration layer no longer receives a plural name (Task 31 removes the heuristic anyway; if Task 31 comes later, pass the `Table` variant's name through).
- **Recorded output:** pure refactoring.

### Task 27: Parse the `#[dsl]` arguments, then validate them

*Moved from the report* — `derive-input/src/internal.rs` › `internal.rs`: `fn try_parse_dsl(args: &proc_macro2::TokenStream) -> syn::Result<DSLData>`

> **Violates:** Single Responsibility Principle (SRP); Curly's Law; DRY; Self-Documenting Code — no redundant or misleading comments
>
> One function parses every `#[dsl(...)]` argument and validates how they combine. The `before(...)` and `after(...)` branches are copies that differ only in the variables they write, and each hook travels as its own `Option<Span>` local and then as its own `bool` in `DSLData` (see the hook matrix in `internal/dsl/hook.rs`). The comment above the function says it parses `plural_name`, which is a small part of what it does, and the comment in `try_parse` about passing the parsed `plural_name` refers to an argument that is not passed.
>
> Recommendation: Separate parsing from validation, and parse hooks with one helper that serves `before` and `after` and fills a structure keyed by timing and operation. Correct the comments. The diagnostics are pinned by compile-tests, so the `.stderr` files must stay unchanged.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - Split `try_parse_dsl` into `parse_dsl_arguments(args) -> ParsedDSLArguments` (raw values and their spans, no rules) and `validate(parsed, args) -> syn::Result<DSLData>` (every combination rule). **Keep the order of the checks exactly as it is today**, so the first diagnostic a multi-error input gets does not change.
  - One helper `parse_hooks_of_timing(meta, timing, &mut declared_hook_spans)` serves `before(...)` and `after(...)`. It fills a `BTreeMap<HookKind, Span>` (`HookKind` is introduced here, in `api::dsl::hook`, as Task 28 describes).
  - Correct the comment above the function (it parses every `#[dsl]` argument, not only `plural_name`) and the comment in `try_parse` about passing the plural name.
- **Recorded output:** pure refactoring. **The `.stderr` files must stay unchanged.**

### Task 28: The hook matrix becomes a collection keyed by timing and operation

*Moved from the report* — `derive-input/src/internal/dsl/hook.rs` › `hook.rs`: the hook matrix — `pub fn build(singular_table_name, singleton, declared: DeclaredHooks) -> SpacetimeDSLMethodHooks` and `pub struct DeclaredHooks`

> **Violates:** DRY; Open/Closed; Encapsulate What Changes
>
> The combinations of timing and operation are written out by hand: one `build_any` call per hook here, one field per hook in `DeclaredHooks`, in `SpacetimeDSLMethodHooks` and in `DSLData`, one `Option<Span>` local and copied parser branches in `internal.rs::try_parse_dsl`, and one `hook::build` call per hook in `derive/src/output.rs`. Adding the soft-delete operation had to touch every one of these places, and the struct literal in `build` lists its fields in a different order than their declaration.
>
> Recommendation: Represent the declared hooks as a collection keyed by `(Timing, Operation)` with an iterator over all kinds, so that parsing, building and emitting loop over the kinds; keep named accessors where a generator asks for one specific hook. Pure refactoring.
>
> Developer's decision: The recommendation should be applied.

*Also moved (partially) — `derive/src/output.rs` › `build`:* "Every hook is fetched by field name (`hooks.before_insert` … `hooks.after_soft_delete`), so a new hook kind means editing this function (see the hook matrix in `derive-input/src/internal/dsl/hook.rs`)."

- **Change:**
  - `api/dsl/hook.rs`: public `enum Timing { Before, After }`, `enum Operation { Insert, Update, Delete, SoftDelete }` (moved from `internal`, same declaration order), `struct HookKind { pub timing, pub operation }` deriving `Ord`, `HookKind::ALL: [HookKind; 8]`, and named constants for the kinds generators ask for (`HookKind::BEFORE_UPDATE`, …). `SpacetimeDSLMethodHooks { pub declared: BTreeMap<HookKind, SpacetimeDSLMethodHook> }` with `get(HookKind) -> Option<&SpacetimeDSLMethodHook>` and `iter()`.
  - `DeclaredHooks` becomes `BTreeSet<HookKind>`. `hook::build` loops over the declared kinds. `DSLData` carries the set (from Task 27's spans).
  - Every generator that reads `hooks.before_update` and similar uses `hooks.get(HookKind::BEFORE_UPDATE)`.
  - `derive/src/output.rs::build` emits `hooks.iter().map(hook::build)`.
  - Update the Task 13 contract test.
- **Recorded output:** pure refactoring. The derived `Ord` (`Before < After`, `Insert < Update < Delete < SoftDelete`) is exactly today's emission order; an order diff means the declaration order is wrong.
- **Docs:** `docs/MIGRATION.md` (derive-input consumers): hooks are a map keyed by `HookKind`.

### Task 29: `#[hook]` and the generator share the hook trait name rule

*Moved from the report* — `derive/src/lib.rs` › `lib.rs`: `pub fn hook(_args: TokenStream, item: TokenStream) -> TokenStream`

> **Violates:** Connascence — of Algorithm, across crates; DRY
>
> `#[hook]` derives the trait to implement as the PascalCase form of the function name followed by `Hook`, while `derive-input/src/internal/dsl/hook.rs` builds the same trait name from timing, table and operation. The algorithms agree only implicitly, and a change to either one breaks every hook in user code.
>
> Recommendation: Expose one function from `derive-input` that maps a hook function name to its trait name, use it on both sides, and cover a table name with digits and underscores in a runtime hook.
>
> Developer's decision: The recommendation should be applied.

- **Test first (runtime):** A table whose accessor has digits and underscores (for example `hook_table_2_b`), with `hook(before(insert))` and a `#[spacetimedsl::hook]` function, plus a group asserting the hook ran. Both algorithms agree today, so it is green from the start. Observe the guard by temporarily changing the suffix in `derive/src/lib.rs::hook` (it fails to compile), then revert.
- **Change:** `api::dsl::hook::hook_trait_name(hook_function_name: &Ident) -> Ident` (PascalCase of the function name plus `Hook`). `internal/dsl/hook.rs::build_any` builds the function name first and derives the trait name from it. `#[hook]` calls the same function. Delete `get_trait_name`.
- **Recorded output:** pure refactoring.

### Task 30: One naming function for `Create<Table>`

*Moved from the report* — `derive-input/src/internal/dsl/hook.rs` › `hook.rs`: `fn get_function_args(...)` and `fn get_return_type(...)` — the `Create<Table>` name

> **Violates:** DRY — the code's own FIXME calls it a "Single Source of Truth Violation"
>
> The name of the create-request struct is formatted here, in the arguments and again in the return type, and in `internal/dsl/method/create.rs`, which defines the struct. The hook signature and the struct agree only by convention.
>
> Recommendation: One naming function in `internal/dsl/method/naming.rs`; the FIXME goes with the fix.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `naming::create_request_struct_name(singular_table_name) -> Ident`, used by the hook arguments, the hook return type and `create.rs`. Delete the FIXME.
- **Recorded output:** pure refactoring.

### Task 31: An explicit table selector replaces the plural-name heuristics

*Moved from the report* — `derive-input/src/internal/integration.rs` › `integration.rs`: `fn select_table_with_heuristics(input: &DeriveInput, plural_name: &syn::Ident) -> syn::Result<(TableArgs, ColumnArgs)>`

> **Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"; Code For The Maintainer — Principle of Least Astonishment; KISS; Coupling Awareness — "avoid hidden coupling or magic"
>
> When a struct carries several `#[table]` attributes, `plural_name` doubles as a table selector: exact match first, then "intelligent" matching, then a fallback that sums the characters of the plural name modulo the number of tables — so an arbitrary table is used instead of an error. The substring rule accepts the first table whose name the plural contains, so `offline_players` selects a table named `player` if that `#[table]` comes first. `plural_to_singular` strips `es` from every word that ends in it (`tables` becomes `tabl`); a special case for exactly `tables` patches that, although the substring rule would accept it anyway. `docs/DOCUMENTATION.md` describes none of this. The comments call the fallback "index-based" and a "hash of plural name"; it is neither. The `all_tables.is_empty()` branch can never be taken (see `internal/error.rs`), and the entry function's name, `spacetime_bindings_macro_input`, is a noun for an operation that selects a table.
>
> Recommendation: Replace guessing with a documented rule. This changes which inputs are accepted, so the developers decide. Options: (a) the `#[table]` directly below a `#[dsl]` belongs to it; (b) an explicit selector such as `#[dsl(table = accessor)]` whenever a struct has several `#[table]` attributes; (c) keep the plural-name matching but reject an ambiguous or unmatched name with a diagnostic instead of the fallback. Pin the ambiguous case with a compile-test and the chosen resolution with a snapshot, and document the rule in `docs/DOCUMENTATION.md`. Give the entry function a name that says it selects the table.
>
> Developer's decision: The recommendation (b) should be applied.

*Also moved (partially) — `derive-input/src/internal/error.rs` › `no_table_attribute_found` and `update_method_disabled_with_set_on_update_column`:* the half about `no_table_attribute_found` ("only raised from branches that cannot be reached (see `internal/integration.rs`) …"); resolved here because the branch disappears with the heuristic.

- **Test first (diagnostics, one file per rejection):**
  - `table_selector_missing_with_multiple_table_attributes`: two `#[table]`s, no `table =`. The message names both accessors and the fix `#[dsl(table = <accessor>)]`.
  - `table_selector_names_no_table`: `table = gizmo` with accessors `gadget1`, `gadget2`, and with a single `#[table]` whose accessor differs.
  - `table_selector_repeated`: `table =` twice (`check_duplicate`).

  Red: today each is accepted, or selects silently.
- **Change:**
  - Parse `table = <Ident>` in Task 27's parser. `integration.rs::spacetime_bindings_macro_input` becomes `select_table_attribute(item, table_selector: Option<&Ident>, is_singleton)`, following the decided rule: selector required iff ≥ 2 `#[table]`; with one it is optional but must match; singletons keep "exactly one `#[table]`" and may carry a matching selector.
  - Delete `select_table_with_heuristics`, `is_plural_match`, `plural_to_singular`, `remove_trailing_digits`, `deterministic_selection_by_name`, the unreachable `all_tables.is_empty()` branch and `error::no_table_attribute_found`.
  - Reword `error::missing_table_attribute`, which says `#[dsl]` must be directly above a `#[table]`; that is no longer the rule.
  - Add `table = …` to the fixtures `multiple_table_attributes` (correct its `//!`: it now pins the explicit selector) and `multiple_dsl_attributes`, and to `examples/test/src/component/test.rs::Module`.
- **Recorded output:** the new `.stderr` files; the snapshots of the two fixtures stay identical.
- **Docs:** `docs/DOCUMENTATION.md` *Multiple `#[spacetimedsl::dsl]` + `#[spacetimedb::table]` on Same Struct*: state the rule and add `table = offline_player` / `table = online_player` to the example; document `table` among the `#[dsl]` arguments. `docs/MIGRATION.md` *Newly rejected inputs*: a struct with several `#[table]` attributes needs `table =` on each `#[dsl]`.

### Task 32: `unique_index` rejects unknown and repeated names

*Moved from the report* — `derive-input/src/internal/dsl/table.rs` › `table.rs`: the names in `DSLData::unique_indices`

> **Violates:** Robustness Principle — "Accept unknown inputs only when semantics remain clear"
>
> `#[dsl(unique_index(name = x))]` A misspelled name is accepted without effect and without a diagnostic, and a repeated name is not detected.
>
> Recommendation: Reject names that do not match an eligible index with a compile-test.
>
> Developer's decision: The recommendation should be applied.

- **Decision taken while planning:** "eligible" means *declared in `#[table]`*. A name of a hash or single-column index stays accepted without effect; see the report entry on the types in `DSLData::unique_indices`.
- **Test first (diagnostics):** `unique_index_with_unknown_name` (a misspelled name; the message lists the declared index accessors) and `unique_index_repeated`. Red: both are accepted today.
- **Change:** After the table is selected, in `internal::try_parse` next to `reject_unique_index_on_singleton` (which keeps running first, so its `.stderr` stays): `error::unknown_unique_index(name, declared)` and `error::repeated_unique_index(name)`, underlining the name.
- **Docs:** `docs/DOCUMENTATION.md` where `unique_index` is described. `docs/MIGRATION.md` *Newly rejected inputs*.

### Task 33: One list of field attributes

*Moved from the report* — `derive/src/lib.rs` › `lib.rs`: `pub fn table_helper(_input) -> TokenStream` and `fn derive_table_helper_attr() -> syn::Attribute`

> **Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"; KISS
>
> The helper-attribute list of the `SpacetimeDSL` derive is one more spelling of the field-attribute vocabulary, which `derive-input` recognises through `symbol!` declarations in `internal/dsl.rs` for some attributes and through string literals (`is_ident("set_on_create")`, `"set_on_update"`, `"set_on_soft_delete"`) for others. A new attribute has to be added in several places, and a forgotten entry surfaces as an unknown-attribute error in user code. `derive_table_helper_attr` parses a constant with `unwrap()` calls where `syn::parse_quote!` says the same.
>
> Recommendation: Give the attribute names one list in `derive-input` that all of its parsers use. `proc_macro_derive(attributes(...))` needs literal identifiers, so the helper list cannot be generated from that list; pin their agreement instead, for example with a snapshot fixture that uses every field attribute on an accepted table. Use `syn::parse_quote!`. Keep the reason for the helper derive as a comment.
>
> Developer's decision: The recommendation should be applied.

- **Test first:**
  - Snapshot: fixture `every_field_attribute.rs`, accepted tables that together use `create_wrapper`, `use_wrapper`, `foreign_key`, `referenced_by`, `set_on_create`, `set_on_update`, `set_on_soft_delete` and `auto_gen`.
  - Derive test (the snapshot harness never compiles, so a fixture cannot notice a missing helper attribute): `helper_attributes_match_field_attributes` in `derive/src/characterization_tests.rs` parses `include_str!("lib.rs")`, reads the `attributes(...)` list of `#[proc_macro_derive(SpacetimeDSL, …)]`, and compares it with `spacetimedsl_derive_input::api::attribute::FIELD_ATTRIBUTE_NAMES`. Red: the constant does not exist yet. Then observe it fail with one name removed from either side.
- **Change:** `api/attribute.rs` (from Task 25) gets `FIELD_ATTRIBUTE_NAMES`. The string literals `is_ident("set_on_create")`, `"set_on_update"`, `"set_on_soft_delete"` become `symbol!` declarations in `internal/dsl.rs`, next to the others. `derive_table_helper_attr` uses `syn::parse_quote!`. The TODO above the helper derive becomes a plain comment stating why the derive exists (attribute macros cannot declare helper attributes), and the rust-lang issue link and the `PartialOrd` TODO go to the issue tracker.
- **Recorded output:** only the new fixture's snapshots.

### Task 34: Diagnostics print visibilities as written

*Moved from the report* — `derive-input/src/internal/error.rs` › `error.rs`: `fn visibility_variant_name(visibility: &Visibility) -> &'static str`

> **Violates:** Hide Implementation Details; Code For The Maintainer — Principle of Least Astonishment
>
> User-facing messages print names of `syn` types — a column should have `Visibility::Inherited`, found `Visibility::Public` — instead of Rust syntax, while neighbouring messages print the visibility as the user wrote it.
>
> Recommendation: Render the visibility as written (`pub`, `pub(crate)`, no modifier) and regenerate the affected `.stderr` files.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Replace `visibility_variant_name` with a renderer: `pub`, `pub(crate)`, `pub(in …)` from `to_token_stream()`, and "no visibility modifier" for `Inherited`.
- **Test first:** Regenerate with `TRYBUILD=overwrite`, read the diff, and check that only the visibility wording moved (`created_at_attribute_public`, `updated_at_attribute_restricted`, `public_marker_column`, `auto_gen_public`, …).
- **Docs:** `docs/MIGRATION.md` *Changed messages* (one line).

### Task 35: The missing-update-method message names `#[set_on_update]`

*Moved from the report* — `derive-input/src/internal/error.rs` › `error.rs`: `pub fn missing_update_method_with_only_private_columns(struct_name: &Ident) -> Error`

> **Violates:** Code For The Maintainer; DRY — "Sync related artifacts—code, docs, tests—whenever knowledge changes"
>
> The message tells users that a mutable table needs a non-private column or one named `modified_at` / `updated_at`, without mentioning `#[set_on_update]`, which has the same effect.
>
> Recommendation: Name the attribute next to the conventional names and regenerate the `.stderr` files.
>
> Developer's decision: The recommendation should be applied.

- **Test first (diagnostics):** `update_method_missing_with_only_private_columns` (no `.stderr` pins this message yet). Write the `.stderr` with the new wording, which names `#[set_on_update]` next to `modified_at` / `updated_at`. The red is the old wording.
- **Change:** Reword `error::missing_update_method_with_only_private_columns`. After Task 38, build the listed names from the role constants.

---

## Phase 6 — Types and conventional names

### Task 36: One type classification for every type check

*Moved from the report* — `derive-input/src/internal/column.rs` › `column.rs`: `ColumnTypeKind::of(type_name_or_path: &Path) -> ColumnTypeKind` and the string-based type checks elsewhere

> **Violates:** DRY — "One authoritative source for each business rule"; Robustness Principle
>
> `ColumnTypeKind::of` classifies a column's type by its path and accepts qualified spellings. Timestamps and flags, however, are checked by comparing stringified tokens (such as `"Option < spacetimedb :: Timestamp >"`) in `internal/dsl/table.rs` and `internal/dsl/soft_delete.rs`, the singleton key type in `internal/dsl/singleton.rs`, and foreign-key type equality in `internal/dsl/method/foreign_key.rs`. The accepted spellings therefore differ from check to check: `::spacetimedb::Timestamp` and `std::option::Option<Timestamp>` fail the string checks, although `ColumnTypeKind` treats a qualified `Option` as optional.
>
> Recommendation: Extend the classification (for example with timestamp, optional timestamp and flag kinds) and use it for every type check. Extend the `qualified_type_spellings` fixture and add compile-tests for qualified spellings before switching.
>
> Developer's decision: The recommendation should be applied.

*Also moved (partially) — `derive-input/src/internal/dsl/singleton.rs` › `is_primary_key_column`:* "`is_primary_key_column` compares the type as text with `"u8"`."

- **Test first:**
  - Snapshot: extend `qualified_type_spellings.rs` with `::spacetimedb::Timestamp` (set-on-create), `std::option::Option<spacetimedb::Timestamp>` (set-on-update), `core::option::Option<::spacetimedb::Timestamp>` (soft-delete marker), `core::primitive::bool` (soft-delete flag in a second struct) and `core::primitive::u64` columns. Red: the expansion fails with a type-mismatch diagnostic.
  - Diagnostics: `set_on_update_column_with_qualified_wrong_type` (`std::option::Option<u64>`), which must name the expected types exactly as today.
- **Change (`internal/column.rs`):**
  - `ColumnTypeKind` gains `Bool` and `Timestamp`.
  - `UnsignedInteger` also matches `core::primitive::uN` / `std::primitive::uN`, and every std path may start with `::`.
  - `Timestamp` and `UUID` match bare, `spacetimedb::X` and `::spacetimedb::X`.
  - Add `ColumnTypeKind::of_option_argument(&Path) -> Option<ColumnTypeKind>` (the kind of `T` in `Option<T>`), and `canonical_type(&Path) -> String` (known types reduced to their bare name, generic arguments canonicalised recursively, everything else as written without a leading `::`), used by Task 51 and by `singleton::is_primary_key_column` (`canonical_type == "u8"`).
  - Correct the doc that says a qualified primitive is a different type.
  - Replace `is_bare_timestamp_type`, `is_optional_timestamp_type` and the inline `field_type.eq("…")` chains in `internal/dsl/table.rs` with the classification.
- **Recorded output:** new snapshots of the extended fixture and the new `.stderr`. Every other file unchanged.
- **Docs:** `docs/DOCUMENTATION.md`: where column types are listed, state the accepted spellings once. `docs/MIGRATION.md`: previously rejected qualified spellings are accepted (not breaking).

### Task 37: Soft-delete markers use the classification

*Moved from the report* — `derive-input/src/internal/dsl/soft_delete.rs` › `soft_delete.rs`: `fn claimed_kind(field: &SatsField<'_>) -> syn::Result<Option<SoftDeleteMarkerKind>>` and `fn type_fits(kind: SoftDeleteMarkerKind, field_type: &str) -> bool`

> **Violates:** DRY
>
> The marker's type is checked by comparing stringified tokens, restating `is_optional_timestamp_type` from `internal/dsl/table.rs`, and the conventional names live in constants while the timestamp roles use inline strings.
>
> Recommendation: Resolve together with the type classification in `internal/column.rs` and the role names in `internal/dsl/table.rs`.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `type_fits` and `claimed_kind` take the field's `Path` (from `RustField`, or parsed once) and ask `ColumnTypeKind`: `Flag` ↔ `Bool`, `Timestamp` ↔ `of_option_argument == Some(Timestamp)`. The conventional names come from Task 38.
- **Test first:** The qualified marker spellings from Task 36's fixture are red until this task. Commit Task 36 with that struct commented out of the fixture and restore it here, or order Task 37 before Task 36's commit; the red has to be observed either way.
- **Recorded output:** the snapshots of the marker struct; the `.stderr` of `marker_column_with_wrong_type` and `set_on_soft_delete_attribute_wrong_type` unchanged.

### Task 38: The conventional column names live in one module

*Moved from the report* — `derive-input/src/internal/dsl/table.rs` › `table.rs`: `fn get_timestamp_role(field: &SatsField<'_>) -> syn::Result<Option<TimestampRole>>`

> **Violates:** DRY; Coupling Awareness — "avoid hidden coupling or magic"; KISS
>
> Column names claim framework roles implicitly — `created_at`, `inserted_at`, `modified_at` and `updated_at` here, `deleted`, `removed`, `deleted_at` and `removed_at` in `internal/dsl/soft_delete.rs`. The behaviour is documented, but the timestamp roles and the soft-delete roles use different mechanisms (inline string comparisons here, constant arrays there), so renaming a role or adding one follows no single pattern. `(true, true) => unreachable!()` restates a case the early return already excluded.
>
> Recommendation: Keep the conventional names of all roles as constants in one module and look them up the same way.
>
> Developer's decision: The recommendation should be applied.

- **Change:** New `internal/dsl/column_role.rs` with `SET_ON_CREATE_COLUMN_NAMES = ["created_at", "inserted_at"]`, `SET_ON_UPDATE_COLUMN_NAMES = ["modified_at", "updated_at"]`, `SOFT_DELETE_FLAG_COLUMN_NAMES = ["deleted", "removed"]`, `SOFT_DELETE_TIMESTAMP_COLUMN_NAMES = ["deleted_at", "removed_at"]`, and one `fn claims(names, column_name) -> bool`. `get_timestamp_role` and `soft_delete.rs::claimed_kind` look up the same way. Delete the `(true, true) => unreachable!()` arm (a `match` over `(bool, bool)` returning `None` for `(false, false)` after the early return, or an `if` chain). Diagnostics listing these names (`soft_delete_method_without_marker_column`, `missing_update_method_with_only_private_columns`) render the lists from the constants, in the same words.
- **Recorded output:** pure refactoring; the `.stderr` files unchanged.

### Task 39: Array, tuple, reference and other non-path column types get a diagnostic

*Moved from the report* — `derive-input/src/internal/rust/column.rs` › `column.rs`: `RustField::map(field: &SatsField<'_>) -> RustField`

> **Violates:** Robustness Principle
>
> The field type is re-parsed as a `Path` with an `expect`, so a field whose type is not a path (an array, a tuple or a reference) makes the macro panic. `#[dsl]` expands before `#[table]`, so the user sees a panic from SpacetimeDSL instead of a spanned diagnostic from either crate. `internal/dsl/wrapper.rs` re-parses types the same way.
>
> Recommendation: The developers decide which column types SpacetimeDSL supports. For unsupported ones, return a spanned error built in `internal/error.rs` (compile-test first: red is the panic, green the diagnostic); for supported ones, keep a `syn::Type` instead of a `Path`.
>
> Developer's decision: Arrays, tuples and references are unsupported. The recommendation should be applied.

- **Test first (diagnostics):** `column_with_array_type` (`[u8; 4]`), `column_with_tuple_type` (`(u8, u8)`), `column_with_reference_type` (`&'static str`). Red: `proc macro panicked … should be parseable as Path`. Read it.
- **Change:** `RustField::map` returns `syn::Result<RustField>`. It unwraps `Type::Group` and `Type::Paren`, accepts `Type::Path` without `qself`, and rejects everything else through one `error::unsupported_column_type(ty)` whose message names the kind (array, tuple, reference, slice, function pointer, trait object, `impl Trait`, never type, qualified path) and says only path types are supported. Validate every field at the start of `internal/table.rs::try_parse`, before any other column analysis, so this diagnostic comes first.
- **Docs:** `docs/DOCUMENTATION.md`: supported column types are path types. `docs/MIGRATION.md`: these inputs used to panic and now get a diagnostic.

---

## Phase 7 — Indices and columns

Order within the phase: 42 → 43 → 40 → 41 → 44.

### Task 42: Assign indices with queries and build `SpacetimeDBTable` once

*Moved from the report* — `derive-input/src/internal/table.rs` › `table.rs`: `pub fn try_parse(input, dsl_data, table_args, column_args) -> syn::Result<Table>`

> **Violates:** Command Query Separation (CQS); Connascence — of Execution; KISS
>
> `SpacetimeDBTable` is moved into `SpacetimeDSLTable::try_parse` and into `column::try_parse`, and each returns it changed: the first sets `is_unique` on indices, the second removes the single-column indices. Each of these internal functions is a command and a query at once, and the order of the calls carries meaning nothing states.
>
> Recommendation: Compute the index assignment — which index belongs to which column, which multi-column indices the DSL treats as unique — with queries that return new values, then assemble `SpacetimeDBTable` once. Pure refactoring.
>
> Developer's decision: The recommendation should be applied.

- **Change:** In `internal/table.rs`:
  - `fn assign_indices(table_args, column_names) -> IndexAssignment { single_column_index_by_column: BTreeMap<Ident, Index>, multi_column_indices: Vec<Index> }`, a pure query that keeps declaration order.
  - `fn dsl_unique_index_names(dsl_data, &multi_column_indices) -> BTreeSet<Ident>`.
  - Then `SpacetimeDBTable::map(table_args, &assignment, &dsl_unique_index_names)` is assembled once.
  - `SpacetimeDSLTable::try_parse` and `column::try_parse` no longer take or return a `SpacetimeDBTable`, and `SpacetimeDBColumn::map` receives its assigned index.
- **Recorded output:** pure refactoring. The old `swap_remove` reordered the remaining indices; if a snapshot's `internal_methods.snap` or method order moves, check that only order moved and that declaration order is now used, and describe it in the commit message.

### Task 43: `column::try_parse` returns a named result

*Moved from the report* — `derive-input/src/internal/column.rs` › `column.rs`: `pub fn try_parse(column_args, rust_struct, spacetimedb_table, spacetimedsl_table) -> syn::Result<(SpacetimeDBTable, Vec<Column>, Column, Vec<InternalColumn>, InternalColumn)>`

> **Violates:** Connascence — of Position; KISS; Self-Documenting Code — no abbreviations
>
> The result is a positional tuple kept behind `#[allow(clippy::type_complexity)]`. The primary key column is searched for in the column list and again in the list of internal columns, by comparing the `to_string()` of identifiers, which can be compared directly; `res.0`, `res.1` and the message "PK column should be present" use abbreviations.
>
> Recommendation: A named result struct; compare identifiers directly; find the primary key once.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `struct AnalysedColumns { columns, primary_key_column, internal_columns, internal_primary_key_column }` (no `SpacetimeDBTable` after Task 42). Find the primary key once, by its position in `column_args.fields`, comparing `Ident`s directly. Remove `#[allow(clippy::type_complexity)]`, `res.0` / `res.1` and "PK column should be present" (the `expect` states the invariant: "`ColumnArgs` guarantees the primary key is one of the fields").
- **Recorded output:** pure refactoring.

### Task 40: `multi_column_indices` holds what its name says; the singleton rule moves to the DSL layer

*Moved from the report* — `derive-input/src/internal/db/table.rs` › `table.rs`: `SpacetimeDBTable::map(table: &TableArgs, is_singleton: bool) -> syn::Result<SpacetimeDBTable>` and the field `multi_column_indices`

> **Violates:** Code For The Maintainer — Principle of Least Astonishment; Connascence — of Execution; Separation of Concerns
>
> The comment on the field admits that `multi_column_indices` contains all indices during processing; the name only becomes true once the columns have taken their indices. The mapping also enforces a DSL rule (no multi-column index on a singleton).
>
> Recommendation: Keep all indices in an accurately named collection while processing (or split them up front, see `internal/table.rs`), and move the singleton rule to the DSL layer.
>
> Developer's decision: The recommendation should be applied.

- **Change:** After Task 42, `multi_column_indices` is built from `IndexAssignment::multi_column_indices`; delete the comment admitting it holds all indices. Move the "no multi-column index on a singleton" check into `internal/dsl/singleton.rs::reject_multi_column_indices(&assignment)`, called at the same point of `internal::try_parse` that raises it today, so the order of diagnostics holds.
- **Recorded output:** pure refactoring. `multi_column_index_on_singleton.stderr` unchanged.

### Task 41: Column validation leaves the SpacetimeDB mapping; a second single-column index is rejected

*Moved from the report* — `derive-input/src/internal/db/column.rs` › `column.rs`: `SpacetimeDBColumn::map(rust_field, spacetimedb_table, auto_inc_column_names, primary_key_column_name, is_singleton) -> syn::Result<(SpacetimeDBTable, SpacetimeDBColumn)>`

> **Violates:** Separation of Concerns; Command Query Separation (CQS); DRY; Robustness Principle
>
> The mapping of the SpacetimeDB column also enforces DSL rules (no primary key prefixed with the table name, no index on a singleton's column). It walks the indices with the same `match` for the validation and again for the lookup. The lookup stops at the first single-column index, so a column with more than one single-column index leaves the others in `multi_column_indices`, where `SpacetimeDSLTableMethods::generate` has to skip them — silently, without a diagnostic and without methods.
>
> Recommendation: Move the DSL validation to the DSL layer and return the index assignment as data (see `internal/table.rs`). An additional single-column index on one column must be rejected with a diagnostic. Pin that with a compile-test.
>
> Developer's decision: The recommendation should be applied.

- **Test first (diagnostics):** `column_with_two_single_column_indices`: `#[unique]` on a column that also has `index(accessor = …, btree(columns = [that_column]))`. Red: accepted today, the second index silently without methods. First run `grep` over `examples/` and the fixtures for a column carrying two single-column indices (`#[primary_key]` together with a single-column `index(...)` counts). If one exists, stop and ask the developer before continuing.
- **Change:**
  - `assign_indices` (Task 42) returns an error when a column gets a second single-column index: `error::multiple_single_column_indices_on_column(column, first, second)`, underlining the second index's accessor.
  - Move `primary_key_prefixed_with_table_name` and `single_column_index_on_singleton` out of `SpacetimeDBColumn::map` into the DSL layer (`internal/dsl/column.rs` validators and `internal/dsl/singleton.rs`), called in today's order.
  - The one index walk remains in `assign_indices`, so `SpacetimeDBColumn::map` stays a pure mapping. `SpacetimeDSLTableMethods::generate` no longer needs to skip single-column entries in `multi_column_indices`; delete that skip.
- **Docs:** `docs/MIGRATION.md` *Newly rejected inputs*.

### Task 44: `ScheduledReducer` keeps the path

*Moved from the report* — `derive-input/src/internal/db/table.rs` › `table.rs`: `ScheduledReducer::map(scheduled: &ScheduledArg) -> ScheduledReducer`

> **Violates:** Robustness Principle; YAGNI
>
> The reducer name becomes an `Ident` by formatting its tokens into a string. If the bindings parser accepts a qualified path there, such as `scheduled(crate::timers::tick)`, `format_ident!` panics. The value this produces, `SpacetimeDBTable::scheduled_reducer`, is read nowhere in the repository.
>
> Recommendation: Keep the `Path` and pin a qualified path in a snapshot fixture.
>
> Developer's decision: The recommendation should be applied.

- **Test first (snapshot):** Extend `scheduled_table.rs` with a second struct scheduled as `scheduled(crate::timers::tick)`. Red: `format_ident!` panics. If the bindings parser rejects a qualified path, the fixture cannot exist: record that in the commit message and keep the `Path` anyway, since that is the value the bindings hand over.
- **Change:** `ScheduledReducer { pub reducer_path: syn::Path }` (was `reducer_name: Ident`), with its doc stating that SpacetimeDSL does not read it itself and keeps it for consumers. Update the contract test.
- **Docs:** `docs/MIGRATION.md` (derive-input consumers).

---

## Phase 8 — Wrappers and accessors

### Task 45: `WrapperType::try_parse` keeps `syn` values and only parses

*Moved from the report* — `derive-input/src/internal/dsl/wrapper.rs` › `wrapper.rs`: `WrapperType::try_parse(rust_struct: &RustStruct, rust_field: &RustField, field: &SatsField<'_>) -> syn::Result<Option<WrapperType>>`

> **Violates:** KISS; Connascence — of Meaning; Robustness Principle; Single Responsibility Principle (SRP); Scope & Goal Discipline — "Future ideas captured outside codebase"
>
> Names and types are turned into `String` and parsed back with `expect`; a local declared without a value is assigned `Some` in some branches and unwrapped with `expect` right after; the comparison with `create_wrapper` is repeated. Parsing the attribute and generating the wrapper struct (`get_wrapper_impl`) happen in one flow.
>
> Recommendation: Keep `syn` values (`Ident`, `Path`, `Type`) from parsing to generation, and separate parsing from generating the wrapper.
>
> Developer's decision: The recommendation should be applied.

- **Change:** A parse step turns each `#[create_wrapper]` / `#[use_wrapper]` into `WrapperAttribute::Created { name: Ident }` or `::Used { path: Path }`; the path comparison is made once. Generation (`created_wrapper_impl(struct_name, wrapper_name, wrapped_type: &Path, field_name, wraps_uuid)`) is a separate function called afterwards, with no `String` round trip, no `parse_str` and no `expect`. Move `// TODO: Doc comments on Wrapper Types` to the issue tracker.
- **Recorded output:** pure refactoring.

### Task 46: One `&self` accessor each for the wrapper type and the wrapped type

*Moved from the report* — `derive-input/src/internal/dsl/wrapper.rs` › `wrapper.rs`: `WrapperType::map(value: &WrapperType) -> Type`, `WrapperType::map_to_wrapped_type(value: &WrapperType) -> Type` and `WrapperType::struct_name_or_path_tokens(&self) -> TokenStream`

> **Violates:** DRY; Self-Documenting Code — don't document the past; KISS
>
> `map` and `struct_name_or_path_tokens` both produce the wrapper's name or path, one of them by formatting and re-parsing it. The panic messages name `WrapperType::map_to_wrapper_type`, `WrapperType::Wrap` and `WrapperType::Wrapped`, none of which exist (the current names are `map`, `Created` and `Used`), and `map_to_wrapped_type` claims to parse an `Ident` while it parses a `Type`. `map` and `map_to_wrapped_type` take the wrapper as an argument while their siblings take `&self`.
>
> Recommendation: One `&self` method that returns the wrapper type without a string round trip and one for the wrapped type; correct the messages.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `WrapperType::wrapper_path(&self) -> Path` (a created wrapper's `Ident` as a one-segment path) and `WrapperType::wrapped_type(&self) -> &Path` replace `map`, `map_to_wrapped_type` and `struct_name_or_path_tokens`. With no parsing left, the wrong panic messages go with them. Update every caller (`grep -rn "WrapperType::map\|struct_name_or_path_tokens"`).
- **Recorded output:** pure refactoring.

### Task 47: Generated code maps an optional wrapper with `Option::map`

*Moved from the report* — `derive-input/src/internal/dsl/wrapper.rs` › `wrapper.rs`: `pub fn map_wrapper_type_option_to_wrapped_type_option(column_name: &Ident, wrapper_type_name_or_path: &Type) -> TokenStream`

> **Violates:** KISS — the generated code is read by users in the rustdoc "Implementation" section
>
> The generated code declares a mutable `None`, tests `is_some()` and reassigns through `expect`, which is `Option::map` written out by hand.
>
> Recommendation: Emit `Option::map`. The snapshots change.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `map_wrapper_type_option_to_wrapped_type_option` emits `let #column_name = #column_name.map(|value| Into::<#wrapper>::into(value).value());`.
- **Recorded output:** the snapshots of every create method and setter with an optional used wrapper (`wrapper_optional_index`, …) change to the one-line form. The runtime gate proves they still compile.
- **Docs:** `docs/MIGRATION.md` *Changed … generated code* (visible in rustdoc).

### Task 48: Setter, getter and mut getter build each branch as one value

*Moved from the report* — `derive-input/src/internal/dsl/setter.rs` › `setter.rs`: `Setter::map(rust_field: &RustField, is_option: bool, wrapper_type: &Option<WrapperType>) -> Option<Setter>`

> **Violates:** KISS; Self-Documenting Code — no abbreviations
>
> The locals `method_arg`, `return_type` and `return_expr` are declared first and assigned in nested branches, and `method_impl` is initialised, then prepended to in one branch and replaced in another, so the resulting method body is hard to predict. The generated `match` over `old_value` is `Option::map` written out. `rt` and `return_expr` are abbreviations. `getter.rs` and `mut_getter.rs` use the same deferred style, and `mut_getter.rs` matches on `wrapper_type` only to return early in one arm.
>
> Recommendation: Let each branch produce its argument, return type and body as one expression, emit `Option::map`, and spell names out. The snapshots show any change in the generated accessors.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Each branch of `Setter::map` returns `SetterParts { argument, return_type, body }` as one expression; the generated `match old_value` becomes `old_value.map(#wrapper::new)`; `rt` → `column_type`, `return_expr` → `returned_old_value`. `Getter::map` builds its return type and body per branch the same way. `MutGetter::map` starts with `if wrapper_type.is_some() { return None; }`.
- **Recorded output:** setter snapshots for optional used wrappers change to `Option::map`; all others unchanged.

### Task 49: Accessor names come from `naming.rs`

*Moved from the report* — `derive-input/src/internal/dsl/getter.rs` › `getter.rs`: `pub fn get_getter_method_name(column_name: &Ident) -> Ident` and `mut_getter.rs::get_mut_getter_method_name`

> **Violates:** DRY; Hide Implementation Details — "Minimize class and member accessibility"
>
> The getter naming rule has a helper, but `internal/dsl/method/reference_integrity.rs` and `internal/dsl/method/upsert.rs` format `get_{column}` again. The helpers are `pub` without callers outside their own files.
>
> Recommendation: Move all naming of generated methods into `internal/dsl/method/naming.rs`, and reduce the visibility.
>
> Developer's decision: The recommendation should be applied.

- **Change:** Move `get_getter_method_name`, `get_mut_getter_method_name` and the setter name into `internal/dsl/method/naming.rs` as `pub(crate) fn getter_name`, `mut_getter_name`, `setter_name`, and use them everywhere `format_ident!("get_{…}")` builds one (`grep -rn 'format_ident!("get_{' derive-input/src`), except the `get_<table>_by_<index>` rule, which is Task 56. Widen the module doc of `naming.rs`: the identifiers generated code and user code agree on by name.
- **Recorded output:** pure refactoring.

---

## Phase 9 — Foreign keys

### Task 50: `SetZero` checks the column type and supports `Uuid::NIL`

*Moved from the report* — `derive-input/src/internal/dsl/foreign_key.rs` › `foreign_key.rs`: `ForeignKey::try_parse(has_delete_method: &bool, is_soft_deletable: bool, is_singleton: bool, field: &SatsField<'_>) -> syn::Result<Option<ForeignKey>>`

> **Violates:** Robustness Principle; DRY; KISS
>
> `OnDeleteStrategy::SetZero` is documented as available only for numeric columns, but nothing checks the column type — the TODO on `try_parse_for_on_delete` lists the missing checks. A `Uuid` foreign key with `on_delete = SetZero` generates an assignment of `0`, which fails to compile inside the expanded code instead of at the attribute. Whether the column has an index is re-derived by scanning its raw attributes, although `SpacetimeDBColumn` already knows it (`reference.rs` does the same for `#[primary_key]`). A private visibility is detected by comparing token strings, while `internal/dsl/table.rs` uses `matches!`. `has_delete_method` is passed as `&bool`.
>
> Recommendation: Add the type check with a diagnostic (compile-test first), pass the facts `SpacetimeDBColumn` already holds instead of re-scanning attributes, use `matches!(field.vis, syn::Visibility::Inherited)`, and pass `bool` by value.
>
> Developer's decision: The recommendation should be applied. Allow `SetZero` also on the `Uuid` type, which should set `Uuid::NIL` rather than `0`. Treat `Uuid::NIL` like `0` in create/update/delete/soft_delete DSL methods ("no row of another table referenced").

- **Test first:**
  - Diagnostics: `on_delete_set_zero_on_string_column` (a `String` foreign key with `on_delete = SetZero`). Red today: an error inside the expansion, at the wrong span, instead of at the attribute.
  - Snapshot: `on_delete_set_zero_uuid.rs`, a `Uuid` foreign key with `on_delete = SetZero`. Red: the fixture is new.
  - Runtime (`component/uuid_reference_test.rs`): deleting the referenced row sets the foreign key to `Uuid::NIL`; creating and updating a row whose foreign key is `Uuid::NIL` succeeds (it references no row); a non-NIL key without a referenced row still fails the integrity check.
- **Change:**
  - `ForeignKey::try_parse(has_delete_method: bool, …, spacetimedb_column: &SpacetimeDBColumn, …)`: `has_index` comes from `spacetimedb_column` (primary key or single-column index) instead of scanning the raw attributes; the private check is `matches!(field.vis, syn::Visibility::Inherited)`. `SetZero` is accepted only when `ColumnTypeKind` is `UnsignedInteger` or `UUID`, otherwise `error::set_zero_strategy_on_unsupported_type(field.ty)`.
  - The `SetZero` arm of `on_delete_strategy.rs` writes `0` or `api::spacetimedb::uuid_nil()` by kind.
  - `reference_integrity.rs::reference_integrity_checks` guards `UUID` with `ne(&<NIL>)`. `upsert.rs::ForeignKeyColumnScope::CheckedOnCreate` covers `UUID`.
  - Delete paths unchanged (decision; see the new report entry in `on_delete_strategy.rs`).
  - Move the TODO on `try_parse_for_on_delete` to the issue tracker; its `SetNone` note already has issue #32.
- **Recorded output:** the new files; snapshots of `Uuid` foreign keys in create/update/upsert gain the NIL guard (read the diff: only guards added).
- **Docs:** the `SetZero` doc of both `OnDeleteStrategy` copies (`src/delete.rs`, `api/dsl/foreign_key.rs`) and `docs/DOCUMENTATION.md` *SetZero*: unsigned integers get `0`, `Uuid` gets `Uuid::NIL`, and both mean "references no row" in create and update. `docs/MIGRATION.md`: newly rejected `SetZero` types; `Uuid` foreign keys equal to `NIL` are no longer an integrity violation on create/update.

### Task 51: `for_foreign_key` binds each foreign key once and compares types canonically

*Moved from the report* — `derive-input/src/internal/dsl/method/foreign_key.rs` › `foreign_key.rs`: `pub fn for_foreign_key(removal, one_or_multiple, referencing_tables, context, referenced_table_name, columns_with_foreign_key) -> syn::Result<(SpacetimeDSLMethod, TableContributions)>`

> **Violates:** KISS; DRY; Robustness Principle; Self-Documenting Code
>
> Within one loop iteration, the same `foreign_key` option is unwrapped in different ways (`expect` and `unwrap_or_else` with `panic!`). Type and path equality between the grouped columns are decided by comparing token strings, so `u64` and `core::primitive::u64`, or a relative and an absolute path to the same module, are reported as mismatches.
>
> Recommendation: Bind the foreign key once per column, and compare types through the classification in `internal/column.rs`.
>
> Developer's decision: The recommendation should be applied.

- **Test first:**
  - Snapshot: `foreign_keys_with_equivalent_spellings.rs`, two foreign-key columns to one table typed `u64` and `core::primitive::u64`, with paths `::other_crate::tables` and `other_crate::tables`. Red: rejected today as mismatched types, then paths.
  - `foreign_keys_with_mismatched_types` and `foreign_keys_with_mismatched_paths` keep their rejections. The paths message adds "spell both paths the same way"; regenerate that `.stderr` and read it.
- **Change:** `let foreign_key = column.spacetimedsl_column.foreign_key.as_ref().expect("<invariant: columns are grouped by their foreign key>");` once per iteration. Compare types with `canonical_type` (Task 36), and paths after stripping a leading `::`.

---

## Phase 10 — Generators without copies

Order within the phase: 52 → 53 → 54 → 55 → 56 → 57 → 58 → 59 → 60 → 61 → 62 → 63.

### Task 52: One module owns every message shape

*Moved from the report* — `derive-input/src/internal/dsl/method/index.rs` › `index.rs`: `pub fn column_names_and_row_values(column_names: &[Ident]) -> String` and the message formats elsewhere

> **Violates:** DRY; Connascence — of Meaning
>
> The shape of "column : value" messages is stated here, restated in `internal/dsl/singleton.rs::rendered_primary_key`, formatted by hand for a whole row in `create.rs` and `upsert.rs`, and formatted again for a single column in `reference_integrity.rs`. Format strings that are built at generation time and escaped for a second `format!` at run time are hard to read and easy to break.
>
> Recommendation: One module that owns every message shape and returns the finished `format!` tokens.
>
> Developer's decision: The recommendation should be applied.

*Also moved (partially) — `derive-input/src/internal/dsl/singleton.rs` › `rendered_primary_key`:* "`rendered_primary_key` restates the message format of `internal/dsl/method/index.rs::column_names_and_row_values`, as its doc comment says."

- **Change:** New `internal/dsl/method/message.rs`, returning finished `format!(…)` token streams:
  - `column_names_and_row_values(columns, value_getters)`;
  - `single_column_and_value(column, value)`;
  - `whole_row(struct_name, row)`, used by the create and upsert insert errors;
  - `singleton_primary_key()`, replacing `singleton::rendered_primary_key`.

  Callers stop building generation-time format strings that are escaped for a run-time `format!`.
- **Recorded output:** pure refactoring. The emitted `format!` calls may be spelled differently while producing identical text; if the snapshot moves, check that only the literal's shape moved and that the rendered text is identical (a runtime assertion on one not-found message makes this binary; add it if none exists).

### Task 53: One insert helper for `create_<table>` and `upsert_<table>`

*Moved from the report* — `derive-input/src/internal/dsl/method/create.rs` › `create.rs`: `pub fn for_create(context: &MethodGenerationContext) -> (SpacetimeDSLMethod, TableContributions)`

> **Violates:** DRY; Scope & Goal Discipline — "Future ideas captured outside codebase"
>
> The `try_insert` statement with its mapping of `UniqueConstraintViolation` and `AutoIncOverflow`, the message format that renders the whole row, and the FIXMEs "Only show unique columns here" and "No clone?" are copied into the insert path of `upsert.rs`.
>
> Recommendation: One helper that emits the insert-and-map-errors statement for `for_create` and `for_singleton_upsert`.
>
> Developer's decision: The recommendation should be applied.

*Moved from the report* — `derive-input/src/internal/dsl/method/upsert.rs` › `upsert.rs`: `pub fn for_singleton_upsert(context: &MethodGenerationContext) -> SpacetimeDSLMethod`

> **Violates:** DRY
>
> The insert path copies the `try_insert` statement of `for_create`, its error mapping, its message format and its FIXMEs (see `create.rs`).
>
> Recommendation: Use the shared insert helper proposed in the `create.rs` entry.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `fn insert_and_map_errors(context, row: &Ident) -> TokenStream` in `create.rs` emits the `try_insert` statement, the mapping of `UniqueConstraintViolation` / `AutoIncOverflow` (through `api::spacetimedb::try_insert_error`), and the message from Task 52. `for_create` and `for_singleton_upsert` call it. The FIXMEs "Only show unique columns here" and "No clone?" go to the issue tracker.
- **Recorded output:** pure refactoring.

### Task 54: Classify each column's role once

*Moved from the report* — `derive-input/src/internal/dsl/method/create.rs` › `create.rs`: `fn create_method_column_parts(spacetimedsl_table: &SpacetimeDSLTable, internal_column: &InternalColumn) -> CreateMethodColumnParts`

> **Violates:** KISS; Open/Closed; YAGNI
>
> A chain of `if … else if` over column roles — injected singleton key, generated UUID, auto-increment, created-at, updated-at, soft-delete marker, then the wrapper kinds — assigns locals declared without a value and returns early. The conditions are written as block expressions (`&& { … }`), and inside them the local `column_name` is shadowed by the table's timestamp column name. A special case for `String` columns yields `String` as the argument type where the general branch would yield the column's own type, which for such a column is `String` as well — it looks vestigial. Each new column role adds another link to the chain.
>
> Recommendation: Classify each column into a role once — an enum that `upsert.rs`, which re-implements the created-at and updated-at assignments, can use too — and map the role to its parts with a `match`. For the `String` special case, some spelling may depend on it; the snapshot diff after removing it will show.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - `enum CreateColumnRole { SingletonPrimaryKey, GeneratedUuid(UUIDVersion), AutoIncrement, SetOnCreate, SetOnUpdate { optional: bool }, SoftDeleteMarker(SoftDeleteMarkerKind), CreatedWrapper, UsedWrapper { optional: bool }, Plain }` with `fn of(spacetimedsl_table, internal_column) -> CreateColumnRole`.
  - `create_method_column_parts` becomes a `match` on the role that returns `CreateMethodColumnParts` from each arm, with no uninitialised locals, no `&& { … }` conditions and no shadowed `column_name`.
  - `upsert.rs`'s created-at / updated-at assignments ask the same `CreateColumnRole::of`. The move of those helpers out of `upsert.rs` is the independent entry and is not done here.
  - Remove the `String` special case.
- **Recorded output:** only `qualified_type_spellings` may move, where a `std::string::String` column now shows its own spelling instead of `String`. Read the diff; any other change means the special case mattered, so stop and report.

### Task 55: The unique-constraint-violation error is built once

*Moved from the report* — `derive-input/src/internal/dsl/method/reference_integrity.rs` › `reference_integrity.rs`: DRY in `pub fn multi_column_index_checks(...)` and `pub fn unique_multi_column_index_check(...)`

> **Violates:** DRY
>
> Both functions build the same unique-constraint-violation error.
>
> Recommendation: Build the error once and pass it on.
>
> Developer's decision: The recommendation should be applied.

- **Change:** A private `fn unique_multi_column_index_violation(context, index, one_or_multiple) -> TokenStream` in `reference_integrity.rs`, used by `multi_column_index_checks` and `unique_multi_column_index_check`.
- **Recorded output:** pure refactoring.

### Task 56: The cross-table identifiers come from one place

*Moved from the report* — `derive-input/src/internal/dsl/method/reference_integrity.rs` › `reference_integrity.rs`: the identifiers `the_same_or_another_<table>`, `get_<column>` and `get_<table>_by_<key>`

> **Violates:** DRY; Connascence — of Name, across tables; Code For The Maintainer
>
> `MethodGenerationContext` documents itself as the one place that states the name `field_name_for_found_value`, yet several functions here format `the_same_or_another_{table}` again. Getter names are formatted again as well (see `internal/dsl/getter.rs`). The method a referencing table calls on the referenced table, `get_<table>_by_<key>`, must match what `get.rs` generates in the other table's expansion — exactly the kind of name `internal/dsl/method/naming.rs` says it owns, but it is not there.
>
> Recommendation: Pass `context.field_name_for_found_value` in, move the `get_<table>_by_<index>` rule into `naming.rs` and use it in `get.rs` and here, and use the getter helper.
>
> Developer's decision: The recommendation should be applied.

- **Change:** The reference-integrity builders receive `context.field_name_for_found_value`. `naming::get_by_index_method_name(table, index) -> Ident` is used by `get.rs` and by `reference_integrity.rs` for the `get_<table>_by_<key>` call into the other table. Getter names come from Task 49.
- **Recorded output:** pure refactoring.

### Task 57: `SpacetimeDSLTableMethods::generate`, one function per concern

*Moved from the report* — `derive-input/src/internal/dsl/method.rs` › `method.rs`: `SpacetimeDSLTableMethods::generate(context: &MethodGenerationContext, columns: &[Column]) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)>`

> **Violates:** Single Responsibility Principle (SRP); DRY; KISS
>
> One function produces the create, get-all and count methods, the referenced side's cascade entry points, the grouping of foreign-key columns by referenced table, the referencing side's strategy entry points, the wrapper methods and the multi-column index methods. The block "for each kind of removal: build the one-row and the many-row method, merge the contributions, assign to `on_deletion` or `on_soft_deletion`" is written for the referenced side and again for the referencing side. The grouping uses `contains_key`, `insert` and `get_mut().expect()` where `entry(key).or_default()` suffices; `internal/dsl/method/foreign_key.rs` does the same.
>
> Recommendation: One function per concern, a shared helper for the entry points per kind of removal, and `entry().or_default()`. Pure refactoring.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - Split into `table_level_methods`, `referenced_side_entry_points`, `foreign_key_columns_by_referenced_table`, `referencing_side_entry_points`, `wrapper_methods` and `multi_column_index_methods`.
  - One helper `entry_points_per_removal(removal_kinds, build_one_row, build_multiple_rows) -> (Option<CascadeEntryPoints>, Option<CascadeEntryPoints>, TableContributions)` serves both sides.
  - Grouping uses `entry(key).or_default()`, here and in `method/foreign_key.rs`.
- **Recorded output:** pure refactoring.

### Task 58: The cascade entry-point types yield the entry points they hold

*Moved from the report* — `derive-input/src/api/dsl/table.rs` › `table.rs`: `pub struct OnDeleteStrategiesOfReferencingTables` and `pub struct OnDeleteStrategiesOfTheReferencedTable`

> **Violates:** DRY
>
> These types have the same shape — an optional pair of entry points per kind of removal — and consumers iterate their fields the same way in several places.
>
> Recommendation: Options: (a) one shared type with an iterator over the entry points it holds; (b) keep both names for their different roles and give each such an iterator. Either one removes the repeated iteration in `derive/src/output.rs`.
>
> Developer's decision: The recommendation (b) should be applied.

*Also moved (partially) — `derive/src/output.rs` › `build`:* "The iteration over `[&strategies.on_deletion, &strategies.on_soft_deletion]` is written out for the referencing and for the referenced side." Recommendation part: "Give the cascade entry-point types a method that yields the entry points they hold […] so `build` does not need to know their fields."

- **Change:** Following decision (b), both types keep their names and gain `pub fn entry_points(&self) -> impl Iterator<Item = &CascadeEntryPoints>` (deletion first, then soft deletion). `derive/src/output.rs::build` uses it on both sides, keeping the "every one-row method before any many-row method" order. Task 57's helper uses it too.
- **Recorded output:** pure refactoring.

### Task 59: One dispatcher signature, one removal wording

*Moved from the report* — `derive-input/src/internal/dsl/method/referenced_by.rs` › `referenced_by.rs`: `pub fn for_referenced_by(removal, one_or_multiple, spacetimedb_table, spacetimedsl_table, primary_key_column) -> (SpacetimeDSLMethod, TableContributions)`

> **Violates:** DRY
>
> The dispatcher signature (DSL, strategy, one key or a slice of keys, entries in a `Vec` or a `HashMap`, the failure type) is built here and again in `method/foreign_key.rs::for_foreign_key`. Both contain the same `past_tense` wording, which `naming.rs::removal_suffix` repeats in snake case.
>
> Recommendation: One helper for the dispatcher signature, and one table for the removal wording from which both the prose and the snake-case suffix are derived.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - `Removal` gets `fn past_tense(self, &OneOrMultiple) -> &'static str` ("was deleted", "were soft-deleted", …). `naming::removal_suffix` derives the snake-case form from it (spaces and hyphens become `_`), so the wording is stated once.
  - One helper `dispatcher_signature(removal, one_or_multiple, primary_key_column_type) -> (Vec<SpacetimeDSLArg>, TokenStream /* return type */)` serves `for_referenced_by` and `method/foreign_key.rs::for_foreign_key`.
- **Recorded output:** pure refactoring.

### Task 60: One removal skeleton for one row and many rows

*Moved from the report* — `derive-input/src/internal/dsl/method/removal.rs` › `removal.rs`: `pub fn for_removal_many(removal: Removal, shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod` and `pub fn for_removal_one(removal: Removal, shape: &IndexShape, context: &MethodGenerationContext) -> SpacetimeDSLMethod`

> **Violates:** DRY; Single Responsibility Principle (SRP); Code For The Maintainer — Principle of Least Astonishment; Connascence — of Name
>
> `for_removal_many` and `for_removal_one` repeat one structure — the hook blocks of a hard deletion, the reported strategy, the deletion-result entry, the handler for an error after the state changed, the call of the `Error` strategy, the strategies after the write, the final assembly and the naming — and differ mainly in how rows are found and how results are collected. The errors raised after a state change call themselves "Delete Many Error" or "Delete One Error" for soft deletions as well. `retire_row_named_old_row` expects the surrounding code to have bound a variable called `old_row`, a contract carried by the function's name instead of a parameter.
>
> Recommendation: Extract the shared skeleton, parameterised by one or many rows, as the module already does for hard and soft removal; let the messages name the actual removal; pass the row binding as an `Ident` parameter. The snapshots guard the refactoring.
>
> Developer's decision: The recommendation should be applied.

- **Change:**
  - Extract the shared skeleton of `for_removal_many` / `for_removal_one`, parameterised by `RowCount::{One, Many}`, next to the existing `Removal` parameter: the hooks of a hard deletion, the reported strategy, the entry, the after-state-change handler, the `Error` strategy call, the strategies after the write, the assembly and the naming.
  - The messages name the actual removal: "Soft Delete Many Error", "Soft Delete One Error".
  - `retire_row_named_old_row` becomes `retire_row(row: &Ident, …)`.
- **Recorded output:** the soft-delete method snapshots change in the error messages only ("Delete … Error" → "Soft Delete … Error"). Hard-delete snapshots unchanged.
- **Docs:** `docs/MIGRATION.md` *Changed messages*.

### Task 61: `delete_<singleton>` shares the removal helpers

*Moved from the report* — `derive-input/src/internal/dsl/method/singleton_table.rs` › `singleton_table.rs`: `pub fn for_singleton_delete(context: &MethodGenerationContext) -> SpacetimeDSLMethod`

> **Violates:** DRY; YAGNI
>
> The hook blocks and the text of the count-mismatch error are copied from `removal.rs::for_removal_one`, and the generated body imports `Itertools` without calling any of its methods.
>
> Recommendation: Share the hook and message helpers with `removal.rs` and drop the import; the snapshot shows the change.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `for_singleton_delete` uses the hook and count-mismatch message helpers from Tasks 52 and 60 and drops the unused `Itertools` import.
- **Recorded output:** the `singleton*` `delete_*` snapshots lose the import line; nothing else moves.

### Task 62: Name the bindings shared between cascade fragments once

*Moved from the report* — `derive-input/src/internal/dsl/method/on_delete_strategy.rs` › `on_delete_strategy.rs`: the bindings shared with `method/foreign_key.rs`

> **Violates:** Connascence — of Name, only checked after expansion; Hide Implementation Details
>
> The fragments generated here depend on bindings that `for_foreign_key` declares in another module — `dsl`, `entries`, `error`, `error_from_hook`, the label `'outer` and `primary_key_value_of_a_row_of_another_table_to_delete` — and on bindings their sibling fragments introduce. A rename on one side still compiles in `derive-input` and fails only when a user's crate expands the macro, because the snapshot harness never compiles tokens.
>
> Recommendation: Name the shared bindings once in `naming.rs`, so a rename is one change. The runtime tests remain the only gate that compiles these fragments; keep every strategy covered there.
>
> Developer's decision: The recommendation should be applied.

- **Change:** `naming.rs` gets `pub(crate) fn` for each binding the fragments share: `dsl`, `entries`, `error`, `error_from_hook`, the label `'outer`, `primary_key_value_of_a_row_of_another_table_to_delete`, `primary_key_values_of_rows_to_delete`, `child_entries_by_primary_key_value_of_row_to_delete`, and whichever others `grep` finds used by both `on_delete_strategy.rs` and `method/foreign_key.rs`. Both modules interpolate them instead of spelling them.
- **Recorded output:** pure refactoring. Only the runtime gate proves the bindings still meet, so run it.

### Task 63: Named hook fragments and `expect` messages that state the invariant

*Moved from the report (partially)* — `derive-input/src/internal/dsl/method/on_delete_strategy.rs` › `hooks_around_the_write` and `store_the_row`. The part moved here:

> **Violates:** Connascence — of Position; Robustness Principle
>
> The nested tuple can only be read by position. […] the generated cascade also panics through `expect` calls with messages such as "Should exist".
>
> Recommendation: A named struct for the hook fragments; […] give the remaining `expect` calls messages that state the broken invariant.
>
> Developer's decision: The recommendation should be applied. (Planning decision: the `update` panic part stays in the report with the independent `update.rs` entry.)

- **Change:** `struct HookFragments { use_trait: TokenStream, call: TokenStream }` and `struct HooksAroundTheWrite { before: HookFragments, after: HookFragments }` replace the nested tuple. Every generated `expect` in `on_delete_strategy.rs` and `removal.rs` states what guaranteed the value, for example `"every primary key value of a row to delete was given an entry list before its strategies ran"`, instead of "Should exist" or "`{…}` should exist in …".
- **Recorded output:** snapshots of every cascade change in `expect` strings only. Read the diff for that.

---

## Phase 11 — Release

### Task 64: `0.24.0`

- Bump `spacetimedsl`, `spacetimedsl_derive` and `spacetimedsl_derive-input` to `0.24.0` in `Cargo.toml` (the workspace dependency pins too) and wherever the docs name the version. Regenerate `Cargo.lock` through a build run by `x.ps1`.
- Read `docs/MIGRATION.md` end to end against `git diff main --stat`. Every breaking change of Tasks 13–63 must have an entry with before/after code.
- Check that `CODE_QUALITY_REPORT.md` contains none of the entries of this plan (`grep` for each moved heading).
- Gates: `.\x.ps1 unit-test`, `.\x.ps1 test`, `.\x.ps1 lint`, and `.\x.ps1 format` twice with no change on the second run.

---

## What this plan does not do

Every entry left in `CODE_QUALITY_REPORT.md` is out of scope: entries without a decision, entries whose decision says to fix them independently (among them the new entry about `0` / `Uuid::NIL` in the delete cascades), and the remaining halves of the partially moved entries.
