/// A rule of `#[disallow(...)]`: what the value of a column must not be.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Disallowed {
    /// `zero`: the value `0`, or `Uuid::NIL` for a `Uuid` column.
    Zero,
}
