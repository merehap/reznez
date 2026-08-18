use std::collections::{BTreeSet, BTreeMap};
use std::sync::LazyLock;

use crate::circuit::component::{Component, ComponentName};
use crate::circuit::component_types::d_latch::DLatch;
use crate::circuit::component_types::nor::Nor;
use crate::circuit::component_types::sr_latch::SRLatch;
use crate::circuit::label::Label;
use crate::circuit::signal::Signal;
use crate::circuit::signal_state::SignalState;
use crate::circuit::sink::{PrimarySink, Sink, SinkTerminal};
use crate::circuit::source::{PrimarySource, Source, SourceTerminal};
use crate::memory::ppu::ppu_pinout::PpuPinout;

static READ_2007_COMPONENTS: LazyLock<ComponentGraph> = LazyLock::new(|| {
    let mut graph = ComponentGraphBuilder::new()
        .component(Component::new("DLatch_0", Box::new(DLatch::new(false))))
        .component(Component::new("DLatch_1", Box::new(DLatch::new(true))))
        .component(Component::new("DLatch_2", Box::new(DLatch::new(false))))
        .component(Component::new("DLatch_3", Box::new(DLatch::new(true))))
        .component(Component::new("DLatch_4", Box::new(DLatch::new(false))))
        // This latch storing "true" at startup ensures that calculation doesn't start until a Read2007 occurs.
        .component(Component::new("SRLatch", Box::new(SRLatch::new(true))))
        .component(Component::new("NOR_0", Box::new(Nor)))
        .component(Component::new("NOR_ALE", Box::new(Nor)))
        .component(Component::new("NOR_READ", Box::new(Nor)))
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockFirstHalf),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_0", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_2", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_4", 'E')),
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockSecondHalf),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_1", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_3", 'E')),
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Read2007),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_0", 'A'))
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Read2007),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("SRLatch", 'S'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("SRLatch", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_0", 'B'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_0", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_0", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_0", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_1", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_1", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_2", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_2", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_READ", 'B')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_2", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_3", 'D')),
                Sink::Internal(SinkTerminal::from_raw("NOR_ALE", 'A')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_3", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_4", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_3", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("SRLatch", 'R'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_4", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_ALE", 'B')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_4", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_READ", 'A')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_ALE", 'Q')),
            sinks: BTreeSet::from([
                Sink::Primary(PrimarySink::Ale)
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_READ", 'Q')),
            sinks: BTreeSet::from([
                Sink::Primary(PrimarySink::Read)
            ]),
        })
        .build();

    graph.compute_primary_sinks(&STARTUP_PRIMARY_SOURCE_STATES);

    graph
});

static WRITE_2007_COMPONENTS: LazyLock<ComponentGraph> = LazyLock::new(|| {
    let mut graph = ComponentGraphBuilder::new()
        .component(Component::new("DLatch_0", Box::new(DLatch::new(false))))
        .component(Component::new("DLatch_1", Box::new(DLatch::new(true))))
        .component(Component::new("DLatch_2", Box::new(DLatch::new(false))))
        .component(Component::new("DLatch_3", Box::new(DLatch::new(true))))
        .component(Component::new("DLatch_4", Box::new(DLatch::new(false))))
        // This latch storing "true" at startup ensures that calculation doesn't start until a Write2007 occurs.
        .component(Component::new("SRLatch", Box::new(SRLatch::new(true))))
        .component(Component::new("NOR_0", Box::new(Nor)))
        .component(Component::new("NOR_ALE", Box::new(Nor)))
        .component(Component::new("NOR_WRITE", Box::new(Nor)))
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockFirstHalf),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_0", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_2", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_4", 'E')),
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::PpuClockSecondHalf),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_1", 'E')),
                Sink::Internal(SinkTerminal::from_raw("DLatch_3", 'E')),
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Write2007),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_0", 'A'))
            ]),
        })
        .signal(Signal {
            source: Source::Primary(PrimarySource::Write2007),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("SRLatch", 'S'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("SRLatch", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_0", 'B'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_0", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_0", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_0", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_1", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_1", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_2", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_1", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_WRITE", 'B')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_2", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_3", 'D')),
                Sink::Internal(SinkTerminal::from_raw("NOR_ALE", 'A')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_3", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("DLatch_4", 'D'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_3", 'N')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("SRLatch", 'R'))
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_4", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_ALE", 'B')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("DLatch_3", 'Q')),
            sinks: BTreeSet::from([
                Sink::Internal(SinkTerminal::from_raw("NOR_WRITE", 'A')),
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_ALE", 'Q')),
            sinks: BTreeSet::from([
                Sink::Primary(PrimarySink::Ale)
            ]),
        })
        .signal(Signal {
            source: Source::Internal(SourceTerminal::from_raw("NOR_WRITE", 'Q')),
            sinks: BTreeSet::from([
                Sink::Primary(PrimarySink::Write)
            ]),
        })
        .build();

    graph.compute_primary_sinks(&STARTUP_PRIMARY_SOURCE_STATES);
    graph
});

