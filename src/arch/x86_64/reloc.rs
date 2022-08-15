#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum RelocKind {
    Unknown(u32),
}

impl RelocKind {
    pub const fn from_u32(kind: u32) -> RelocKind {
        match kind {
            _ => RelocKind::Unknown(kind),
        }
    }
}
