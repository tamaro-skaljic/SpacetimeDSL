use syn::Ident;

/// Which of the two shapes a soft-delete marker column has.
///
/// The shape decides what a soft deletion writes into the column and how the generated
/// code asks whether a row is already retired.
#[derive(Clone, Copy, PartialEq)]
pub enum SoftDeleteMarkerKind {
    /// A `bool`, set to `true`.
    Flag,
    /// An `Option<Timestamp>`, set to the current timestamp.
    Timestamp,
}

/// The one column a soft deletion writes.
///
/// A table has at most one. Its presence is what makes the table soft-deletable: the
/// `method(soft_delete)` flag and this column are rejected unless they agree.
#[derive(Clone)]
pub struct SoftDeleteMarker {
    pub column_name: Ident,
    pub kind: SoftDeleteMarkerKind,
}