const STARTUP_PRIMARY_SOURCE_STATES: [bool; std::mem::variant_count::<PrimarySource>()] = [
    false, // PpuClockFirstHalf
    true,  // PpuClockSecondHalf
    false, // Read2007
    false, // Write2007
];

pub struct CircuitBoard {
    primary_source_states: [bool; std::mem::variant_count::<PrimarySource>()],
    read_2007_components: ComponentGraph,
    write_2007_components: ComponentGraph,
}

impl CircuitBoard {
    pub fn new() -> Self {
        let mut circuit_board = Self {
            primary_source_states: STARTUP_PRIMARY_SOURCE_STATES,
            read_2007_components: READ_2007_COMPONENTS.clone(),
            write_2007_components: WRITE_2007_COMPONENTS.clone(),
        };
        // This way the first PPU tick will start the first half of the PPU dot.
        circuit_board.set_source(PrimarySource::PpuClockSecondHalf, true);

        circuit_board
    }

    pub fn source(&self, input: PrimarySource) -> bool {
        self.primary_source_states[input as usize]
    }

    pub fn set_source(&mut self, primary_source: PrimarySource, value: bool) {
        self.primary_source_states[primary_source as usize] = value;
    }

    pub fn calculate_read_2007_outputs(&mut self, ppu_pinout: &mut PpuPinout) {
        self.read_2007_components.compute_primary_sinks(&self.primary_source_states);

        ppu_pinout.ppu_data_ale = self.read_2007_components.primary_sinks[&PrimarySink::Ale];
        ppu_pinout.ppu_data_read = self.read_2007_components.primary_sinks[&PrimarySink::Read];
    }

    pub fn is_read_2007_circuit_in_base_state(&self) -> bool {
        !self.source(PrimarySource::Read2007) && self.read_2007_components.signal_states_equal(&READ_2007_COMPONENTS)
    }

    pub fn calculate_write_2007_outputs(&mut self, ppu_pinout: &mut PpuPinout) {
        self.write_2007_components.compute_primary_sinks(&self.primary_source_states);

        ppu_pinout.ppu_data_ale = self.write_2007_components.primary_sinks[&PrimarySink::Ale];
        ppu_pinout.ppu_data_write = self.write_2007_components.primary_sinks[&PrimarySink::Write];
    }

    pub fn is_write_2007_circuit_in_base_state(&self) -> bool {
        !self.source(PrimarySource::Write2007) && self.write_2007_components.signal_states_equal(&WRITE_2007_COMPONENTS)
    }
}

#[derive(Clone)]
pub struct ComponentGraph {
    primary_sinks: BTreeMap<PrimarySink, bool>,
    sink_label_and_signal_by_component_name: BTreeMap<ComponentName, (Component, Vec<(Label, Signal)>)>,
    signals_by_source_terminal: BTreeMap<SourceTerminal, BTreeSet<Signal>>,
    signal_states: BTreeMap<Signal, SignalState>,
}

impl ComponentGraph {
    // FIXME: This should fail to compile if called between two different types of ComponentGraphs.
    pub fn signal_states_equal(&self, other: &ComponentGraph) -> bool {
        assert_eq!(self.signal_states.len(), other.signal_states.len());
        for (signal, state) in &self.signal_states {
            if other.signal_states[&signal] != *state {
                return false
            }
        }

        true
    }

