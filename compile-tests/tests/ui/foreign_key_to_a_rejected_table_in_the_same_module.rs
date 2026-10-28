//! A rejected `parent` table in the module of the `child` table which references it. The
//! foreign key of `child` reports that `parent` has to be fixed first, although the module
//! holds the `compile_error!` of `parent`.
//!
//! The error after the first is that one, on the `#[foreign_key]`.

::spacetimedsl::spacetimedsl!();

pub mod family {
    #[spacetimedsl::dsl(method(update = false, delete = true))]
    #[spacetimedb::table(accessor = parent, public)]
    pub struct Parent {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        #[referenced_by(path = self, table = child)]
        id: u64,
    }

    #[spacetimedsl::dsl(plural_name = children, method(update = false, delete = true))]
    #[spacetimedb::table(accessor = child, public)]
    pub struct Child {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,

        #[index(btree)]
        #[use_wrapper(ParentId)]
        #[foreign_key(path = self, table = parent, column = id, on_delete = Delete)]
        parent_id: u64,
    }
}

fn main() {}
