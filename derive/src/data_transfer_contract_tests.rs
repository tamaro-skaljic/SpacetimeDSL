//! Pins the public data-transfer structure of `spacetimedsl_derive-input` from the side of a
//! crate that consumes it.
//!
//! Every public struct reachable from [`Table`] is destructured without `..`, and every public
//! enum is matched without a wildcard arm. Adding, removing or renaming a field or a variant
//! therefore fails to compile here, which is what it does to every crate building on the model:
//! a change of this structure is a breaking change. The assertions read a few values of three
//! parsed fixture tables, so the test also checks what the structure holds.

use {
    proc_macro2::TokenStream,
    quote::{ToTokens, quote},
    spacetimedsl_derive_input::api::{
        Column, Table,
        db::{
            column::SpacetimeDBColumn,
            index::{Index, IndexType},
            reducer::ScheduledReducer,
            table::{SpacetimeDBTable, SpacetimeDBTableVisibility},
        },
        dsl::{
            auto_gen::UUIDVersion,
            column::{
                SpacetimeDSLColumn, SpacetimeDSLColumnMethods, SpacetimeDSLColumnMethodsForIndex,
                SpacetimeDSLColumnMethodsForUniqueIndex,
            },
            foreign_key::{ForeignKey, OnDeleteStrategy},
            getter::Getter,
            hook::{HookKind, Operation, SpacetimeDSLMethodHook, SpacetimeDSLMethodHooks, Timing},
            method::{SpacetimeDSLArg, SpacetimeDSLArgType, SpacetimeDSLMethod},
            mut_getter::MutGetter,
            reference::ReferencingTable,
            setter::Setter,
            soft_delete::{SoftDeleteMarker, SoftDeleteMarkerKind},
            table::{
                CascadeEntryPoints, CreateDSLMethodArg, OnDeleteStrategiesOfReferencingTables,
                OnDeleteStrategiesOfTheReferencedTable, SingletonKind, SpacetimeDSLTable,
                SpacetimeDSLTableKind, SpacetimeDSLTableMethods,
            },
            wrapper::{CreatedWrapper, UsedWrapper, WrapperMethod, WrapperType},
        },
        rust::{column::RustField, table::RustStruct, visibility::RustVisibility},
    },
};