    pub fn compute_primary_sinks(&mut self, primary_sources: &[bool]) {
        for (_current_component_name, (_component, labeled_signals)) in &self.sink_label_and_signal_by_component_name {
            for (_, signal) in labeled_signals {
                if let Source::Primary(primary_source) = &signal.source {
                    let primary_source_value = primary_sources[*primary_source as usize];
                    self.signal_states.insert(signal.clone(), SignalState::Resolved(primary_source_value));
                }
            }
        }

        // TODO: Find a proper way to terminate this rather than just iterating 10 times and hoping for the best.
        for _ in 0..3 {
            for (component_name, (component, labeled_signals)) in &mut self.sink_label_and_signal_by_component_name {
                let mut input_signal_states = BTreeMap::new();
                for (label, signal) in labeled_signals.iter() {
                    let signal_state = self.signal_states[&signal];
                    input_signal_states.insert(*label, signal_state);
                }

                let output_signal_states = component.calculate_output_signal_states(input_signal_states);
                for (source_label, signal_state) in output_signal_states {
                    let source_terminal = SourceTerminal::new(component_name.clone(), source_label);
                    let Some(signals) = self.signals_by_source_terminal.get(&source_terminal) else {
                        continue;
                    };

                    for signal in signals {
                        for sink in &signal.sinks {
                            match sink {
                                Sink::Primary(primary_sink) => {
                                    if let SignalState::Resolved(resolved_state) = signal_state {
                                        self.primary_sinks.insert(*primary_sink, resolved_state);
                                    }
                                }
                                Sink::Internal(_) => {
                                    *self.signal_states.get_mut(&signal).unwrap() = signal_state;
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}

pub struct ComponentGraphBuilder  {
    components: BTreeMap<ComponentName, Component>,
    signals: BTreeSet<Signal>,
}

impl ComponentGraphBuilder {
    pub fn new() -> Self {
        Self {
            components: BTreeMap::new(),
            signals: BTreeSet::new(),
        }
    }

    pub fn component(mut self, component: Component) -> Self {
        self.components.insert(component.name(), component);
        self
    }

    pub fn signal(mut self, signal: Signal) -> Self {
        self.signals.insert(signal);
        self
    }

    pub fn build(mut self) -> ComponentGraph {
        let mut primary_sources = BTreeMap::new();
        for signal in &self.signals {
            if let Source::Primary(primary_source) = signal.source {
                primary_sources.insert(primary_source, false);
            }
        }

        let mut primary_sinks = BTreeMap::new();
        for signal in &self.signals {
            for sink in &signal.sinks {
                if let Sink::Primary(primary_sink) = sink {
                    primary_sinks.insert(*primary_sink, false);
                }
            }
        }

        let mut sink_label_and_signal_by_component_name: BTreeMap<ComponentName, (Component, Vec<(Label, Signal)>)> = BTreeMap::new();
        for (signal, sink_terminal) in self.signals.iter().flat_map(|signal| signal.clone().sink_terminals().into_iter().map(|sink_terminal| (signal.clone(), sink_terminal))) {
            if let Some((_, signals)) = sink_label_and_signal_by_component_name.get_mut(&sink_terminal.component_name()) {
                signals.push((sink_terminal.label(), signal));
            } else {
                let component = self.components.remove(&sink_terminal.component_name())
                    .expect(&format!("Signal refers to a component ({:?}) that doesn't exist.", sink_terminal.component_name()));
                let signals = vec![(sink_terminal.label(), signal)];
                sink_label_and_signal_by_component_name.insert(component.name(), (component, signals));
            }
        }

        let mut signals_by_source_terminal: BTreeMap<SourceTerminal, BTreeSet<Signal>> = BTreeMap::new();
        for signal in self.signals {
            match signal.source {
                Source::Primary(_) => {}
                Source::Internal(ref terminal) => {
                    if let Some(signals) = signals_by_source_terminal.get_mut(terminal) {
                        signals.insert(signal.clone());
                    } else {
                        signals_by_source_terminal.insert(terminal.clone(), BTreeSet::from([signal]));
                    }
                }
            }
        }

        let mut signal_states: BTreeMap<Signal, SignalState> = BTreeMap::new();
        for (_current_component_name, (_component, labeled_signals)) in &sink_label_and_signal_by_component_name {
            for (_, signal) in labeled_signals {
                signal_states.insert(signal.clone(), SignalState::Unknown);
            }
        }

        ComponentGraph {
            primary_sinks,
            sink_label_and_signal_by_component_name,
            signals_by_source_terminal,
            signal_states,
        }
    }
}