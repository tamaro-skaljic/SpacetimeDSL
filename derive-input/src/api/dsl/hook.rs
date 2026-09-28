use std::collections::BTreeMap;

use ident_case::RenameRule;
use proc_macro2::TokenStream;
use quote::format_ident;
use syn::Ident;

use crate::api::dsl::method::SpacetimeDSLArg;

/// When a hook runs: before or after the write it hooks into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Timing {
    Before,
    After,
}

/// The write a hook hooks into.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum Operation {
    Insert,
    Update,
    Delete,
    SoftDelete,
}

/// A hook `#[dsl(hook(<timing>(<operation>)))]` can declare.
///
/// The derived order — every before hook ahead of every after hook, each timing in the
/// order insert, update, delete, soft delete — is the order the hooks are emitted in.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct HookKind {
    pub timing: Timing,
    pub operation: Operation,
}

impl HookKind {
    pub const BEFORE_INSERT: HookKind = HookKind::new(Timing::Before, Operation::Insert);
    pub const BEFORE_UPDATE: HookKind = HookKind::new(Timing::Before, Operation::Update);
    pub const BEFORE_DELETE: HookKind = HookKind::new(Timing::Before, Operation::Delete);
    pub const BEFORE_SOFT_DELETE: HookKind = HookKind::new(Timing::Before, Operation::SoftDelete);
    pub const AFTER_INSERT: HookKind = HookKind::new(Timing::After, Operation::Insert);
    pub const AFTER_UPDATE: HookKind = HookKind::new(Timing::After, Operation::Update);
    pub const AFTER_DELETE: HookKind = HookKind::new(Timing::After, Operation::Delete);
    pub const AFTER_SOFT_DELETE: HookKind = HookKind::new(Timing::After, Operation::SoftDelete);

    /// Every kind, in their order.
    pub const ALL: [HookKind; 8] = [
        HookKind::BEFORE_INSERT,
        HookKind::BEFORE_UPDATE,
        HookKind::BEFORE_DELETE,
        HookKind::BEFORE_SOFT_DELETE,
        HookKind::AFTER_INSERT,
        HookKind::AFTER_UPDATE,
        HookKind::AFTER_DELETE,
        HookKind::AFTER_SOFT_DELETE,
    ];

    pub const fn new(timing: Timing, operation: Operation) -> HookKind {
        HookKind { timing, operation }
    }
}

/// The hooks `#[dsl(hook(before(...), after(...)))]` declares.
#[derive(Clone)]
pub struct SpacetimeDSLMethodHooks {
    /// Every declared hook under its kind. A kind the table does not declare has no entry.
    pub declared: BTreeMap<HookKind, SpacetimeDSLMethodHook>,
}

impl SpacetimeDSLMethodHooks {
    /// The hook of `kind`, or `None` when the table does not declare it.
    pub fn get(&self, kind: HookKind) -> Option<&SpacetimeDSLMethodHook> {
        self.declared.get(&kind)
    }

    /// Every declared hook, in the order of their kinds.
    pub fn iter(&self) -> impl Iterator<Item = &SpacetimeDSLMethodHook> {
        self.declared.values()
    }
}

/// One declared hook: a trait the generated code declares and calls, which the user
/// implements with a `#[spacetimedsl::hook]` function.
#[derive(Clone)]
pub struct SpacetimeDSLMethodHook {
    /// `<Timing><Table><Operation>Hook`, such as `BeforeEntityInsertHook`.
    pub trait_name: Ident,
    /// `<timing>_<table>_<operation>`, such as `before_entity_insert`: the name the
    /// `#[spacetimedsl::hook]` function has to have.
    pub function_name: Ident,
    /// The parameters of the hook function: the DSL, then the row or rows it sees.
    pub function_args: Vec<SpacetimeDSLArg>,
    /// What the hook function returns: for a before hook of an insert, update or soft
    /// deletion the value to write, otherwise `()`, in a `Result` with `SpacetimeDSLError`.
    pub return_type: TokenStream,
}

/// The trait a hook function implements: the PascalCase form of `hook_function_name`
/// followed by `Hook`, such as `BeforeEntityInsertHook` for `before_entity_insert`.
///
/// The generator declares the trait under this name and `#[spacetimedsl::hook]` implements
/// it under this name, so both have to call this one function.
pub fn hook_trait_name(hook_function_name: &Ident) -> Ident {
    format_ident!(
        "{}Hook",
        RenameRule::PascalCase.apply_to_field(hook_function_name.to_string())
    )
}
