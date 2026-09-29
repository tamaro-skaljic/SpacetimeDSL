# Foreign Key Messages, Pairing and Relationship Docs — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Resolve issues #183, #182, #181, #180, #179, #173, #172 and #35 in one pull request. Generated error messages name the right key and column. Foreign keys pair with `#[referenced_by]` one by one and may reference tables whose rows are never removed. The generated documentation shows every relationship.

**Architecture:** Every change lives in the generator crate `derive-input` (`src/internal/dsl/method/…`) and in the output crate `derive`. The runtime crate `src/` changes only the `Display` text of one error. A new module `derive-input/src/internal/dsl/method/pairing.rs` owns every marker trait that pairs a `#[foreign_key]` with its `#[referenced_by]`. A new module `derive-input/src/internal/dsl/method/relationship_doc.rs` owns every sentence the generated documentation says about relationships.

**Tech Stack:** Rust 2024 (toolchain pinned in `rust-toolchain.toml`), `syn` / `quote` / `proc-macro2`, SpacetimeDB `=2.10.1`, `insta` snapshots, `trybuild` compile tests, the `examples/test` SpacetimeDB module as the runtime gate, PowerShell `x.ps1` as the only build entry point.

**Spec:** The eight GitHub issues, read together with *Decisions taken while planning* below, which fill the gaps the issues left open:

- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/183 — name the key in the invariant panics of the generated cascades
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/182 — `update_*_by_*` reports a missing row by its foreign key value
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/181 — avoid cloning the row before `try_insert`
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/180 — only the unique columns in the unique-constraint error
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/179 — check `on_delete = SetZero` on primary key and unique foreign key columns
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/173 — the create-side reference-integrity error names the column by its value
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/172 — `#[referenced_by]` on tables without delete methods, `#[foreign_key]` without strategies
- https://github.com/tamaro-skaljic/SpacetimeDSL/issues/35 — `#[referenced_by]`s and `#[foreign_key]`s influence doc comments (empty issue body; scope decided below)

Found while planning and moved out of scope: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/185 — a `String` primary key cannot be referenced by a foreign key.

## Global Constraints

- Build and test only through `x.ps1`. Never run `cargo` directly: a workspace member built on its own fails to link against SpacetimeDB.
- The crates stay at version `0.24.0`, which is not released yet. Every user-visible change extends the section `## 0.23 → 0.24` of `docs/MIGRATION.md`.
- One branch, one pull request, one commit per task, in the order of this plan.
- Every commit message ends with the line `Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>`.
- Names follow AGENTS.md *Self-Documenting Code*: no abbreviations, no comment that repeats the code, no comment about the past or about an issue or task.
- Every user-facing `syn::Error` is written in `derive-input/src/internal/error.rs`.
- `derive-input/src/api/` gains only public fields and types another crate reads. Helper functions go into `derive-input/src/internal/` as plain `pub`.
- A change of a public `derive-input` type updates `derive/src/data_transfer_contract_tests.rs` and adds an entry under *For crates building on `spacetimedsl_derive-input`* in `docs/MIGRATION.md`, in the same commit.
- A runtime test group is one file in `examples/test/src` with a `pub(crate) fn run_tests`, registered in `TEST_GROUPS` in `examples/test/src/lib.rs`. Its table accessors are unique across the whole module, and every count it asserts is relative.
- Knowledge in `AGENTS.md`, `docs/DOCUMENTATION.md`, `docs/MIGRATION.md` and `CODE_QUALITY_REPORT.md` changes in the same commit as the code it describes.

## Decisions taken while planning

| Topic | Decision |
| --- | --- |
| Delivery | One pull request, one commit per task. |
| Version | No bump. `0.24.0` is unreleased, so every entry extends `## 0.23 → 0.24` in `docs/MIGRATION.md`. |
| Plan location | `docs/plans/`, as the previous plan. |
| #183 panic | `.unwrap_or_else(\|\| panic!("<invariant>, which does not hold for {}", key))`. The key is formatted with `{}`, like in every other generated message, and only when the lookup fails. |
| #182 message | The `NotFoundError` names the primary key and the value the lookup used: `{ id : 7 }`, or the literal `{ id : 0 }` for a singleton. The `expect` next to it states its invariant instead of naming a generator variable. The dead `OneOrMultiple::Multiple` branch of `reference_integrity_checks_on_update` goes. |
| #180 columns | Only the columns SpacetimeDB checks: the primary key and the `#[unique]` columns. A `unique_index(name = …)` multi-column index is checked by SpacetimeDSL before the insert and cannot cause this error. An `#[auto_inc]` column shows the value handed to SpacetimeDB (`0`). The `Display` text becomes *here are the unique columns and the values handed to SpacetimeDB*. |
| #181 copy | The unique column values are cloned into one tuple before `try_insert`, and the row moves into `try_insert`. |
| #173 value | The raw value, read through the getter: `{ warehouse_id : 3 }`, identical to the update side. The runtime test uses `u64` keys. `String` foreign keys are #185. |
| #179 shapes | Reject `SetZero` on a primary key column, on a `#[unique]` column and on a column of a `unique_index(name = …)` index; one compile-test per shape. A temporary runtime probe per shape shows the failure first; the probes are removed before the commit. |
| #172 traits | Pair-specific. For each `#[referenced_by(table = B)]` and each removal A cannot perform, A declares `this_compilation_error_occurs_because_your_foreign_key_referencing_the_{A}_table_needs_to_define_a_strategy_for_on_{delete,soft_delete}_or_the_{A}_table_has_no_referenced_by_attribute_referencing_the_{B}_table`. |
| #172 per foreign key | Each `#[foreign_key]` is checked on its own: a group of foreign keys to one table imports the trait for a declared strategy when one of them declares it, and the trait for a missing strategy when one of them lacks it. |
| #172 back reference | B declares `this_compilation_error_occurs_because_the_{B}_table_has_no_foreign_key_attribute_referencing_the_{A}_table` once per referenced table. A imports it once per `#[referenced_by]`, whatever A can remove. It replaces A's two per-removal imports. |
| #172 import home | Imports go into one `const _: () = { use …; };` block per table, from the new field `SpacetimeDSLTable::compile_error_check_imports: Vec<syn::Path>`. |
| #172 one module | `derive-input/src/internal/dsl/method/pairing.rs` computes every declared trait and every import; the cascade generators lose that responsibility. |
| #172 `delete` default | "Explicitly or implicitly `delete = false`" means the resolved `has_delete_method`. `delete` keeps defaulting to `true`. |
| #35 placements | Write methods (`create_*`, `update_*_by_*`, `upsert_*`), removal methods (`delete_*`, `soft_delete_*`), the getter and setter of a foreign key column, and the struct. |
| #35 style | Rustdoc headings and bullets. Struct headings always name the table: *Foreign keys of the `shipment` table*, *Tables referencing the `warehouse` table*. |
| String foreign keys | Out of scope; filed as #185. |

## Review Focus

These inputs are the most likely to bite a user, and no test pins them today. Each line names the test its owning task adds.

1. **Two foreign keys from one table to a table without delete methods.** The pairing imports must stay deduplicated per referenced table, since a repeated `use` in one block is error E0252. Pinned by `CatalogProduct` (`category_id`, `secondary_category_id`) in Task 9's runtime group.
2. **Two tables of one module referencing the same table.** Their `const _` blocks must not clash. Pinned by `CatalogProduct` and `CatalogBundle`, which share one module, in Task 9's runtime group.
3. **Two foreign keys to one deletable table, only one of them with `on_delete`.** It must be rejected, because deleting the referenced row would leave the other foreign key dangling. Pinned by `foreign_key_without_on_delete_to_deletable_table` in Task 8.
4. **A `String` `#[unique]` column in `create_*`.** The #181 copy must clone it, not move it out of the row before `try_insert`. Pinned by `ConstraintBadge.code` in Task 4's runtime group, which Task 5 runs again.
5. **A table which references itself.** After the pairing move, its `const _` block imports its own traits through `self::`. Pinned by the existing `self_referencing_cascade` fixture and `self_referencing_cascade_test` runtime group; Tasks 7 and 8 read that diff and run the runtime gate.

---

## Conventions for every task

**The local server** has to run for the runtime gate: start `spacetime start` in a terminal of its own, and check it with `spacetime server ping local`.

**The unit gate** (snapshots and compile tests):

```powershell
.\x.ps1 unit-test 2>&1 | Select-String -Pattern "test result:|FAILED|^error|^warning: " | Select-Object -First 20
```

**The runtime gate:**

```powershell
$output = .\x.ps1 test 2>&1 | Out-String
if ($LASTEXITCODE -eq 0) {
    "PASSED"
} else {
    "FAILED - relevant output:"
    $output -split "`n" | Select-String -Pattern "^error|-->|panic|should|failed" | Select-Object -First 30
}
```

**Regenerating snapshots**, then reading every hunk of the diff:

```powershell
$env:INSTA_FORCE_UPDATE = "1"
.\x.ps1 unit-test
$env:INSTA_FORCE_UPDATE = $null
git diff --stat -- derive/tests/snapshots
git diff -- derive/tests/snapshots
```

**Regenerating compile-test output**, then reading every hunk of the diff:

```powershell
$env:TRYBUILD = "overwrite"
.\x.ps1 unit-test
$env:TRYBUILD = $null
git diff -- compile-tests/tests/ui
```

The regeneration rewrites the line endings of every file it touches, so `git status` lists far more files than the diff shows. `git diff` normalizes line endings, so trust it. Stage with `git add .`; git handles the line endings.

**Green includes the formatter:** run `.\x.ps1 format` until a second run changes nothing, then `.\x.ps1 lint`, which must exit 0.

**Red is an observation:** read the failure text of a new test before changing production code, and check that it fails for the reason under test.

---

## Task 0: Record the baseline and commit this plan

**Files:**
- Create: `docs/plans/2026-09-29-foreign-key-messages-pairing-and-docs.md` (this file)

- [ ] **Step 1: Run the unit gate**

Expected: every `test result:` line says `ok`, no `FAILED`.

- [ ] **Step 2: Run the runtime gate**

Expected: `PASSED`.

- [ ] **Step 3: Run the lint gate**

Run: `.\x.ps1 lint; $LASTEXITCODE`
Expected: `0`.

A red baseline is fixed or reported before Task 1 starts; it is not carried into the plan.

- [ ] **Step 4: Commit the plan**

```powershell
git add docs/plans/2026-09-29-foreign-key-messages-pairing-and-docs.md
git commit -m @'
Add the plan for foreign key messages, pairing and relationship docs

Plans issues #183, #182, #181, #180, #179, #173, #172 and #35.

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
'@
```

---

## Task 1: Name the key in the invariant panics of the generated cascades (#183)

**Files:**
- Modify: `derive-input/src/internal/dsl/method/on_delete_strategy.rs` (import line, top of `on_delete_strategy_implementation`, nine lookups, `referenced_table_function_call_for_strategy_implementation`, the constants at the end)
- Modify: `derive-input/src/internal/dsl/method/referenced_by.rs` (import, `referenced_table_function_call_for_dsl_method`, `for_referenced_by`)
- Modify: `derive-input/src/internal/dsl/method/naming.rs` (`cascade_binding`)
- Modify: `docs/MIGRATION.md`
- Recorded output: 48 files under `derive/tests/snapshots` (every `internal_methods.snap` of a cascade, and the `delete_*` methods of referenced tables)

**Interfaces:**
- Produces: `naming::cascade_binding::primary_key_value_of_a_row_to_delete() -> Ident`; `on_delete_strategy::child_entries_of_a_row_to_delete_or_panic() -> TokenStream` (`pub(super)`). `CHILD_ENTRIES_ONLY_FOR_ROWS_TO_DELETE` becomes private.

The lookups the issue names can only fail when the generator emitted inconsistent code, so no runtime test can trigger them. The snapshots are the test, and the runtime gate proves that the new code compiles.

- [ ] **Step 1: Add the binding to `naming::cascade_binding`**

In `derive-input/src/internal/dsl/method/naming.rs`, add after `primary_key_values_of_rows_to_delete()`:

```rust
    /// The primary key value of one of this table's rows a strategy removes, under which a
    /// referencing table returns the entries its own strategies produced for that row.
    pub fn primary_key_value_of_a_row_to_delete() -> Ident {
        format_ident!("primary_key_value_of_a_row_to_delete")
    }
```

- [ ] **Step 2: Replace the constants and add the panic builders**

In `on_delete_strategy.rs`, replace the block that starts with `// Why the lookups in the generated cascades cannot fail.` and ends with the `CHILD_ENTRIES_ONLY_FOR_ROWS_TO_DELETE` constant by:

```rust
// Why the lookups in the generated cascades cannot fail. Each starts the panic message of the
// lookup that relies on it, which goes on to name the key the invariant broke for.

const ENTRY_LIST_PREPARED: &str = "every primary key value of a removed row of the referenced table was given an entry list before its strategies ran";

const ROW_LIST_PREPARED: &str = "every primary key value of a removed row of the referenced table was given a list of rows to delete before its rows were found";

const ROW_RECORDED: &str =
    "every primary key value of a row to delete was recorded with its row when the row was found";

const CHILD_ENTRY_LIST_PREPARED: &str = "every primary key value of a row to delete was given a child entry list before its strategies ran";

const CHILD_ENTRIES_ONLY_FOR_ROWS_TO_DELETE: &str = "the referencing tables return child entries only for the primary key values of the rows this table deletes";

/// `.unwrap_or_else(|| panic!("<invariant>, which does not hold for {}", <key>))`, the end of
/// a lookup in a generated cascade which fails only if SpacetimeDSL generated inconsistent
/// code. The message is formatted only when the lookup fails, not on every lookup.
fn unwrap_or_panic_naming_the_key(invariant: &str, key: &impl ToTokens) -> TokenStream {
    let message = format!("{invariant}, which does not hold for {{}}");

    quote! {
        .unwrap_or_else(|| panic!(#message, #key))
    }
}

/// The end of the lookup of the child entries a referencing table returned under
/// `primary_key_value_of_a_row_to_delete`, a row this table deletes.
pub(super) fn child_entries_of_a_row_to_delete_or_panic() -> TokenStream {
    unwrap_or_panic_naming_the_key(
        CHILD_ENTRIES_ONLY_FOR_ROWS_TO_DELETE,
        &cascade_binding::primary_key_value_of_a_row_to_delete(),
    )
}
```

Change the import `quote::{TokenStreamExt, format_ident, quote},` to `quote::{ToTokens, TokenStreamExt, format_ident, quote},`.

- [ ] **Step 3: Use the builders in `on_delete_strategy_implementation`**

After `let spacetimedb_call_prefix = quote! { … };` add:

```rust
    let entry_list_or_panic = unwrap_or_panic_naming_the_key(
        ENTRY_LIST_PREPARED,
        &primary_key_value_of_a_row_of_another_table_to_delete,
    );
    let row_list_or_panic = unwrap_or_panic_naming_the_key(
        ROW_LIST_PREPARED,
        &primary_key_value_of_a_row_of_another_table_to_delete,
    );
    let recorded_row_or_panic =
        unwrap_or_panic_naming_the_key(ROW_RECORDED, primary_key_column_name);
    let child_entry_list_or_panic =
        unwrap_or_panic_naming_the_key(CHILD_ENTRY_LIST_PREPARED, primary_key_column_name);
```

Then replace, inside the `quote!` bodies of the same function:

| Old tokens (occurrences) | New tokens |
| --- | --- |
| `.expect(#ENTRY_LIST_PREPARED)` (1) | `#entry_list_or_panic` |
| `.expect(#CHILD_ENTRY_LIST_PREPARED)` (2) | `#child_entry_list_or_panic` |
| `.expect(#ROW_LIST_PREPARED)` (2) | `#row_list_or_panic` |
| `.expect(#ROW_RECORDED)` (2) | `#recorded_row_or_panic` |

For example, `#entries.get_mut(#primary_key_value_of_a_row_of_another_table_to_delete).expect(#ENTRY_LIST_PREPARED).push(#create_entry);` becomes `#entries.get_mut(#primary_key_value_of_a_row_of_another_table_to_delete)#entry_list_or_panic.push(#create_entry);`.

- [ ] **Step 4: Use the builder in `referenced_table_function_call_for_strategy_implementation`**

Add after `let child_entries_by_primary_key_value_of_row_to_delete = …;`:

```rust
    let primary_key_value_of_a_row_to_delete =
        cascade_binding::primary_key_value_of_a_row_to_delete();
    let child_entries_or_panic = child_entries_of_a_row_to_delete_or_panic();
```

and replace the returned `quote!` by:

```rust
    quote! {
        match #referenced_table_call {
            Err(failure) => {
                for (#primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                    #child_entries_by_primary_key_value_of_row_to_delete.get_mut(#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.append(&mut child_entries);
                }

                if #error_from_hook.is_none() {
                    #error_from_hook = failure.error_from_hook;
                }

                #on_error_handler
            },
            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                for (#primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                    #child_entries_by_primary_key_value_of_row_to_delete.get_mut(#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.append(&mut child_entries);
                }
            }
        };
    }
```

- [ ] **Step 5: Use the builder in `referenced_by.rs`**

Change the import `on_delete_strategy::CHILD_ENTRIES_ONLY_FOR_ROWS_TO_DELETE,` to `on_delete_strategy::child_entries_of_a_row_to_delete_or_panic,`, and add `cascade_binding,` to the `naming::{…}` import.

In `referenced_table_function_call_for_dsl_method`, replace the `OneOrMultiple::Multiple` arm's `quote!` by:

```rust
            let primary_key_value_of_a_row_to_delete =
                cascade_binding::primary_key_value_of_a_row_to_delete();
            let child_entries_or_panic = child_entries_of_a_row_to_delete_or_panic();

            quote! {
                match #referenced_table_call {
                    Err(failure) => {
                        for (#primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                            deletion_result_entries.get_mut(#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.child_entries.append(&mut child_entries);
                        }

                        let error_from_hook = failure.error_from_hook;

                        #on_error_handler
                    },
                    Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                        for (#primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                            deletion_result_entries.get_mut(#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.child_entries.append(&mut child_entries);
                        }
                    }
                };
            }
```

In `for_referenced_by`, spell both bindings through `cascade_binding`, so the loops agree with the name the panic reads. Add at the top of the function:

```rust
    let primary_key_value_of_a_row_to_delete =
        cascade_binding::primary_key_value_of_a_row_to_delete();
    let primary_key_values_of_rows_to_delete =
        cascade_binding::primary_key_values_of_rows_to_delete();
```

In the `(doc_comment, arg_name)` match, keep both `format!` doc strings and replace `format_ident!("primary_key_value_of_a_row_to_delete")` by `primary_key_value_of_a_row_to_delete.clone()` and `format_ident!("primary_key_values_of_rows_to_delete")` by `primary_key_values_of_rows_to_delete.clone()`.

Replace the `OneOrMultiple::Multiple` arm of `create_entries` by:

```rust
        OneOrMultiple::Multiple => {
            quote! {
                let mut entries = std::collections::HashMap::new();

                if #primary_key_values_of_rows_to_delete.is_empty() {
                    return Ok(entries);
                }

                for #primary_key_value_of_a_row_to_delete in #primary_key_values_of_rows_to_delete {
                    entries.insert(#primary_key_value_of_a_row_to_delete, vec![]);
                }
            }
        }
```