#[test]
fn the_model_holds_what_a_table_declares() {
    let gadget = parse_table(
        quote! {
            plural_name = gadgets,
            method(update = true, delete = true, soft_delete = true),
            hook(before(insert)),
            unique_index(name = owner_id_and_name),
        },
        quote! {
            #[spacetimedb::table(
                accessor = gadget,
                public,
                index(accessor = owner_id_and_name, btree(columns = [owner_id, name])),
            )]
            pub struct Gadget {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                #[referenced_by(path = crate::part, table = part)]
                id: u64,

                #[index(btree)]
                #[use_wrapper(crate::owner::OwnerId)]
                #[foreign_key(path = crate::owner, table = owner, column = id, on_delete = Delete)]
                pub owner_id: u64,

                pub name: String,

                #[creation_default(1)]
                pub revision: u32,

                #[unique]
                #[create_wrapper]
                #[auto_gen(v7)]
                serial_number: spacetimedb::Uuid,

                created_at: spacetimedb::Timestamp,

                deleted: bool,
            }
        },
    );
    visit_table(&gadget);

    assert_eq!(gadget.rust_struct.name, "Gadget");
    assert!(matches!(
        gadget.rust_struct.visibility,
        RustVisibility::Public
    ));
    assert_eq!(gadget.columns.len(), 7);
    assert_eq!(gadget.primary_key_column.rust_field.name, "id");

    let spacetimedb_table = &gadget.spacetimedb_table;
    assert_eq!(spacetimedb_table.singular_name, "gadget");
    assert!(matches!(
        spacetimedb_table.visibility,
        SpacetimeDBTableVisibility::Public
    ));
    assert!(spacetimedb_table.scheduled_reducer.is_none());
    let [multi_column_index] = spacetimedb_table.multi_column_indices.as_slice() else {
        panic!("the table declares one multi-column index");
    };
    assert_eq!(multi_column_index.name, "owner_id_and_name");
    assert!(multi_column_index.is_unique);
    let IndexType::BTreeMultiColumn { columns } = &multi_column_index.index_type else {
        panic!("`owner_id_and_name` is a btree index over two columns");
    };
    assert_eq!(columns, &["owner_id", "name"]);

    let spacetimedsl_table = &gadget.spacetimedsl_table;
    let SpacetimeDSLTableKind::Normal { plural_name } = &spacetimedsl_table.kind else {
        panic!("`gadget` is not a singleton");
    };
    assert_eq!(plural_name, "gadgets");
    assert!(spacetimedsl_table.has_update_method);
    assert!(spacetimedsl_table.has_delete_method);
    let soft_delete_marker = spacetimedsl_table
        .soft_delete_marker
        .as_ref()
        .expect("`deleted` is the soft-delete marker");
    assert_eq!(soft_delete_marker.column_name, "deleted");
    assert!(matches!(
        soft_delete_marker.kind,
        SoftDeleteMarkerKind::Flag
    ));
    assert_eq!(
        spacetimedsl_table
            .on_insert_set_current_timestamp_column_name
            .as_ref()
            .expect("`created_at` is set on insert"),
        "created_at"
    );
    assert!(
        spacetimedsl_table
            .on_update_set_current_timestamp_column_name
            .is_none()
    );
    let [referencing_table] = spacetimedsl_table.referencing_tables.as_slice() else {
        panic!("one table references the table");
    };
    assert_eq!(referencing_table.table_name, "part");
    let compile_error_check_imports: Vec<String> = spacetimedsl_table
        .compile_error_check_imports
        .iter()
        .map(|path| path.to_token_stream().to_string())
        .collect();
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
    assert_eq!(
        spacetimedsl_table.struct_doc_comment,
        "# Foreign keys of the `gadget` table\n\n- `owner_id` references the `id` column of the `owner` table (`crate::owner`).\n  - On delete: `Delete`\n  - On soft delete: none, the `owner` table is not soft-deletable\n\n# Tables referencing the `gadget` table\n\n- the `part` table (`crate::part`)"
    );
    let create_dsl_method_arg = spacetimedsl_table
        .create_dsl_method_arg
        .as_ref()
        .expect("the table has a create method");
    assert_eq!(create_dsl_method_arg.struct_name, "CreateGadget");
    let owner_id_member = create_dsl_method_arg
        .struct_members
        .iter()
        .find(|member| member.arg_name == "owner_id")
        .expect("the caller supplies `owner_id`");
    assert_eq!(
        owner_id_member.arg_type.actual_type().to_string(),
        "crate :: owner :: OwnerId"
    );
    let before_insert = spacetimedsl_table
        .hooks
        .get(HookKind::BEFORE_INSERT)
        .expect("`before(insert)` is declared");
    assert_eq!(before_insert.function_name, "before_gadget_insert");
    assert!(
        spacetimedsl_table
            .hooks
            .get(HookKind::AFTER_INSERT)
            .is_none()
    );

    let id = column(&gadget, "id");
    assert!(id.spacetimedb_column.is_primary_key);
    assert!(id.spacetimedb_column.is_auto_inc);
    let Some(WrapperType::Created(created_wrapper)) = &id.spacetimedsl_column.wrapper_type else {
        panic!("`id` has `#[create_wrapper]`");
    };
    assert_eq!(created_wrapper.wrapper_struct_name, "GadgetId");
    let Some(SpacetimeDSLColumnMethods::ForUniqueIndex(id_methods)) = &id.spacetimedsl_methods
    else {
        panic!("the primary key is a unique index");
    };
    assert_eq!(id_methods.get_one_option.method_name, "get_gadget_by_id");
    assert!(id_methods.get_one_option.read_context_compatible);
    assert!(id_methods.update.is_some());
    assert!(id_methods.delete_one.is_some());
    assert!(id_methods.soft_delete_one.is_some());

    let owner_id = column(&gadget, "owner_id");
    let Some(WrapperType::Used(used_wrapper)) = &owner_id.spacetimedsl_column.wrapper_type else {
        panic!("`owner_id` has `#[use_wrapper]`");
    };
    assert_eq!(
        used_wrapper
            .wrapper_struct_name_or_path
            .to_token_stream()
            .to_string(),
        "crate :: owner :: OwnerId"
    );
    let foreign_key = owner_id
        .spacetimedsl_column
        .foreign_key
        .as_ref()
        .expect("`owner_id` has `#[foreign_key]`");
    assert_eq!(foreign_key.table_name, "owner");
    assert_eq!(foreign_key.primary_key_column_name, "id");
    assert_eq!(
        foreign_key.on_delete_strategy,
        Some(OnDeleteStrategy::Delete)
    );
    assert_eq!(
        owner_id
            .spacetimedsl_column
            .getter
            .as_ref()
            .expect("every column has a getter")
            .doc_comment,
        "References the `id` column of the `owner` table (`crate::owner`).\n\n- On delete: `Delete`\n- On soft delete: none, the `owner` table is not soft-deletable"
    );
    let Some(SpacetimeDSLColumnMethods::ForIndex(owner_id_methods)) =
        &owner_id.spacetimedsl_methods
    else {
        panic!("`owner_id` has a non-unique index");
    };
    assert_eq!(
        owner_id_methods.get_many.method_name,
        "get_gadgets_by_owner_id"
    );
    assert!(owner_id_methods.delete_many.is_some());

    let name = column(&gadget, "name");
    assert!(matches!(name.rust_field.visibility, RustVisibility::Public));
    assert_eq!(
        name.rust_field
            .type_name_or_path
            .to_token_stream()
            .to_string(),
        "String"
    );
    assert_eq!(
        name.spacetimedsl_column
            .getter
            .as_ref()
            .expect("every column has a getter")
            .method_name,
        "get_name"
    );
    assert_eq!(
        name.spacetimedsl_column
            .mut_getter
            .as_ref()
            .expect("a public column without a wrapper has a mut getter")
            .method_name,
        "get_name_mut"
    );
    assert_eq!(
        name.spacetimedsl_column
            .setter
            .as_ref()
            .expect("a public column has a setter")
            .method_name,
        "set_name"
    );

    let revision = column(&gadget, "revision");
    assert_eq!(
        revision
            .spacetimedsl_column
            .creation_default
            .as_ref()
            .expect("`revision` has `#[creation_default(1)]`")
            .to_token_stream()
            .to_string(),
        "1"
    );
    assert!(
        create_dsl_method_arg
            .struct_members
            .iter()
            .all(|member| member.arg_name != "revision"),
        "a column with `#[creation_default]` is not asked of the caller"
    );

    let serial_number = column(&gadget, "serial_number");
    assert!(matches!(
        serial_number
            .spacetimedsl_column
            .auto_generated_uuid_version,
        Some(UUIDVersion::V7)
    ));

    let spacetimedsl_methods = &gadget.spacetimedsl_methods;
    assert!(spacetimedsl_methods.create.is_some());
    assert_eq!(
        spacetimedsl_methods
            .get_all
            .as_ref()
            .expect("every table has get_all")
            .method_name,
        "get_all_gadgets"
    );
    assert!(spacetimedsl_methods.get_count.is_some());
    let entry_points_of_referenced_table = spacetimedsl_methods
        .on_delete_strategies_of_referencing_tables
        .as_ref()
        .expect("`part` references the table");
    assert!(entry_points_of_referenced_table.on_deletion.is_some());
    assert!(entry_points_of_referenced_table.on_soft_deletion.is_some());
    let [strategies_for_owner] = spacetimedsl_methods
        .on_delete_strategies_of_this_table
        .as_slice()
    else {
        panic!("the table references one table");
    };
    assert!(strategies_for_owner.on_deletion.is_some());
    assert_eq!(spacetimedsl_methods.multi_column_indices.len(), 1);
    let [wrapper_method] = spacetimedsl_methods.wrapper_methods.as_slice() else {
        panic!("the one foreign key column adds one method to its wrapper type");
    };
    assert_eq!(wrapper_method.method_name, "get_gadgets");

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

    let cleanup_timer = parse_table(
        quote! { plural_name = cleanup_timers, method(update = false) },
        quote! {
            #[spacetimedb::table(accessor = cleanup_timer, scheduled(run_cleanup))]
            pub struct CleanupTimer {
                #[primary_key]
                #[auto_inc]
                #[create_wrapper]
                scheduled_id: u64,

                scheduled_at: spacetimedb::ScheduleAt,
            }
        },
    );
    visit_table(&cleanup_timer);

    assert!(matches!(
        cleanup_timer.spacetimedb_table.visibility,
        SpacetimeDBTableVisibility::Private
    ));
    assert!(
        cleanup_timer
            .spacetimedb_table
            .scheduled_reducer
            .as_ref()
            .expect("the table declares `scheduled(run_cleanup)`")
            .reducer_path
            .is_ident("run_cleanup")
    );
    assert!(!cleanup_timer.spacetimedsl_table.has_update_method);

    // The struct carries the `#[primary_key] id: u8` that `#[dsl]` injects into a singleton
    // before it calls `Table::try_parse`.
    let feature_flags = parse_table(
        quote! { singleton(with_default), method(update = true) },
        quote! {
            #[spacetimedb::table(accessor = feature_flags, public)]
            pub struct FeatureFlags {
                #[primary_key]
                id: u8,

                pub tutorial_is_enabled: bool,
            }
        },
    );
    visit_table(&feature_flags);

    assert!(matches!(
        feature_flags.spacetimedsl_table.kind,
        SpacetimeDSLTableKind::Singleton(SingletonKind::WithDefault)
    ));
}

