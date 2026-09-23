/// The UUID version an `#[auto_gen(v4)]` or `#[auto_gen(v7)]` column is generated with.
#[derive(Clone, Copy, PartialEq)]
pub enum UUIDVersion {
    V4,
    V7,
}
