use std::marker::ConstParamTy_;

use crate::circuit::component::{Component, ToIndex};
use crate::circuit::signal::Signal;
use crate::circuit::source::{PrimarySource, Source};

pub enum ComponentTypeName {
    Nor,
    DLatch,
    SRLatch,
}

#[derive(Clone)]
pub enum Factor<ComponentName: ConstParamTy_> {
    PrimarySource(PrimarySource),
    Nor { a: Signal<ComponentName>, b: Signal<ComponentName> },
    DLatchQ { e: Signal<ComponentName>, d: Signal<ComponentName> },
    DLatchN { e: Signal<ComponentName>, d: Signal<ComponentName> },
    SRLatchQ { s: Signal<ComponentName>, r: Signal<ComponentName> },
    SRLatchN { s: Signal<ComponentName>, r: Signal<ComponentName> },
}

pub struct FactorGraph<ComponentName: ConstParamTy_> {
    factors_with_source: Vec<(Factor<ComponentName>, Source<ComponentName>)>,
}

impl <ComponentName: ConstParamTy_ + Clone + Copy + ToIndex> FactorGraph<ComponentName> {
    pub fn new(primary_sources: &[PrimarySource]) -> Self {
        Self {
            factors_with_source: primary_sources.iter()
                .map(|primary_source| (Factor::PrimarySource(*primary_source), Source::Primary(*primary_source)))
                .collect()
        }
    }

    pub fn push_component(&mut self, signals: &[Signal<ComponentName>], component: &Component<ComponentName>) {
        self.factors_with_source.append(&mut component.factors(signals));
    }

    pub fn to_node_graph(self) -> NodeGraph {
        let signal_to_source_index = |signal: Signal<ComponentName>| -> Index {
            for (i, (_, source)) in self.factors_with_source.iter().enumerate() {
                if signal.source == source.clone() {
                    return i;
                }
            }

            panic!();
        };

        let factor_to_node = |factor: Factor<ComponentName>| -> Node {
            match factor {
                Factor::PrimarySource(primary_source) => Node::PrimarySource(primary_source),
                Factor::Nor { a, b } => Node::Nor(signal_to_source_index(a), signal_to_source_index(b)),
                Factor::DLatchQ { e, d } => Node::DLatchQ { e: signal_to_source_index(e), d: signal_to_source_index(d) },
                Factor::DLatchN { e, d } => Node::DLatchN { e: signal_to_source_index(e), d: signal_to_source_index(d) },
                Factor::SRLatchQ { s, r } => Node::SRLatchQ { s: signal_to_source_index(s), r: signal_to_source_index(r) },
                Factor::SRLatchN { s, r } => Node::SRLatchN { s: signal_to_source_index(s), r: signal_to_source_index(r) },
            }
        };

        NodeGraph {
            nodes: self.factors_with_source.iter()
                .map(|(factor, _)| factor_to_node(factor.clone()))
                .collect(),
        }
    }
}

#[derive(PartialEq, Eq, Clone)]
pub enum Node {
    Constant,

    PrimarySource(PrimarySource),
    Identity(Index),

    Not(Index),

    Or(Index, Index),
    Nor(Index, Index),
    NotFirstOr(Index, Index),
    NotFirstAnd(Index, Index),
    DLatchQ { e: Index, d: Index },
    DLatchN { e: Index, d: Index },
    SRLatchQ { s: Index, r: Index },
    SRLatchN { s: Index , r: Index },
}

pub type Index = usize;

#[derive(PartialEq, Eq, Clone)]
pub struct NodeGraph {
    nodes: Vec<Node>,
}

impl NodeGraph {
    pub fn propagate_constants(&mut self, states: &mut [bool]) {
        assert_eq!(states.len(), self.nodes.len());

        loop {
            let reduced = self.reduce(states);
            if !reduced {
                return;
            }
        }
    }

