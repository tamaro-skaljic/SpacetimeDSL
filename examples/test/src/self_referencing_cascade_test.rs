use crate::spacetimedsl::prelude::*;

/// A `Folder` references its own table, so deleting or retiring one cascades into the
/// same table again, down to the folders which no other folder references.
#[spacetimedsl::dsl(
    plural_name = folders,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(
    accessor = folder,
    public,
)]
pub struct Folder {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = crate::self_referencing_cascade_test, table = folder)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(FolderId)]
    #[foreign_key(
        path = crate::self_referencing_cascade_test,
        table = folder,
        column = id,
        on_delete = Delete,
        on_soft_delete = SoftDelete,
    )]
    pub parent_folder_id: u64,

    deleted: bool,
}

fn create_folder_in<T: WriteContext>(
    dsl: &DSL<'_, T>,
    parent_folder_id: FolderId,
) -> Result<Folder, String> {
    Ok(dsl.create_folder(CreateFolder { parent_folder_id })?)
}

/// Exercises `on_delete = Delete` and `on_soft_delete = SoftDelete` on a table which
/// references itself: a folder no other folder references, a tree of folders, and a
/// folder which references itself.
pub fn run_tests<T: WriteContext>(dsl: &DSL<'_, T>) -> Result<(), String> {
    let no_parent = FolderId::new(0);

    let lone_folder = create_folder_in(dsl, no_parent.clone())?;

    dsl.delete_folder_by_id(&lone_folder).map_err(|error| {
        format!("Should be able to delete a Folder without subfolders! Got:\n{error}")
    })?;

    if dsl.get_folder_by_id(&lone_folder).is_ok() {
        return Err("A deleted Folder without subfolders should be gone!".to_string());
    }

    let root = create_folder_in(dsl, no_parent.clone())?;
    let child = create_folder_in(dsl, root.get_id())?;
    let grandchild = create_folder_in(dsl, child.get_id())?;
    let unrelated = create_folder_in(dsl, no_parent.clone())?;

    dsl.delete_folder_by_id(&root).map_err(|error| {
        format!("Should be able to delete a Folder with subfolders! Got:\n{error}")
    })?;

    if dsl.get_folder_by_id(&child).is_ok() || dsl.get_folder_by_id(&grandchild).is_ok() {
        return Err(
            "on_delete = Delete should have deleted every subfolder of the deleted Folder!"
                .to_string(),
        );
    }

    if dsl.get_folder_by_id(&unrelated).is_err() {
        return Err("Deleting a Folder shouldn't delete a Folder outside its tree!".to_string());
    }

    let mut own_parent = create_folder_in(dsl, no_parent.clone())?;
    own_parent.set_parent_folder_id(own_parent.get_id());
    let own_parent = dsl.update_folder_by_id(own_parent)?;

    dsl.delete_folder_by_id(&own_parent).map_err(|error| {
        format!("Should be able to delete a Folder which is its own parent! Got:\n{error}")
    })?;

    if dsl.get_folder_by_id(&own_parent).is_ok() {
        return Err("A deleted Folder which is its own parent should be gone!".to_string());
    }

    let root = create_folder_in(dsl, no_parent)?;
    let child = create_folder_in(dsl, root.get_id())?;
    let grandchild = create_folder_in(dsl, child.get_id())?;

    dsl.soft_delete_folder_by_id(&root).map_err(|error| {
        format!("Should be able to soft-delete a Folder with subfolders! Got:\n{error}")
    })?;

    for folder in [&root, &child, &grandchild] {
        if !dsl.get_folder_by_id(folder)?.get_deleted() {
            return Err(
                "on_soft_delete = SoftDelete should have retired the Folder and every subfolder!"
                    .to_string(),
            );
        }
    }

    Ok(())
}