fn parse_table(args: TokenStream, item: TokenStream) -> Table {
    let derive_input: syn::DeriveInput = syn::parse2(item).expect("the fixture should be a struct");

    Table::try_parse(args, &derive_input)
        .unwrap_or_else(|error| panic!("the fixture should be accepted: {error}"))
}

fn column<'a>(table: &'a Table, column_name: &str) -> &'a Column {
    table
        .columns
        .iter()
        .find(|column| column.rust_field.name == column_name)
        .unwrap_or_else(|| panic!("the fixture should have a column `{column_name}`"))
}

fn visit_table(table: &Table) {
    let Table {
        rust_struct,
        spacetimedb_table,
        spacetimedsl_table,
        columns,
        primary_key_column,
        spacetimedsl_methods,
    } = table;

    visit_rust_struct(rust_struct);
    visit_spacetimedb_table(spacetimedb_table);
    visit_spacetimedsl_table(spacetimedsl_table);
    columns.iter().for_each(visit_column);
    visit_column(primary_key_column);
    visit_spacetimedsl_table_methods(spacetimedsl_methods);
}

fn visit_rust_struct(rust_struct: &RustStruct) {
    let RustStruct {
        visibility,
        name: _,
    } = rust_struct;

    visit_rust_visibility(visibility);
}