    fn reduce(&mut self, states: &mut [bool]) -> bool {
        let original = self.clone();

        use Node::*;
        for i in 0..self.nodes.len() {
            match self.nodes[i].clone() {
                Constant => {}
                PrimarySource(_) => {}
                Identity(index) => {
                    if self.nodes[index] == Constant {
                        states[i] = states[index];
                        self.nodes[i] = Constant;
                    }
                }
                Not(index) => {
                    if self.nodes[index] == Constant {
                        states[i] = !states[index];
                        self.nodes[i] = Constant;
                    }
                }
                Or(a, b) => {
                    match (&self.nodes[a], &self.nodes[b]) {
                        (Constant, Constant) => {
                            states[i] = states[a] || states[b];
                            self.nodes[i] = Constant;
                        }
                        (Constant, _) => {
                            if states[a] {
                                states[i] = true;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Identity(b);
                            }
                        }
                        (_, Constant) => {
                            if states[b] {
                               states[i] = true;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Identity(a);
                            }
                        }
                        (_, _) => {}
                    }
                }
                Nor(a, b) => {
                    match (&self.nodes[a], &self.nodes[b]) {
                        (Constant, Constant) => {
                            states[i] = !states[a] && !states[b];
                            self.nodes[i] = Identity(i);
                        }
                        (Constant, _) => {
                            if states[a] {
                                states[i] = false;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Not(b);
                            }
                        }
                        (_, Constant) => {
                            if states[b] {
                                states[i] = false;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Not(a);
                            }
                        }
                        (_, _) => {}
                    }
                }
                NotFirstOr(a, b) => {
                    match (&self.nodes[a], &self.nodes[b]) {
                        (Constant, Constant) => {
                            states[i] = !states[a] || states[b];
                            self.nodes[i] = Identity(i);
                        }
                        (Constant, _) => {
                            if states[a] {
                                self.nodes[i] = Identity(b);
                            } else {
                                states[i] = true;
                                self.nodes[i] = Constant;
                            }
                        }
                        (_, Constant) => {
                            if states[b] {
                                states[i] = true;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Not(a);
                            }
                        }
                        (_, _) => {}
                    }
                }
                NotFirstAnd(a, b) => {
                    match (&self.nodes[a], &self.nodes[b]) {
                        (Constant, Constant) => {
                            states[i] = !states[a] && states[b];
                            self.nodes[i] = Identity(i);
                        }
                        (Constant, _) => {
                            if states[a] {
                                states[i] = false;
                                self.nodes[i] = Constant;
                            } else {
                                self.nodes[i] = Identity(b);
                            }
                        }
                        (_, Constant) => {
                            if states[b] {
                                self.nodes[i] = Not(a);
                            } else {
                                states[i] = false;
                                self.nodes[i] = Constant;
                            }
                        }
                        (_, _) => {}
                    }
                }
                DLatchQ { e, d } => {
                    match (&self.nodes[e], &self.nodes[d]) {
                        (Constant, Constant) => {
                            states[i] = states[d];
                            self.nodes[i] = Constant;
                        }
                        (Constant, _) => {
                            if states[e] {
                                self.nodes[i] = Identity(d);
                            } else {
                                states[i] = states[d];
                                self.nodes[i] = Constant;
                            }
                        }
                        (_, Constant) => {
                            if states[d] {
                                self.nodes[i] = Or(e, i);
                            } else {
                                self.nodes[i] = NotFirstAnd(e, i);
                            }
                        }
                        (_, _) => {}
                    }
                }
                DLatchN { e, d } => {
                    match (&self.nodes[e], &self.nodes[d]) {
                        (Constant, Constant) => {
                            states[i] = !states[d];
                            self.nodes[i] = Constant;
                        }
                        (Constant, _) => {
                            if states[e] {
                                self.nodes[i] = Not(d);
                            } else {
                                states[i] = !states[d];
                                self.nodes[i] = Constant;
                            }
                        }
                        (_, Constant) => {
                            if states[d] {
                                self.nodes[i] = Nor(e, i);
                            } else {
                                self.nodes[i] = NotFirstOr(i, e);
                            }
                        }
                        (_, _) => {}
                    }
                }
                SRLatchQ { s, r } => {
                    match (&self.nodes[s], &self.nodes[r]) {
                        (Constant, Constant) => {
                            self.nodes[i] == Constant;
                            match (states[s], states[r]) {
                                (false, false) => { /* Keep the current state. */ }
                                (true , false) => states[i] = true,
                                (_    , true ) => states[i] = false,
                            }
                        }
                        (Constant, _) => {
                            if states[s] {
                                self.nodes[i] = Not(r);
                            } else {
                                self.nodes[i] = NotFirstAnd(r, i);
                            }
                        }
                        (_, Constant) => {
                            if states[r] {
                                self.nodes[i] = Constant;
                                states[i] = false;
                            } else {
                                self.nodes[i] = Or(s, i);
                            }
                        }
                        (_, _) => {}
                    }
                }
                SRLatchN { s, r } => {
                    match (&self.nodes[s], &self.nodes[r]) {
                        (Constant, Constant) => {
                            self.nodes[i] == Constant;
                            match (states[s], states[r]) {
                                (false, false) => { /* Keep the current state. */ }
                                (false, true ) => states[i] = true,
                                (true , _    ) => states[i] = false,
                            }
                        }
                        (Constant, _) => {
                            if states[s] {
                                self.nodes[i] = Identity(r);
                            } else {
                                self.nodes[i] = Or(r, i);
                            }
                        }
                        (_, Constant) => {
                            if states[r] {
                                self.nodes[i] = Not(s);
                            } else {
                                self.nodes[i] = NotFirstOr(s, i);
                            }
                        }
                        (_, _) => {}
                    }
                }
            }
        }

        let reduced = *self != original;
        reduced
    }
}