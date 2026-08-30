use crate::circuit::label::Label;

#[derive(Clone)]
pub enum LabeledGate {
    Constant(bool),
    State,
    Unary(UnaryGateType, Label),
    Binary(BinaryGateType, Label, Label),
}

#[derive(Clone, Copy)]
pub enum UnaryGateType {
    Identity,
    Not,
    StateOr,
    StateNor,
    StateAnd,
    NotStateOr,
    StateAndNot,
}

#[derive(Clone, Copy)]
pub enum BinaryGateType {
    Nor
}

/*
impl LabeledGate {
    pub fn to_indexed<F>(&self, current_index: Index, f: F) -> IndexedGate
    where F: Fn(Label) -> Index {
        match self {
            Self::Constant(constant) => IndexedGate::Constant(*constant),
            Self::State => IndexedGate::Unary(UnaryGateType::Identity, current_index),
            Self::Unary(gate_type, label) => IndexedGate::Unary(*gate_type, f(*label))
        }
    }
}
*/