fn visit_rust_visibility(visibility: &RustVisibility) {
    match visibility {
        RustVisibility::Public | RustVisibility::Private => {}
        RustVisibility::Restricted(_path) => {}
    }
}

fn visit_spacetimedb_table(spacetimedb_table: &SpacetimeDBTable) {
    let SpacetimeDBTable {
        singular_name: _,
        visibility,
        multi_column_indices,
        scheduled_reducer,
    } = spacetimedb_table;

    match visibility {
        SpacetimeDBTableVisibility::Public | SpacetimeDBTableVisibility::Private => {}
    }
    multi_column_indices.iter().for_each(visit_index);
    if let Some(ScheduledReducer { reducer_path: _ }) = scheduled_reducer {}
}

fn visit_index(index: &Index) {
    let Index {
        name: _,
        is_unique: _,
        index_type,
    } = index;

    match index_type {
        IndexType::BTreeMultiColumn { columns: _ } | IndexType::HashMultiColumn { columns: _ } => {}
        IndexType::BTreeSingleColumn { column: _ }
        | IndexType::HashSingleColumn { column: _ }
        | IndexType::Direct { column: _ } => {}
    }
}

fn visit_spacetimedsl_table(spacetimedsl_table: &SpacetimeDSLTable) {
    let SpacetimeDSLTable {
        kind,
        has_update_method: _,
        has_delete_method: _,
        soft_delete_marker,
        on_insert_set_current_timestamp_column_name: _,
        on_update_set_current_timestamp_column_name: _,
        referencing_tables,
        compile_error_checks: _,
        compile_error_check_imports: _,
        create_dsl_method_arg,
        hooks,
        struct_doc_comment: _,
    } = spacetimedsl_table;

    match kind {
        SpacetimeDSLTableKind::Normal { plural_name: _ } => {}
        SpacetimeDSLTableKind::Singleton(
            SingletonKind::WithoutDefault | SingletonKind::WithDefault,
        ) => {}
    }
    if let Some(SoftDeleteMarker {
        column_name: _,
        kind,
    }) = soft_delete_marker
    {
        match kind {
            SoftDeleteMarkerKind::Flag | SoftDeleteMarkerKind::Timestamp => {}
        }
    }
    for ReferencingTable {
        path: _,
        table_name: _,
    } in referencing_tables
    {}
    if let Some(CreateDSLMethodArg {
        struct_name: _,
        struct_members,
        struct_impl: _,
    }) = create_dsl_method_arg
    {
        struct_members.iter().for_each(visit_arg);
    }
    visit_hooks(hooks);
}

