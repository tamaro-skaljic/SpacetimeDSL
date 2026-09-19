//! Covers columns the generator fills in by itself: created timestamps are set once on insert,
//! and updated timestamps are set on every update. Both the plain and the `Option` spelling are
//! present, because they are set differently.
//!
//! `modified_at` is also what makes `update = true` legal here although no column is
//! public.

use spacetimedb::Timestamp;

#[spacetimedsl::dsl(plural_name = signed_documents, method(update = true))]
#[spacetimedb::table(accessor = signed_document, public)]
pub struct SignedDocument {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    title: String,

    signature: SignatureId,

    #[created_at]
    created_at: Timestamp,

    #[updated_at]
    modified_at: Option<Timestamp>,
}

#[spacetimedsl::dsl(plural_name = signatures, method(update = true))]
#[spacetimedb::table(accessor = signature, public)]
pub struct Signature {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[create_wrapper]
    #[index(btree)]
    value: String,

    created_at: Timestamp,

    modified_at: Timestamp,
}

#[spacetimedsl::dsl(plural_name = timer_records, method(update = true))]
#[spacetimedb::table(accessor = timer_record, public)]
pub struct TimerRecord {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[created_at]
    started_at: Timestamp,

    #[updated_at]
    finished_at: Option<Timestamp>,
}