Keep the comment above that arm. Replace the `OneOrMultiple::Multiple` arm of `strategy_calls` by:

```rust
                OneOrMultiple::Multiple => {
                    let child_entries_or_panic = child_entries_of_a_row_to_delete_or_panic();

                    quote! {
                        match #referencing_table_call {
                            Err(failure) => {
                                for (#primary_key_value_of_a_row_to_delete, mut child_entries) in failure.entries {
                                    entries.get_mut(&#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.append(&mut child_entries);
                                }

                                if error_from_hook.is_none() {
                                    error_from_hook = failure.error_from_hook;
                                }

                                error = true;
                            },
                            Ok(child_entries_by_primary_key_value_of_a_row_to_delete) => {
                                for (#primary_key_value_of_a_row_to_delete, mut child_entries) in child_entries_by_primary_key_value_of_a_row_to_delete {
                                    entries.get_mut(&#primary_key_value_of_a_row_to_delete)#child_entries_or_panic.append(&mut child_entries);
                                }
                            },
                        };
                    }
                },
```

Remove `format_ident` from the `quote` import if nothing else in the file uses it.

- [ ] **Step 6: Observe the red**

Run the unit gate.
Expected: `FAILED` in `derive`'s snapshot tests. Read one reported diff; it has to show exactly `.expect("every primary key value …")` turning into `.unwrap_or_else(|| panic!("every primary key value …, which does not hold for {}", primary_key_value_of_a_row_of_another_table_to_delete))`.

- [ ] **Step 7: Regenerate the snapshots and check the diff**

Regenerate as described in *Conventions*, then run:

```powershell
git grep -l "which does not hold for" -- derive/tests/snapshots | Measure-Object -Line
git grep -n -e "every primary key value" -e "the referencing tables return child entries" -- derive/tests/snapshots | Select-String -NotMatch "which does not hold for"
```

Expected: `48` lines from the first command, no output from the second. Read the whole `git diff`; no hunk may change anything but these tails.

- [ ] **Step 8: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. The runtime gate compiles the new panics in every cascade of `examples/test`.

- [ ] **Step 9: Document the change**

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### A panic inside a generated cascade names its invariant and its key

The lookups inside the generated delete and soft-delete cascades can only fail if SpacetimeDSL generated inconsistent code. Their panic used to name a variable of the generated code, such as *7 should exist in entries.* It now states the invariant that broke and the key it broke for, such as *the referencing tables return child entries only for the primary key values of the rows this table deletes, which does not hold for 7*. The message is formatted only when a lookup fails.
```

- [ ] **Step 10: Format, lint, commit**

```powershell
.\x.ps1 format
.\x.ps1 format
.\x.ps1 lint; $LASTEXITCODE
git add .
git commit -m @'
Name the key in the invariant panics of the generated cascades

The lookups inside the generated cascades end in
`unwrap_or_else(|| panic!("<invariant>, which does not hold for {}", key))`: the panic states
the broken invariant and the key it broke for, and its message is formatted only on failure.

Recorded output: in 48 snapshots, each `.expect("<invariant>")` became that lazy panic.

Closes #183

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
'@
```

---

## Task 2: The create-side reference-integrity error names the column (#173)

**Files:**
- Create: `examples/test/src/reference_integrity_message_test.rs`
- Modify: `examples/test/src/lib.rs` (module list, `TEST_GROUPS`)
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (`reference_integrity_checks_on_create`)
- Modify: `docs/DOCUMENTATION.md` (*Example Error Messages*), `docs/MIGRATION.md`
- Recorded output: every `create_*.snap` of a table with a foreign key, and every `upsert_*.snap` of a singleton with a foreign key

**Interfaces:**
- Produces: the runtime group `reference_integrity_message_test` with the tables `IntegrityDepot` (accessor `integrity_depot`, wrapper `IntegrityDepotId`) and `IntegrityParcel` (accessor `integrity_parcel`, wrapper `IntegrityParcelId`, `CreateIntegrityParcel { id: u64, depot_id: IntegrityDepotId }`). Task 3 extends this group.

- [ ] **Step 1: Write the failing runtime test**

Create `examples/test/src/reference_integrity_message_test.rs`:

```rust
//! The reference-integrity checks of `create_<table>` and `update_<table>_by_<key>` report
//! the row they could not use by column name and value, the way every other generated error
//! does.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = integrity_depots, method(update = false, delete = true))]
#[spacetimedb::table(accessor = integrity_depot)]
pub struct IntegrityDepot {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(IntegrityDepotId)]
    #[referenced_by(path = crate::reference_integrity_message_test, table = integrity_parcel)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = integrity_parcels, method(update = true, delete = true))]
#[spacetimedb::table(accessor = integrity_parcel)]
pub struct IntegrityParcel {
    #[primary_key]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(IntegrityDepotId)]
    #[foreign_key(
        path = crate::reference_integrity_message_test,
        table = integrity_depot,
        column = id,
        on_delete = Delete
    )]
    pub depot_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_names_the_column_which_references_no_row(dsl)?;

    Ok(())
}