fn visit_hooks(hooks: &SpacetimeDSLMethodHooks) {
    let SpacetimeDSLMethodHooks { declared } = hooks;

    for (kind, hook) in declared {
        let HookKind { timing, operation } = kind;
        let _: (&Timing, &Operation) = (timing, operation);
        visit_hook(hook);
    }
}

fn visit_hook(hook: &SpacetimeDSLMethodHook) {
    let SpacetimeDSLMethodHook {
        trait_name: _,
        function_name: _,
        function_args,
        return_type: _,
    } = hook;

    function_args.iter().for_each(visit_arg);
}

fn visit_column(column: &Column) {
    let Column {
        rust_field,
        spacetimedb_column,
        spacetimedsl_column,
        spacetimedsl_methods,
    } = column;

    let RustField {
        visibility,
        name: _,
        type_name_or_path: _,
    } = rust_field;
    visit_rust_visibility(visibility);

    let SpacetimeDBColumn {
        is_primary_key: _,
        single_column_index,
        is_auto_inc: _,
    } = spacetimedb_column;
    if let Some(index) = single_column_index {
        visit_index(index);
    }

    visit_spacetimedsl_column(spacetimedsl_column);

    if let Some(column_methods) = spacetimedsl_methods {
        visit_column_methods(column_methods);
    }
}

fn visit_spacetimedsl_column(spacetimedsl_column: &SpacetimeDSLColumn) {
    let SpacetimeDSLColumn {
        is_option: _,
        wrapper_type,
        foreign_key,
        auto_generated_uuid_version,
        creation_default: _,
        getter,
        mut_getter,
        setter,
    } = spacetimedsl_column;

    match wrapper_type {
        None => {}
        Some(WrapperType::Created(CreatedWrapper {
            wrapper_struct_name: _,
            wrapped_type_name_or_path: _,
            wrapper_impl: _,
        })) => {}
        Some(WrapperType::Used(UsedWrapper {
            wrapper_struct_name_or_path: _,
            wrapped_type_name_or_path: _,
        })) => {}
    }
    if let Some(ForeignKey {
        path: _,
        table_name: _,
        primary_key_column_name: _,
        on_delete_strategy,
        on_soft_delete_strategy,
    }) = foreign_key
    {
        [on_delete_strategy, on_soft_delete_strategy]
            .into_iter()
            .flatten()
            .for_each(visit_on_delete_strategy);
    }
    match auto_generated_uuid_version {
        None | Some(UUIDVersion::V4) | Some(UUIDVersion::V7) => {}
    }
    if let Some(Getter {
        doc_comment: _,
        method_name: _,
        return_type: _,
        method_impl: _,
    }) = getter
    {}
    if let Some(MutGetter {
        method_visibility,
        method_name: _,
        return_type: _,
        method_impl: _,
    }) = mut_getter
    {
        visit_rust_visibility(method_visibility);
    }
    if let Some(Setter {
        doc_comment: _,
        method_visibility,
        method_name: _,
        method_arg: _,
        return_type: _,
        method_impl: _,
    }) = setter
    {
        visit_rust_visibility(method_visibility);
    }
}

