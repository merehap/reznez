use std::fmt;

const MAX_WIDTH: u8 = 32;

/**
 * ```text
 * *** EXAMPLE RESOLVED PRG ROM ADDRESS TEMPLATE ***
 *
 *                +------------------------- Outer bank number (width is outer_bank_count())
 *                |
 *                |        +---------------- Inner bank number (width is inner_bank_count())
 *                |        |
 *                |        |                 Base address (width is inner_bank_size())
 *                |        |                        |
 *                v        v                        v
 * Components   O₀₁O₀₀ I₀₂I₀₁I₀₀ A₁₃A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 * Full Address A₁₈A₁₇ A₁₆A₁₅A₁₄ A₁₃A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 *              |      |         |  |         Page size  (always 8 KiB)   |
 *              |      |         |  +-------------------------------------|
 *              |      |         |            Inner bank size  (16 KiB)   |
 *              |      |         +----------------------------------------|
 *              |      |                      Outer bank size (128 KiB)   |
 *              |      +--------------------------------------------------|
 *              |                             ROM size        (512 KiB)   |
 *              +---------------------------------------------------------+
 * 
 * 
 * *** EXAMPLE RESOLVED PRG ROM ADDRESS TEMPLATE WITH SUB-PAGES ***
 *
 *                +--------------------------------- Outer bank number (width is outer_bank_count())
 *                |
 *                |        +------------------------ Inner bank number (width is inner_bank_count())
 *                |        |
 *                |        |                 +------ Sub-page number
 *                |        |                 |
 *                |        |                 |       Base address (width is 128 B)
 *                |        |                 |                    |
 *                v        v                 v                    v
 * Components   O₀₁O₀₀ I₀₂I₀₁I₀₀ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇ A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 * Full Address A₁₈A₁₇ A₁₆A₁₅A₁₄ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇ A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
 *              |      |         |   |                  | Sub-page (128 B)  |
 *              |      |         |   |                  +-------------------|
 *              |      |         |   |        Page size  (always 8 KiB)     |
 *              |      |         |   +--------------------------------------|
 *              |      |         |            Inner bank size  (16 KiB)     |
 *              |      |         +------------------------------------------|
 *              |      |                      Outer bank size (128 KiB)     |
 *              |      +----------------------------------------------------|
 *              |                             ROM size        (512 KiB)     |
 *              +-----------------------------------------------------------+
 * ```
**/ 
#[derive(Clone, Debug, Default)]
pub struct AddressTemplate {
    // Bit widths
    total_width: u8,
    outer_bank_number_width: u8,
    inner_bank_number_width: u8,
    base_address_width: u8,

    outer_bank_mask: u8,
    inner_bank_mask: u16,
    base_address_mask: u16,

    inner_bank_low_bit_index: u8,

    fixed_inner_bank_number: Option<u16>,
}

impl AddressTemplate {
    pub const PRG_PAGE_NUMBER_WIDTH: u8 = 13;
    pub const PRG_PAGE_SIZE: u16 = 2u16.pow(Self::PRG_PAGE_NUMBER_WIDTH as u32);
    const PRG_SUB_PAGE_SIZE: u16 = Self::PRG_PAGE_SIZE / 64;

    pub fn new(
        (outer_bank_total_width, outer_bank_low_bit_index): (u8, u8),
        (inner_bank_total_width, inner_bank_low_bit_index): (u8, u8),
        (mut base_address_width, base_address_low_bit_index): (u8, u8),
        fixed_inner_bank_number: Option<u16>,
    ) -> Self {
        assert_eq!(base_address_low_bit_index, 0);
        assert_eq!(outer_bank_low_bit_index, 0);

        let outer_bank_number_width = outer_bank_total_width.strict_sub(inner_bank_total_width);
        // If the ROM is undersized, reduce the base address bit count, effectively mirroring the ROM until it's the right size.
        base_address_width = std::cmp::min(base_address_width, outer_bank_total_width);
        let inner_bank_number_width = inner_bank_total_width.strict_sub(base_address_width);

        let address_template = Self {
            total_width: outer_bank_total_width,
            outer_bank_number_width,
            inner_bank_number_width,
            base_address_width,

            outer_bank_mask: create_mask(outer_bank_number_width, outer_bank_low_bit_index).try_into().unwrap(),
            inner_bank_mask: create_mask(inner_bank_number_width, inner_bank_low_bit_index),
            base_address_mask: create_mask(base_address_width, base_address_low_bit_index),

            inner_bank_low_bit_index,

            fixed_inner_bank_number,
        };
        assert!(address_template.total_width() <= MAX_WIDTH);

        address_template
    }

    pub fn total_width(&self) -> u8 {
        self.total_width
    }

    pub fn inner_bank_size(&self) -> u16 {
        1 << (self.base_address_width - self.inner_bank_low_bit_index)
    }

    pub fn inner_bank_count(&self) -> u16 {
        1 << self.inner_bank_number_width
    }

    pub fn outer_bank_count(&self) -> u8 {
        1 << self.outer_bank_number_width
    }

    pub fn outer_bank_size(&self) -> u32 {
        u32::from(self.inner_bank_count()) * u32::from(self.inner_bank_size())
    }

    pub fn rom_size(&self) -> u32 {
        u32::from(self.outer_bank_count()) * self.outer_bank_size()
    }

