/// A rule of `#[disallow(...)]`: what the value of a column must not be or do.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Disallowed {
    /// `zero`: the value `0`, or `Uuid::NIL` for a `Uuid` column.
    Zero,
    /// `decreasing`: a write which makes the value smaller than the stored one.
    Decreasing,
    /// `increasing`: a write which makes the value larger than the stored one.
    Increasing,
}
