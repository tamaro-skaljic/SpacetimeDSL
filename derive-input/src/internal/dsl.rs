use spacetime_bindings_macro_input::{sym::Symbol, symbol};

pub mod table;

pub mod hook;

pub mod column;

pub mod wrapper;

pub mod foreign_key;

pub mod reference;

pub mod getter;

pub mod mut_getter;

pub mod setter;

pub mod method;

pub mod one_or_multiple;

pub mod singleton;

symbol!(table);
symbol!(singleton);
symbol!(with_default);
symbol!(plural_name);
symbol!(unique_index);
symbol!(method);
symbol!(r#true);
symbol!(r#false);
symbol!(hook);
symbol!(before);
symbol!(after);
symbol!(insert);
symbol!(update);
symbol!(delete);
symbol!(soft_delete);
symbol!(foreign_key);
symbol!(referenced_by);
symbol!(path);
symbol!(column);
symbol!(on_delete);
symbol!(create_wrapper);
symbol!(use_wrapper);
