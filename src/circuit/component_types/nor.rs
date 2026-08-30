use std::collections::BTreeMap;

use crate::circuit::component_type::{CofactorId, ComponentType};
use crate::circuit::gate::LabeledGate;
use crate::circuit::gate::UnaryGateType::*;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::label::Label::*;
use crate::circuit::node::ComponentTypeName;
use crate::circuit::signal_state::SignalState;

const SINK_LABELS: &'static [Label] = &[A, B];
const SOURCE_LABELS: &'static [Label] = &[Q];

#[derive(Clone, Debug)]
pub struct Nor;

impl ComponentType for Nor {
    fn calculate_output_signal_states(&mut self, inputs: &[SignalState; LABEL_COUNT], outputs: &mut [SignalState; LABEL_COUNT]) {
        use SignalState::*;
        let q = match (&inputs[A as usize], &inputs[B as usize]) {
            (Resolved(false), Resolved(false)) => Resolved(true),
            (Resolved(true) , _              ) => Resolved(false),
            (_              , Resolved(true) ) => Resolved(false),
            (Unknown        , _              ) => Unknown,
            (_              , Unknown        ) => Unknown,
        };


        outputs[Q as usize] = q;
    }

    fn cofactors(&self) -> BTreeMap<CofactorId, LabeledGate> {
        BTreeMap::from([
            (CofactorId::new((A, false), Q), LabeledGate::Unary(Not, B)),
            (CofactorId::new((A, true ), Q), LabeledGate::Constant(false)),
            (CofactorId::new((B, false), Q), LabeledGate::Unary(Not, A)),
            (CofactorId::new((B, true ), Q), LabeledGate::Constant(false)),
        ])
    }

    fn component_type_name(&self) -> ComponentTypeName {
        ComponentTypeName::Nor
    }

    fn sink_labels(&self) -> &[Label] {
        SINK_LABELS
    }

    fn source_labels(&self) -> &[Label] {
        SOURCE_LABELS
    }
}
