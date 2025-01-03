#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum NameTableMirroring {
    Vertical,
    Horizontal,
    FourScreen,
    OneScreenLeftBank,
    OneScreenRightBank,
}
