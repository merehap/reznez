use itertools::Itertools;

use crate::memory::address_template::segment::{Label, LabelOrConstant, Segment};
use crate::memory::register_ids::bank::RegisterId;
use crate::util::const_vec::ConstVec;

const MAX_SEGMENT_COUNT: usize = 4;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct BitTemplate<ID: const RegisterId> {
    // Segments are stored right-to-left, the reverse of how they are rendered.
    segments: ConstVec<Segment<ID>, MAX_SEGMENT_COUNT>,
}

impl <ID: const RegisterId> BitTemplate<ID> {
    pub const fn right_to_left(mut segments: ConstVec<Segment<ID>, MAX_SEGMENT_COUNT>, forced_width: Option<u8>) -> Self {
        // Chop high template bits off if width exceeds forced width.
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

        let mut segments: ConstVec<Segment<ID>, MAX_SEGMENT_COUNT> = ConstVec::new();
        while !bytes.is_empty() {
            let segment;
            (segment, bytes) = Segment::parse(bytes)?;
            segments.push_front(segment);
        }

        Ok(BitTemplate::right_to_left(segments, None))
    }

    pub const fn empty_segment_inserted(&self, index: u8) -> Self {
        let mut result = *self;
        result.segments.insert(index, Segment::EMPTY_UNLABELED);
        result
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

    pub const fn label_at(&self, segment_index: u8) -> Label<ID> {
        self.segments.get(segment_index).label()
    }

    pub fn constant_at(&self, segment_index: u8) -> Option<u16> {
        Some(self.segments.maybe_get(segment_index)?.constant())
    }

    pub const fn index_of_label(&self, target: char) -> Option<u8> {
        let mut i = 0;
        while i < self.segments.len() {
            if self.segments.get(i).label().to_char() == Some(target) {
                return Some(i);
            }

            i += 1;
        }

        None
    }

    pub const fn has_inner_bank(&self) -> bool {
        self.segment_count() > 1 && !matches!(self.segments.get(1).label(), Label::OuterBank)
    }

    pub fn resolve(&self, address_bus_value: u16) -> u32 {
        let mut segments = self.segments.as_iter();
        let address_bus_segment = segments.next().unwrap();

        let mut result: u32 = address_bus_segment.resolve_with(address_bus_value).into();
        let mut offset = address_bus_segment.width();
        for segment in segments {
            result += segment.resolve_shifted(offset);
            offset += segment.width();
        }

        result
    }

    // TODO: Change this to be resolve_inner_bank_number
    pub fn resolve_segment(&self, segment_index: u8) -> u16 {
        match self.segments.maybe_get(segment_index) {
            None => 0,
            Some(segment) => segment.resolve(),
        }
    }

    pub fn formatted(&self) -> String {
        let mut atoms: Vec<(char, u8)> = Vec::new();
        for segment in self.segments.as_iter() {
            for (si, subscript) in segment.subscripts().iter().enumerate() {
                let atom = match segment.label_at(si as u8) {
                    LabelOrConstant::Label(c) => (c, *subscript),
                    LabelOrConstant::Zero => ('0', *subscript),
                    LabelOrConstant::One => ('1', *subscript),
                };
                atoms.push(atom);
            }
        }

        atoms.into_iter()
            .rev()
            .map(|(label, subscript)| [ label.to_string(), subscript_byte_to_string(subscript)].concat())
            .join("")
    }

    pub fn set_raw_value_at(&mut self, segment_index: u8, raw_value: u16) {
        self.segments.get_mut(segment_index).set_raw_value(raw_value);
    }

    pub fn segments_mut(&mut self) -> impl DoubleEndedIterator<Item = &mut Segment<ID>> {
        self.segments.iter_mut()
    }
}

fn subscript_byte_to_string(value: u8) -> String {
    let subscript_of = |c| match c {
        '0' => '₀',
        '1' => '₁',
        '2' => '₂',
        '3' => '₃',
        '4' => '₄',
        '5' => '₅',
        '6' => '₆',
        '7' => '₇',
        '8' => '₈',
        '9' => '₉',
        _ => unreachable!(),
    };

    format!("{value:02}")
        .to_string()
        .chars()
        .map(subscript_of)
        .collect()
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::memory::address_template::segment::{Segment, Label};
    use crate::memory::register_ids::bank::PrgBankRegisterId;

    #[test]
    fn template_from_formatted() {
        let text = "o₀₀p₀₀a₀₀";
        let bit_template = BitTemplate::from_formatted(text).unwrap();
        let segments: Vec<Segment<PrgBankRegisterId>> = bit_template.segments.as_iter().collect();
        assert_eq!(segments[0].label(), Label::new('a').unwrap());
        assert_eq!(segments[1].label(), Label::new('p').unwrap());
        assert_eq!(segments[2].label(), Label::new('o').unwrap());
        assert_eq!(bit_template.formatted(), text);
    }

    #[test]
    fn ignored_low_bits_from_formatted() {
        let text = "p₀₃p₀₂p₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::from_formatted(text).unwrap();
        let segments: Vec<Segment<PrgBankRegisterId>> = bit_template.segments.as_iter().collect();
        assert_eq!(segments[0].label(), Label::new('a').unwrap());
        assert_eq!(segments[0].magnitude(), 15);
        assert_eq!(segments[0].ignored_low_count(), 0);

        assert_eq!(segments[1].label(), Label::new('p').unwrap());
        assert_eq!(segments[1].magnitude(), 4);
        assert_eq!(segments[1].ignored_low_count(), 1);

        assert_eq!(bit_template.formatted(), text);
    }

    #[test]
    fn contiguous_constants_are_part_of_segment() {
        let text = "o₀₀p₀₃1₀₂p₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::<PrgBankRegisterId>::from_formatted(text).unwrap();
        assert_eq!(bit_template.segments.len(), 3);
    }

    #[test]
    fn discontiguous_constants_are_separate_segments() {
        let text = "1₀₂p₀₃p₀₂p₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::<PrgBankRegisterId>::from_formatted(text).unwrap();
        assert_eq!(bit_template.segments.len(), 3);
    }

    #[test]
    fn constant() {
        let text = "o₀₀1₀₃1₀₂0₀₁1₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::<PrgBankRegisterId>::from_formatted(text).unwrap();
        let value = bit_template.segments.get(1).resolve();
        assert_eq!(value, 0b1101);
    }

    #[test]
    fn shifted_constant() {
        let text = "o₀₀1₁₃1₁₂0₁₁1₁₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::<PrgBankRegisterId>::from_formatted(text).unwrap();
        assert_eq!(bit_template.segments.len(), 3);
        let value = bit_template.segments.get(1).resolve();
        assert_eq!(value, 0b1101_0000000000);
    }

    #[test]
    fn embedded_constant() {
        let text = "o₀₀p₀₃1₀₂p₀₁a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let bit_template = BitTemplate::<PrgBankRegisterId>::from_formatted(text).unwrap();
        let value = bit_template.segments.get(1).resolve();
        assert_eq!(value, 0b0100);
    }
}