#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Writability {
    Rom,
    Ram,
    RomRam,
}
