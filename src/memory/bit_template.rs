use itertools::Itertools;

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
        self.segments.as_iter()
            .rev()
            .map(Segment::formatted)
            .join("")
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

        Ok(BitTemplate::right_to_left(segments))
    }

    pub const fn increase_segment_magnitude(&mut self, segment_index: u8, new_magnitude: u8) {
        assert!(segment_index < self.segments.len());

        let mut ignored_low_count = self.segments.get_mut(segment_index).increase_magnitude_to(new_magnitude);

        let mut index = segment_index + 1;
        while index < self.segments.len() {
            ignored_low_count = self.segments.get_mut(index).increase_ignored_low_count(ignored_low_count);
            assert!(ignored_low_count == 0, "Overshift occurred. Outer bank bits shouldn't be lost to large inner bank sizes.");
            index += 1;
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Segment {
    label: Label,
    magnitude: u8,
    ignored_low_count: u8,
}

impl Segment {
    const SEGMENT_ATOM_LENGTH: usize = 7;

    pub const fn named(name: char, magnitude: u8) -> Self {
        Self {
            label: Label::Name(name),
            magnitude,
            ignored_low_count: 0,
        }
    }

    pub const fn constant(value: u16, lowest_subscript: u8, magnitude: u8, ignored_low_count: u8) -> Self {
        Self {
            label: Label::Constant { value, lowest_subscript },
            magnitude,
            ignored_low_count,
        }
    }

    pub const fn parse(bytes: &[u8]) -> Result<(Self, &[u8]), &'static str> {
        if bytes.len() < Self::SEGMENT_ATOM_LENGTH {
            return Err("A segment must have at least one atom in it.");
        }

        let (mut label, magnitude) = Segment::atom_from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
        let mut index = 0;
        while index < bytes.len() {
            let extended = label.extend(bytes[index] as char);
            if !extended {
                break;
            }

            index += Self::SEGMENT_ATOM_LENGTH;
        }

        let segment = Segment { label, magnitude, ignored_low_count: 0 };
        let unparsed_remainder = &bytes[index..];
        Ok((segment, unparsed_remainder))
    }

    const fn atom_from_bytes(bytes: &[u8]) -> Result<(Label, u8), &'static str> {
        assert!(bytes.len() == 7);

        let label = Label::from_char(bytes[0] as char);
        let tens_digit = subscript_utf8_bytes_to_digit(bytes[1], bytes[2], bytes[3])?;
        let ones_digit = subscript_utf8_bytes_to_digit(bytes[4], bytes[5], bytes[6])?;
        let subscript = 10 * tens_digit + ones_digit;
        let magnitude = subscript + 1;
        Ok((label, magnitude))
    }

    pub const fn magnitude(&self) -> u8 {
        self.magnitude
    }

    pub const fn width(&self) -> u8 {
        self.magnitude.saturating_sub(self.ignored_low_count)
    }

    pub const fn ignored_low_count(&self) -> u8 {
        self.ignored_low_count
    }

    // TODO: Cache mask.
    pub fn resolve(&self, raw_value: u16) -> u16 {
        match &self.label {
            Label::Name(_) => {
                let max_value = (1 << self.magnitude) - 1;
                let ignored_low = (1 << self.ignored_low_count) - 1;
                let mask = max_value & !ignored_low;
                raw_value & mask
            }
            Label::Constant { value, .. } => {
                *value & !((1 << self.ignored_low_count) - 1)
            }
        }
    }

    pub fn resolve_shifted(&self, raw_value: u16, shift: u8) -> u32 {
        u32::from(self.resolve(raw_value)) << shift
    }

    pub fn formatted(self) -> String {
        match self.label {
            Label::Name(_) => {
                (self.ignored_low_count..self.magnitude).rev()
                    .map(|i| [self.label_text_at(i - self.ignored_low_count), subscript_digit_to_string(i)].concat())
                    .join("")
            }
            Label::Constant { lowest_subscript, .. } => {
                let lowest_visible_subscript = lowest_subscript + self.ignored_low_count;
                (lowest_visible_subscript..lowest_visible_subscript + self.width()).rev()
                    .map(|i| [self.label_text_at(i - lowest_subscript), subscript_digit_to_string(i)].concat())
                    .join("")
            }
        }
    }

    const fn increase_magnitude_to(&mut self, magnitude: u8) -> u8 {
        let increase_amount = magnitude.strict_sub(self.magnitude);
        self.magnitude = magnitude;
        increase_amount
    }

    const fn increase_ignored_low_count(&mut self, increase_amount: u8) -> u8 {
        let already_empty = self.width() == 0;
        self.ignored_low_count = self.ignored_low_count.strict_add(increase_amount);

        // Return the overshift (usually zero)
        if already_empty {
            increase_amount
        } else {
            self.ignored_low_count.saturating_sub(self.magnitude)
        }
    }

    fn label_text_at(&self, index: u8) -> String {
        match &self.label {
            Label::Name(name) => (*name).to_string(),
            Label::Constant { value, .. } => ((value >> index) & 1).to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Label {
    Name(char),
    Constant { value: u16, lowest_subscript: u8 },
}

impl Label {
    const fn from_char(c: char) -> Self {
        match c {
            '0' => Self::Constant { value: 0, lowest_subscript: 0 },
            '1' => Self::Constant { value: 1, lowest_subscript: 0 },
            c => {
                assert!(c.is_ascii_alphabetic());
                Self::Name(c)
            }
        }
    }

    const fn extend(&mut self, c: char) -> bool {
        let new = Self::from_char(c);
        use Label::*;
        match (new, *self) {
            // If both the new label and old label are Constants, then append the new bit onto the old constant.
            (Constant { value: old_value, .. }, Constant { value: new_value, lowest_subscript }) => {
                *self = Constant {
                    value: (old_value << 1) | new_value,
                    lowest_subscript
                };
                true
            }
            // If the name stayed the same, then the current Label is "extended" by doing nothing.
            (Name(new_name), Name(old_name)) if new_name == old_name => {
                true
            }
            // If the name changed, or it's a new variant, then a new Label must be made (the old one can't be extended further).
            (Name(_), Name(_) | Constant {..}) | (Constant {..}, Name(_)) => {
                false
            }
        }
    }
}

// TODO: Remove this? Why have a default at all?
impl Default for Label {
    fn default() -> Self {
        Self::Name('a')
    }
}

fn subscript_digit_to_string(value: u8) -> String {
    let subscript_of = |c| {
        match c {
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
        }
    };

    format!("{value:02}")
        .to_string()
        .chars()
        .map(subscript_of)
        .collect()
}

const fn subscript_utf8_bytes_to_digit(top: u8, mid: u8, bot: u8) -> Result<u8, &'static str> {
    // Standard library UTF8 decoding isn't available in const contexts, so re-implement a little bit here.
    Ok(match (top, mid, bot) {
        (0xE2, 0x82, 0x80) => 0, // '₀'
        (0xE2, 0x82, 0x81) => 1, // '₁'
        (0xE2, 0x82, 0x82) => 2, // '₂'
        (0xE2, 0x82, 0x83) => 3, // '₃'
        (0xE2, 0x82, 0x84) => 4, // '₄'
        (0xE2, 0x82, 0x85) => 5, // '₅'
        (0xE2, 0x82, 0x86) => 6, // '₆'
        (0xE2, 0x82, 0x87) => 7, // '₇'
        (0xE2, 0x82, 0x88) => 8, // '₈'
        (0xE2, 0x82, 0x89) => 9, // '₉'
        _ => return Err("Non-subscript character specified."),
    })
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn subscript_digit() {
        assert_eq!(subscript_utf8_bytes_to_digit(0xE2, 0x82, 0x80), Ok(0));
    }

    #[test]
    fn segment_from_bytes() {
        let (label, magnitude) = Segment::atom_from_bytes(&[0x61, 0xE2, 0x82, 0x81, 0xE2, 0x82, 0x82]).unwrap();
        assert_eq!(label, Label::Name('a'));
        assert_eq!(magnitude, 13);
    }

    #[test]
    fn segment_from_formatted() {
        let text = "o₀₀i₀₀a₀₀";
        let bit_template = BitTemplate::from_formatted(text).unwrap();
        let segments: Vec<Segment> = bit_template.segments.as_iter().collect();
        assert_eq!(segments[0].label, Label::Name('a'));
        assert_eq!(segments[1].label, Label::Name('i'));
        assert_eq!(segments[2].label, Label::Name('o'));
        assert_eq!(bit_template.formatted(), text);
    }
}