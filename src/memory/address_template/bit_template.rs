use itertools::Itertools;

use crate::memory::address_template::segment::{Label, Segment};
use crate::util::const_vec::ConstVec;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BitTemplate {
    // Segments are stored right-to-left, the reverse of how they are rendered.
    segments: ConstVec<Segment, 3>,
}

impl BitTemplate {
    pub const fn right_to_left(mut segments: ConstVec<Segment, 3>, forced_width: Option<u8>) -> Self {
        if let Some(forced_width) = forced_width {
            let mut width_accum = 0;
            let mut i = 0;
            while i < segments.len() {
                width_accum += segments.get(i).width();
                if width_accum > forced_width {
                    let overshoot = segments.get_mut(i).decrease_width_by(width_accum - forced_width);
                    assert!(overshoot == 0);
                    // If we decreased the length of the current segment, then the higher segments don't even exist.
                    segments.decrease_len_to(i + 1);
                    break;
                }

                i += 1;
            }
        }

        Self { segments }
    }

    pub const fn from_formatted(text: &str) -> Result<Self, &'static str> {
        const SEGMENT_ATOM_LENGTH: usize = 7;

        let mut bytes = text.as_bytes();
        if !bytes.len().is_multiple_of(SEGMENT_ATOM_LENGTH) {
            return Err("BitTemplate byte length must be a multiple of 7 (subscript chars are 3 bytes each).");
        }

        if bytes.len() < SEGMENT_ATOM_LENGTH {
            return Err("BitTemplate must have at least one segment (minimally a label and two subscript chars).");
        }

        let mut segments: ConstVec<Segment, 3> = ConstVec::new();

        while !bytes.is_empty() {
            let segment;
            (segment, bytes) = Segment::parse(bytes)?;
            segments.push_front(segment);
        }

        Ok(BitTemplate::right_to_left(segments, None))
    }

    pub fn shortened(&self, new_width: u8) -> Self {
        let mut result = *self;
        let mut shorten_amount = result.width().strict_sub(new_width);
        for i in (0..result.segments.len()).rev() {
            shorten_amount = result.segments.get_mut(i).decrease_width_by(shorten_amount);
        }

        assert_eq!(shorten_amount, 0);
        result
    }

    pub const fn segment_count(&self) -> u8 {
        self.segments.len()
    }

    pub const fn width(&self) -> u8 {
        let mut width = 0;
        let mut index = 0;
        while index < self.segments.len() {
            if let Some(segment) = self.segments.maybe_get(index) {
                width += segment.width();
            }

            index += 1;
        }

        width
    }

    pub const fn width_of(&self, label: char) -> u8 {
        if let Some(segment) = self.segment_with_label(label) {
            segment.width()
        } else {
            0
        }
    }

    pub const fn ignored_low_count_of(&self, label: char) -> u8 {
        if let Some(segment) = self.segment_with_label(label) {
            segment.ignored_low_count()
        } else {
            0
        }
    }

    pub const fn label_at(&self, segment_index: u8) -> Option<Label> {
        let segment = self.segments.maybe_get(segment_index)?;
        Some(segment.label)
    }

    pub fn resolve(&self, raw_values: &[u16]) -> u32 {
        let mut result = 0;
        let mut index = 0;
        for (segment, &raw_value) in self.segments.as_iter().zip(raw_values) {
            result += segment.resolve_shifted(raw_value, index);
            index += segment.width();
        }

        result
    }

    pub fn resolve_segment(&self, segment_index: u8, raw_value: u16) -> u16 {
        match self.segments.maybe_get(segment_index) {
            None => 0,
            Some(segment) => segment.resolve(raw_value),
        }
    }

    pub fn formatted(&self) -> String {
        self.segments
            .as_iter()
            .rev()
            .map(Segment::formatted)
            .join("")
    }

    const fn segment_with_label(&self, label: char) -> Option<&Segment> {
        let mut i = 0;
        while i < self.segment_count() {
            if let Label::Name(segment_label) = self.segments.get_ref(i).label && segment_label == label {
                return Some(self.segments.get_ref(i));
            }

            i += 1;
        }

        None
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::address_template::segment::{Segment, Label};

    #[test]
    fn template_from_formatted() {
        let text = "o₀₀i₀₀a₀₀";
        let bit_template = BitTemplate::from_formatted(text).unwrap();
        let segments: Vec<Segment> = bit_template.segments.as_iter().collect();
        assert_eq!(segments[0].label, Label::Name('a'));
        assert_eq!(segments[1].label, Label::Name('i'));
        assert_eq!(segments[2].label, Label::Name('o'));
        assert_eq!(bit_template.formatted(), text);
    }

    #[test]
    fn ignored_low_bits_from_formatted() {
        let text = "i₀₃i₀₂i₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::from_formatted(text).unwrap();
        let segments: Vec<Segment> = bit_template.segments.as_iter().collect();
        assert_eq!(
            segments[0],
            Segment {
                label: Label::Name('a'),
                magnitude: 15,
                ignored_low_count: 0
            }
        );
        assert_eq!(
            segments[1],
            Segment {
                label: Label::Name('i'),
                magnitude: 4,
                ignored_low_count: 1
            }
        );
        assert_eq!(bit_template.formatted(), text);
    }
}
