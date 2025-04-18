use spacetimedb::ReducerContext;

pub struct DSL<'a> {
    pub ctx: &'a ReducerContext,
}

pub fn dsl<'a>(ctx: &'a ReducerContext) -> DSL<'a> {
    DSL { ctx }
}

pub trait Wrapper<WrappedType: Clone + Default, WrapperType> {
    fn new(value: WrappedType) -> WrapperType;
    fn default() -> WrapperType;
    fn value(&self) -> WrappedType;
}

pub trait DSLContext {
    fn ctx<'a>(&'a self) -> &'a ReducerContext;
}

impl DSLContext for DSL<'_> {
    fn ctx<'a>(&'a self) -> &'a ReducerContext {
        self.ctx
    }
}
