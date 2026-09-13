//! `#[dsl]` reads the table accessor, the indices and the column attributes out of the
//! `#[table]` attribute below it, so it has nothing to work with on its own.

::spacetimedsl::spacetimedsl!();

pub mod thing {
    #[spacetimedsl::dsl(plural_name = things, method(update = false))]
    pub struct Thing {
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
