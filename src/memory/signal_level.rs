use std::marker::ConstParamTy;

#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, ConstParamTy)]
pub enum SignalLevel {
    #[default]
    High,
    Low,
}