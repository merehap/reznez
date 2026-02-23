#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Segment {
    label: Option<Label>,
    raw_constant: u16,
    constant_mask: u16,
    magnitude: u8,
    ignored_low_count: u8,
}

impl Segment {
    const SEGMENT_ATOM_LENGTH: usize = 7;

    pub const fn labeled(name: char, magnitude: u8) -> Self {
        Self {
            label: Some(Label(name)),
            raw_constant: 0,
            constant_mask: 0b0000_0000_0000_0000,
            magnitude,
            ignored_low_count: 0,
        }
    }

    pub const fn unlabeled(raw_constant: u16, magnitude: u8) -> Self {
        Self {
            label: None,
            raw_constant,
            constant_mask: 0b1111_1111_1111_1111,
            magnitude,
            ignored_low_count: 0,
        }
    }

    pub const fn parse(mut bytes: &[u8]) -> Result<(Segment, &[u8]), &'static str> {
        if bytes.len() < Self::SEGMENT_ATOM_LENGTH {
            return Err("A segment must have at least one atom in it.");
        }

        // TODO: Move this case into the loop. Just have to extract the magnitude before the loop.
        let (label_parse, mut subscript) = Self::atom_from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
        if subscript >= 16 {
            return Err("The maximum allowed value for a subscript is 15.");
        }

        let magnitude = subscript + 1;
        bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];

        let (mut label, mut raw_constant, mut constant_mask) = match label_parse {
            LabelParse::Label(label) => (Some(label), 0, 0),
            LabelParse::Constant { raw_constant } => (None, raw_constant << subscript, 1 << subscript),
        };

        let mut expected_subscript = subscript;
        while !bytes.is_empty() && subscript > 0 {
            expected_subscript -= 1;

            let (new_label_parse, new_subscript) = Self::atom_from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
            let new_label = new_label_parse.to_label();
            if new_subscript != expected_subscript {
                if let (Some(new), Some(old)) = (new_label, label) && new.to_char() == old.to_char() {
                    return Err("Contiguous segment elements must have decrementing subscripts.");
                }

                // The subscript isn't one less than the previous subscript, so we've found the end of the segment.
                break;
            }

            if let (Some(new), Some(old)) = (new_label, label) && new.to_char() != old.to_char() {
                // If we switched labels, then the new segment is about to start, so wrap up the current segment.
                break;
            }

            // We're still on the same label, so we can update all the state.
            subscript = new_subscript;
            bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];
            if label.is_none() {
                label = new_label;
            }

            if let LabelParse::Constant { raw_constant: new_raw_constant } = new_label_parse {
                raw_constant |= new_raw_constant << subscript;
                constant_mask |= 1 << subscript;
            }
        }

        let ignored_low_count = subscript;
        let full_mask: u32 = ((1u32 << magnitude) - 1) & !((1 << ignored_low_count) - 1);
        assert!(full_mask <= u16::MAX as u32);
        if constant_mask != full_mask as u16 {
            assert!(label.is_some(), "How is there an incomplete constant mask, but no label to fill in the blanks?");
        }

        let segment = Self { label, raw_constant, constant_mask, magnitude, ignored_low_count };
        Ok((segment, bytes))
    }

    const fn atom_from_bytes(bytes: &[u8]) -> Result<(LabelParse, u8), &'static str> {
        assert!(bytes.len() == 7);

        let label_parse = LabelParse::from_char(bytes[0] as char);
        let tens_digit = subscript_utf8_bytes_to_digit(bytes[1], bytes[2], bytes[3])?;
        let ones_digit = subscript_utf8_bytes_to_digit(bytes[4], bytes[5], bytes[6])?;
        let subscript = 10 * tens_digit + ones_digit;
        Ok((label_parse, subscript))
    }

    pub const fn label(&self) -> Option<Label> {
        self.label
    }

    pub fn label_at(&self, index: u8) -> LabelOrConstant {
        if let Some(label) = self.label {
            LabelOrConstant::Label(label)
        } else if (self.constant() >> index) & 1 == 1 {
            LabelOrConstant::One
        } else {
            LabelOrConstant::Zero
        }
    }

    pub fn constant(&self) -> u16 {
        let mask = ((1 << self.magnitude()) - 1) & !((1 << self.ignored_low_count()) - 1);
        self.raw_constant & mask
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
    pub fn resolve(&self, raw_label_value: u16) -> u16 {
        let max_value = (1 << self.magnitude) - 1;
        let ignored_low = (1 << self.ignored_low_count) - 1;
        let mask = max_value & !ignored_low;

        let mut raw_value = self.raw_constant & self.constant_mask;
        if self.label().is_some() {
            raw_value |= raw_label_value & !self.constant_mask;
        }

        raw_value & mask
    }

    pub fn resolve_shifted(&self, raw_value: u16, shift: u8) -> u32 {
        u32::from(self.resolve(raw_value) >> self.ignored_low_count) << shift
    }

    pub fn subscripts(&self) -> Vec<u8> {
        (self.ignored_low_count..self.magnitude).collect()
    }

    pub const fn increase_magnitude_to(&mut self, magnitude: u8) -> u8 {
        let increase_amount = magnitude.strict_sub(self.magnitude);
        self.magnitude = magnitude;
        increase_amount
    }

    pub const fn increase_ignored_low_count(&mut self, increase_amount: u8) -> u8 {
        let already_empty = self.width() == 0;
        self.ignored_low_count = self.ignored_low_count.strict_add(increase_amount);

        // Return the overshift (usually zero)
        if already_empty {
            increase_amount
        } else {
            self.ignored_low_count.saturating_sub(self.magnitude)
        }
    }

    pub const fn decrease_width_by(&mut self, mut amount: u8) -> u8 {
        if self.width() >= amount {
            self.magnitude -= amount;
            amount = 0;
        } else {
            self.magnitude = 0;
            amount -= self.width();
        }

        amount
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum LabelOrConstant {
    Label(Label),
    Zero,
    One,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum LabelParse {
    Label(Label),
    Constant { raw_constant: u16 },
}

impl LabelParse {
    const fn from_char(c: char) -> Self {
        match c {
            '0' => Self::Constant { raw_constant: 0 },
            '1' => Self::Constant { raw_constant: 1 },
            c => {
                match Label::new(c) {
                    Ok(label) => Self::Label(label),
                    Err(err) => const_panic::concat_panic!("Bad label char: ", c, ". ", err),
                }
            }
        }
    }

    const fn to_label(self) -> Option<Label> {
        if let Self::Label(l) = self {
            Some(l)
        } else {
            None
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Label(char);

impl Label {
    pub const fn new(c: char) -> Result<Self, &'static str> {
        match c {
            'a' | 'o'..='z' => Ok(Self(c)),
            'A' | 'O'..='Z' => Err("Template labels must be lower-case."),
            _ => Err("Bad template label char"),
        }
    }

    pub const fn to_char(self) -> char {
        self.0
    }
}

const fn subscript_utf8_bytes_to_digit(
    top: u8,
    mid: u8,
    bot: u8,
) -> Result<u8, &'static str> {
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
    fn segment_atom_from_bytes() {
        let (label, subscript) = Segment::atom_from_bytes(&[0x61, 0xE2, 0x82, 0x81, 0xE2, 0x82, 0x82]).unwrap();
        assert_eq!(label, LabelParse::Label(Label::new('a').unwrap()));
        assert_eq!(subscript, 12);
    }
}