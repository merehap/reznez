use std::fmt;

use crate::mapper::CpuAddress;
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::address_template::bit_template::BitTemplate;
use crate::memory::address_template::segment::{Label, Segment};
use crate::memory::bank::bank::PrgBankNumberProvider;
use crate::memory::bank::bank_number::PrgBankRegisters;
use crate::memory::window::PrgWindow;
use crate::util::const_vec::ConstVec;

const MAX_WIDTH: u8 = 32;
const BASE_ADDRESS_SEGMENT: u8 = 0;
const INNER_BANK_SEGMENT: u8 = 1;

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
 * Components   o₀₁o₀₀ p₀₂p₀₁p₀₀ a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
 * Full Address a₁₈a₁₇ a₁₆a₁₅a₁₄ a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
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
 * Components   o₀₁o₀₀ p₀₂p₀₁p₀₀ a₁₃ a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇ a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
 * Full Address a₁₈a₁₇ a₁₆a₁₅a₁₄ a₁₃ a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇ a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
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
#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub struct AddressResolver {
    // Never changed after initialization.
    bit_template: BitTemplate,

    // TODO: This should only be present for RAM resolvers.
    work_ram_start_inner_bank_number: u16,
}

impl AddressResolver {
    pub const PRG_PAGE_NUMBER_WIDTH: u8 = 13;
    pub const PRG_PAGE_SIZE: u16 = 2u16.pow(Self::PRG_PAGE_NUMBER_WIDTH as u32);

    /**
     * PRG Address                           a₁₇a₁₆a₁₅a₁₄ a₁₃ a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
     * Components Before (8 KiB inner banks) o₀₁o₀₀p₀₂p₀₁ p₀₀ a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
     * Components After (16 KiB inner banks) o₀₁o₀₀p₀₂p₀₁ a₁₃ a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
     */
    pub const fn prg(window: &PrgWindow, bank_sizes: &BankSizes, work_ram_start_inner_bank_number: u16) -> Self {
        let inner_bank_width = bank_sizes.inner_bank_width();
        let address_bus_segment = Segment::labeled(Label::AddressBus, inner_bank_width);
        let inner_bank_segment = match window.bank().prg_bank_number_provider() {
            PrgBankNumberProvider::Fixed(bank_number) => {
                // o₀₁o₀₀1₁₆1₁₅1₁₄1₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::constant_inner_bank(bank_number.to_raw(), bank_sizes.inner_bank_number_width())
            }
            PrgBankNumberProvider::Switchable(reg_id) => {
                // o₀₁o₀₀p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::labeled(Label::InnerBankSegment(Some(reg_id)), bank_sizes.inner_bank_number_width())
            }
        };

        let outer_bank_segment = Segment::labeled(Label::OuterBank, bank_sizes.outer_bank_number_width());

        let mut segments = ConstVec::new();
        segments.push(address_bus_segment);
        segments.push(inner_bank_segment);
        segments.push(outer_bank_segment);

        // Don't expand the bank size larger than the total memory size.
        let new_base_address_bit_count = std::cmp::min(window.size().bit_count(), bank_sizes.full_width());
        if new_base_address_bit_count > segments.get_mut(BASE_ADDRESS_SEGMENT).magnitude() {
            let mut ignored_low_count = segments.get_mut(BASE_ADDRESS_SEGMENT).increase_magnitude_to(new_base_address_bit_count);
            ignored_low_count = match segments.maybe_get_mut(INNER_BANK_SEGMENT) {
                None => ignored_low_count,
                Some(segment) => segment.increase_ignored_low_count(ignored_low_count),
            };
            assert!(
                ignored_low_count == 0,
                "Overshift occurred. Outer bank bits shouldn't be lost to large inner bank sizes."
            );
        }

        let bit_template = BitTemplate::right_to_left(segments, Some(bank_sizes.full_width()));
        let address_template = Self {
            bit_template,
            work_ram_start_inner_bank_number,
        };
        assert!(address_template.total_width() <= MAX_WIDTH);

        address_template
    }

