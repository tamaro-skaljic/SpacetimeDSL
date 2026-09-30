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

pub mod auto_gen;
pub mod creation_default;
pub mod disallow;

pub mod soft_delete;

pub mod column_role;

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
symbol!(on_soft_delete);
symbol!(create_wrapper);
symbol!(use_wrapper);
symbol!(auto_gen);
symbol!(creation_default);
symbol!(disallow);
symbol!(zero);
symbol!(decreasing);
symbol!(increasing);
symbol!(set_on_create);
symbol!(set_on_update);
symbol!(set_on_soft_delete);
