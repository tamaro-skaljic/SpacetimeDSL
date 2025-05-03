use spacetime_bindings_macro_input::{sym::Symbol, symbol};

pub mod wrapper;

pub mod foreign_key;

pub mod getter;

pub mod setter;

pub mod method;

pub mod quote;

symbol!(table);
symbol!(plural_name);
symbol!(unique_index);
symbol!(foreign_key);
symbol!(path);
symbol!(column);
symbol!(on_delete);
symbol!(wrap);
symbol!(wrapped);
