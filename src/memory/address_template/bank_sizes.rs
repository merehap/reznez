use crate::util::unit::KIBIBYTE;

#[derive(Clone, Debug)]
pub struct BankSizes {
    full_size: u32,
    outer_bank_size: u32,
    inner_bank_size: u32,
}

impl BankSizes {
    pub const fn new(
        full_size: u32,
        mut outer_bank_size: u32,
        mut inner_bank_size: u32,
    ) -> Self {
        outer_bank_size = std::cmp::min(outer_bank_size, full_size);
        assert!(outer_bank_size.is_multiple_of(8 * KIBIBYTE));
        inner_bank_size = std::cmp::min(inner_bank_size, outer_bank_size);
        Self { full_size, outer_bank_size, inner_bank_size }
    }

    pub const fn full_size(&self) -> u32 {
        self.full_size
    }
    pub const fn outer_bank_size(&self) -> u32 {
        self.outer_bank_size
    }
    pub const fn inner_bank_size(&self) -> u32 {
        self.inner_bank_size
    }

    pub const fn full_width(&self) -> u8 {
        size_to_width(self.full_size)
    }

    pub const fn outer_bank_width(&self) -> u8 {
        size_to_width(self.outer_bank_size)
    }

    pub const fn inner_bank_width(&self) -> u8 {
        size_to_width(self.inner_bank_size)
    }

    pub const fn outer_bank_number_width(&self) -> u8 {
        self.full_width() - self.outer_bank_width()
    }

    pub const fn inner_bank_number_width(&self) -> u8 {
        self.outer_bank_width() - self.inner_bank_width()
    }
}

const fn size_to_width(size: u32) -> u8 {
    if size == 0 {
        return 0;
    }

    assert!(size.is_power_of_two());
    (size - 1).count_ones() as u8
}
