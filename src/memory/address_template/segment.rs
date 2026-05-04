use crate::mapper::PrgBankRegisterId;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Segment {
    label: Label,
    raw_constant: u16,
    constant_mask: u16,
    magnitude: u8,
    ignored_low_count: u8,

    raw_value: u16,
}

impl Segment {
    pub const EMPTY_UNLABELED: Self = Self::constant_inner_bank(0, 0);
    const SEGMENT_ATOM_LENGTH: usize = 7;

    pub const fn labeled(label: Label, magnitude: u8) -> Self {
        Self {
            label,
            raw_constant: 0,
            constant_mask: 0b0000_0000_0000_0000,
            magnitude,
            ignored_low_count: 0,

            raw_value: 0,
        }
    }

    pub const fn constant_inner_bank(raw_constant: u16, magnitude: u8) -> Self {
        Self {
            label: Label::InnerBankSegment(None),
            raw_constant,
            constant_mask: 0b1111_1111_1111_1111,
            magnitude,
            ignored_low_count: 0,

            raw_value: 0,
        }
    }

    pub const fn parse(mut bytes: &[u8]) -> Result<(Segment, &[u8]), &'static str> {
        if bytes.len() < Self::SEGMENT_ATOM_LENGTH {
            return Err("A segment must have at least one atom in it.");
        }

        // TODO: Move this case into the loop. Just have to extract the magnitude before the loop.
        let atom = Atom::from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
        if atom.subscript >= 16 {
            return Err("The maximum allowed value for a subscript is 15.");
        }

        let magnitude = atom.subscript + 1;
        bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];

        let (mut raw_constant, mut constant_mask) = if let Some(raw_constant) = atom.raw_constant {
            ((raw_constant as u16) << atom.subscript, 1 << atom.subscript)
        } else {
            (0, 0)
        };

        let mut subscript = atom.subscript;
        let mut label = atom.label;
        let mut expected_subscript = atom.subscript;
        while !bytes.is_empty() && subscript > 0 {
            expected_subscript -= 1;

            let next_atom = Atom::from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
            if next_atom.subscript != expected_subscript {
                if let (Some(new), Some(old)) = (next_atom.label.to_char(), label.to_char()) && new == old {
                    return Err("Contiguous segment elements must have decrementing subscripts.");
                }

                // The subscript isn't one less than the previous subscript, so we've found the end of the segment.
                break;
            }

            if let (Some(new), Some(old)) = (next_atom.label.to_char(), label.to_char()) && new != old {
                // If we switched labels, then the new segment is about to start, so wrap up the current segment.
                break;
            }

            // We're still on the same label, so we can update all the state.
            subscript = next_atom.subscript;
            bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];
            if matches!(label, Label::InnerBankSegment(None)) {
                label = next_atom.label;
            }

            if let Some(new_raw_constant) = next_atom.raw_constant {
                raw_constant |= (new_raw_constant as u16) << subscript;
                constant_mask |= 1 << subscript;
            }
        }

        let ignored_low_count = subscript;
        let full_mask: u32 = ((1u32 << magnitude) - 1) & !((1 << ignored_low_count) - 1);
        assert!(full_mask <= u16::MAX as u32);
        if constant_mask != full_mask as u16 {
            assert!(label.to_char().is_some(), "How is there an incomplete constant mask, but no label to fill in the blanks?");
        }

        let segment = Self { label, raw_constant, constant_mask, magnitude, ignored_low_count, raw_value: 0 };
        Ok((segment, bytes))
    }

    pub fn set_raw_value(&mut self, value: u16) {
        self.raw_value = value;
    }

    pub const fn label(&self) -> Label {
        self.label
    }

    pub const fn register_id(&self) -> Option<PrgBankRegisterId> {
        self.label().register_id()
    }

    pub fn label_at(&self, index: u8) -> LabelOrConstant {
        if let Some(c) = self.label.to_char() {
            LabelOrConstant::Label(c)
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

    pub fn resolve_shifted(&self, shift: u8) -> u32 {
        u32::from(self.resolve() >> self.ignored_low_count) << shift
    }

    pub fn resolve(&self) -> u16 {
        self.resolve_with(self.raw_value)
    }

    // TODO: Cache mask.
    pub fn resolve_with(&self, raw_value: u16) -> u16 {
        let max_value = (1 << self.magnitude) - 1;
        let ignored_low = (1 << self.ignored_low_count) - 1;
        let mask = max_value & !ignored_low;

        let mut value = self.raw_constant & self.constant_mask;
        if self.label().to_char().is_some() {
            value |= raw_value & !self.constant_mask;
        }

        value & mask
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
    Label(char),
    Zero,
    One,
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
struct Atom {
    label: Label,
    raw_constant: Option<bool>,
    subscript: u8,
}

impl Atom {
    const fn from_bytes(bytes: &[u8]) -> Result<Self, &'static str> {
        assert!(bytes.len() == 7);
        let c = bytes[0] as char;
        let tens_digit = subscript_utf8_bytes_to_digit(bytes[1], bytes[2], bytes[3])?;
        let ones_digit = subscript_utf8_bytes_to_digit(bytes[4], bytes[5], bytes[6])?;
        let subscript = 10 * tens_digit + ones_digit;

        match c {
            '0' => Ok(Self {
                label: Label::InnerBankSegment(None),
                raw_constant: Some(false),
                subscript,
            }),
            '1' => Ok(Self {
                label: Label::InnerBankSegment(None),
                raw_constant: Some(true),
                subscript,
            }),
            c => {
                match Label::new(c) {
                    Ok(label) => Ok(Self { label, raw_constant: None, subscript }),
                    Err(err) => const_panic::concat_panic!("Bad label char: ", c, ". ", err),
                }
            }
        }
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum Label {
    OuterBank,
    InnerBankSegment(Option<PrgBankRegisterId>),
    AddressBus,
}

impl Label {
    pub const fn new(c: char) -> Result<Self, &'static str> {
        match c {
            'o' => Ok(Self::OuterBank),
            'a' => Ok(Self::AddressBus),
            'p'..='z' => PrgBankRegisterId::from_char(c)
                .map(Some)
                .map(Label::InnerBankSegment)
                .ok_or("Bad label char"),
            'A' | 'O'..='Z' => Err("Template labels must be lower-case."),
            _ => Err("Bad template label char"),
        }
    }

    pub const fn to_char(self) -> Option<char> {
        match self {
            Self::OuterBank => Some('o'),
            Self::InnerBankSegment(None) => None,
            Self::InnerBankSegment(Some(reg_id)) => Some(reg_id.to_char()),
            Self::AddressBus => Some('a'),
        }
    }

    pub const fn register_id(self) -> Option<PrgBankRegisterId> {
        if let Self::InnerBankSegment(reg_id) = self {
            reg_id
        } else {
            None
        }
    }
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
    fn segment_atom_from_bytes() {
        let Atom { label, raw_constant, subscript } = Atom::from_bytes(&[0x61, 0xE2, 0x82, 0x81, 0xE2, 0x82, 0x82]).unwrap();
        assert_eq!(label, Label::new('a').unwrap());
        assert_eq!(raw_constant, None);
        assert_eq!(subscript, 12);
    }
}