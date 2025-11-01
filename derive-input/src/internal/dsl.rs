use spacetime_bindings_macro_input::{sym::Symbol, symbol};

pub mod table;

pub mod hooks;

pub mod column;

pub mod wrapper;

pub mod foreign_key;

pub mod reference;

pub mod getter;

pub mod setter;

pub mod method;

symbol!(table);
symbol!(plural_name);
symbol!(unique_index);
symbol!(before_insert_hook);
symbol!(before_update_hook);
symbol!(before_delete_hook);
symbol!(after_insert_hook);
symbol!(after_update_hook);
symbol!(after_delete_hook);
symbol!(foreign_key);
symbol!(referenced_by);
symbol!(path);
symbol!(column);
symbol!(on_delete);
symbol!(create_wrapper);
symbol!(use_wrapper);
