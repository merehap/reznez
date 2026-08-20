use crate::circuit::component_type::ComponentType;
use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;

const SOURCE_LABELS: &'static [Label] = &[Label::new('Q'), Label::new('N')];
const SINK_LABELS: &'static [Label] = &[Label::new('E'), Label::new('D')];

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
        let new_state = match (&inputs[Label::E as usize], &inputs[Label::D as usize]) {
            (Resolved(true) , d      ) => *d,
            (Resolved(false), _      ) => Resolved(self.state),
            (Unknown        , _      ) => panic!("Don't know what to do if DLatch is neither enabled nor disabled."),
        };

        if let SignalState::Resolved(state) = new_state {
            self.state = state;
        }

        outputs[Label::Q as usize] = new_state;
        outputs[Label::N as usize] = new_state.negate();
    }

    fn sink_labels(&self) -> &'static [Label] {
        SINK_LABELS
    }

    fn source_labels(&self) -> &[Label] {
        SOURCE_LABELS
    }
}