use itertools::Itertools;

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct Segment {
    pub label: Label,
    pub magnitude: u8,
    pub ignored_low_count: u8,
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

    pub const fn constant(
        value: u16,
        lowest_subscript: u8,
        magnitude: u8,
        ignored_low_count: u8,
    ) -> Self {
        Self {
            label: Label::Constant { value, lowest_subscript },
            magnitude,
            ignored_low_count,
        }
    }

    pub const fn parse(mut bytes: &[u8]) -> Result<(Self, &[u8]), &'static str> {
        if bytes.len() < Self::SEGMENT_ATOM_LENGTH {
            return Err("A segment must have at least one atom in it.");
        }

        // TODO: Move this case into the loop. Just have to extract the magnitude before the loop.
        let (mut label, mut subscript) =
            Self::atom_from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
        let magnitude = subscript + 1;
        bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];

        let mut expected_subscript = subscript;
        while !bytes.is_empty() && subscript > 0 {
            let (new_label, new_subscript) =
                Self::atom_from_bytes(&bytes[0..Self::SEGMENT_ATOM_LENGTH])?;
            use Label::*;
            match (new_label, label) {
                // If both the new label and old label are Constants, then append the new bit onto the old constant.
                (
                    Constant { value: old_value, .. },
                    Constant { value: new_value, lowest_subscript },
                ) => {
                    label = Constant {
                        value: (old_value << 1) | new_value,
                        lowest_subscript,
                    };
                    bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];
                    subscript = new_subscript;
                }
                // If the name stayed the same, then the current Label is "extended" by doing nothing.
                (Name(new_name), Name(old_name)) if new_name == old_name => {
                    bytes = &bytes[Self::SEGMENT_ATOM_LENGTH..];
                    subscript = new_subscript;
                }
                // If the name changed, or it's a new variant, then a new Label must be made (the old one can't be extended further).
                (Name(_), Name(_) | Constant { .. }) | (Constant { .. }, Name(_)) => {
                    break;
                }
            }

            expected_subscript -= 1;
            if subscript != expected_subscript {
                return Err("Subscripts must be decrementing within a segment.");
            }
        }

        let segment = Segment { label, magnitude, ignored_low_count: subscript };
        Ok((segment, bytes))
    }

    const fn atom_from_bytes(bytes: &[u8]) -> Result<(Label, u8), &'static str> {
        assert!(bytes.len() == 7);

        let label = Label::from_char(bytes[0] as char);
        let tens_digit = subscript_utf8_bytes_to_digit(bytes[1], bytes[2], bytes[3])?;
        let ones_digit = subscript_utf8_bytes_to_digit(bytes[4], bytes[5], bytes[6])?;
        let subscript = 10 * tens_digit + ones_digit;
        Ok((label, subscript))
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
            Label::Name(_) => (self.ignored_low_count..self.magnitude)
                .rev()
                .map(|i| {
                    [
                        self.label_text_at(i - self.ignored_low_count),
                        subscript_digit_to_string(i),
                    ]
                    .concat()
                })
                .join(""),
            Label::Constant { lowest_subscript, .. } => {
                let lowest_visible_subscript = lowest_subscript + self.ignored_low_count;
                (lowest_visible_subscript..lowest_visible_subscript + self.width())
                    .rev()
                    .map(|i| {
                        [
                            self.label_text_at(i - lowest_subscript),
                            subscript_digit_to_string(i),
                        ]
                        .concat()
                    })
                    .join("")
            }
        }
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
}

fn subscript_digit_to_string(value: u8) -> String {
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
        let (label, subscript) =
            Segment::atom_from_bytes(&[0x61, 0xE2, 0x82, 0x81, 0xE2, 0x82, 0x82])
                .unwrap();
        assert_eq!(label, Label::Name('a'));
        assert_eq!(subscript, 12);
    }
}
