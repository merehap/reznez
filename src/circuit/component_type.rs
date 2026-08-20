use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use crate::circuit::label::{LABEL_COUNT, Label};
use crate::circuit::signal_state::SignalState;

#[allow(private_bounds)]
pub trait ComponentType: ComponentTypeClone {
    fn calculate_output_signal_states(&mut self, input_signal_states: &[SignalState; LABEL_COUNT]) -> BTreeMap<Label, SignalState>;
    #[allow(dead_code)]
    fn input_labels(&self) -> BTreeSet<Label>;
    #[allow(dead_code)]
    fn output_labels(&self) -> BTreeSet<Label>;
}

impl Clone for Box<dyn ComponentType + Send + Sync> {
    fn clone(&self) -> Self {
        self.clone_box()
    }
}

trait ComponentTypeClone {
    fn clone_box(&self) -> Box<dyn ComponentType + Send + Sync>;
}

impl<T> ComponentTypeClone for T
where T: ComponentType + Clone + fmt::Debug + Send + Sync + 'static
{
    fn clone_box(&self) -> Box<dyn ComponentType + Send + Sync> {
        Box::new(self.clone())
    }
}