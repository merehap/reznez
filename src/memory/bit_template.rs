use itertools::Itertools;

#[derive(PartialEq, Eq, Clone, Debug, Default)]
pub struct BitTemplate {
    // Segments are stored right-to-left, the reverse of how they are rendered.
    segments: Vec<Segment>,
}

impl BitTemplate {
    pub fn right_to_left(raw: &[(&str, u8)]) -> Self {
        let segments = raw.iter()
            .map(|(label, magnitude)| {
                let label = (*label).to_string();
                let label = Box::leak(Box::new(label));
                Segment::named(label, *magnitude)
            })
            .collect();
        Self { segments }
    }

    pub fn width(&self) -> u8 {
        self.segments.iter()
            .map(Segment::width)
            .sum()
    }

    pub fn width_of(&self, segment_index: u8) -> u8 {
        self.segments[segment_index as usize].width()
    }

    pub fn original_magnitude_of(&self, segment_index: u8) -> u8 {
        self.segments[segment_index as usize].original_magnitude()
    }

    pub fn magnitude_of(&self, segment_index: u8) -> u8 {
        self.segments[segment_index as usize].magnitude()
    }

    pub fn resolve(&self, raw_values: &[u16]) -> u32 {
        let mut result = 0;
        let mut index = 0;
        for (segment, &raw_value) in self.segments.iter().zip(raw_values) {
            result += segment.resolve_shifted(raw_value, index);
            index += segment.width();
        }

        result
    }

    pub fn resolve_segment(&self, segment_index: u8, raw_value: u16) -> u16 {
        self.segments[segment_index as usize].resolve(raw_value)
    }

    pub fn formatted(&self) -> String {
        self.segments.iter()
            .rev()
            .map(Segment::formatted)
            .join("")
    }

    /**
     * ```text
     * value == 0b1111_1111_1111_1111
     * Before: o₀₁o₀₀i₀₃i₀₂i₀₁i₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
     * After:  o₀₁o₀₀1₁₅1₁₄1₁₃1₁₂a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
     * ```
     */
    pub fn constify_segment(&mut self, segment_index: u8, value: u16) {
        let segment_index: usize = segment_index.into();
        let offset: u8 = self.segments[0..segment_index].iter()
            .map(Segment::width)
            .sum();
        self.segments[segment_index].constify(value, offset);
    }

    pub fn increase_segment_magnitude(&mut self, segment_index: u8, new_magnitude: u8) {
        let segment_index: usize = segment_index.into();
        assert!(segment_index < self.segments.len());

        let mut ignored_low_count = self.segments[segment_index].increase_magnitude_to(new_magnitude);

        for segment in &mut self.segments[segment_index + 1 ..] {
            ignored_low_count = segment.increase_ignored_low_count(ignored_low_count);
            assert_eq!(ignored_low_count, 0, "Overshift occurred. Outer bank bits shouldn't be lost to large inner bank sizes.");
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Segment {
    label: Label,
    original_magnitude: u8,
    magnitude: u8,
    ignored_low_count: u8,
}

impl Segment {
    pub const fn named(name: &'static str, magnitude: u8) -> Self {
        Self {
            label: Label::Name(name),
            original_magnitude: magnitude,
            magnitude,
            ignored_low_count: 0,
        }
    }

    pub fn original_magnitude(&self) -> u8 {
        self.original_magnitude
    }

    pub fn magnitude(&self) -> u8 {
        self.magnitude
    }

    pub fn width(&self) -> u8 {
        self.magnitude.saturating_sub(self.ignored_low_count)
    }

    pub fn ignored_low_count(&self) -> u8 {
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

    pub fn formatted(&self) -> String {
        match self.label {
            Label::Name(_) => {
                (self.ignored_low_count..self.magnitude).rev()
                    .map(|i| [self.label_text_at(i - self.ignored_low_count), to_subscript(i)].concat())
                    .join("")
            }
            Label::Constant { lowest_subscript, .. } => {
                let lowest_visible_subscript = lowest_subscript + self.ignored_low_count;
                (lowest_visible_subscript..lowest_visible_subscript + self.width()).rev()
                    .map(|i| [self.label_text_at(i - lowest_subscript), to_subscript(i)].concat())
                    .join("")
            }
        }
    }

    pub fn increase_magnitude_to(&mut self, magnitude: u8) -> u8 {
        let increase_amount = magnitude.strict_sub(self.magnitude);
        self.magnitude = magnitude;
        increase_amount
    }

    pub fn increase_ignored_low_count(&mut self, increase_amount: u8) -> u8 {
        let already_empty = self.width() == 0;
        self.ignored_low_count = self.ignored_low_count.strict_add(increase_amount);

        // Return the overshift (usually zero)
        if already_empty {
            increase_amount
        } else {
            self.ignored_low_count.saturating_sub(self.magnitude)
        }
    }

    pub fn constify(&mut self, value: u16, lowest_subscript: u8 ) {
        self.label = Label::Constant { value, lowest_subscript };
    }

    fn label_text_at(&self, index: u8) -> String {
        match &self.label {
            Label::Name(name) => (*name).to_string(),
            Label::Constant { value, .. } => ((value >> index) & 1).to_string(),
        }
    }
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub enum Label {
    Name(&'static str),
    Constant { value: u16, lowest_subscript: u8 },
}

impl Default for Label {
    fn default() -> Self {
        Self::Name("a")
    }
}

fn to_subscript(value: u8) -> String {
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