    pub const fn from_formatted(text: &'static str, work_ram_start_inner_bank_number: u16) -> Result<Self, &'static str> {
        let mut bit_template = BitTemplate::from_formatted(text)?;
        let base_address_index = bit_template.index_of_label('a').expect("Base Address Segment");

        // If the outer bank segment and base address segment are adjacent,
        // then insert an empty inner bank segment between them.
        if let Some(outer_segment_index) = bit_template.index_of_label('o')
                && outer_segment_index.strict_sub(base_address_index) == 1 {
            bit_template = bit_template.empty_segment_inserted(outer_segment_index);
        }

        Self::from_bit_template(bit_template, work_ram_start_inner_bank_number)
    }

    const fn from_bit_template(bit_template: BitTemplate, work_ram_start_inner_bank_number: u16) -> Result<Self, &'static str> {
        if bit_template.width() > 32 {
            return Err("AddressTemplate must not be longer than 32 bits.");
        }

        Ok(Self { bit_template, work_ram_start_inner_bank_number })
    }

    pub fn segment_constants(&self) -> Vec<u16> {
        (0..self.bit_template.segment_count())
            .map(|i| self.bit_template.constant_at(i).unwrap())
            .collect()
    }

    pub fn reduced(&self, bank_sizes: &BankSizes) -> Self {
        let mut result = *self;
        result.bit_template = result.bit_template.shortened(bank_sizes.full_width());
        result
    }

    pub const fn total_width(&self) -> u8 {
        self.bit_template.width()
    }

    pub fn is_currently_resolving_to_save_ram(&self) -> bool {
        self.resolve_inner_bank_number() < self.work_ram_start_inner_bank_number
    }

    pub fn resolve_inner_bank_number(&self) -> u16 {
        if self.bit_template.has_inner_bank() {
            self.bit_template.resolve_segment(INNER_BANK_SEGMENT)
        } else {
            0
        }
    }

    pub fn resolve_index(&self, addr: CpuAddress) -> u32 {
        self.bit_template.resolve(*addr)
    }

    pub fn formatted(&self) -> String {
        self.bit_template.formatted()
    }

    pub const fn segment_count(&self) -> u8 {
        self.bit_template.segment_count()
    }

    pub fn set_raw_outer_bank_number(&mut self, number: u16) {
        let last_segment = self.bit_template.segment_count() - 1;
        if self.bit_template.label_at(last_segment) == Label::OuterBank {
            self.bit_template.set_raw_value_at(last_segment, number);
        }
    }

    pub fn update_inner_bank_number(&mut self, regs: &PrgBankRegisters) {
        let mut segments: Vec<_> = self.bit_template.segments_mut().collect();
        for index in 0..segments.len() {
            if let Some(reg_id) = segments[index].register_id() {
            let raw_value = regs.get(reg_id).index().unwrap().to_raw();
                segments[index].set_raw_value(raw_value);
            }
        }
    }
}

impl fmt::Display for AddressResolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}

#[cfg(test)]
mod test {
    use crate::mapper::KIBIBYTE;

    use super::*;

    #[test]
    fn no_inner_bank() {
        let text = "o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀";
        let original_resolver = AddressResolver::from_formatted(text, 0).unwrap();
        assert_eq!(original_resolver.total_width(), 23);
        assert_eq!(original_resolver.resolve_inner_bank_number(), 0);
        assert_eq!(original_resolver.formatted(), "o₀₇o₀₆o₀₅o₀₄o₀₃o₀₂o₀₁o₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀");

        let bank_sizes = BankSizes::new(512 * KIBIBYTE, 32 * KIBIBYTE, 32 * KIBIBYTE);
        let mut reduced_resolver = original_resolver.reduced(&bank_sizes);
        // Set raw outer bank number
        reduced_resolver.bit_template.set_raw_value_at(1, 0b1111_1111_1111_1111);
        assert_eq!(reduced_resolver.total_width(), 19);
        assert_eq!(reduced_resolver.resolve_inner_bank_number(), 0);
        assert_eq!(reduced_resolver.formatted(), "o₀₃o₀₂o₀₁o₀₀a₁₄a₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀");

        assert_eq!(reduced_resolver.segment_count(), 3, "Missing empty inner bank segment?");
    }
}