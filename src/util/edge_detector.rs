use std::marker::ConstParamTy_;

use crate::ppu::pattern_table_side::PatternTableSide;

pub struct EdgeDetector<V: ConstParamTy_> {
    target_value: V,

    previous_value: V,
    value: V,
}

impl EdgeDetector<PatternTableSide> {
    pub const fn pattern_table_side_detector(target_side: PatternTableSide) -> Self {
        Self {
            target_value: target_side,
            previous_value: PatternTableSide::Left,
            value: PatternTableSide::Left,
        }
    }
}

impl <V: PartialEq + Eq + Clone + Copy + Default + ConstParamTy_> EdgeDetector<V> {
    // Can't be const (yet) because it relies on default().
    pub fn new(target_value: V) -> Self {
        Self {
            target_value,

            previous_value: V::default(),
            value: V::default(),
        }
    }

    pub fn current_value(&self) -> V {
        self.value
    }

    pub fn target_value(&self) -> V {
        self.target_value
    }

    pub fn set_value(&mut self, value: V) {
        self.value = value;
    }

    pub fn detect(&mut self) -> bool {
        let edge_detected = self.value == self.target_value && self.previous_value != self.value;
        self.previous_value = self.value;
        edge_detected
    }

    pub fn set_value_then_detect(&mut self, value: V) -> bool {
        self.set_value(value);
        self.detect()
    }
}