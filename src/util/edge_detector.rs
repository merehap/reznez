use std::marker::ConstParamTy_;

pub struct EdgeDetector<V: ConstParamTy_, const TARGET: V> {
    previous_value: V,
    value: V,
}

impl <V: PartialEq + Eq + Clone + Copy + Default + ConstParamTy_, const TARGET: V> EdgeDetector<V, TARGET> {
    pub fn new() -> Self {
        Self {
            previous_value: V::default(),
            value: V::default(),
        }
    }

    pub fn level(&self) -> V {
        self.value
    }

    pub fn set_value(&mut self, value: V) {
        self.value = value;
    }

    pub fn detect_edge(&mut self) -> bool {
        let edge_detected = self.value == TARGET && self.previous_value != self.value;
        self.previous_value = self.value;
        edge_detected
    }

    pub fn set_value_then_detect_edge(&mut self, value: V) -> bool {
        self.set_value(value);
        self.detect_edge()
    }
}