/// A parcel whose `depot_id` references no depot is rejected, and the error names the column
/// and the value it held.
fn create_names_the_column_which_references_no_row<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let result = dsl.create_integrity_parcel(CreateIntegrityParcel {
        id: 1,
        depot_id: IntegrityDepotId::new(u64::MAX),
    });

    let error = match result {
        Ok(parcel) => {
            return Err(format!(
                "Creating a parcel whose depot_id references no depot should fail! Got:\n{parcel:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = format!(
        "Reference Integrity Violation Error while trying to create a row in the `integrity_parcel` table because of `{{ depot_id : {} }}`!",
        u64::MAX
    );
    if error.to_string() != expected {
        return Err(format!(
            "The reference-integrity error of create_integrity_parcel should name the column and its value!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}
```

In `examples/test/src/lib.rs`, add `pub mod reference_integrity_message_test;` after `pub mod primary_key_foreign_key_cascade_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "reference_integrity_message_test",
        reference_integrity_message_test::run_tests,
    ),
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED`, with a line like `reference_integrity_message_test: The reference-integrity error of create_integrity_parcel should name the column and its value!` whose *Actual* is `` …because of `{ 18446744073709551615 : IntegrityDepotId { id: 18446744073709551615 } }`! ``.

- [ ] **Step 3: Read the value through the getter**

In `reference_integrity_checks_on_create`, replace the `reference_integrity_violation_error` binding, including the comment inside it that points at the issue, by:

```rust
        let reference_integrity_violation_error =
            runtime::reference_integrity_violation_on_create_or_update(
                &referencing_table_name_as_string,
                &quote! { Create },
                &message::single_column_and_value(
                    referencing_table_column_name,
                    &quote! { #referencing_table_name.#referencing_table_column_getter_name().value() },
                ),
            );
```

The local binding of the column is not used: a create method has already moved it into the row.

- [ ] **Step 4: Regenerate the snapshots and read the diff**

Expected hunks only: `format!("{{ {} : {} }}", warehouse_id, inspection.get_warehouse_id())` becomes `format!("{{ warehouse_id : {} }}", inspection.get_warehouse_id().value())`, in `create_*` and in the insert path of `upsert_*`.

- [ ] **Step 5: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 6: Document the change**

In `docs/DOCUMENTATION.md`, *Example Error Messages*, replace the block under `**ReferenceIntegrityViolation (on create/update):**` by:

````markdown
```txt
Reference Integrity Violation Error while trying to create a row in the `position` table because of `{ entity_id : 1 }`!
```
````

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### The reference-integrity error of `create_<table>` names the column

When `create_<table>`, or the insert path of `upsert_<table>`, rejects a row because a foreign key references no row, the error names the column and its value, `{ warehouse_id : 3 }`, the way `update_<table>_by_<key>` does. It used to print the value in place of the name, `{ 3 : WarehouseId { id: 3 } }`.
```

- [ ] **Step 7: Format, lint, commit**

Commit message:

```text
Name the column in the reference-integrity error of create

The create side read the column's name as a value. It now reads the value through the getter
and names the column, as the update side does: `{ warehouse_id : 3 }`.

Test: reference_integrity_message_test pins the message of a rejected create.
Recorded output: every create_* and upsert_* of a table with a foreign key.

Closes #173

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 3: `update_*_by_*` reports a missing row by its primary key (#182)

**Files:**
- Modify: `examples/test/src/reference_integrity_message_test.rs`
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (`reference_integrity_checks_on_update`, new constant)
- Modify: `derive-input/src/internal/dsl/method/update.rs` (call site, `OneOrMultiple` import)
- Modify: `derive-input/src/internal/dsl/method/upsert.rs` (call site, `index_columns`, `OneOrMultiple` import)
- Modify: `docs/DOCUMENTATION.md`, `docs/MIGRATION.md`
- Recorded output: every `update_*.snap` and `upsert_*.snap` of a table with a non-private foreign key column

**Interfaces:**
- Consumes: Task 2's tables `IntegrityDepot`, `IntegrityParcel`.
- Produces: `reference_integrity_checks_on_update(spacetimedb_table: &SpacetimeDBTable, columns: &[InternalColumn], field_name_for_found_value: &Ident, primary_key_column: &InternalColumn, is_singleton: bool) -> Vec<TokenStream>`. The `index_columns` and `one_or_multiple` parameters are gone.

- [ ] **Step 1: Write the failing runtime test**

In `reference_integrity_message_test.rs`, call the new test from `run_tests`, after the create test:

```rust
    update_of_a_missing_row_names_its_primary_key(dsl)?;
```

and add:

```rust
/// Updating a parcel which no longer exists fails with the primary key value the lookup used,
/// not with the value of its foreign key.
fn update_of_a_missing_row_names_its_primary_key<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let depot = dsl.create_integrity_depot()?;
    let parcel = dsl.create_integrity_parcel(CreateIntegrityParcel {
        id: 1_000_000,
        depot_id: depot.get_id(),
    })?;
    dsl.delete_integrity_parcel_by_id(parcel.get_id())?;

    let error = match dsl.update_integrity_parcel_by_id(parcel) {
        Ok(parcel) => {
            return Err(format!(
                "Updating a parcel which no longer exists should fail! Got:\n{parcel:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = "Not Found Error while trying to find a row in the `integrity_parcel` table with `{ id : 1000000 }`!";
    if error.to_string() != expected {
        return Err(format!(
            "Updating a parcel which no longer exists should name the primary key value it looked up!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED` with *Actual* `` …with `{ id : 1 }`! ``, the depot's id where the parcel's belongs.

- [ ] **Step 3: Build the message from the primary key value**

In `reference_integrity.rs`, add above `reference_integrity_checks_on_update`:

```rust
/// Why the generated update may unwrap the stored row: the first foreign key check looks it up
/// by its primary key and returns when it finds none.
const STORED_ROW_LOOKED_UP: &str =
    "the stored row is looked up by its primary key before its foreign key columns are compared";
```

Replace the signature and the head of `reference_integrity_checks_on_update` down to, but not including, `let reference_integrity_violation_error =`, by:

```rust
pub fn reference_integrity_checks_on_update(
    spacetimedb_table: &SpacetimeDBTable,
    columns: &[InternalColumn],
    field_name_for_found_value: &Ident,
    primary_key_column: &InternalColumn,
    is_singleton: bool,
) -> Vec<TokenStream> {
    reference_integrity_checks(columns, true, |column, foreign_key| {
        let referenced_table_name = &foreign_key.table_name;

        let primary_key_column_name_of_referenced_table = &foreign_key.primary_key_column_name;
        let get_row_of_referenced_table_by_primary_key_method_name =
            naming::get_by_index_method_name(
                referenced_table_name,
                primary_key_column_name_of_referenced_table,
            );

        let referencing_table_name = &spacetimedb_table.singular_name;
        let referencing_table_name_as_string = referencing_table_name.to_string();
        let referencing_table_column_name = &column.rust_field_name;
        let primary_key_column_name_of_referencing_table = &primary_key_column.rust_field_name;
        let referencing_table_column_getter_name =
            naming::getter_name(referencing_table_column_name);

        // The stored row is looked up by the primary key value of the row to write, so a
        // missing row is reported with that value.
        let (primary_key_value_of_referencing_table, missing_row) = match is_singleton {
            true => {
                let primary_key_value = singleton::primary_key_value();

                (
                    quote! { &#primary_key_value },
                    message::singleton_primary_key(),
                )
            }
            false => {
                let getter_name = naming::getter_name(primary_key_column_name_of_referencing_table);
                let primary_key_value = quote! { #referencing_table_name.#getter_name().value() };
                let missing_row = message::single_column_and_value(
                    primary_key_column_name_of_referencing_table,
                    &primary_key_value,
                );

                (primary_key_value, missing_row)
            }
        };

        let not_found_error =
            runtime::not_found_error(&referencing_table_name_as_string, &missing_row);
```

Keep the `reference_integrity_violation_error` binding as it is. In the returned `quote!`, replace `.expect("field_name_for_found_value should be Some(_)")` by `.expect(#STORED_ROW_LOOKED_UP)`. The `row_value_getters` and `format_for_not_found_error` bindings are gone. Keep the doc comment of the function.

- [ ] **Step 4: Update the two callers**

In `update.rs`, delete the `let one_or_multiple = match shape.is_multi_column { … };` binding and the import `internal::dsl::one_or_multiple::OneOrMultiple`, and call:

```rust
    let reference_integrity_checks = reference_integrity_checks_on_update(
        spacetimedb_table,
        internal_columns,
        field_name_for_found_value,
        primary_key_column,
        is_singleton_pk,
    );
```

In `upsert.rs`, delete `let index_columns = vec![primary_key.clone()];`, change the import `dsl::{one_or_multiple::OneOrMultiple, singleton},` to `dsl::singleton,`, and call:

```rust
    let checks_on_update = reference_integrity_checks_on_update(
        spacetimedb_table,
        internal_columns,
        field_name_for_found_value,
        primary_key_column,
        true,
    );
```

- [ ] **Step 5: Regenerate the snapshots and read the diff**

Expected hunks only:
- `format!("{{ id : {} }}", warehouse_id)` becomes `format!("{{ id : {} }}", inspection.get_id().value())` in `update_*`;
- `format!("{{ id : {} }}", region_id)` becomes the literal `"{ id : 0 }"` in `update_<singleton>` and `upsert_<singleton>`;
- `.expect("field_name_for_found_value should be Some(_)")` becomes `.expect("the stored row is looked up by its primary key before its foreign key columns are compared")`.

- [ ] **Step 6: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 7: Document the change**

In `docs/DOCUMENTATION.md`, *Example Error Messages*, replace the block under `**NotFoundError:**` by:

````markdown
```txt
Not Found Error while trying to find a row in the `position` table with `{ entity_id : 1 }`!
```
````

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### A missing row in `update_<table>_by_<key>` is reported by its primary key

When the row to update no longer exists, the `NotFoundError` of `update_<table>_by_<key>`, `update_<singleton>` and `upsert_<singleton>` shows the primary key value it looked up, `{ id : 7 }`. It used to show the value of a foreign key column under the primary key's name.
```

- [ ] **Step 8: Format, lint, commit**

Commit message:

```text
Report a missing row in update by its primary key value

The not-found error of the update-side reference-integrity check printed the value of the
foreign key under the primary key's name. It now prints the primary key value the lookup used,
or `{ id : 0 }` for a singleton. The `expect` next to it states its invariant, and the branch
for a multi-column index is gone: SpacetimeDB updates through the primary key only.

Test: reference_integrity_message_test pins the message of an update of a deleted row.
Recorded output: every update_* and upsert_* of a table with a non-private foreign key.

Closes #182

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 4: The unique-constraint error of create lists only the unique columns (#180)

**Files:**
- Create: `examples/test/src/unique_constraint_message_test.rs`
- Modify: `examples/test/src/lib.rs`
- Modify: `derive-input/src/internal/column.rs` (`InternalColumn`, its construction)
- Modify: `derive-input/src/internal/dsl/method/create.rs` (`insert_and_map_errors`)
- Modify: `derive-input/src/internal/dsl/method/message.rs` (remove `whole_row`)
- Modify: `src/error.rs` (`Display` of `UniqueConstraintViolation` from SpacetimeDB)
- Modify: `docs/DOCUMENTATION.md`, `docs/MIGRATION.md`
- Recorded output: every `create_*.snap` and every `upsert_*.snap`; possibly `compile-tests/tests/ui/wrapper_optional_unique_index.stderr`

**Interfaces:**
- Produces: `InternalColumn::spacetimedb_column_is_unique: bool`; the runtime group `unique_constraint_message_test` with the table `ConstraintBadge` (accessor `constraint_badge`, `CreateConstraintBadge { id: u64, code: String, note: String }`). Task 5 relies on both.

- [ ] **Step 1: Write the failing runtime test**

Create `examples/test/src/unique_constraint_message_test.rs`:

```rust
//! SpacetimeDB's unique-constraint error does not say which constraint a row broke, so the
//! error of `create_<table>` lists every unique column with the value handed to SpacetimeDB.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = constraint_badges, method(update = false, delete = false))]
#[spacetimedb::table(accessor = constraint_badge)]
pub struct ConstraintBadge {
    #[primary_key]
    #[create_wrapper]
    id: u64,

    #[unique]
    code: String,

    note: String,
}

/// Creating a badge with the id of a stored one names the primary key and the `#[unique]`
/// column with the values handed over, and leaves out the column that cannot collide.
pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    dsl.create_constraint_badge(CreateConstraintBadge {
        id: 1,
        code: "first".to_string(),
        note: "the stored badge".to_string(),
    })?;

    let result = dsl.create_constraint_badge(CreateConstraintBadge {
        id: 1,
        code: "second".to_string(),
        note: "a badge with the id of the stored one".to_string(),
    });

    let error = match result {
        Ok(badge) => {
            return Err(format!(
                "Creating a badge with the id of a stored one should fail! Got:\n{badge:?}"
            ));
        }
        Err(error) => error,
    };

    let expected = "Unique Constraint Violation Error while trying to create a row in the `constraint_badge` table! Unfortunately SpacetimeDB doesn't provide more information, so here are the unique columns and the values handed to SpacetimeDB: `{ id : 1, code : second }`.";
    if error.to_string() != expected {
        return Err(format!(
            "A unique-constraint violation should list only the unique columns and the values handed to SpacetimeDB!\n\nExpected:\n{expected}\n\nActual:\n{error}"
        ));
    }

    Ok(())
}
```

Register it in `examples/test/src/lib.rs`: `pub mod unique_constraint_message_test;` after `pub mod timestamp_helper_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "unique_constraint_message_test",
        unique_constraint_message_test::run_tests,
    ),
```

- [ ] **Step 2: Observe the red**

Run the runtime gate.
Expected: `FAILED`, with *Actual* ending in `` so here are all columns and their values: `{ constraint_badge : ConstraintBadge { id: 1, code: "second", note: "a badge with the id of the stored one" } }`. ``

- [ ] **Step 3: Record which columns SpacetimeDB checks for uniqueness**

In `derive-input/src/internal/column.rs`, add to `InternalColumn` after `spacetimedb_column_is_auto_inc`:

```rust
    /// Whether SpacetimeDB rejects a second row with the same value in this column: the
    /// primary key, or a column with a unique single-column index.
    pub spacetimedb_column_is_unique: bool,
```

and set it where `InternalColumn` is built, after `spacetimedb_column_is_auto_inc: spacetimedb_column.is_auto_inc,`:

```rust
            spacetimedb_column_is_unique: spacetimedb_column.is_primary_key
                || spacetimedb_column
                    .single_column_index
                    .as_ref()
                    .is_some_and(|index| index.is_unique),
```

- [ ] **Step 4: List the unique columns in the error**

In `create.rs`, replace the head of `insert_and_map_errors` down to, but not including, `let auto_inc_overflow_error =`, by:

```rust
pub(super) fn insert_and_map_errors(
    context: &MethodGenerationContext,
    after_insert_hook: &TokenStream,
) -> TokenStream {
    let MethodGenerationContext {
        internal_columns,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    // SpacetimeDB does not say which unique constraint the row broke, so the message lists
    // every column SpacetimeDB checks for uniqueness.
    let unique_column_names = internal_columns
        .iter()
        .filter(|internal_column| internal_column.spacetimedb_column_is_unique)
        .map(|internal_column| internal_column.rust_field_name.clone())
        .collect_vec();
    let unique_column_values = unique_column_names
        .iter()
        .map(|column_name| quote! { #singular_table_name.#column_name })
        .collect_vec();

    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        singular_table_name_as_string,
        &quote! { Create },
        &quote! { SpacetimeDB },
        &OneOrMultiple::One,
        &message::column_names_and_row_values(&unique_column_names, &unique_column_values),
    );
```

In `message.rs`, delete `whole_row`, which has no caller left.

- [ ] **Step 5: Correct the `Display` text**

In `src/error.rs`, replace the `ErrorFrom::SpacetimeDB` arm by:

```rust
                    ErrorFrom::SpacetimeDB => write!(
                        f,
                        "! {spacetimedb_gives_no_details}, so here are the unique columns and the values handed to SpacetimeDB: `{column_names_and_row_values}`."
                    ),
```

- [ ] **Step 6: Regenerate the snapshots and the compile-test output, then read both diffs**

Expected snapshot hunks only: `format!("{{ inspection : {:?} }}", inspection)` becomes `format!("{{ id : {} }}", inspection.id)`, with one `column : {}` pair per unique column. The primary key is always one of them; a singleton lists `id`.

`wrapper_optional_unique_index.stderr` may gain one more *`Option<spacetimedb::Timestamp>` doesn't implement `std::fmt::Display`* error for the create method. Its `//!` comment already names that limitation. Any other `.stderr` hunk is unexpected: revert it with `git checkout -- <path>` and investigate.

- [ ] **Step 7: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 8: Document the change**

In `docs/DOCUMENTATION.md`, *Example Error Messages*, replace the block under `**UniqueConstraintViolation (from SpacetimeDB):**` by:

````markdown
```txt
Unique Constraint Violation Error while trying to create a row in the `user` table! Unfortunately SpacetimeDB doesn't provide more information, so here are the unique columns and the values handed to SpacetimeDB: `{ id : 0, email : alice@example.com }`.
```
````

In the block under `**UniqueConstraintViolation (from SpacetimeDSL — multi-column):**`, correct `{{ parent_entity_id : 1, child_entity_id : 2 }}` to `` `{ parent_entity_id : 1, child_entity_id : 2 }` ``, which is what the error prints.

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### The unique-constraint error of `create_<table>` lists the unique columns

When SpacetimeDB rejects the row of `create_<table>`, or of the insert path of `upsert_<table>`, for a unique-constraint violation, the error lists the primary key and the `#[unique]` columns with the values handed to SpacetimeDB, `{ id : 1, code : second }`, instead of the whole row. An `#[auto_inc]` column shows `0`, the value SpacetimeDB replaces. The `Display` text says *here are the unique columns and the values handed to SpacetimeDB* instead of *here are all columns and their values*.
```

- [ ] **Step 9: Format, lint, commit**

Commit message:

```text
List only the unique columns in the unique-constraint error of create

SpacetimeDB reports a unique-constraint violation without saying which constraint broke, so
the error listed the whole row. It now lists the primary key and the #[unique] columns with
the values handed to SpacetimeDB. A unique_index multi-column index is checked by SpacetimeDSL
before the insert and never causes this error.

Test: unique_constraint_message_test pins the message.
Recorded output: every create_* and upsert_*.

Closes #180

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 5: Move the row into `try_insert` instead of cloning it (#181)

**Files:**
- Modify: `derive-input/src/internal/dsl/method/create.rs` (`insert_and_map_errors`)
- Modify: `docs/MIGRATION.md`
- Recorded output: every `create_*.snap` and every `upsert_*.snap`

**Interfaces:**
- Consumes: `InternalColumn::spacetimedb_column_is_unique` and the runtime group `unique_constraint_message_test` from Task 4.

The test is the snapshot diff, which shows the clone gone. Task 4's runtime group proves that the copied values still make the same message, including for the `String` column `code`, which has to be cloned rather than moved out of the row.

- [ ] **Step 1: Copy the unique values and move the row**

Replace `insert_and_map_errors` in `create.rs` by:

```rust
/// The `try_insert` of the row bound to the table's singular name, with SpacetimeDB's insert
/// errors mapped to `SpacetimeDSLError` and `after_insert_hook` run on success, shared by
/// `create_<table>` and the insert path of `upsert_<singleton>`.
pub(super) fn insert_and_map_errors(
    context: &MethodGenerationContext,
    after_insert_hook: &TokenStream,
) -> TokenStream {
    let MethodGenerationContext {
        internal_columns,
        singular_table_name,
        singular_table_name_as_string,
        ..
    } = context;

    // SpacetimeDB does not say which unique constraint the row broke, so the message lists
    // every column SpacetimeDB checks for uniqueness. `try_insert` consumes the row and its
    // error carries nothing, so those values are copied first: only they, not the whole row.
    let unique_column_names = internal_columns
        .iter()
        .filter(|internal_column| internal_column.spacetimedb_column_is_unique)
        .map(|internal_column| internal_column.rust_field_name.clone())
        .collect_vec();
    let unique_column_values = format_ident!("unique_column_values");
    let unique_column_copies = unique_column_names
        .iter()
        .map(|column_name| quote! { #singular_table_name.#column_name.clone() })
        .collect_vec();
    let unique_column_value_in_message = (0..unique_column_names.len())
        .map(syn::Index::from)
        .map(|position| quote! { #unique_column_values.#position })
        .collect_vec();

    let unique_constraint_violation_error = runtime::unique_constraint_violation(
        singular_table_name_as_string,
        &quote! { Create },
        &quote! { SpacetimeDB },
        &OneOrMultiple::One,
        &message::column_names_and_row_values(
            &unique_column_names,
            &unique_column_value_in_message,
        ),
    );
    let auto_inc_overflow_error = runtime::auto_inc_overflow(singular_table_name_as_string);
    let unique_constraint_violation =
        spacetimedb::try_insert_error(&quote! { UniqueConstraintViolation });
    let auto_inc_overflow = spacetimedb::try_insert_error(&quote! { AutoIncOverflow });

    quote! {
        let #unique_column_values = (#(#unique_column_copies,)*);

        match self
            .db()
            .#singular_table_name()
            .try_insert(#singular_table_name) {
            Ok(entity) => {
                #after_insert_hook

                Ok(entity)
            },
            Err(error) => match error {
                #unique_constraint_violation(_) => {
                    Err(#unique_constraint_violation_error)
                }
                #auto_inc_overflow(_) => {
                    Err(#auto_inc_overflow_error)
                }
            },
        }
    }
}
```

A table always has a primary key, so the tuple has at least one element; the trailing comma makes a one-element tuple. The copies are built outside the `quote!`, because a repetition in `quote!` may only interpolate iterators.

- [ ] **Step 2: Observe the red**

Run the unit gate. Expected: `FAILED` snapshot tests whose diff adds `let unique_column_values = (inspection.id.clone(),);` and replaces `try_insert(inspection.clone())` by `try_insert(inspection)`.

- [ ] **Step 3: Regenerate the snapshots and read the diff**

Expected hunks only: the added tuple, `.clone()` gone from `try_insert`, and `unique_column_values.0` (`.1`, …) in the message in place of `inspection.id` (…).

- [ ] **Step 4: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`; `unique_constraint_message_test` still passes.

- [ ] **Step 5: Document the change**

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### `create_<table>` moves the row into `try_insert`

`create_<table>` and the insert path of `upsert_<table>` no longer clone the whole row before they insert it. They copy only the values of the unique columns, which the unique-constraint error needs.
```

- [ ] **Step 6: Format, lint, commit**

Commit message:

```text
Move the row into try_insert instead of cloning it

try_insert takes the row by value and its errors carry no data. Only the unique-constraint
message needs values of the row, so create and the insert path of upsert copy the unique
column values into a tuple and move the row into try_insert.

Recorded output: every create_* and upsert_*.

Closes #181

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 6: Check foreign key groups before generating their cascades (#172, part 1)

A pure refactoring. The type and path checks between foreign keys to one table run inside `for_foreign_key` today. That function only runs for a removal some of the keys declare a strategy for, so a group of foreign keys without strategies (Task 9) would skip the checks. The checks move to where the groups are built.

**Files:**
- Modify: `derive-input/src/internal/dsl/method.rs` (`foreign_key_columns_by_referenced_table`, its call, new helpers, imports)
- Modify: `derive-input/src/internal/dsl/method/foreign_key.rs` (remove the checks and `canonical_path`, lift `foreign_key_of`, infallible `for_foreign_key`)

**Interfaces:**
- Produces: `foreign_key::foreign_key_of(column: &Column) -> &ForeignKey` (`pub`, module `method::foreign_key`); `for_foreign_key(…) -> (SpacetimeDSLMethod, TableContributions)` without `syn::Result`; `foreign_key_columns_by_referenced_table(columns: &[Column]) -> syn::Result<BTreeMap<&syn::Ident, Vec<&Column>>>`.

- [ ] **Step 1: Confirm the guarding tests are green**

The compile tests `foreign_keys_with_mismatched_types` and `foreign_keys_with_mismatched_paths` and the fixture `foreign_keys_with_equivalent_spellings` guard this move. Run the unit gate; expected: all `ok`.

- [ ] **Step 2: Lift `foreign_key_of` and remove the checks from `for_foreign_key`**

In `method/foreign_key.rs`:
- Delete `fn canonical_path`.
- Move the nested `fn foreign_key_of` out of `for_foreign_key`, as:

```rust
/// The foreign key of a column which `foreign_key_columns_by_referenced_table` grouped by it.
pub fn foreign_key_of(column: &Column) -> &ForeignKey {
    column
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("columns are grouped by their foreign key, so every one carries it")
}
```

- Delete `canonical_referenced_table_path`, `canonical_referenced_primary_key_type`, and the two `if … { return Err(…) }` blocks inside the loop, together with the `TODO` comment above the first one (it moves to `method.rs` in Step 3).
- Change the return type to `(SpacetimeDSLMethod, TableContributions)` and the last line to `(method, contributions)`.
- Remove `column::canonical_type` and `error` from the `internal::{…}` import.

- [ ] **Step 3: Check the groups where they are built**

In `method.rs`, replace `foreign_key_columns_by_referenced_table` by:

```rust
/// The columns with a foreign key, grouped by the table the foreign key names.
///
/// The columns of a group share the cascade functions and the pairing checks of their
/// referenced table, so they have to agree on the type of its primary key and on its path.
/// That is checked for every group, whether or not its foreign keys declare a strategy.
fn foreign_key_columns_by_referenced_table(
    columns: &[Column],
) -> syn::Result<BTreeMap<&syn::Ident, Vec<&Column>>> {
    let mut columns_by_referenced_table: BTreeMap<&syn::Ident, Vec<&Column>> = BTreeMap::new();

    for column in columns {
        let Some(foreign_key) = &column.spacetimedsl_column.foreign_key else {
            continue;
        };

        let columns_of_the_referenced_table = columns_by_referenced_table
            .entry(&foreign_key.table_name)
            .or_default();

        if let Some(first_column) = columns_of_the_referenced_table.first() {
            reject_foreign_key_disagreeing_with(first_column, column)?;
        }

        columns_of_the_referenced_table.push(column);
    }

    Ok(columns_by_referenced_table)
}

/// Rejects `column` when its type, or the path of its referenced table, differs from
/// `first_column`'s, which references the same table.
fn reject_foreign_key_disagreeing_with(first_column: &Column, column: &Column) -> syn::Result<()> {
    // TODO: https://github.com/tamaro-skaljic/SpacetimeDSL/issues/32 If Option is supported, the type of the primary key values needs to be without option and it's allowed to have both, option and non-option columns.
    if canonical_type(&column.rust_field.type_name_or_path)
        != canonical_type(&first_column.rust_field.type_name_or_path)
    {
        return Err(error::foreign_key_columns_type_mismatch(
            &column.rust_field.name,
        ));
    }

    if canonical_path(&foreign_key_of(column).path)
        != canonical_path(&foreign_key_of(first_column).path)
    {
        return Err(error::foreign_key_columns_path_mismatch(
            &column.rust_field.name,
        ));
    }

    Ok(())
}

/// A module path in one spelling per module, so `::other_crate::tables` and
/// `other_crate::tables` compare equal: the segments without a leading `::`.
fn canonical_path(path: &syn::Path) -> String {
    path.segments
        .iter()
        .map(|segment| segment.ident.to_string())
        .collect::<Vec<_>>()
        .join("::")
}
```

In `SpacetimeDSLTableMethods::generate`, call it with `?`:

```rust
        let foreign_key_columns_by_referenced_table =
            foreign_key_columns_by_referenced_table(columns)?;
```

In `referencing_side_entry_points`, wrap the call in the closure as `Ok(for_foreign_key(…))`, the way `referenced_side_entry_points` wraps `for_referenced_by`.

Imports of `method.rs`: change `foreign_key::for_foreign_key,` to `foreign_key::{for_foreign_key, foreign_key_of},`, and add `internal::{column::canonical_type, error},` next to `internal::dsl::one_or_multiple::OneOrMultiple` in the `crate::{…}` group.

- [ ] **Step 4: Verify the refactoring is pure**

Run the unit gate, then:

```powershell
git diff --stat -- derive/tests/snapshots compile-tests/tests/ui
```

Expected: all `ok`, and no file listed.

- [ ] **Step 5: Format, lint, commit**

Commit message:

```text
Check foreign key groups before generating their cascades

The type and path checks between foreign keys to one table ran inside for_foreign_key, which
only runs for a removal the keys declare a strategy for. They now run where the groups are
built, for every group. A pure refactoring: snapshots and diagnostics are unchanged.

Part of #172

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 7: One pairing module declares and imports every pairing trait (#172, part 2)

The pairing traits keep their names and their conditions in this task; only where they are computed and where their imports sit changes. The imports leave the cascade function bodies for one `const _` block per table. A block scope keeps the imports of two tables in one module apart.

**Files:**
- Create: `derive-input/src/internal/dsl/method/pairing.rs`
- Modify: `derive-input/src/internal/dsl/method/removal.rs` (`Removal::ALL`, `is_performed_by`, `strategy_declared_by`, `strategy_argument`)
- Modify: `derive-input/src/internal/dsl/method.rs` (`mod pairing;`, `generate`, `entry_points_per_removal`, both `*_side_entry_points`, imports)
- Modify: `derive-input/src/internal/dsl/method/foreign_key.rs` (drop the pairing code, return `SpacetimeDSLMethod`)
- Modify: `derive-input/src/internal/dsl/method/referenced_by.rs` (drop the pairing code, return `SpacetimeDSLMethod`)
- Modify: `derive-input/src/internal/dsl/method/naming.rs` (remove the four check names and the paragraph about them)
- Modify: `derive-input/src/internal/dsl/method/context.rs` (`TableContributions::compile_error_check_imports`)
- Modify: `derive-input/src/api/dsl/table.rs` (`SpacetimeDSLTable::compile_error_check_imports`)
- Modify: `derive-input/src/internal/dsl/table.rs` (initialise the field)
- Modify: `derive/src/output.rs` (emit the `const _` block)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Modify: `docs/MIGRATION.md`, `CODE_QUALITY_REPORT.md`
- Recorded output: every snapshot of a table with `#[foreign_key]` or `#[referenced_by]`; possibly the `.stderr` files which quote a pairing trait

**Interfaces:**
- Consumes: `foreign_key_of` and the checked groups from Task 6.
- Produces: `pairing::for_table(context: &MethodGenerationContext, foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>) -> TableContributions`; `Removal::ALL: [Removal; 2]`, `Removal::is_performed_by(self, &SpacetimeDSLTable) -> bool`, `Removal::strategy_declared_by(self, &ForeignKey) -> Option<&OnDeleteStrategy>`, `Removal::strategy_argument(self) -> &'static str` (`"on_delete"` / `"on_soft_delete"`); `SpacetimeDSLTable::compile_error_check_imports: Vec<syn::Path>`; `TableContributions::compile_error_check_imports: Vec<syn::Path>`; `for_foreign_key(…) -> SpacetimeDSLMethod`; `for_referenced_by(…) -> SpacetimeDSLMethod`.

- [ ] **Step 1: Pin the imports in the contract test (the red)**

In `derive/src/data_transfer_contract_tests.rs`, add `compile_error_check_imports: _,` after `compile_error_checks: _,` in `visit_spacetimedsl_table`, and add after the `referencing_table` assertion in `the_model_holds_what_a_table_declares`:

```rust
    let compile_error_check_imports: Vec<String> = spacetimedsl_table
        .compile_error_check_imports
        .iter()
        .map(|path| path.to_token_stream().to_string())
        .collect();
    assert_eq!(
        compile_error_check_imports,
        [
            "crate :: part :: this_compilation_error_occurs_because_the_part_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_gadget_table",
            "crate :: part :: this_compilation_error_occurs_because_the_part_table_has_no_foreign_key_attribute_with_on_soft_delete_defined_referencing_the_gadget_table",
            "crate :: owner :: this_compilation_error_occurs_because_the_owner_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_gadget_table",
        ]
    );
```

Run the unit gate. Expected: the `derive` crate fails to compile with `error[E0026]: struct SpacetimeDSLTable does not have a field named compile_error_check_imports`.

- [ ] **Step 2: Add the field to the public model**

In `derive-input/src/api/dsl/table.rs`, add after `compile_error_checks`:

```rust
    /// The marker traits the tables on the other side of a foreign key have to declare, as the
    /// paths the expansion imports them from. A missing one is an unresolved import whose name
    /// says what to change.
    pub compile_error_check_imports: Vec<syn::Path>,
```

In `derive-input/src/internal/dsl/table.rs`, initialise it next to `compile_error_checks` and let one comment cover both:

```rust
            referencing_tables,
            // `TableContributions::apply_to` fills these in, after the methods are generated.
            compile_error_checks: BTreeSet::new(),
            compile_error_check_imports: vec![],
            create_dsl_method_arg: None,
```

In `method/context.rs`, add to `TableContributions`:

```rust
    pub compile_error_check_imports: Vec<syn::Path>,
```

extend `merge` with `self.compile_error_check_imports.extend(other.compile_error_check_imports);` and `apply_to` with:

```rust
        spacetimedsl_table
            .compile_error_check_imports
            .extend(self.compile_error_check_imports);
```

- [ ] **Step 3: Give `Removal` the facts the pairing and the cascades share**

In `method/removal.rs`, extend `impl Removal` (add `api::dsl::foreign_key::ForeignKey` to the imports; `OnDeleteStrategy` and `SpacetimeDSLTable` are imported already):

```rust
    /// Both kinds of removal, deletion first.
    pub const ALL: [Removal; 2] = [Removal::Hard, Removal::Soft];

    /// Whether `spacetimedsl_table` performs this kind of removal: `Hard` with a delete
    /// method, `Soft` when it is soft-deletable.
    pub fn is_performed_by(self, spacetimedsl_table: &SpacetimeDSLTable) -> bool {
        match self {
            Removal::Hard => spacetimedsl_table.has_delete_method,
            Removal::Soft => spacetimedsl_table.is_soft_deletable(),
        }
    }

    /// The strategy `foreign_key` declares for this kind of removal of the referenced row.
    pub fn strategy_declared_by(self, foreign_key: &ForeignKey) -> Option<&OnDeleteStrategy> {
        match self {
            Removal::Hard => foreign_key.on_delete_strategy.as_ref(),
            Removal::Soft => foreign_key.on_soft_delete_strategy.as_ref(),
        }
    }

    /// The `#[foreign_key]` argument which declares the strategy for this kind of removal.
    pub fn strategy_argument(self) -> &'static str {
        match self {
            Removal::Hard => "on_delete",
            Removal::Soft => "on_soft_delete",
        }
    }
```

- [ ] **Step 4: Create the pairing module**

Create `derive-input/src/internal/dsl/method/pairing.rs`:

```rust
//! The compile-time checks which pair a table with the tables on the other side of its foreign
//! keys.
//!
//! A table cannot see how the tables on the other side are declared, only what their expansion
//! emits. So each side declares marker traits for what it offers and imports from the other
//! side the traits it needs. A counterpart which does not offer one is an unresolved import,
//! and the trait's name says what to change.
//!
//! These names are part of the generated API: both sides build them here, from the same two
//! table names, and changing one breaks every module generated against the previous name
//! until it is regenerated.

use {
    super::{context::{MethodGenerationContext, TableContributions}, foreign_key::foreign_key_of, removal::Removal},
    crate::api::Column,
    quote::format_ident,
    std::collections::BTreeMap,
    syn::{Ident, Path, PathSegment},
};

/// The traits the table of `context` declares and imports, as the referenced table its
/// `#[referenced_by]` attributes describe, and as the referencing table its foreign keys make
/// it.
pub fn for_table(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = as_referenced_table(context);

    contributions.merge(as_referencing_table(
        &context.singular_table_name,
        foreign_key_columns_by_referenced_table,
    ));

    contributions
}

/// For every table the `#[referenced_by]` attributes name and every removal this table
/// performs: the trait which says so, and the import of the trait the other table declares
/// when a foreign key of it declares a strategy for that removal.
fn as_referenced_table(context: &MethodGenerationContext) -> TableContributions {
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        ..
    } = context;

    let mut contributions = TableContributions::default();

    for referencing_table in &spacetimedsl_table.referencing_tables {
        for removal in Removal::ALL {
            if !removal.is_performed_by(spacetimedsl_table) {
                continue;
            }

            contributions
                .compile_error_checks
                .insert(referenced_table_compile_error_check(
                    removal,
                    singular_table_name,
                    &referencing_table.table_name,
                ));

            contributions.compile_error_check_imports.push(imported_from(
                &referencing_table.path,
                referencing_table_compile_error_check(
                    removal,
                    &referencing_table.table_name,
                    singular_table_name,
                ),
            ));
        }
    }

    contributions
}

/// For every table the foreign keys reference and every removal one of them declares a
/// strategy for: the trait which says so, and the import of the trait the referenced table
/// declares when it performs that removal.
fn as_referencing_table(
    singular_table_name: &Ident,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = TableContributions::default();

    for (referenced_table_name, columns_with_foreign_key) in foreign_key_columns_by_referenced_table {
        let referenced_table_path = &foreign_key_of(columns_with_foreign_key[0]).path;

        for removal in Removal::ALL {
            let declares_a_strategy = columns_with_foreign_key
                .iter()
                .any(|column| removal.strategy_declared_by(foreign_key_of(column)).is_some());

            if !declares_a_strategy {
                continue;
            }

            contributions
                .compile_error_checks
                .insert(referencing_table_compile_error_check(
                    removal,
                    singular_table_name,
                    referenced_table_name,
                ));

            contributions.compile_error_check_imports.push(imported_from(
                referenced_table_path,
                referenced_table_compile_error_check(
                    removal,
                    referenced_table_name,
                    singular_table_name,
                ),
            ));
        }
    }

    contributions
}

/// `this_compilation_error_occurs_because_the_<referenced>_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_<referencing>_table`,
/// or `…_soft_deletable…` for a soft deletion.
fn referenced_table_compile_error_check(
    removal: Removal,
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    let capability = match removal {
        Removal::Hard => "deletable",
        Removal::Soft => "soft_deletable",
    };

    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_{capability}_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

/// `this_compilation_error_occurs_because_the_<referencing>_table_has_no_foreign_key_attribute_with_on_delete_defined_referencing_the_<referenced>_table`,
/// or `…on_soft_delete…` for a soft deletion.
fn referencing_table_compile_error_check(
    removal: Removal,
    referencing_table_name: &Ident,
    referenced_table_name: &Ident,
) -> Ident {
    let strategy_argument = removal.strategy_argument();

    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_with_{strategy_argument}_defined_referencing_the_{referenced_table_name}_table"
    )
}

/// `<module_path>::<trait_name>`: where the expansion imports another table's trait from.
fn imported_from(module_path: &Path, trait_name: Ident) -> Path {
    let mut path = module_path.clone();
    path.segments.push(PathSegment::from(trait_name));
    path
}
```

Let `.\x.ps1 format` lay out the imports.

- [ ] **Step 5: Remove the pairing from the cascade generators**

In `method/foreign_key.rs`: delete everything from `// This table emits the half it declares a strategy for, …` through `let compile_error_check_usage = quote! { use #referenced_table_path::#compile_error_check; };`. Delete `#compile_error_check_usage` from `function_impl`, the `let mut contributions = TableContributions::default();` line, and the `referenced_table_path` bindings. Return `method` with the return type `SpacetimeDSLMethod`. Drop the four `*_compile_error_check_*` names and `TableContributions` from the imports.

In `method/referenced_by.rs`: delete the `compile_error_check_usages` vector, the two `let compile_error_check = …` blocks with their comment and the `contributions` insertion inside the loop, the `referencing_table_path` binding, and `#(#compile_error_check_usages)*` from `function_impl`. Return `method` with the return type `SpacetimeDSLMethod`. Drop the four names and `TableContributions` from the imports.

In `method/naming.rs`: delete the four functions `referenced_table_compile_error_check_for_deletions`, `referenced_table_compile_error_check_for_soft_deletions`, `referencing_table_compile_error_check_for_deletions` and `referencing_table_compile_error_check_for_soft_deletions`. Delete the module-doc paragraph starting *Each direction of the paired check is split by capability*, and end the first paragraph's list at *the trait a hook function implements, and the identifiers two tables in a foreign key relationship share*.

- [ ] **Step 6: Wire the pairing into `generate`**

In `method.rs`, add `mod pairing;` to the module list, and replace `generate`, `entry_points_per_removal`, `referenced_side_entry_points` and `referencing_side_entry_points` by:

```rust
impl SpacetimeDSLTableMethods {
    pub(crate) fn generate(
        context: &MethodGenerationContext,
        columns: &[Column],
    ) -> syn::Result<(SpacetimeDSLTableMethods, TableContributions)> {
        let mut contributions = TableContributions::default();

        let (create, get_all, get_count) = table_level_methods(context, &mut contributions);

        let on_delete_strategies_of_referencing_tables = referenced_side_entry_points(context);

        let foreign_key_columns_by_referenced_table =
            foreign_key_columns_by_referenced_table(columns)?;

        let on_delete_strategies_of_this_table =
            referencing_side_entry_points(context, &foreign_key_columns_by_referenced_table);

        contributions.merge(pairing::for_table(
            context,
            &foreign_key_columns_by_referenced_table,
        ));

        let methods = SpacetimeDSLTableMethods {
            create,
            get_all,
            get_count,
            on_delete_strategies_of_referencing_tables,
            on_delete_strategies_of_this_table,
            multi_column_indices: multi_column_index_methods(context),
            wrapper_methods: wrapper_methods(context, &foreign_key_columns_by_referenced_table),
        };

        Ok((methods, contributions))
    }
}

/// The one-row and the many-row entry point for each kind of removal in `removal_kinds`, as
/// `(on_deletion, on_soft_deletion)`. `build` generates one entry point.
fn entry_points_per_removal(
    removal_kinds: impl IntoIterator<Item = Removal>,
    mut build: impl FnMut(Removal, &OneOrMultiple) -> SpacetimeDSLMethod,
) -> (Option<CascadeEntryPoints>, Option<CascadeEntryPoints>) {
    let mut on_deletion = None;
    let mut on_soft_deletion = None;

    for removal in removal_kinds {
        let entry_points = Some(CascadeEntryPoints {
            after_one_row: build(removal, &OneOrMultiple::One),
            after_multiple_rows: build(removal, &OneOrMultiple::Multiple),
        });

        match removal {
            Removal::Hard => on_deletion = entry_points,
            Removal::Soft => on_soft_deletion = entry_points,
        }
    }

    (on_deletion, on_soft_deletion)
}

/// The entry points a referenced table offers its referencing tables: one pair per kind of
/// removal it can perform, because the cascade a referencing table runs depends on which of
/// the two reached it.
fn referenced_side_entry_points(
    context: &MethodGenerationContext,
) -> Option<OnDeleteStrategiesOfReferencingTables> {
    let MethodGenerationContext {
        spacetimedb_table,
        spacetimedsl_table,
        primary_key_column,
        ..
    } = context;

    if spacetimedsl_table.referencing_tables.is_empty() {
        return None;
    }

    let removal_kinds = Removal::ALL
        .into_iter()
        .filter(|removal| removal.is_performed_by(spacetimedsl_table));

    let (on_deletion, on_soft_deletion) =
        entry_points_per_removal(removal_kinds, |removal, one_or_multiple| {
            for_referenced_by(
                removal,
                one_or_multiple,
                spacetimedb_table,
                spacetimedsl_table,
                primary_key_column,
            )
        });

    Some(OnDeleteStrategiesOfReferencingTables {
        on_deletion,
        on_soft_deletion,
    })
}

/// The entry points this table offers each table it references: one pair per kind of
/// removal its foreign keys to that table declare a strategy for. A key that sets only
/// `on_soft_delete` contributes nothing to the deletion pair, and the other way round.
fn referencing_side_entry_points(
    context: &MethodGenerationContext,
    foreign_key_columns_by_referenced_table: &BTreeMap<&syn::Ident, Vec<&Column>>,
) -> Vec<OnDeleteStrategiesOfTheReferencedTable> {
    let referencing_tables = match context.spacetimedsl_table.referencing_tables.is_empty() {
        true => ReferencingTables::Absent,
        false => ReferencingTables::Present,
    };

    foreign_key_columns_by_referenced_table
        .iter()
        .map(|(referenced_table_name, columns_with_foreign_key)| {
            let removal_kinds = Removal::ALL.into_iter().filter(|removal| {
                columns_with_foreign_key
                    .iter()
                    .any(|column| removal.strategy_declared_by(foreign_key_of(column)).is_some())
            });

            let (on_deletion, on_soft_deletion) =
                entry_points_per_removal(removal_kinds, |removal, one_or_multiple| {
                    for_foreign_key(
                        removal,
                        one_or_multiple,
                        referencing_tables,
                        context,
                        referenced_table_name,
                        columns_with_foreign_key,
                    )
                });

            OnDeleteStrategiesOfTheReferencedTable {
                on_deletion,
                on_soft_deletion,
            }
        })
        .collect()
}
```

Remove the `itertools::Itertools` import if nothing in `method.rs` uses it any more.

- [ ] **Step 7: Emit the imports in a block scope**

In `derive/src/output.rs`, after the loop that builds `compile_error_checks`, add:

```rust
    let compile_error_check_imports = &input.spacetimedsl_table.compile_error_check_imports;

    // A block scope, so the imports of two tables in one module cannot clash.
    let compile_error_check_usages = match compile_error_check_imports.is_empty() {
        true => TokenStream::default(),
        false => quote! {
            const _: () = {
                #(use #compile_error_check_imports;)*
            };
        },
    };
```

and emit it right after the traits:

```rust
        items_outside_dsl_methods: quote! {
            #(#compile_error_checks)*

            #compile_error_check_usages

            #(#wrapper_types)*
```

- [ ] **Step 8: Observe the contract test turn green**

Run the unit gate. Expected: `the_model_holds_what_a_table_declares` passes; snapshot tests fail with diffs that move `use …::this_compilation_error_…;` out of the cascade functions into a `const _: () = { … };` block in `table.snap`.

- [ ] **Step 9: Regenerate the snapshots and the compile-test output, then read both diffs**

Expected snapshot hunks only: the `use` lines leave `internal_methods.snap`, and each `table.snap` of a table with `#[foreign_key]` or `#[referenced_by]` gains one `const _` block with exactly those imports. Check `self_referencing_cascade/Folder/table.snap`: its block imports `self::…` traits which the same file declares. Check `foreign_keys_with_equivalent_spellings`: its imports keep the path as the first column spells it.

Expected `.stderr` hunks: none, or a changed order of the `E0432` errors. Any changed trait name is unexpected.

- [ ] **Step 10: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. The runtime gate compiles every `const _` block, including those of the self-referencing tables.

- [ ] **Step 11: Document the change**

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### The foreign key pairing is imported once per table

The `use` statements which pair a `#[foreign_key]` with its `#[referenced_by]` moved out of the generated cascade methods into one `const _: () = { … };` block per table.
```

Under `### For crates building on spacetimedsl_derive-input`, add:

```markdown
#### `SpacetimeDSLTable::compile_error_check_imports` lists the pairing imports

`SpacetimeDSLTable::compile_error_check_imports: Vec<syn::Path>` holds the marker traits the tables on the other side of the table's foreign keys have to declare, as the paths to import them from. The `method_impl` of a cascade method no longer contains these imports. Emit them in a block scope, as `spacetimedsl_derive` does with `const _: () = { use …; };`, or the pairing checks are lost.
```

In `CODE_QUALITY_REPORT.md`, entry *`table.rs`: `pub struct SpacetimeDSLTable`*, replace the finding and the recommendation by:

```markdown
`SpacetimeDSLTable::try_parse` builds the table with `create_dsl_method_arg: None` and empty `compile_error_checks` and `compile_error_check_imports`, and method generation fills them in later through `TableContributions::apply_to`. In between, the value looks complete but is not; a generator reading those fields during generation would silently see the empty state.

Recommendation: Keep generation results out of the parsed table — for example move `create_dsl_method_arg`, `compile_error_checks` and `compile_error_check_imports` into `SpacetimeDSLTableMethods` or into a separate result type — so that `SpacetimeDSLTable` does not change after parsing. This changes the public API, whose fields the plan's Task 13 documents and pins as a data-transfer contract.
```

- [ ] **Step 12: Format, lint, commit**

Commit message:

```text
Declare and import every pairing trait from one module

pairing.rs computes the marker traits a table declares and the ones it imports from the
tables on the other side of its foreign keys. The imports leave the cascade function bodies
for one const _ block per table, from the new field
SpacetimeDSLTable::compile_error_check_imports. Names and conditions are unchanged.

Recorded output: the use lines moved from internal_methods.snap into table.snap.

Part of #172

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 8: Pair every foreign key and every `#[referenced_by]` (#172, part 3)

The new rules:
- A referenced table declares, for each `#[referenced_by(table = B)]` and each removal, one of two traits. It declares `…is_not_deletable…` when it performs the removal, and `…your_foreign_key_…needs_to_define_a_strategy_for_on_delete_or…` when it does not.
- A referencing table imports, for each referenced table and removal, the trait matching each of its foreign keys. A group of foreign keys where one declares the strategy and one does not imports both, and one of them is missing.
- A referencing table declares `…the_{B}_table_has_no_foreign_key_attribute_referencing_the_{A}_table` once per referenced table. The referenced table imports it once per `#[referenced_by]`. It replaces the two per-removal traits `…with_on_delete_defined…` and `…with_on_soft_delete_defined…`.

**Files:**
- Modify: `derive-input/src/internal/dsl/method/pairing.rs`
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Create: `compile-tests/tests/ui/foreign_key_without_on_delete_to_deletable_table.rs` (+ `.stderr`)
- Create: `compile-tests/tests/ui/foreign_key_without_on_soft_delete_to_soft_deletable_table.rs` (+ `.stderr`)
- Create: `compile-tests/tests/ui/referenced_by_without_foreign_key_back.rs` (+ `.stderr`)
- Modify: `docs/DOCUMENTATION.md` (*Pairing Requirement*, *DSL-Specific Mistakes*), `docs/MIGRATION.md`
- Recorded output: every `table.snap` of a table with `#[foreign_key]` or `#[referenced_by]`; the `.stderr` files which quote a pairing trait

**Interfaces:**
- Consumes: `Removal::ALL`, `is_performed_by`, `strategy_declared_by`, `strategy_argument` and `imported_from` from Task 7.
- Produces: the trait names `removal_is_possible`, `removal_is_impossible` and `foreign_key_exists` build (private to `pairing.rs`), which Task 9's diagnostics quote.

- [ ] **Step 1: Write the failing compile-tests**

Create `compile-tests/tests/ui/foreign_key_without_on_delete_to_deletable_table.rs`:

```rust
//! Every foreign key declares a strategy for each removal its referenced table performs, and
//! the foreign keys of one table are checked one by one. `destination_warehouse_id` leaves
//! out `on_delete` although `warehouse` has a delete method, and `origin_warehouse_id`
//! setting it does not cover for it: deleting a warehouse would leave the destinations
//! dangling.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(
        plural_name = warehouses,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        pub name: String,

        deleted: bool,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = Delete, on_soft_delete = Ignore)]
        pub origin_warehouse_id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_soft_delete = Ignore)]
        pub destination_warehouse_id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/foreign_key_without_on_soft_delete_to_soft_deletable_table.rs`:

```rust
//! Every foreign key declares a strategy for each removal its referenced table performs.
//! `warehouse` is soft-deletable, but the foreign key leaves out `on_soft_delete`, so retiring
//! a warehouse would find no strategy for its shipments.
//!
//! The two errors after the first follow from the same gap: the soft-deletion cascade of
//! `warehouse` calls the two functions an `on_soft_delete` strategy would have generated.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(
        plural_name = warehouses,
        method(update = true, delete = true, soft_delete = true),
    )]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,

        pub name: String,

        deleted: bool,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = Delete)]
        pub warehouse_id: u64,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/referenced_by_without_foreign_key_back.rs`:

```rust
//! A `#[referenced_by]` names a table which has a foreign key back. `shipment` has none, so
//! the attribute claims a relationship that does not exist.
//!
//! The two errors after the first follow from the same gap: the deletion cascade of
//! `warehouse` calls the two functions a foreign key of `shipment` would have generated.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
```

Each is valid except for its one rejection. Adding `on_delete = Error` to `destination_warehouse_id`, `on_soft_delete = Ignore` to `warehouse_id`, or removing the `#[referenced_by]` line makes it compile.

- [ ] **Step 2: Observe the red**

Run the unit gate. Expected: `invalid_table_definitions_are_rejected` fails.
- `foreign_key_without_on_delete_to_deletable_table` *compiles* — trybuild reports that it expected a compile failure. That is the gap under test.
- The other two fail with the old names (`…has_no_foreign_key_attribute_with_on_soft_delete_defined…`, `…with_on_delete_defined…`) and have no `.stderr` yet. Read their error text.

- [ ] **Step 3: Pin the new rules in the contract test**

In `the_model_holds_what_a_table_declares`, replace the expected imports and add the declared traits:

```rust
    assert_eq!(
        compile_error_check_imports,
        [
            "crate :: part :: this_compilation_error_occurs_because_the_part_table_has_no_foreign_key_attribute_referencing_the_gadget_table",
            "crate :: owner :: this_compilation_error_occurs_because_the_owner_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_gadget_table",
            "crate :: owner :: this_compilation_error_occurs_because_your_foreign_key_referencing_the_owner_table_needs_to_define_a_strategy_for_on_soft_delete_or_the_owner_table_has_no_referenced_by_attribute_referencing_the_gadget_table",
        ]
    );
    let compile_error_checks: Vec<String> = spacetimedsl_table
        .compile_error_checks
        .iter()
        .map(ToString::to_string)
        .collect();
    assert_eq!(
        compile_error_checks,
        [
            "this_compilation_error_occurs_because_the_gadget_table_has_no_foreign_key_attribute_referencing_the_owner_table",
            "this_compilation_error_occurs_because_the_gadget_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_part_table",
            "this_compilation_error_occurs_because_the_gadget_table_is_not_soft_deletable_or_has_no_referenced_by_attribute_referencing_the_part_table",
        ]
    );
```

- [ ] **Step 4: Implement the rules**

In `pairing.rs`, replace `as_referenced_table`, `as_referencing_table`, `referenced_table_compile_error_check` and `referencing_table_compile_error_check` by:

```rust
/// For every table the `#[referenced_by]` attributes name: per removal, the trait which says
/// whether this table performs it, and the import of the trait which says that the other
/// table has a foreign key back.
fn as_referenced_table(context: &MethodGenerationContext) -> TableContributions {
    let MethodGenerationContext {
        spacetimedsl_table,
        singular_table_name,
        ..
    } = context;

    let mut contributions = TableContributions::default();

    for referencing_table in &spacetimedsl_table.referencing_tables {
        let referencing_table_name = &referencing_table.table_name;

        for removal in Removal::ALL {
            let declared = match removal.is_performed_by(spacetimedsl_table) {
                true => removal_is_possible(removal, singular_table_name, referencing_table_name),
                false => {
                    removal_is_impossible(removal, singular_table_name, referencing_table_name)
                }
            };

            contributions.compile_error_checks.insert(declared);
        }

        contributions.compile_error_check_imports.push(imported_from(
            &referencing_table.path,
            foreign_key_exists(referencing_table_name, singular_table_name),
        ));
    }

    contributions
}

/// For every table the foreign keys reference: the trait which says that this table has a
/// foreign key to it, and per removal the import of the trait matching each foreign key.
///
/// A foreign key declares a strategy exactly when the referenced table performs the removal,
/// so a group where one foreign key declares it and another does not imports both traits,
/// and one of them is missing.
fn as_referencing_table(
    singular_table_name: &Ident,
    foreign_key_columns_by_referenced_table: &BTreeMap<&Ident, Vec<&Column>>,
) -> TableContributions {
    let mut contributions = TableContributions::default();

    for (referenced_table_name, columns_with_foreign_key) in foreign_key_columns_by_referenced_table {
        let referenced_table_path = &foreign_key_of(columns_with_foreign_key[0]).path;

        contributions
            .compile_error_checks
            .insert(foreign_key_exists(singular_table_name, referenced_table_name));

        for removal in Removal::ALL {
            let declares_a_strategy =
                |column: &&Column| removal.strategy_declared_by(foreign_key_of(column)).is_some();

            if columns_with_foreign_key.iter().any(declares_a_strategy) {
                contributions.compile_error_check_imports.push(imported_from(
                    referenced_table_path,
                    removal_is_possible(removal, referenced_table_name, singular_table_name),
                ));
            }

            if !columns_with_foreign_key.iter().all(declares_a_strategy) {
                contributions.compile_error_check_imports.push(imported_from(
                    referenced_table_path,
                    removal_is_impossible(removal, referenced_table_name, singular_table_name),
                ));
            }
        }
    }

    contributions
}

/// Declared by a referenced table for each table its `#[referenced_by]` names and each
/// removal it performs. A foreign key declaring a strategy for that removal imports it.
fn removal_is_possible(
    removal: Removal,
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    let capability = match removal {
        Removal::Hard => "deletable",
        Removal::Soft => "soft_deletable",
    };

    format_ident!(
        "this_compilation_error_occurs_because_the_{referenced_table_name}_table_is_not_{capability}_or_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

/// Declared by a referenced table for each table its `#[referenced_by]` names and each
/// removal it does not perform. A foreign key declaring no strategy for that removal imports
/// it.
fn removal_is_impossible(
    removal: Removal,
    referenced_table_name: &Ident,
    referencing_table_name: &Ident,
) -> Ident {
    let strategy_argument = removal.strategy_argument();

    format_ident!(
        "this_compilation_error_occurs_because_your_foreign_key_referencing_the_{referenced_table_name}_table_needs_to_define_a_strategy_for_{strategy_argument}_or_the_{referenced_table_name}_table_has_no_referenced_by_attribute_referencing_the_{referencing_table_name}_table"
    )
}

/// Declared by a referencing table for each table its foreign keys reference. The referenced
/// table imports it for each table its `#[referenced_by]` names.
fn foreign_key_exists(referencing_table_name: &Ident, referenced_table_name: &Ident) -> Ident {
    format_ident!(
        "this_compilation_error_occurs_because_the_{referencing_table_name}_table_has_no_foreign_key_attribute_referencing_the_{referenced_table_name}_table"
    )
}
```

Update the doc comment of `for_table`: *The traits the table of `context` declares and imports, as the table its `#[referenced_by]` attributes describe and as the table whose foreign keys reference others.*

- [ ] **Step 5: Regenerate the compile-test output and read it**

Expected new `.stderr` files:
- `foreign_key_without_on_delete_to_deletable_table`: one `E0432` for `crate::warehouse::this_compilation_error_occurs_because_your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`, nothing else.
- `foreign_key_without_on_soft_delete_to_soft_deletable_table`: one `E0432` for `crate::warehouse::…needs_to_define_a_strategy_for_on_soft_delete_or…_shipment_table`, followed by two `E0599` for the missing `…after_one_row_of_the_warehouse_table_was_soft_deleted` / `…after_multiple_rows_of_the_warehouse_table_were_soft_deleted`.
- `referenced_by_without_foreign_key_back`: one `E0432` for `crate::shipment::this_compilation_error_occurs_because_the_shipment_table_has_no_foreign_key_attribute_referencing_the_warehouse_table`, followed by two `E0599` for the missing deletion cascade functions.

Expected changes of existing `.stderr` files: in the follow-on errors, `…has_no_foreign_key_attribute_with_on_delete_defined_referencing…` and `…with_on_soft_delete_defined…` become `…has_no_foreign_key_attribute_referencing…`. A referencing table without `on_soft_delete` may gain a follow-on `E0432` for `…needs_to_define_a_strategy_for_on_soft_delete…`. Read every hunk; the first error of each file must not change.

- [ ] **Step 6: Regenerate the snapshots and read the diff**

Expected hunks only, in `table.snap`:
- referenced tables declare `…needs_to_define_a_strategy_for_on_soft_delete…` for each `#[referenced_by]` when they are not soft-deletable (and the `on_delete` twin when they are not deletable);
- referencing tables declare `…has_no_foreign_key_attribute_referencing…` instead of `…with_on_delete_defined…` / `…with_on_soft_delete_defined…`;
- the `const _` blocks import accordingly.

- [ ] **Step 7: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`. Every existing example declares a strategy for exactly the removals its referenced table performs; a failure here names an example that did not, which is the new rejection doing its job. Fix that example by adding the missing strategy, and name it in the commit message.

- [ ] **Step 8: Document the change**

In `docs/DOCUMENTATION.md`, replace the section `### Pairing Requirement` up to `### path Parameter` by:

````markdown
### Pairing Requirement

Every `#[foreign_key]` needs a `#[referenced_by]` naming its table on the referenced table's primary key, and every `#[referenced_by]` needs a `#[foreign_key]` back in the table it names. Each foreign key declares a strategy for exactly the removals its referenced table performs: `on_delete` when it has a delete method, `on_soft_delete` when it is soft-deletable. The foreign keys of one table are checked one by one. A broken rule is an unresolved import whose name says what to change:

```txt
unresolved import crate::entity::this_compilation_error_occurs_because_the_entity_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_position_table
unresolved import crate::entity::this_compilation_error_occurs_because_your_foreign_key_referencing_the_entity_table_needs_to_define_a_strategy_for_on_delete_or_the_entity_table_has_no_referenced_by_attribute_referencing_the_position_table
unresolved import crate::position::this_compilation_error_occurs_because_the_position_table_has_no_foreign_key_attribute_referencing_the_entity_table
```

- The first: a foreign key of `position` sets `on_delete`, but `entity` has no delete method, or no `#[referenced_by]` naming `position`.
- The second: `entity` has a delete method, but a foreign key of `position` to it does not set `on_delete`, or `entity` has no `#[referenced_by]` naming `position`.
- The third: `entity` names `position` in `#[referenced_by]`, but `position` has no foreign key to `entity`.

Soft deletion is checked the same way, with `soft_deletable` and `on_soft_delete` in the names.

````

In *DSL-Specific Mistakes*, add after the row *Missing `#[foreign_key]` for a `#[referenced_by]`*:

```markdown
| A foreign key without `on_delete` to a deletable table | Set `on_delete`; each foreign key reacts to every removal its referenced table performs |
```

In `docs/MIGRATION.md`, under `### Newly rejected inputs`, add:

```markdown
#### A foreign key which leaves out the strategy of a removal its table performs

Every `#[foreign_key]` has to set `on_delete` when the referenced table has a delete method, and `on_soft_delete` when it is soft-deletable. Two foreign keys of one table to the same table used to be checked together, so one of them could leave out `on_delete` while the other set it; deleting a referenced row then left the rows of the first one pointing at nothing. Each is checked on its own now: *unresolved import `…::this_compilation_error_occurs_because_your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`*. Add the missing strategy.
```

Under `### Changed messages and generated code`, add:

```markdown
#### The pairing errors name what to change

A broken pairing between `#[foreign_key]` and `#[referenced_by]` is still an unresolved import, with these names:

- A foreign key missing a strategy for a removal its table performs: `…your_foreign_key_referencing_the_<table>_table_needs_to_define_a_strategy_for_on_delete_or_the_<table>_table_has_no_referenced_by_attribute_referencing_the_<other>_table` (or `…on_soft_delete…`).
- A `#[referenced_by]` naming a table without a foreign key back: `…the_<other>_table_has_no_foreign_key_attribute_referencing_the_<table>_table`, which replaces `…has_no_foreign_key_attribute_with_on_delete_defined…` and `…with_on_soft_delete_defined…`.
- A foreign key setting a strategy its table cannot use keeps `…the_<table>_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_<other>_table` (or `…is_not_soft_deletable…`).
```

- [ ] **Step 9: Format, lint, commit**

Commit message:

```text
Pair every foreign key and every referenced_by

Each foreign key now has to declare a strategy for exactly the removals its referenced table
performs: a referenced table declares, per #[referenced_by] and removal, whether it performs
it, and each foreign key imports the matching trait. A #[referenced_by] imports a trait the
other table declares for each table its foreign keys reference, which replaces the two
per-removal traits.

Tests: foreign_key_without_on_delete_to_deletable_table (newly rejected),
foreign_key_without_on_soft_delete_to_soft_deletable_table and
referenced_by_without_foreign_key_back (new names); the contract test pins the traits.
Recorded output: table.snap of every table with a foreign key or #[referenced_by], and the
follow-on errors of the compile tests.

Part of #172

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 9: Reference tables whose rows are never removed (#172, part 4)

**Files:**
- Modify: `derive-input/src/internal/dsl/reference.rs` (drop the rejection and two parameters)
- Modify: `derive-input/src/internal/dsl/table.rs` (the call)
- Modify: `derive-input/src/internal/dsl/foreign_key.rs` (drop the "at least one strategy" rejection)
- Modify: `derive-input/src/internal/error.rs` (delete two diagnostics)
- Modify: `derive-input/src/internal/dsl/method.rs` (entry points only where a cascade runs)
- Modify: `derive-input/src/api/dsl/table.rs` (docs of `OnDeleteStrategiesOfReferencingTables` and two fields)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Rename: `compile-tests/tests/ui/referenced_by_without_delete_or_soft_delete_method.*` to `foreign_key_with_on_delete_to_table_without_delete_method.*`
- Rename: `compile-tests/tests/ui/foreign_key_without_any_on_delete_field.*` to `foreign_key_without_strategy_to_deletable_table.*`
- Create: `compile-tests/tests/ui/foreign_keys_with_mismatched_types_without_strategy.rs` (+ `.stderr`)
- Create: `compile-tests/tests/ui/referenced_by_without_foreign_key_back_on_table_without_delete_methods.rs` (+ `.stderr`)
- Create: `derive/tests/fixtures/foreign_key_to_table_without_delete_methods.rs`; register in `derive/src/characterization_tests.rs`
- Create: `examples/test/src/referenced_table_without_delete_methods_test.rs`; register in `examples/test/src/lib.rs`
- Modify: `docs/DOCUMENTATION.md` (*Method Configuration*, *Declaration*, *Pairing Requirement*), `docs/MIGRATION.md`, `CODE_QUALITY_REPORT.md`

**Interfaces:**
- Consumes: the pairing of Task 8, which already declares the `removal_is_impossible` traits for tables that cannot remove rows.
- Produces: `ReferencingTable::try_parse(field: &SatsField<'_>) -> syn::Result<Vec<ReferencingTable>>`. `on_delete_strategies_of_referencing_tables` is `None` for a table which performs no removal. `on_delete_strategies_of_this_table` leaves out a referenced table whose foreign keys declare no strategy.

- [ ] **Step 1: Write the failing contract assertions**

In `the_model_holds_what_a_table_declares`, before `let cleanup_timer = …`, add:

```rust
    let currency = parse_table(
        quote! { plural_name = currencies, method(update = false, delete = false) },
        quote! {
            #[spacetimedb::table(accessor = currency, public)]
            pub struct Currency {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                #[referenced_by(path = crate::price, table = price)]
                id: u64,
            }
        },
    );
    visit_table(&currency);

    assert!(
        currency
            .spacetimedsl_methods
            .on_delete_strategies_of_referencing_tables
            .is_none(),
        "a table which neither deletes nor soft-deletes rows offers no cascade entry points"
    );

    let price = parse_table(
        quote! { plural_name = prices, method(update = true, delete = true) },
        quote! {
            #[spacetimedb::table(accessor = price, public)]
            pub struct Price {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                id: u64,

                #[index(btree)]
                #[use_wrapper(crate::currency::CurrencyId)]
                #[foreign_key(path = crate::currency, table = currency, column = id)]
                pub currency_id: u64,
            }
        },
    );
    visit_table(&price);

    assert!(
        price
            .spacetimedsl_methods
            .on_delete_strategies_of_this_table
            .is_empty(),
        "a foreign key without strategies has no strategy implementations"
    );
```

- [ ] **Step 2: Write the failing snapshot fixture, compile-tests and runtime group**

Create `derive/tests/fixtures/foreign_key_to_table_without_delete_methods.rs`:

```rust
//! Covers a table whose rows are never removed, `method(delete = false)` without
//! `method(soft_delete = true)`, and a foreign key to it. The referenced table offers no
//! cascade entry point and declares the traits which let a foreign key without strategies
//! compile; the referencing table imports them and generates no strategy implementation.

#[spacetimedsl::dsl(plural_name = currencies, method(update = false, delete = false))]
#[spacetimedb::table(accessor = currency, public)]
pub struct Currency {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(CurrencyId)]
    #[referenced_by(path = self, table = price)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = prices, method(update = true, delete = true))]
#[spacetimedb::table(accessor = price, public)]
pub struct Price {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CurrencyId)]
    #[foreign_key(path = self, table = currency, column = id)]
    pub currency_id: u64,
}
```

Register it in `derive/src/characterization_tests.rs`, after `fn foreign_key_with_table_level_index`:

```rust
#[test]
fn foreign_key_to_table_without_delete_methods() {
    snapshot_fixture("foreign_key_to_table_without_delete_methods");
}
```

Create `compile-tests/tests/ui/foreign_keys_with_mismatched_types_without_strategy.rs`:

```rust
//! Two foreign keys pointing at the same referenced table share its pairing checks, so they
//! have to agree on the type of its primary key, also when they declare no strategy.
//!
//! The diagnostic is spanned on the second of the two columns, the one whose type
//! contradicts the first.
//!
//! The error after the first follows from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait the `shipment` table would have
//! declared for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = self, table = shipment)]
        id: u64,
    }

    #[spacetimedsl::dsl(plural_name = shipments, method(update = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(WarehouseId)]
        #[foreign_key(path = self, table = warehouse, column = id)]
        pub origin_warehouse_id: u64,

        #[index(btree)]
        #[use_wrapper(WarehouseId)]
        #[foreign_key(path = self, table = warehouse, column = id)]
        pub destination_warehouse_id: u128,
    }
}

fn main() {}
```

Create `compile-tests/tests/ui/referenced_by_without_foreign_key_back_on_table_without_delete_methods.rs`:

```rust
//! A `#[referenced_by]` names a table which has a foreign key back, also on a table which
//! neither deletes nor soft-deletes rows. `shipment` has none, so the attribute claims a
//! relationship that does not exist.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = false))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
```

Rename the two old files and rewrite their comments:

```powershell
git mv compile-tests/tests/ui/referenced_by_without_delete_or_soft_delete_method.rs compile-tests/tests/ui/foreign_key_with_on_delete_to_table_without_delete_method.rs
git mv compile-tests/tests/ui/referenced_by_without_delete_or_soft_delete_method.stderr compile-tests/tests/ui/foreign_key_with_on_delete_to_table_without_delete_method.stderr
git mv compile-tests/tests/ui/foreign_key_without_any_on_delete_field.rs compile-tests/tests/ui/foreign_key_without_strategy_to_deletable_table.rs
git mv compile-tests/tests/ui/foreign_key_without_any_on_delete_field.stderr compile-tests/tests/ui/foreign_key_without_strategy_to_deletable_table.stderr
```

The new `//!` comment of `foreign_key_with_on_delete_to_table_without_delete_method.rs`:

```rust
//! A foreign key declares `on_delete` only for a referenced table which deletes rows. The
//! `warehouse` table has `method(delete = false)` and is not soft-deletable, so its rows are
//! never removed, and `on_delete = Delete` describes a deletion that cannot happen.
```

The new `//!` comment of `foreign_key_without_strategy_to_deletable_table.rs`:

```rust
//! A foreign key declares `on_delete` when its referenced table deletes rows, so the cascade
//! finds a strategy for every row it reaches. This one declares no strategy at all, although
//! `warehouse` has a delete method.
//!
//! The two errors after the first follow from the same gap: the deletion cascade of
//! `warehouse` calls the two functions an `on_delete` strategy would have generated.
```

Create `examples/test/src/referenced_table_without_delete_methods_test.rs`:

```rust
//! A table whose rows are never removed, `method(delete = false)` without
//! `method(soft_delete = true)`, can be referenced. Its foreign keys declare no strategy,
//! because there is no removal to react to, and create and update still check that they
//! reference a row.

use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = catalog_categories, method(update = false, delete = false))]
#[spacetimedb::table(accessor = catalog_category)]
pub struct CatalogCategory {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(CatalogCategoryId)]
    #[referenced_by(path = crate::referenced_table_without_delete_methods_test, table = catalog_product)]
    #[referenced_by(path = crate::referenced_table_without_delete_methods_test, table = catalog_bundle)]
    id: u64,
}

/// Two foreign keys to one table, whose pairing imports have to stay deduplicated.
#[spacetimedsl::dsl(plural_name = catalog_products, method(update = true, delete = true))]
#[spacetimedb::table(accessor = catalog_product)]
pub struct CatalogProduct {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    pub category_id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    pub secondary_category_id: u64,
}

/// A second table of the same module referencing the same table, whose pairing imports must
/// not clash with those of `CatalogProduct`.
#[spacetimedsl::dsl(plural_name = catalog_bundles, method(update = false, delete = true))]
#[spacetimedb::table(accessor = catalog_bundle)]
pub struct CatalogBundle {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(CatalogCategoryId)]
    #[foreign_key(
        path = crate::referenced_table_without_delete_methods_test,
        table = catalog_category,
        column = id
    )]
    category_id: u64,
}

pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    create_checks_the_reference(dsl)?;
    update_checks_the_reference(dsl)?;

    Ok(())
}

/// Creating rows which reference an existing category works, and creating one which
/// references no category is rejected.
fn create_checks_the_reference<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let category = dsl.create_catalog_category()?;

    dsl.create_catalog_product(CreateCatalogProduct {
        category_id: category.get_id(),
        secondary_category_id: category.get_id(),
    })
    .map_err(|error| {
        format!("A product referencing an existing category should be created! Got:\n{error}")
    })?;
    dsl.create_catalog_bundle(CreateCatalogBundle {
        category_id: category.get_id(),
    })
    .map_err(|error| {
        format!("A bundle referencing an existing category should be created! Got:\n{error}")
    })?;

    match dsl.create_catalog_product(CreateCatalogProduct {
        category_id: CatalogCategoryId::new(u64::MAX),
        secondary_category_id: category.get_id(),
    }) {
        Err(SpacetimeDSLError::ReferenceIntegrityViolation(_)) => Ok(()),
        other => Err(format!(
            "A product whose category_id references no category should be rejected! Got:\n{other:?}"
        )),
    }
}

/// Updating a product so that a foreign key references no category is rejected.
fn update_checks_the_reference<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let category = dsl.create_catalog_category()?;
    let mut product = dsl.create_catalog_product(CreateCatalogProduct {
        category_id: category.get_id(),
        secondary_category_id: category.get_id(),
    })?;
    product.set_secondary_category_id(CatalogCategoryId::new(u64::MAX));

    match dsl.update_catalog_product_by_id(product) {
        Err(SpacetimeDSLError::ReferenceIntegrityViolation(_)) => Ok(()),
        other => Err(format!(
            "Updating a product so that secondary_category_id references no category should be rejected! Got:\n{other:?}"
        )),
    }
}
```

Register it in `examples/test/src/lib.rs`: `pub mod referenced_table_without_delete_methods_test;` after `pub mod reference_integrity_message_test;`, and append to `TEST_GROUPS`:

```rust
    (
        "referenced_table_without_delete_methods_test",
        referenced_table_without_delete_methods_test::run_tests,
    ),
```

- [ ] **Step 3: Observe the red**

Run the unit gate. Expected:
- the contract test panics with *the fixture should be accepted: `#[referenced_by]` is only allowed when the table has a delete method …*;
- `foreign_key_to_table_without_delete_methods` panics with the same rejection;
- the two new compile-tests have no `.stderr` yet, and the renamed ones still print the old first error.

Run the runtime gate. Expected: `FAILED`, the module does not compile, with the same `#[referenced_by]` rejection.

- [ ] **Step 4: Accept `#[referenced_by]` on a table that removes no rows**

In `internal/dsl/reference.rs`, change the signature to `pub(crate) fn try_parse(field: &SatsField<'_>) -> syn::Result<Vec<ReferencingTable>>` and delete the block:

```rust
            if !has_delete_method && !is_soft_deletable {
                return Err(error::referenced_by_without_delete_or_soft_delete_method(
                    attr,
                ));
            }
```

In `internal/dsl/table.rs`, call `ReferencingTable::try_parse(field)?`.

- [ ] **Step 5: Accept a `#[foreign_key]` without strategies**

In `internal/dsl/foreign_key.rs`, delete:

```rust
            if on_delete_strategy.is_none() && on_soft_delete_strategy.is_none() {
                return Err(error::foreign_key_without_on_delete_strategy(&attr.meta));
            }
```

In `internal/error.rs`, delete `foreign_key_without_on_delete_strategy` and `referenced_by_without_delete_or_soft_delete_method`.

- [ ] **Step 6: Offer entry points only where a cascade runs**

In `method.rs`, end `referenced_side_entry_points` with:

```rust
    // A table which neither deletes nor soft-deletes rows runs no cascade, so it offers no
    // entry point, even though other tables reference it.
    (on_deletion.is_some() || on_soft_deletion.is_some()).then_some(
        OnDeleteStrategiesOfReferencingTables {
            on_deletion,
            on_soft_deletion,
        },
    )
```

In `referencing_side_entry_points`, change `.map(` to `.filter_map(` and end the closure with:

```rust
            (on_deletion.is_some() || on_soft_deletion.is_some()).then_some(
                OnDeleteStrategiesOfTheReferencedTable {
                    on_deletion,
                    on_soft_deletion,
                },
            )
```

In `derive-input/src/api/dsl/table.rs`:
- `SpacetimeDSLTableMethods::on_delete_strategies_of_referencing_tables`: *The cascade entry points the tables named in `#[referenced_by(...)]` call when a row of this table is removed. `None` when no table references this one, or when this table neither deletes nor soft-deletes rows.*
- `SpacetimeDSLTableMethods::on_delete_strategies_of_this_table`: *The strategy implementations for each table this table references with a foreign key which declares a strategy.*
- `OnDeleteStrategiesOfReferencingTables`: add the sentence *At least one of the two is `Some`.*

- [ ] **Step 7: Regenerate the compile-test output and the snapshots, then read both diffs**

Expected `.stderr`:
- `foreign_key_with_on_delete_to_table_without_delete_method`: exactly one `E0432` for `crate::warehouse::this_compilation_error_occurs_because_the_warehouse_table_is_not_deletable_or_has_no_referenced_by_attribute_referencing_the_shipment_table`.
- `foreign_key_without_strategy_to_deletable_table`: one `E0432` for `…your_foreign_key_referencing_the_warehouse_table_needs_to_define_a_strategy_for_on_delete_or_the_warehouse_table_has_no_referenced_by_attribute_referencing_the_shipment_table`, then two `E0599` for the missing deletion cascade functions of `shipment`.
- `foreign_keys_with_mismatched_types_without_strategy`: the type mismatch, then one `E0432` for `self::this_compilation_error_occurs_because_the_shipment_table_has_no_foreign_key_attribute_referencing_the_warehouse_table`.
- `referenced_by_without_foreign_key_back_on_table_without_delete_methods`: exactly one `E0432` for `crate::shipment::…the_shipment_table_has_no_foreign_key_attribute_referencing_the_warehouse_table`.

Expected new snapshots, `foreign_key_to_table_without_delete_methods`:
- `Currency/table.snap` declares `…needs_to_define_a_strategy_for_on_delete_or_the_currency_table_has_no_referenced_by_attribute_referencing_the_price_table` and its `on_soft_delete` twin, and imports `self::…the_price_table_has_no_foreign_key_attribute_referencing_the_currency_table`; there is no `internal_methods.snap` and no `delete_*` method.
- `Price/table.snap` declares `…the_price_table_has_no_foreign_key_attribute_referencing_the_currency_table` and imports the two `needs_to_define_a_strategy` traits; there is no `internal_methods.snap`; `create_price.snap` and `update_price_by_id.snap` still check the reference; `wrapper_methods.snap` adds `get_prices` to `CurrencyId`.

No existing snapshot changes.

- [ ] **Step 8: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 9: Document the change**

In `docs/DOCUMENTATION.md`, *Method Configuration*, replace the line starting with `` - `delete = true` is therefore required on a table which declares `#[referenced_by]` `` by:

```markdown
- A table with `delete = false` may still declare `#[referenced_by]`. Its foreign keys then declare no `on_delete` strategy, because its rows are never deleted through the DSL
```

In *Declaration*, replace the paragraph from `At least one of the two strategy parameters is required` through `…through the pairing below.` by:

```markdown
The two strategy parameters follow what the referenced table does. Set `on_delete` exactly when it has a delete method, and `on_soft_delete` exactly when it is soft-deletable:

- `on_delete` — what happens to this table's rows when a referenced row is **deleted**: `Error`, `Delete`, `SoftDelete`, `SetZero`, or `Ignore`
- `on_soft_delete` — what happens to them when a referenced row is **soft-deleted**: `Error`, `SoftDelete`, or `Ignore`

A foreign key to a table with `method(delete = false)` and without `method(soft_delete = true)` sets neither: the rows it references are never removed. Create and update still check that it references a row.
```

In *Pairing Requirement*, add after its first paragraph:

```markdown
A table which neither deletes nor soft-deletes rows is paired the same way: its `#[referenced_by]` names every table with a foreign key to it, and those foreign keys set no strategy.
```

In `docs/MIGRATION.md`, add after the section `### Newly rejected inputs` a new section:

```markdown
### Newly accepted inputs

#### `#[referenced_by]` on a table without delete and soft-delete methods

A table with `method(delete = false)` and without `method(soft_delete = true)` may carry `#[referenced_by]`. Its rows are never removed through the DSL, so it generates no cascade. *`#[referenced_by]` is only allowed when the table has a delete method …* is gone.

#### `#[foreign_key]` without `on_delete` and `on_soft_delete`

A foreign key to such a table sets no strategy. *A `#[foreign_key]` must set `on_delete`, `on_soft_delete`, or both* is gone; a foreign key to a table which performs a removal still has to set its strategy. Create and update still check that it references a row.
```

Under `### For crates building on spacetimedsl_derive-input`, add:

```markdown
#### Cascade entry points only where a cascade runs

`SpacetimeDSLTableMethods::on_delete_strategies_of_referencing_tables` is `None` for a table which neither deletes nor soft-deletes rows, even when other tables reference it, and `on_delete_strategies_of_this_table` leaves out a referenced table whose foreign keys declare no strategy. `ForeignKey::on_delete_strategy` and `on_soft_delete_strategy` are `None` exactly when the referenced table does not perform that removal.
```

In `CODE_QUALITY_REPORT.md`, replace the entry under `## derive-input/src/internal/dsl/reference.rs` by:

```markdown
### `reference.rs`: `ReferencingTable::try_parse(field: &SatsField<'_>) -> syn::Result<Vec<ReferencingTable>>`

**Violates:** DRY

Whether the field is the primary key is re-derived from its raw attributes, although `SpacetimeDBColumn::is_primary_key` already states it.

Recommendation: Take the primary-key fact from `SpacetimeDBColumn`.
```

- [ ] **Step 10: Format, lint, commit**

Commit message:

```text
Reference tables whose rows are never removed

#[referenced_by] is allowed on a table without delete and soft-delete methods, and
#[foreign_key] without on_delete and on_soft_delete. Such a table offers no cascade entry
point, and a foreign key without strategies generates no strategy implementation. The
pairing still requires a strategy for every removal the referenced table performs, and
create and update still check the reference.

Tests: contract assertions for both inputs; the fixture
foreign_key_to_table_without_delete_methods; compile tests
foreign_keys_with_mismatched_types_without_strategy and
referenced_by_without_foreign_key_back_on_table_without_delete_methods, and the two renamed
ones whose first error is now a pairing error; the runtime group
referenced_table_without_delete_methods_test.

Closes #172

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 10: Reject `SetZero` on a primary key or unique foreign key column (#179)

**Files:**
- Temporary: `examples/test/src/set_zero_probe_test.rs` (removed before the commit)
- Create: `compile-tests/tests/ui/on_delete_set_zero_on_primary_key_column.rs` (+ `.stderr`)
- Create: `compile-tests/tests/ui/on_delete_set_zero_on_unique_column.rs` (+ `.stderr`)
- Create: `compile-tests/tests/ui/on_delete_set_zero_on_unique_multi_column_index_column.rs` (+ `.stderr`)
- Modify: `derive-input/src/internal/error.rs` (three diagnostics)
- Modify: `derive-input/src/internal/dsl/foreign_key.rs` (`ForeignKey::try_parse`)
- Modify: `derive-input/src/internal/dsl/column.rs` (`SpacetimeDSLColumn::try_parse`)
- Modify: `derive-input/src/internal/column.rs` (`try_parse`)
- Modify: `examples/test/src/component/position.rs` (strategy of `Position::entity_id`, the last assertion of `run_tests`)
- Modify: `examples/test/src/component/test.rs` (strategy of `Test::unique`)
- Modify: `docs/DOCUMENTATION.md` (*OnDeleteStrategy*, `SetZero`), `docs/MIGRATION.md`

**Interfaces:**
- Produces: `ForeignKey::try_parse(has_delete_method, is_soft_deletable, is_singleton, field, spacetimedb_column, column_type_kind, unique_multi_column_index: Option<&Ident>)`; `SpacetimeDSLColumn::try_parse(spacetimedsl_table, field, rust_struct, rust_field, spacetimedb_column, unique_multi_column_index: Option<&Ident>)`; `error::set_zero_strategy_on_primary_key_column`, `error::set_zero_strategy_on_unique_column`, `error::set_zero_strategy_on_unique_multi_column_index_column`.

- [ ] **Step 1: Prove the failures at run time**

The issue asks to confirm both cases before choosing the fix. Create `examples/test/src/set_zero_probe_test.rs`:

```rust
use crate::spacetimedsl::prelude::*;

#[spacetimedsl::dsl(plural_name = probe_parents, method(update = false, delete = true))]
#[spacetimedb::table(accessor = probe_parent)]
pub struct ProbeParent {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ProbeParentId)]
    #[referenced_by(path = crate::set_zero_probe_test, table = probe_primary_key_child)]
    #[referenced_by(path = crate::set_zero_probe_test, table = probe_unique_child)]
    #[referenced_by(path = crate::set_zero_probe_test, table = probe_indexed_child)]
    id: u64,
}

#[spacetimedsl::dsl(plural_name = probe_primary_key_children, method(update = true, delete = true))]
#[spacetimedb::table(accessor = probe_primary_key_child)]
pub struct ProbePrimaryKeyChild {
    #[primary_key]
    #[use_wrapper(ProbeParentId)]
    #[foreign_key(path = crate::set_zero_probe_test, table = probe_parent, column = id, on_delete = SetZero)]
    pub parent_id: u64,
}

#[spacetimedsl::dsl(plural_name = probe_unique_children, method(update = true, delete = true))]
#[spacetimedb::table(accessor = probe_unique_child)]
pub struct ProbeUniqueChild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[unique]
    #[use_wrapper(ProbeParentId)]
    #[foreign_key(path = crate::set_zero_probe_test, table = probe_parent, column = id, on_delete = SetZero)]
    pub parent_id: u64,
}

#[spacetimedsl::dsl(
    plural_name = probe_indexed_children,
    method(update = true, delete = true),
    unique_index(name = parent_and_slot),
)]
#[spacetimedb::table(
    accessor = probe_indexed_child,
    index(accessor = parent_and_slot, btree(columns = [parent_id, slot])),
)]
pub struct ProbeIndexedChild {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ProbeParentId)]
    #[foreign_key(path = crate::set_zero_probe_test, table = probe_parent, column = id, on_delete = SetZero)]
    pub parent_id: u64,

    pub slot: u64,
}

pub(crate) fn probe_primary_key<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let parent = dsl.create_probe_parent()?;
    dsl.create_probe_primary_key_child(CreateProbePrimaryKeyChild {
        parent_id: parent.get_id(),
    })?;

    dsl.delete_probe_parent_by_id(parent.get_id())?;

    Err("Deleting the parent returned; SetZero on a primary key did not fail".to_string())
}

pub(crate) fn probe_unique<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let first_parent = dsl.create_probe_parent()?;
    let second_parent = dsl.create_probe_parent()?;
    for parent in [&first_parent, &second_parent] {
        dsl.create_probe_unique_child(CreateProbeUniqueChild {
            parent_id: parent.get_id(),
        })?;
    }

    dsl.delete_probe_parent_by_id(first_parent.get_id())?;
    dsl.delete_probe_parent_by_id(second_parent.get_id())?;

    Err("Deleting both parents returned; SetZero on a #[unique] column did not fail".to_string())
}

pub(crate) fn probe_unique_multi_column_index<T: WriteContext>(
    dsl: &DSL<'_, T>,
) -> Result<(), String> {
    let first_parent = dsl.create_probe_parent()?;
    let second_parent = dsl.create_probe_parent()?;
    for parent in [&first_parent, &second_parent] {
        dsl.create_probe_indexed_child(CreateProbeIndexedChild {
            parent_id: parent.get_id(),
            slot: 5,
        })?;
    }

    dsl.delete_probe_parent_by_id(first_parent.get_id())?;
    dsl.delete_probe_parent_by_id(second_parent.get_id())?;

    let lookup = dsl.get_probe_indexed_child_by_parent_and_slot(ProbeParentId::new(0), &5);

    Err(format!(
        "Both children now hold parent_id 0 and slot 5; looking one up returns:\n{lookup:?}"
    ))
}
```

Add `pub mod set_zero_probe_test;` to `lib.rs`. Then, one probe at a time, put exactly one entry into `TEST_GROUPS`, run the runtime gate, and write down the relevant lines:
1. `("set_zero_probe_test", set_zero_probe_test::probe_primary_key)`: expected a panic inside SpacetimeDB's `update`, which finds no row under the key `0`.
2. `("set_zero_probe_test", set_zero_probe_test::probe_unique)`: expected a panic inside `update` for the unique-constraint violation of the second cleared row.
3. `("set_zero_probe_test", set_zero_probe_test::probe_unique_multi_column_index)`: expected no panic, and a lookup error saying the unique multi-column index holds two rows.

**Stop condition:** if a probe does not fail as expected, stop and report the observed output to the developer. The fix depends on it.

Remove the probe file, its `mod` line and its `TEST_GROUPS` entry. Check `git status -- examples/test` shows nothing.

- [ ] **Step 2: Write the failing compile-tests**

`compile-tests/tests/ui/on_delete_set_zero_on_primary_key_column.rs`:

```rust
//! `SetZero` clears the foreign key column and writes the row back through its primary key.
//! When the foreign key column is the primary key, the write goes to the key `0`, where it
//! finds no row or overwrites another one.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `dock` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::dock, table = dock)]
        id: u64,
    }
}

pub mod dock {
    #[spacetimedsl::dsl(plural_name = docks, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = dock, public)]
    pub struct Dock {
        #[primary_key]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        pub warehouse_id: u64,
    }
}

fn main() {}
```

`compile-tests/tests/ui/on_delete_set_zero_on_unique_column.rs`:

```rust
//! `SetZero` writes `0` into the foreign key column. On a `#[unique]` column, the second
//! cleared row repeats that value, which the unique constraint rejects inside the cascade.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(plural_name = shipments, method(update = true, delete = true))]
    #[spacetimedb::table(accessor = shipment, public)]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[unique]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        pub warehouse_id: u64,
    }
}

fn main() {}
```

`compile-tests/tests/ui/on_delete_set_zero_on_unique_multi_column_index_column.rs`:

```rust
//! `SetZero` writes `0` into the foreign key column. On a column of a unique multi-column
//! index, two cleared rows which agree in the index's other columns repeat its values, and
//! SpacetimeDSL checks that index only when a row is created or updated.
//!
//! The errors after the first follow from it: a rejected `#[dsl]` emits nothing else, so
//! the `warehouse` table's expansion misses the trait and the two cascade functions the
//! `shipment` table would have generated for it.

::spacetimedsl::spacetimedsl!();

pub mod warehouse {
    #[spacetimedsl::dsl(plural_name = warehouses, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = warehouse, public)]
    pub struct Warehouse {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper(WarehouseId)]
        #[referenced_by(path = crate::shipment, table = shipment)]
        id: u64,
    }
}

pub mod shipment {
    #[spacetimedsl::dsl(
        plural_name = shipments,
        method(update = true, delete = true),
        unique_index(name = warehouse_and_slot),
    )]
    #[spacetimedb::table(
        accessor = shipment,
        public,
        index(accessor = warehouse_and_slot, btree(columns = [warehouse_id, slot])),
    )]
    pub struct Shipment {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(crate::warehouse::WarehouseId)]
        #[foreign_key(path = crate::warehouse, table = warehouse, column = id, on_delete = SetZero)]
        pub warehouse_id: u64,

        pub slot: u64,
    }
}

fn main() {}
```

Each compiles once `on_delete = SetZero` becomes `on_delete = Delete`: that is the fix its diagnostic suggests.

- [ ] **Step 3: Observe the red**

Run the unit gate. Expected: trybuild reports that each of the three *compiled* although a failure was expected.

- [ ] **Step 4: Write the three diagnostics**

In `internal/error.rs`, after `set_zero_strategy_on_unsupported_type`, add:

```rust
pub fn set_zero_strategy_on_primary_key_column(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::SetZero` is not allowed on a primary key column!\nClearing it would write the row back under the key `0` or `Uuid::NIL`, where the write finds no row or another one. Choose another strategy, such as `on_delete = Delete`.",
    )
}

pub fn set_zero_strategy_on_unique_column(foreign_key_meta: &impl ToTokens) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        "`OnDeleteStrategy::SetZero` is not allowed on a `#[unique]` column!\nA second cleared row would repeat `0` or `Uuid::NIL`, which the unique constraint rejects. Remove `#[unique]`, or choose another strategy, such as `on_delete = Delete`.",
    )
}

pub fn set_zero_strategy_on_unique_multi_column_index_column(
    foreign_key_meta: &impl ToTokens,
    unique_index_name: &Ident,
) -> Error {
    Error::new_spanned(
        foreign_key_meta,
        format!(
            "`OnDeleteStrategy::SetZero` is not allowed on a column of the unique multi-column index `{unique_index_name}`!\nTwo cleared rows which agree in the index's other columns would repeat its values, and SpacetimeDSL checks the index only when a row is created or updated. Remove `unique_index(name = {unique_index_name})`, or choose another strategy, such as `on_delete = Delete`."
        ),
    )
}
```

- [ ] **Step 5: Reject the three shapes**

In `internal/column.rs`, add `index::IndexType` to the `db::{…}` import, and before `let spacetimedsl_column = SpacetimeDSLColumn::try_parse(` add:

```rust
        let unique_multi_column_index = spacetimedb_table
            .multi_column_indices
            .iter()
            .find(|index| {
                index.is_unique
                    && matches!(
                        &index.index_type,
                        IndexType::BTreeMultiColumn { columns } if columns.contains(&rust_field.name)
                    )
            })
            .map(|index| &index.name);
```

and pass `unique_multi_column_index` as a sixth argument. In `internal/dsl/column.rs`, add the parameter `unique_multi_column_index: Option<&Ident>` to `SpacetimeDSLColumn::try_parse` and pass it to `ForeignKey::try_parse` as its seventh argument.

In `internal/dsl/foreign_key.rs`, add the parameter `unique_multi_column_index: Option<&Ident>` to `ForeignKey::try_parse`, and extend the `SetZero` block after the private-column check:

```rust
                // Clearing the column writes `0` or `Uuid::NIL`, which a second cleared row
                // would repeat and a cleared primary key would write the row back under.
                if spacetimedb_column.is_primary_key {
                    return Err(error::set_zero_strategy_on_primary_key_column(&attr.meta));
                }

                if spacetimedb_column
                    .single_column_index
                    .as_ref()
                    .is_some_and(|index| index.is_unique)
                {
                    return Err(error::set_zero_strategy_on_unique_column(&attr.meta));
                }

                if let Some(unique_index_name) = unique_multi_column_index {
                    return Err(error::set_zero_strategy_on_unique_multi_column_index_column(
                        &attr.meta,
                        unique_index_name,
                    ));
                }
```

- [ ] **Step 6: Regenerate the compile-test output and read it**

Expected: each new `.stderr` starts with its diagnostic, spanned on the `#[foreign_key(…)]` attribute, followed by the follow-on errors its `//!` comment names. No existing `.stderr` changes.

- [ ] **Step 7: Move the two examples off the rejected shape**

Two runtime examples use `SetZero` on a `#[unique]` column. They worked only because no test cleared a second row. Run the runtime gate first. Expected: `FAILED`, the module does not compile, with the new *`OnDeleteStrategy::SetZero` is not allowed on a `#[unique]` column!* at `component/position.rs` and `component/test.rs`.

Both keep `#[unique]`, which `EntityId::get_position` relies on, and switch to `Delete`. `SetZero` stays covered at run time by `entity.rs`, `component/uuid_reference_test.rs` and `update_and_soft_delete_hook_test.rs`.

In `examples/test/src/component/position.rs`, change the foreign key of `Position::entity_id` (the one below `/// The unique ID of the Entity the Position belongs to.`) to:

```rust
    #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
```

and replace the last assertion of `run_tests`:

```rust
    if dsl
        .get_position_by_id(&player_reflection_position.get_id())
        .expect("should exist")
        .get_entity_id()
        .value()
        .ne(&0)
    {
        return Err("The entity_id of the position which was previously for the player_reflection entity should be 0 because the entity was deleted and the foreign key has a SetZero strategy!".to_string());
    }
```

by:

```rust
    if dsl
        .get_position_by_id(&player_reflection_position.get_id())
        .is_ok()
    {
        return Err(
            "Deleting the player_reflection Entity should delete its Position through on_delete = Delete!"
                .to_string(),
        );
    }
```

In `examples/test/src/component/test.rs`, change the foreign key of `Test::unique` to:

```rust
    #[foreign_key(path = crate::entity, table = entity, column = obj_id, on_delete = Delete)]
```

- [ ] **Step 8: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 9: Document the change**

In `docs/DOCUMENTATION.md`, in the `SetZero` bullet list, add after *Requires the foreign key column to be `pub` (so a setter exists)*:

```markdown
- Not allowed on a primary key column, whose cleared value would move the row to another key, nor on a `#[unique]` column or a column of a unique multi-column index, where a second cleared row would repeat the value
```

In `docs/MIGRATION.md`, under `### Newly rejected inputs`, add (with the observed probe output in mind):

```markdown
#### `on_delete = SetZero` on a primary key, a `#[unique]` column or a column of a unique multi-column index

`SetZero` writes `0` or `Uuid::NIL` into the foreign key column and writes the row back through its primary key. On a primary key column that write finds no row or another one; on a `#[unique]` column, or a column of a `unique_index(name = …)` index, a second cleared row repeats the value. SpacetimeDB's `update` panicked inside the cascade in the first two cases, and the third silently broke the index's uniqueness. All three are rejected at the `#[foreign_key]` attribute. Choose another strategy, such as `Delete`, or remove the uniqueness.
```

- [ ] **Step 10: Format, lint, commit**

Commit message (paste the observed probe lines where marked):

```text
Reject SetZero on primary key and unique foreign key columns

on_delete = SetZero is rejected on a primary key column, on a #[unique] column and on a
column of a unique multi-column index, each with its own diagnostic at the #[foreign_key]
attribute.

Observed at run time before the fix, with probes that were removed again:
- primary key: (the panic line probe 1 printed in Step 1)
- #[unique]: (the panic line probe 2 printed in Step 1)
- unique multi-column index: (the lookup result probe 3 printed in Step 1)

Tests: on_delete_set_zero_on_primary_key_column, on_delete_set_zero_on_unique_column,
on_delete_set_zero_on_unique_multi_column_index_column.

The runtime examples Position::entity_id and Test::unique used SetZero on a #[unique]
column; they switch to Delete and keep #[unique]. SetZero stays covered at run time by
entity.rs, uuid_reference_test.rs and update_and_soft_delete_hook_test.rs.

Closes #179

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 11: Document foreign keys and cascades on the generated methods (#35, part 1)

**Files:**
- Create: `derive-input/src/internal/dsl/method/relationship_doc.rs`
- Modify: `derive-input/src/internal/dsl/method.rs` (`pub mod relationship_doc;`)
- Modify: `derive-input/src/internal/dsl/method/reference_integrity.rs` (`documented_value_referencing_no_row`)
- Modify: `derive-input/src/internal/dsl/method/create.rs`, `update.rs`, `upsert.rs`, `removal.rs` (doc comments)
- Modify: `docs/DOCUMENTATION.md`, `docs/MIGRATION.md`
- Recorded output: `create_*`, `update_*_by_*`, `upsert_*` of tables with foreign keys; `delete_*` and `soft_delete_*` of referenced tables

**Interfaces:**
- Consumes: `Removal::strategy_argument` from Task 7.
- Produces: `relationship_doc::with_section(doc: String, section: String) -> String`, `relationship_doc::reference_checks(lead: &str, checked_columns: &[&InternalColumn]) -> String`, `relationship_doc::cascade(removal: Removal, referencing_tables: &[ReferencingTable]) -> String`, and the private helpers `section` and `table_and_module`, which Tasks 12 and 13 extend.

- [ ] **Step 1: Name the value that references no row next to the guard which skips it**

In `reference_integrity.rs`, add after `reference_integrity_checks`:

```rust
/// How documentation names the value a foreign key column of this kind holds when it
/// references no row: the value the guard in `reference_integrity_checks` lets through
/// unchecked. `None` for a kind every value of which is checked.
pub fn documented_value_referencing_no_row(kind: ColumnTypeKind) -> Option<&'static str> {
    match kind {
        ColumnTypeKind::UnsignedInteger => Some("`0`"),
        ColumnTypeKind::Optional => Some("`None`"),
        ColumnTypeKind::UUID => Some("`Uuid::NIL`"),
        ColumnTypeKind::String
        | ColumnTypeKind::Bool
        | ColumnTypeKind::Timestamp
        | ColumnTypeKind::Other => None,
    }
}
```

- [ ] **Step 2: Create the module**

Create `derive-input/src/internal/dsl/method/relationship_doc.rs`:

```rust
//! The rustdoc text which says how a table relates to other tables through `#[foreign_key]`
//! and `#[referenced_by]`.
//!
//! The write and removal methods, the accessors of a foreign key column and the struct show
//! the same relationships, so each fact is phrased once here.

use {
    super::{reference_integrity, removal::Removal},
    crate::{api::dsl::reference::ReferencingTable, internal::column::InternalColumn},
    quote::ToTokens,
    syn::{Ident, Path},
};

/// `doc`, followed by `section` as a paragraph of its own, or `doc` alone when `section` is
/// empty.
pub fn with_section(doc: String, section: String) -> String {
    match section.is_empty() {
        true => doc,
        false => format!("{doc}\n\n{section}"),
    }
}

/// The `# Foreign keys` section of a method which checks, before it writes, that the foreign
/// keys among `checked_columns` reference a row. Empty when none of them has a foreign key.
pub fn reference_checks(lead: &str, checked_columns: &[&InternalColumn]) -> String {
    let bullets: Vec<String> = checked_columns
        .iter()
        .filter_map(|column| {
            let foreign_key = column.spacetimedsl_column_foreign_key.as_ref()?;
            let value_referencing_no_row =
                reference_integrity::documented_value_referencing_no_row(column.rust_field_type_kind)
                    .map(|value| format!(", or {value}"))
                    .unwrap_or_default();

            Some(format!(
                "- `{}`: a row of {}{value_referencing_no_row}",
                column.rust_field_name,
                table_and_module(&foreign_key.table_name, &foreign_key.path),
            ))
        })
        .collect();

    section("Foreign keys", Some(lead), &bullets)
}

/// The `# Cascade` section of a method which removes rows other tables reference: the tables
/// whose strategies for `removal` it runs.
pub fn cascade(removal: Removal, referencing_tables: &[ReferencingTable]) -> String {
    let bullets: Vec<String> = referencing_tables
        .iter()
        .map(|referencing_table| {
            format!(
                "- {}",
                table_and_module(&referencing_table.table_name, &referencing_table.path)
            )
        })
        .collect();

    let lead = format!(
        "Runs the `{}` strategies which the foreign keys of these tables declare:",
        removal.strategy_argument()
    );

    section("Cascade", Some(lead.as_str()), &bullets)
}

/// A doc comment section: its heading, a lead sentence when there is one, and its bullets.
/// Nothing when there are no bullets.
fn section(heading: &str, lead: Option<&str>, bullets: &[String]) -> String {
    if bullets.is_empty() {
        return String::new();
    }

    let lead = lead
        .map(|lead| format!("{lead}\n\n"))
        .unwrap_or_default();

    format!("# {heading}\n\n{lead}{}", bullets.join("\n"))
}

/// "the `warehouse` table (`self`)": a table and the module path its attribute names it by,
/// written the way the user wrote it.
fn table_and_module(table_name: &Ident, module_path: &Path) -> String {
    let module_path = module_path.to_token_stream().to_string().replace(' ', "");

    format!("the `{table_name}` table (`{module_path}`)")
}
```

In `method.rs`, add `pub mod relationship_doc;` next to `pub mod naming;`.

- [ ] **Step 3: Add the sections to the doc comments**

In `create.rs` (`for_create`), replace the `doc_comment` field by:

```rust
        doc_comment: relationship_doc::with_section(
            format!("Create a row in the `{singular_table_name}` table."),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each column references a row:",
                &internal_columns.iter().collect_vec(),
            ),
        ),
```

In `update.rs` (`for_update`), bind before the method:

```rust
    // The check skips a private column, which has no setter and cannot change.
    let columns_with_a_setter = internal_columns
        .iter()
        .filter(|internal_column| {
            !matches!(internal_column.rust_field_visibility, RustVisibility::Private)
        })
        .collect_vec();
```

and wrap the existing `doc_comment` expression:

```rust
        doc_comment: relationship_doc::with_section(
            match is_singleton_pk {
                true => format!(
                    "Try to update the `{struct_name}` row of the singleton `{singular_table_name}` table."
                ),
                false => format!(
                    "{unique_multi_column_index_hint}\n\nTry to update a `{struct_name}` row of the `{singular_table_name}` table {described_as}."
                ),
            },
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each of these columns references a row whenever its value changes:",
                &columns_with_a_setter,
            ),
        ),
```

In `upsert.rs` (`for_singleton_upsert`), replace the `doc_comment` field by:

```rust
        doc_comment: relationship_doc::with_section(
            format!(
                "Write the `{struct_name}` row of the singleton `{singular_table_name}` table, whether or not it exists yet."
            ),
            relationship_doc::reference_checks(
                "Fails with `ReferenceIntegrityViolation` unless each column references a row. While the row exists, only the columns with a setter are checked, whenever their value changes:",
                &internal_columns.iter().collect_vec(),
            ),
        ),
```

In `removal.rs` (`removal_method`), after the `let (doc_comment, method_name) = match one_or_multiple { … };` binding, add:

```rust
    let doc_comment = relationship_doc::with_section(
        doc_comment,
        relationship_doc::cascade(removal, &spacetimedsl_table.referencing_tables),
    );
```

Add the imports each file needs: `relationship_doc` to the `super::{…}` group; `itertools::Itertools` and `api::rust::visibility::RustVisibility` in `update.rs`; `itertools::Itertools` in `upsert.rs`.

- [ ] **Step 4: Observe the red, regenerate the snapshots and read the diff**

Run the unit gate. Expected: `FAILED` snapshots whose diff only appends sections to doc comments. Regenerate, then check `foreign_key_and_referenced_by`:
- `Shipment/create_shipment.snap` ends its doc with *# Foreign keys … - `origin_warehouse_id`: a row of the `warehouse` table (`self`), or `0` - `destination_warehouse_id`: …*;
- `Warehouse/delete_warehouse_by_id.snap` ends with *# Cascade Runs the `on_delete` strategies which the foreign keys of these tables declare: - the `shipment` table (`self`) - the `inspection` table (`self`)*;
- `on_soft_delete_cascade`'s `soft_delete_*` methods say `on_soft_delete`;
- `on_delete_set_zero_uuid` says *or `Uuid::NIL`*.

No method body changes.

- [ ] **Step 5: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 6: Document the change**

In `docs/DOCUMENTATION.md`, add before `### Critical Rule` in *Foreign Keys & Referential Integrity*:

```markdown
### Generated Documentation

The generated code documents the relationships it acts on:

- `create_<table>`, `update_<table>_by_<key>` and `upsert_<table>` list under *Foreign keys* each foreign key column they check, the table it has to reference, and the value (`0`, `Uuid::NIL`) which references no row.
- The delete and soft-delete methods of a table with `#[referenced_by]` list under *Cascade* the tables whose `on_delete` or `on_soft_delete` strategies they run.
```

In `docs/MIGRATION.md`, under `### Changed messages and generated code`, add:

```markdown
#### The generated documentation shows foreign keys and cascades

- `create_<table>`, `update_<table>_by_<key>` and `upsert_<table>` list under *Foreign keys* the foreign key columns they check.
- The delete and soft-delete methods of a table with `#[referenced_by]` list under *Cascade* the tables whose strategies they run.

Only rustdoc output changes.
```

- [ ] **Step 7: Format, lint, commit**

Commit message:

```text
Document foreign keys and cascades on the generated methods

create, update and upsert list the foreign key columns they check, and the delete and
soft-delete methods of a referenced table list the tables whose strategies they run.
relationship_doc.rs phrases each of these facts once.

Recorded output: the doc comments of those methods.

Part of #35

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 12: Document the accessors of a foreign key column (#35, part 2)

**Files:**
- Modify: `derive-input/src/api/dsl/getter.rs`, `derive-input/src/api/dsl/setter.rs` (`doc_comment`)
- Modify: `derive-input/src/internal/dsl/getter.rs`, `derive-input/src/internal/dsl/setter.rs` (`map` takes the foreign key)
- Modify: `derive-input/src/internal/dsl/column.rs` (pass the foreign key)
- Modify: `derive-input/src/internal/dsl/method/relationship_doc.rs` (`foreign_key`, `referenced_column`, `strategies`)
- Modify: `derive/src/output/accessor.rs`
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Modify: `docs/DOCUMENTATION.md`, `docs/MIGRATION.md`
- Recorded output: `table.snap` of every table with a foreign key column

**Interfaces:**
- Consumes: `relationship_doc::table_and_module`, `Removal::strategy_declared_by`.
- Produces: `Getter::doc_comment: String`, `Setter::doc_comment: String`; `relationship_doc::foreign_key(foreign_key: &ForeignKey) -> String`; the private `relationship_doc::referenced_column` and `relationship_doc::strategies`, which Task 13 reuses.

- [ ] **Step 1: Pin the getter's documentation in the contract test (the red)**

In `visit_spacetimedsl_column`, add `doc_comment: _,` to the `Getter { … }` and `Setter { … }` patterns. In `the_model_holds_what_a_table_declares`, after the `foreign_key` assertions of `owner_id`, add:

```rust
    assert_eq!(
        owner_id
            .spacetimedsl_column
            .getter
            .as_ref()
            .expect("every column has a getter")
            .doc_comment,
        "References the `id` column of the `owner` table (`crate::owner`).\n\n- On delete: `Delete`\n- On soft delete: none, the `owner` table is not soft-deletable"
    );
```

Run the unit gate. Expected: `error[E0026]: struct Getter does not have a field named doc_comment`.

- [ ] **Step 2: Add the fields**

In `api/dsl/getter.rs` and `api/dsl/setter.rs`, add as the first field:

```rust
    /// What the column references and the strategies it declares, when it has a foreign
    /// key; empty otherwise.
    pub doc_comment: String,
```

- [ ] **Step 3: Phrase the foreign key**

In `relationship_doc.rs`, add `api::dsl::foreign_key::ForeignKey` to the imports and:

```rust
/// What a foreign key column references, and the strategy it declares for each removal of
/// the referenced row: the documentation of its getter and setter.
pub fn foreign_key(foreign_key: &ForeignKey) -> String {
    let [on_delete, on_soft_delete] = strategies(foreign_key);

    format!(
        "References {}.\n\n- {on_delete}\n- {on_soft_delete}",
        referenced_column(foreign_key)
    )
}

/// "the `id` column of the `warehouse` table (`self`)"
fn referenced_column(foreign_key: &ForeignKey) -> String {
    format!(
        "the `{}` column of {}",
        foreign_key.primary_key_column_name,
        table_and_module(&foreign_key.table_name, &foreign_key.path),
    )
}

/// "On delete: `Delete`" and "On soft delete: none, the `warehouse` table is not
/// soft-deletable". The pairing lets a strategy be missing exactly when the referenced table
/// does not perform that removal.
fn strategies(foreign_key: &ForeignKey) -> [String; 2] {
    let strategy = |removal: Removal, name: &str, capability: &str| {
        match removal.strategy_declared_by(foreign_key) {
            Some(strategy) => format!("{name}: `{strategy:?}`"),
            None => format!(
                "{name}: none, the `{}` table is not {capability}",
                foreign_key.table_name
            ),
        }
    };

    [
        strategy(Removal::Hard, "On delete", "deletable"),
        strategy(Removal::Soft, "On soft delete", "soft-deletable"),
    ]
}
```

- [ ] **Step 4: Fill the fields**

In `internal/dsl/getter.rs`, add the parameter `foreign_key: Option<&ForeignKey>` to `Getter::map` and the field:

```rust
        Getter {
            doc_comment: foreign_key
                .map(relationship_doc::foreign_key)
                .unwrap_or_default(),
            method_name: naming::getter_name(column_name),
            return_type,
            method_impl,
        }
```

In `internal/dsl/setter.rs`, the same parameter and `doc_comment` as the first field of the returned `Setter`. Import `api::dsl::foreign_key::ForeignKey` and `internal::dsl::method::relationship_doc` in both.

In `internal/dsl/column.rs`, pass `foreign_key.as_ref()` as the last argument of `Getter::map` and `Setter::map`.

- [ ] **Step 5: Put the documentation in front of the accessor**

In `derive/src/output/accessor.rs`, add `doc_comment: &'a str,` to `AccessorDefinition`; set it to `&getter.doc_comment`, `""` and `&setter.doc_comment` in the three arms of `definition`; and in `build` replace the doc comment line by:

```rust
    let doc_comment = doc_comment::doc_comment_with_implementation(accessor.doc_comment, method.clone());
```

- [ ] **Step 6: Observe green, regenerate the snapshots and read the diff**

Run the unit gate: the contract test passes, the snapshots fail. Regenerate. Expected hunks only: the getter and setter of each foreign key column in `table.snap` get the *References …* text in place of the empty `///`. No other accessor changes.

- [ ] **Step 7: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 8: Document the change**

In `docs/DOCUMENTATION.md`, *Generated Documentation*, add the bullet:

```markdown
- The getter and the setter of a foreign key column name the column and table it references, and its `on_delete` and `on_soft_delete` strategies.
```

In `docs/MIGRATION.md`, extend the entry *The generated documentation shows foreign keys and cascades* with the bullet *The getter and setter of a foreign key column say which column and table it references and which strategies it declares.* Under `### For crates building on spacetimedsl_derive-input`, add:

```markdown
#### `Getter::doc_comment` and `Setter::doc_comment`

`Getter` and `Setter` gained `doc_comment: String`: what a foreign key column references and the strategies it declares, empty for any other column. Put it in front of the accessor's documentation, as `spacetimedsl_derive` does.
```

- [ ] **Step 9: Format, lint, commit**

Commit message:

```text
Document the accessors of a foreign key column

The getter and setter of a foreign key column say which column of which table it references
and which strategy it declares for a deletion and a soft deletion of that row. Getter and
Setter gained doc_comment, which the derive crate puts in front of the accessor.

Recorded output: table.snap of every table with a foreign key column.

Part of #35

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 13: Append the relationships to the struct's documentation (#35, part 3)

**Files:**
- Modify: `derive-input/src/api/dsl/table.rs` (`SpacetimeDSLTable::struct_doc_comment`)
- Modify: `derive-input/src/internal/dsl/table.rs` (initialise it)
- Modify: `derive-input/src/internal/dsl/method/context.rs` (`TableContributions::struct_doc_comment`)
- Modify: `derive-input/src/internal/dsl/method.rs` (`generate` fills it)
- Modify: `derive-input/src/internal/dsl/method/relationship_doc.rs` (`of_struct`)
- Modify: `derive/src/lib.rs` (`append_documentation`)
- Modify: `derive/src/characterization_tests.rs` (snapshot the appended documentation)
- Modify: `derive/src/data_transfer_contract_tests.rs`
- Modify: `docs/DOCUMENTATION.md`, `docs/MIGRATION.md`, `CODE_QUALITY_REPORT.md`
- Recorded output: `table.snap` of every table with a foreign key or `#[referenced_by]`

**Interfaces:**
- Consumes: `relationship_doc::section`, `table_and_module`, `referenced_column`, `strategies`.
- Produces: `SpacetimeDSLTable::struct_doc_comment: String`; `relationship_doc::of_struct(singular_table_name: &Ident, columns: &[Column], referencing_tables: &[ReferencingTable]) -> String`.

- [ ] **Step 1: Pin the struct documentation in the contract test (the red)**

Add `struct_doc_comment: _,` to the `SpacetimeDSLTable { … }` pattern in `visit_spacetimedsl_table`, and after the `compile_error_checks` assertion:

```rust
    assert_eq!(
        spacetimedsl_table.struct_doc_comment,
        "# Foreign keys of the `gadget` table\n\n- `owner_id` references the `id` column of the `owner` table (`crate::owner`).\n  - On delete: `Delete`\n  - On soft delete: none, the `owner` table is not soft-deletable\n\n# Tables referencing the `gadget` table\n\n- the `part` table (`crate::part`)"
    );
```

Run the unit gate. Expected: `error[E0026]: struct SpacetimeDSLTable does not have a field named struct_doc_comment`.

- [ ] **Step 2: Add the field and carry it through the contributions**

In `api/dsl/table.rs`, add after `hooks`:

```rust
    /// The documentation `#[spacetimedsl::dsl]` appends to the struct's own: the table's
    /// foreign keys and the tables which reference it. Empty when it has neither.
    pub struct_doc_comment: String,
```

In `internal/dsl/table.rs`, initialise `struct_doc_comment: String::new(),` among the fields `TableContributions::apply_to` fills in. In `method/context.rs`, add `pub struct_doc_comment: String,` to `TableContributions`; in `merge`:

```rust
        if !other.struct_doc_comment.is_empty() {
            self.struct_doc_comment = other.struct_doc_comment;
        }
```

and in `apply_to`: `spacetimedsl_table.struct_doc_comment = self.struct_doc_comment;`.

- [ ] **Step 3: Phrase the struct documentation**

In `relationship_doc.rs`, add `api::Column` to the imports and:

```rust
/// The sections `#[spacetimedsl::dsl]` appends to the struct's documentation: the foreign keys
/// of the `singular_table_name` table, and the tables its `#[referenced_by]` attributes name.
/// Empty when it has neither.
pub fn of_struct(
    singular_table_name: &Ident,
    columns: &[Column],
    referencing_tables: &[ReferencingTable],
) -> String {
    let foreign_keys: Vec<String> = columns
        .iter()
        .filter_map(|column| {
            let foreign_key = column.spacetimedsl_column.foreign_key.as_ref()?;
            let [on_delete, on_soft_delete] = strategies(foreign_key);

            Some(format!(
                "- `{}` references {}.\n  - {on_delete}\n  - {on_soft_delete}",
                column.rust_field.name,
                referenced_column(foreign_key),
            ))
        })
        .collect();

    let referencing_tables: Vec<String> = referencing_tables
        .iter()
        .map(|referencing_table| {
            format!(
                "- {}",
                table_and_module(&referencing_table.table_name, &referencing_table.path)
            )
        })
        .collect();

    [
        section(
            &format!("Foreign keys of the `{singular_table_name}` table"),
            None,
            &foreign_keys,
        ),
        section(
            &format!("Tables referencing the `{singular_table_name}` table"),
            None,
            &referencing_tables,
        ),
    ]
    .into_iter()
    .filter(|section| !section.is_empty())
    .collect::<Vec<_>>()
    .join("\n\n")
}
```

In `method.rs` (`generate`), before `let methods = …`, add:

```rust
        contributions.struct_doc_comment = relationship_doc::of_struct(
            &context.singular_table_name,
            columns,
            &context.spacetimedsl_table.referencing_tables,
        );
```

- [ ] **Step 4: Append the documentation to the struct**

In `derive/src/lib.rs`, in `expand_dsl_attribute_parts`, after `let input = Table::try_parse(args, &derive_input)?;` add:

```rust
    append_documentation(&mut derive_input, &input.spacetimedsl_table.struct_doc_comment);
```

and add the function after `derive_table_helper_attr`:

```rust
/// Appends `documentation` to the struct's own doc comment, as a paragraph of its own.
fn append_documentation(derive_input: &mut syn::DeriveInput, documentation: &str) {
    if documentation.is_empty() {
        return;
    }

    derive_input.attrs.push(syn::parse_quote!(#[doc = ""]));
    derive_input.attrs.push(syn::parse_quote!(#[doc = #documentation]));
}
```

- [ ] **Step 5: Snapshot what a pass appended**

In `derive/src/characterization_tests.rs`:
- add `appended_documentation: Vec<String>,` to `FixtureExpansion`, documented as *The doc attributes this pass appended to the struct, in order*;
- in `expand_fixture`, before `expand_dsl_attribute_parts`, bind `let documentation_before = documentation_of(&derive_input.attrs).len();`, and after it bind `let appended_documentation = documentation_of(&echoed_item.attrs).split_off(documentation_before);`, storing it in the `FixtureExpansion`;
- in `snapshot_fixture`, destructure `appended_documentation` and pass `&appended_documentation` to `table_snapshot`;
- add:

```rust
/// The text of every `#[doc = "…"]` attribute, in order.
fn documentation_of(attributes: &[Attribute]) -> Vec<String> {
    attributes
        .iter()
        .filter(|attribute| attribute.path().is_ident("doc"))
        .filter_map(|attribute| match &attribute.meta {
            syn::Meta::NameValue(syn::MetaNameValue {
                value:
                    Expr::Lit(ExprLit {
                        lit: Lit::Str(text),
                        ..
                    }),
                ..
            }) => Some(text.value()),
            _ => None,
        })
        .collect()
}
```

- replace `table_snapshot` by:

```rust
/// Everything the macro emits that is not a DSL method, followed by a manifest of the
/// generated method names, and preceded by the documentation it appended to the struct. The
/// manifest makes an added or removed method fail this snapshot instead of only orphaning a
/// file.
fn table_snapshot(
    generated_output: &GeneratedOutput,
    struct_name: &str,
    appended_documentation: &[String],
) -> String {
    let method_names: BTreeSet<String> = generated_output
        .dsl_methods
        .iter()
        .map(|dsl_method| dsl_method.method_name.to_string())
        .collect();

    assert_eq!(
        method_names.len(),
        generated_output.dsl_methods.len(),
        "`{struct_name}` should generate each DSL method name only once, otherwise its methods cannot be snapshotted separately"
    );

    let appended_documentation = match appended_documentation.is_empty() {
        true => String::new(),
        false => {
            let lines = appended_documentation
                .join("\n")
                .lines()
                .map(|line| format!("/// {line}").trim_end().to_string())
                .collect::<Vec<_>>()
                .join("\n");

            format!("// Documentation appended to the struct:\n{lines}\n\n")
        }
    };

    let items_outside_dsl_methods = format_tokens(&generated_output.items_outside_dsl_methods);
    let manifest = method_names
        .iter()
        .map(|method_name| format!("// {method_name}\n"))
        .collect::<String>();

    format!(
        "{appended_documentation}{items_outside_dsl_methods}\n// Generated DSL methods:\n{manifest}"
    )
}
```

Update the module doc of `characterization_tests.rs`: `table.snap` also starts with the documentation the macro appended to the struct, when it appended any.

- [ ] **Step 6: Observe green, regenerate the snapshots and read the diff**

Run the unit gate: the contract test passes, the snapshots fail. Regenerate. Expected hunks only: `table.snap` of each table with a foreign key or `#[referenced_by]` gains a leading `// Documentation appended to the struct:` block. `foreign_key_and_referenced_by/Warehouse/table.snap` lists *the `shipment` table (`self`)* and *the `inspection` table (`self`)* under *Tables referencing the `warehouse` table*; `Shipment/table.snap` lists two foreign keys, each with *On delete* and *On soft delete*. Tables without relationships are unchanged.

- [ ] **Step 7: Run the unit gate and the runtime gate**

Expected: all `ok`, then `PASSED`.

- [ ] **Step 8: Document the change**

In `docs/DOCUMENTATION.md`, *Generated Documentation*, add the bullet:

```markdown
- The struct gets *Foreign keys of the `<table>` table* and *Tables referencing the `<table>` table* appended to its own documentation.
```

In `docs/MIGRATION.md`, extend the entry *The generated documentation shows foreign keys and cascades* with the bullet *The struct gets its foreign keys and the tables referencing it appended to its documentation.* Under `### For crates building on spacetimedsl_derive-input`, add:

```markdown
#### `SpacetimeDSLTable::struct_doc_comment`

`SpacetimeDSLTable::struct_doc_comment: String` holds the sections `#[spacetimedsl::dsl]` appends to the struct's documentation, empty for a table without foreign keys and without `#[referenced_by]`. Append it to the struct you emit as a `#[doc]` attribute after an empty one, as `spacetimedsl_derive` does.
```

In `CODE_QUALITY_REPORT.md`, entry *`table.rs`: `pub struct SpacetimeDSLTable`*, replace the first sentence of the finding by *`SpacetimeDSLTable::try_parse` builds the table with `create_dsl_method_arg: None`, empty `compile_error_checks` and `compile_error_check_imports`, and an empty `struct_doc_comment`, and method generation fills them in later through `TableContributions::apply_to`.*, and the list in the recommendation by *move `create_dsl_method_arg`, `compile_error_checks`, `compile_error_check_imports` and `struct_doc_comment` into `SpacetimeDSLTableMethods` or into a separate result type*.

- [ ] **Step 9: Format, lint, commit**

Commit message:

```text
Append the relationships to the struct's documentation

The struct's documentation gains "Foreign keys of the <table> table" and "Tables referencing
the <table> table", both naming the table, since a struct with several #[table] attributes is
expanded once per table. SpacetimeDSLTable::struct_doc_comment carries the text; the table
snapshots start with what a pass appended.

Recorded output: table.snap of every table with a foreign key or #[referenced_by].

Closes #35

Co-Authored-By: Claude Opus 5.5 <noreply@anthropic.com>
```

---

## Task 14: Verify the whole branch

**Files:** none changed, unless a check below finds something.

- [ ] **Step 1: Run every gate on a clean tree**

Run the unit gate, the runtime gate, `.\x.ps1 format` twice (the second run changes nothing) and `.\x.ps1 lint`. Expected: all `ok`, `PASSED`, no file changed, exit code `0`.

- [ ] **Step 2: Check the history**

Run: `git log --oneline main..HEAD`
Expected: 14 commits, Task 0 to Task 13, in plan order, each naming its issue.

- [ ] **Step 3: Read the documentation once more**

Read `## 0.23 → 0.24` of `docs/MIGRATION.md` from top to bottom. Every change of Tasks 1–13 has exactly one entry, *Newly accepted inputs* sits after *Newly rejected inputs*, and no entry contradicts a later one. Read *Foreign Keys & Referential Integrity* and *Example Error Messages* in `docs/DOCUMENTATION.md` for the same.

- [ ] **Step 4: Hand over**

Use superpowers:finishing-a-development-branch. The pull request body lists `Closes #183`, `Closes #182`, `Closes #181`, `Closes #180`, `Closes #179`, `Closes #173`, `Closes #172` and `Closes #35`, and names #185 as found and left out. Open it only after the developer agrees.
