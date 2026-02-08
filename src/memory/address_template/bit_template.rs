use itertools::Itertools;

use crate::memory::address_template::segment::{Label, Segment};
use crate::util::const_vec::ConstVec;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BitTemplate {
    // Segments are stored right-to-left, the reverse of how they are rendered.
    segments: ConstVec<Segment, 3>,
}

impl BitTemplate {
    pub const fn right_to_left(segments: ConstVec<Segment, 3>) -> Self {
        Self { segments }
    }

    pub const fn width(&self) -> u8 {
        let mut width = 0;
        let mut index = 0;
        while index < self.segments.len() {
            width += self.segments.get(index).width();
            index += 1;
        }

        width
    }

    pub const fn width_of(&self, segment_index: u8) -> u8 {
        self.segments.get(segment_index).width()
    }

    pub const fn magnitude_of(&self, segment_index: u8) -> u8 {
        self.segments.get(segment_index).magnitude()
    }

    pub const fn ignored_low_count_of(&self, segment_index: u8) -> u8 {
        self.segments.get(segment_index).ignored_low_count()
    }

    pub const fn label_of(&self, segment_index: u8) -> Label {
        self.segments.get(segment_index).label
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
        self.segments.get(segment_index).resolve(raw_value)
    }

    pub fn formatted(&self) -> String {
        self.segments
            .as_iter()
            .rev()
            .map(Segment::formatted)
            .join("")
    }

    pub const fn from_formatted(text: &str) -> Result<Self, &'static str> {
        const SEGMENT_ATOM_LENGTH: usize = 7;

        let mut bytes = text.as_bytes();
        if !bytes.len().is_multiple_of(SEGMENT_ATOM_LENGTH) {
            return Err(
                "BitTemplate byte length must be a multiple of 7 (subscript chars are 3 bytes each).",
            );
        }

        if bytes.len() < SEGMENT_ATOM_LENGTH {
            return Err(
                "BitTemplate must have at least one segment (minimally a label and two subscript chars).",
            );
        }

        let mut segments: ConstVec<Segment, 3> = ConstVec::new();

        while !bytes.is_empty() {
            let segment;
            (segment, bytes) = Segment::parse(bytes)?;
            segments.push_front(segment);
        }

        Ok(BitTemplate::right_to_left(segments))
    }

    pub const fn increase_segment_magnitude(
        &mut self,
        segment_index: u8,
        new_magnitude: u8,
    ) {
        assert!(segment_index < self.segments.len());

        let mut ignored_low_count = self
            .segments
            .get_mut(segment_index)
            .increase_magnitude_to(new_magnitude);

        let mut index = segment_index + 1;
        while index < self.segments.len() {
            ignored_low_count = self
                .segments
                .get_mut(index)
                .increase_ignored_low_count(ignored_low_count);
            assert!(
                ignored_low_count == 0,
                "Overshift occurred. Outer bank bits shouldn't be lost to large inner bank sizes."
            );
            index += 1;
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::address_template::segment::Segment;

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
