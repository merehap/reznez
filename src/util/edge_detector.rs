use std::marker::ConstParamTy_;

use crate::ppu::pattern_table_side::PatternTableSide;

pub struct EdgeDetector<V: ConstParamTy_> {
    target_value: Option<V>,

    previous_value: V,
    value: V,
}

impl EdgeDetector<PatternTableSide> {
    pub const fn pattern_table_side_detector(target_side: PatternTableSide) -> Self {
        Self {
            target_value: Some(target_side),
            previous_value: PatternTableSide::Left,
            value: PatternTableSide::Left,
        }
    }
}

impl <V: PartialEq + Eq + Clone + Copy + Default + ConstParamTy_> EdgeDetector<V> {
    // Can't be const (yet) because it relies on default().
    pub fn target_value(target_value: V) -> Self {
        Self {
            target_value: Some(target_value),

            previous_value: V::default(),
            value: V::default(),
        }
    }

    pub fn any_edge() -> Self {
        Self {
            target_value: None,

            previous_value: V::default(),
            value: V::default(),
        }
    }

    pub fn starting_value(starting_value: V) -> Self {
        Self {
            target_value: None,

            previous_value: starting_value,
            value: starting_value,
        }
    }

    pub fn current_value(&self) -> V {
        self.value
    }

    pub fn matches_target(&self, value: V) -> bool {
        self.target_value.unwrap() == value
    }

    pub fn set_value(&mut self, value: V) {
        self.value = value;
    }

    pub fn detect(&mut self) -> bool {
        let target_value_hit = self.target_value
            .map(|target| self.value == target)
            // If there's no target, then target is considered always hit.
            .unwrap_or(true);
        let edge_detected = target_value_hit && self.previous_value != self.value;
        self.previous_value = self.value;
        edge_detected
    }

    pub fn set_value_then_detect(&mut self, value: V) -> bool {
        self.set_value(value);
        self.detect()
    }
}