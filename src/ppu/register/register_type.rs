use num_derive::FromPrimitive;

#[derive(PartialEq, Eq, Clone, Copy, Debug, FromPrimitive)]
pub enum RegisterType {
    Ctrl,
    Mask,
    Status,
    OamAddr,
    OamData,
    Scroll,
    PpuAddr,
    PpuData,
}
