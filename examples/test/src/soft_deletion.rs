use crate::spacetimedsl::prelude::*;
use spacetimedb::Timestamp;

/// A retired `Archive` keeps its row, so a reducer can still read what was retired.
#[spacetimedsl::dsl(
    plural_name = archives,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(
    accessor = archive,
    public,
)]
pub struct Archive {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ArchiveId)]
    #[referenced_by(path = crate::soft_deletion, table = archive_entry)]
    id: u64,

    pub label: String,

    #[set_on_update]
    modified_at: Option<Timestamp>,

    deleted: bool,
}

/// Retiring an `Archive` retires its entries; deleting one removes them.
#[spacetimedsl::dsl(
    plural_name = archive_entries,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(
    accessor = archive_entry,
    public,
)]
pub struct ArchiveEntry {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper(ArchiveEntryId)]
    #[referenced_by(path = crate::soft_deletion, table = archive_entry_note)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ArchiveId)]
    #[foreign_key(
        path = crate::soft_deletion,
        table = archive,
        column = id,
        on_delete = Delete,
        on_soft_delete = SoftDelete,
    )]
    pub archive_id: u64,

    #[set_on_soft_delete]
    retired_at: Option<Timestamp>,
}

/// Being referenced itself makes `ArchiveEntry` retire its rows in bulk and consult
/// this table in turn, so an `Archive` cascade reaches a second level.
#[spacetimedsl::dsl(
    plural_name = archive_entry_notes,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(
    accessor = archive_entry_note,
    public,
)]
pub struct ArchiveEntryNote {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    id: u64,

    #[index(btree)]
    #[use_wrapper(ArchiveEntryId)]
    #[foreign_key(
        path = crate::soft_deletion,
        table = archive_entry,
        column = id,
        on_delete = Delete,
        on_soft_delete = SoftDelete,
    )]
    pub archive_entry_id: u64,

    deleted: bool,
}

/// Exercises soft deletion end to end: the marker is written, the row stays readable,
/// `modified_at` is left alone, the cascade retires the referencing row through
/// `on_soft_delete = SoftDelete` and on to the row referencing that one, and retiring an
/// already retired row does nothing.
pub(crate) fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let archive = dsl.create_archive(CreateArchive {
        label: "first".to_string(),
    })?;
    let archive_id = archive.get_id();

    let entry = dsl.create_archive_entry(CreateArchiveEntry {
        archive_id: archive_id.clone(),
    })?;
    let entry_id = entry.get_id();

    let note = dsl.create_archive_entry_note(CreateArchiveEntryNote {
        archive_entry_id: entry_id.clone(),
    })?;
    let note_id = note.get_id();

    let modified_at_before_soft_delete = *archive.get_modified_at();

    dsl.soft_delete_archive_by_id(&archive_id)?;

    let retired_archive = match dsl.get_archive_by_id(&archive_id) {
        Err(_) => {
            return Err("A soft-deleted Archive row should still be readable!".to_string());
        }
        Ok(retired_archive) => retired_archive,
    };

    if !retired_archive.get_deleted() {
        return Err("soft_delete_archive_by_id should have set the marker!".to_string());
    }

    if retired_archive
        .get_modified_at()
        .ne(&modified_at_before_soft_delete)
    {
        return Err("A soft deletion should leave modified_at alone!".to_string());
    }

    let retired_entry = match dsl.get_archive_entry_by_id(&entry_id) {
        Err(_) => {
            return Err(
                "An ArchiveEntry of a soft-deleted Archive should still exist!".to_string(),
            );
        }
        Ok(retired_entry) => retired_entry,
    };

    if retired_entry.get_retired_at().is_none() {
        return Err(
            "on_soft_delete = SoftDelete should have retired the ArchiveEntry!".to_string(),
        );
    }

    let retired_note = match dsl.get_archive_entry_note_by_id(&note_id) {
        Err(_) => {
            return Err(
                "An ArchiveEntryNote of a soft-deleted ArchiveEntry should still exist!"
                    .to_string(),
            );
        }
        Ok(retired_note) => retired_note,
    };

    if !retired_note.get_deleted() {
        return Err(
            "Retiring an Archive should have retired the ArchiveEntryNote of its ArchiveEntry!"
                .to_string(),
        );
    }

    let repeated = dsl.soft_delete_archive_by_id(&archive_id)?;

    if !repeated.entries.is_empty() {
        return Err("Soft-deleting an already retired row should report no entries!".to_string());
    }

    Ok(())
}