    pub fn prg_pages_per_inner_bank(&self) -> u8 {
        u8::try_from(self.inner_bank_size() / Self::PRG_PAGE_SIZE).unwrap()
    }

    pub fn prg_pages_per_outer_bank(&self) -> u16 {
        u16::try_from(self.outer_bank_size() / u32::from(Self::PRG_PAGE_SIZE)).unwrap()
    }

    pub fn total_prg_pages(&self) -> u16 {
        u16::try_from(self.rom_size() / u32::from(Self::PRG_PAGE_SIZE)).unwrap()
    }

    pub fn page_number_mask(&self) -> u16 {
        self.prg_pages_per_outer_bank() - 1
    }

    /**
     * PRG Address                            A₁₇A₁₆A₁₅A₁₄ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components Before ( 8 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ I₀₀ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     * Components After  (16 KiB inner banks) O₀₁O₀₀I₀₂I₀₁ A₁₃ A₁₂A₁₁A₁₀A₀₉A₀₈A₀₇A₀₆A₀₅A₀₄A₀₃A₀₂A₀₁A₀₀
     */
    pub fn with_bigger_bank(&self, new_base_address_bit_count: u8, fixed_inner_bank_number: Option<u16>) -> Option<Self> {
        let mut big_banked = self.clone();
        big_banked.fixed_inner_bank_number = fixed_inner_bank_number;

        // Don't expand the bank size larger than the total memory size.
        if new_base_address_bit_count > self.total_width() {
            return Some(big_banked);
        }

        let bank_size_shift = new_base_address_bit_count.checked_sub(self.base_address_width)?;

        big_banked.inner_bank_low_bit_index += bank_size_shift;
        big_banked.base_address_width += bank_size_shift;
        big_banked.base_address_mask = create_mask(big_banked.base_address_width, 0);
        big_banked.inner_bank_mask = create_mask(big_banked.inner_bank_number_width, bank_size_shift);

        assert_eq!(
            big_banked.outer_bank_number_width
                + big_banked.inner_bank_number_width
                + big_banked.base_address_width
                - big_banked.inner_bank_low_bit_index,
            big_banked.total_width()
        );
        Some(big_banked)
    }

    pub fn resolve_page_number(&self, raw_inner_bank_number: u16, page_offset: u16) -> u16 {
        let inner_bank_number = raw_inner_bank_number & self.inner_bank_mask;
        let raw_page_number = inner_bank_number * u16::from(self.prg_pages_per_inner_bank()) + page_offset;
        raw_page_number & self.page_number_mask()
    }

    pub fn resolve_index(&self, raw_outer_bank_number: u8, page_number: u16, offset_in_page: u16) -> u32 {
        let outer_bank_start = u32::from(raw_outer_bank_number & self.outer_bank_mask) * self.outer_bank_size();
        let page_start = u32::from(page_number) * u32::from(Self::PRG_PAGE_SIZE);
        outer_bank_start | page_start | u32::from(offset_in_page)
    }

    pub fn resolve_subpage_index(&self, raw_outer_bank_number: u8, page_number: u16, sub_page_offset: u8, offset_in_page: u16) -> u32 {
        let outer_bank_start = u32::from(raw_outer_bank_number & self.outer_bank_mask) * self.outer_bank_size();
        let page_start = u32::from(page_number) * Self::PRG_PAGE_SIZE as u32;
        let subpage_start = Self::PRG_SUB_PAGE_SIZE as u32 * sub_page_offset as u32;
        let offset_in_subpage = u32::from(offset_in_page % Self::PRG_SUB_PAGE_SIZE);
        outer_bank_start | page_start | subpage_start | offset_in_subpage
    }

    pub fn formatted(&self) -> String {
        let mut entries = Vec::with_capacity(MAX_WIDTH.into());
        let ow = self.outer_bank_number_width;
        let iw = self.inner_bank_number_width.strict_sub(self.inner_bank_low_bit_index);
        let aw = self.base_address_width;
        assert_eq!(ow + iw + aw, self.total_width);
        entries.append(&mut suffix_sequence("a", 0, aw));
        if let Some(fixed) = self.fixed_inner_bank_number && iw > 0 {
            let fixed = (fixed & self.inner_bank_mask) >> self.inner_bank_low_bit_index;
            let width = usize::from(iw);
            let mut fixed: Vec<_> = format!("{fixed:0>width$b}")
                .chars()
                .rev()
                .enumerate()
                .map(|(i, c)| format!("{c}{}", to_subscript(aw + i as u8)))
                .collect();
            entries.append(&mut fixed);
        } else {
            entries.append(&mut suffix_sequence("i", self.inner_bank_low_bit_index, iw));
        }

        entries.append(&mut suffix_sequence("o", 0, ow));

        entries.reverse();
        entries.concat()
    }
}

fn create_mask(bit_count: u8, low_bit_index: u8) -> u16 {
    ((1 << bit_count) - 1) & !((1 << low_bit_index) - 1)
}

fn suffix_sequence(prefix: &str, start_index: u8, count: u8) -> Vec<String> {
    (start_index..start_index + count)
        .map(|i| [prefix, &to_subscript(i)].concat())
        .collect()
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

impl fmt::Display for AddressTemplate {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}