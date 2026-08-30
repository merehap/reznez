use std::collections::BTreeMap;

use crate::circuit::component_type::{CofactorId, ComponentType};
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::label::Label::*;
use crate::circuit::node::ComponentTypeName;
use crate::circuit::signal_state::SignalState;
use crate::circuit::gate::LabeledGate;
use crate::circuit::gate::UnaryGateType::*;

const SINK_LABELS: &'static [Label] = &[E, D];
const SOURCE_LABELS: &'static [Label] = &[Q, N];

#[derive(Clone, Debug)]
pub struct DLatch {
    state: bool,
}

impl DLatch {
    pub fn new(initial_state: bool) -> Self {
        Self { state: initial_state }
    }
}

impl ComponentType for DLatch {
    fn calculate_output_signal_states(
        &mut self,
        inputs: &[SignalState; LABEL_COUNT],
        outputs: &mut [SignalState; LABEL_COUNT],
    ) {

        use SignalState::*;
        let new_state = match (&inputs[E as usize], &inputs[D as usize]) {
            (Resolved(true) , d      ) => *d,
            (Resolved(false), _      ) => Resolved(self.state),
            (Unknown        , _      ) => panic!("Don't know what to do if DLatch is neither enabled nor disabled."),
        };

        if let SignalState::Resolved(state) = new_state {
            self.state = state;
        }

        outputs[Q as usize] = new_state;
        outputs[N as usize] = new_state.negate();
    }

    fn cofactors(&self) -> BTreeMap<CofactorId, LabeledGate> {
        BTreeMap::from([
            (CofactorId::new((E, false), Q), LabeledGate::State),
            (CofactorId::new((E, false), N), LabeledGate::State),
            (CofactorId::new((E, true ), Q), LabeledGate::Unary(Identity, D)),
            (CofactorId::new((E, true ), N), LabeledGate::Unary(Not, D)),
            (CofactorId::new((D, false), Q), LabeledGate::Unary(StateAndNot, E)),
            (CofactorId::new((D, false), N), LabeledGate::Unary(NotStateOr, E)),
            (CofactorId::new((D, true ), Q), LabeledGate::Unary(StateOr, E)),
            (CofactorId::new((D, true ), N), LabeledGate::Unary(StateNor, E)),
        ])
    }

    fn component_type_name(&self) -> ComponentTypeName {
        ComponentTypeName::DLatch
    }

    fn sink_labels(&self) -> &'static [Label] {
        SINK_LABELS
    }

    fn source_labels(&self) -> &[Label] {
        SOURCE_LABELS
    }
}