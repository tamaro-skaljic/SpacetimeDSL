//! Covers a cascade into the table it starts from: `Folder` references its own table with
//! `on_delete = Delete` and `on_soft_delete = SoftDelete`, so each dispatcher of `Folder`
//! reaches itself again through the strategy which removes the subfolders.

#[spacetimedsl::dsl(
    plural_name = folders,
    method(update = true, delete = true, soft_delete = true),
)]
#[spacetimedb::table(accessor = folder, public)]
pub struct Folder {
    #[primary_key]
    #[auto_inc]
    #[create_wrapper]
    #[referenced_by(path = self, table = folder)]
    id: u64,

    #[index(btree)]
    #[use_wrapper(FolderId)]
    #[foreign_key(
        path = self,
        table = folder,
        column = id,
        on_delete = Delete,
        on_soft_delete = SoftDelete,
    )]
    pub parent_folder_id: u64,

    deleted: bool,
}