fn visit_on_delete_strategy(on_delete_strategy: &OnDeleteStrategy) {
    match on_delete_strategy {
        OnDeleteStrategy::Error
        | OnDeleteStrategy::Delete
        | OnDeleteStrategy::SoftDelete
        | OnDeleteStrategy::SetZero
        | OnDeleteStrategy::Ignore => {}
    }
}

fn visit_column_methods(column_methods: &SpacetimeDSLColumnMethods) {
    match column_methods {
        SpacetimeDSLColumnMethods::ForUniqueIndex(SpacetimeDSLColumnMethodsForUniqueIndex {
            get_one_option,
            update,
            delete_one,
            soft_delete_one,
        }) => {
            visit_method(get_one_option);
            [update, delete_one, soft_delete_one]
                .into_iter()
                .flatten()
                .for_each(visit_method);
        }
        SpacetimeDSLColumnMethods::ForIndex(SpacetimeDSLColumnMethodsForIndex {
            get_many,
            delete_many,
            soft_delete_many,
        }) => {
            visit_method(get_many);
            [delete_many, soft_delete_many]
                .into_iter()
                .flatten()
                .for_each(visit_method);
        }
    }
}

fn visit_method(method: &SpacetimeDSLMethod) {
    let SpacetimeDSLMethod {
        doc_comment: _,
        method_name: _,
        method_args,
        return_type: _,
        method_impl: _,
        read_context_compatible: _,
    } = method;

    method_args.iter().for_each(visit_arg);
}

fn visit_arg(arg: &SpacetimeDSLArg) {
    let SpacetimeDSLArg {
        is_option: _,
        arg_name: _,
        arg_type,
    } = arg;

    match arg_type {
        SpacetimeDSLArgType::Normal(_actual_type) => {}
        SpacetimeDSLArgType::Wrapped {
            wrapped_type: _,
            actual_type: _,
        } => {}
    }
}

fn visit_spacetimedsl_table_methods(spacetimedsl_methods: &SpacetimeDSLTableMethods) {
    let SpacetimeDSLTableMethods {
        create,
        get_all,
        get_count,
        on_delete_strategies_of_referencing_tables,
        on_delete_strategies_of_this_table,
        multi_column_indices,
        wrapper_methods,
    } = spacetimedsl_methods;

    [create, get_all, get_count]
        .into_iter()
        .flatten()
        .for_each(visit_method);
    if let Some(OnDeleteStrategiesOfReferencingTables {
        on_deletion,
        on_soft_deletion,
    }) = on_delete_strategies_of_referencing_tables
    {
        [on_deletion, on_soft_deletion]
            .into_iter()
            .flatten()
            .for_each(visit_cascade_entry_points);
    }
    for OnDeleteStrategiesOfTheReferencedTable {
        on_deletion,
        on_soft_deletion,
    } in on_delete_strategies_of_this_table
    {
        [on_deletion, on_soft_deletion]
            .into_iter()
            .flatten()
            .for_each(visit_cascade_entry_points);
    }
    multi_column_indices.iter().for_each(visit_column_methods);
    for WrapperMethod {
        wrapper_type: _,
        doc_comment: _,
        method_name: _,
        return_type: _,
        method_impl: _,
    } in wrapper_methods
    {}
}

fn visit_cascade_entry_points(cascade_entry_points: &CascadeEntryPoints) {
    let CascadeEntryPoints {
        after_one_row,
        after_multiple_rows,
    } = cascade_entry_points;

    visit_method(after_one_row);
    visit_method(after_multiple_rows);
}
