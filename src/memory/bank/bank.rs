use strum_macros::Display;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy, Debug, Display)]
pub enum MemoryPresence {
    Absent,
    Supported,
    Required,
}