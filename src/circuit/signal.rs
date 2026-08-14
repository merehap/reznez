use std::collections::BTreeSet;

use crate::circuit::sink::{Sink, SinkTerminal};
use crate::circuit::source::Source;

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Debug)]
pub struct Signal {
    pub source: Source,
    pub sinks: BTreeSet<Sink>,
}

impl Signal {
    pub fn sink_terminals(&self) -> BTreeSet<SinkTerminal> {
        self.sinks.iter()
            .filter_map(|sink| {
                match sink {
                    Sink::Primary(_) => None,
                    Sink::Internal(terminal) => Some(terminal.clone()),
                }
            })
            .collect()
    }
}
