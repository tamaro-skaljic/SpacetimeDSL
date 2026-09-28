//! `table = <accessor>` has to name the accessor of one of the struct's `#[table]`
//! attributes, whether the struct has several of them or only one.

::spacetimedsl::spacetimedsl!();

pub mod gadget {
    #[spacetimedsl::dsl(plural_name = gadgets, table = gizmo, method(update = false))]
    #[spacetimedb::table(accessor = gadget1, public)]
    #[spacetimedb::table(accessor = gadget2, public)]
    pub struct Gadget {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

pub mod widget {
    #[spacetimedsl::dsl(plural_name = widgets, table = gizmo, method(update = false))]
    #[spacetimedb::table(accessor = widget, public)]
    pub struct Widget {
        #[primary_key]
        #[auto_inc]
        #[create_wrapper]
        id: u64,
    }
}

fn main() {}
