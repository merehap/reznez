#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Board {
    Any,

    // Mapper 0
    Nrom128,
    Nrom256,

    // Mapper 3
    Cnrom128,
    Cnrom256,
}
