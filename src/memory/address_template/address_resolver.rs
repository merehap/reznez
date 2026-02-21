use std::fmt;

use crate::mapper::{CpuAddress, PrgBankRegisterId};
use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::address_template::bit_template::BitTemplate;
use crate::memory::address_template::segment::{Label, Segment};
use crate::memory::bank::bank_number::{BankNumber, PrgBankRegisters};
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
    // Never changed after initialization.
    inner_bank_width: u8,

    raw_outer_bank_number: u16,
    raw_inner_bank_number: u16,

    reg_id: Option<PrgBankRegisterId>,
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
        let fixed_inner_bank_number = window.bank().fixed_bank_number().map(BankNumber::to_raw);

        let inner_bank_width = bank_sizes.inner_bank_width();
        let address_bus_segment = Segment::labeled('a', inner_bank_width);
        let inner_bank_segment =
            if let Some(fixed_inner_bank_number) = fixed_inner_bank_number {
                // o₀₁o₀₀1₁₆1₁₅1₁₄1₁₃a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::unlabeled(fixed_inner_bank_number, bank_sizes.inner_bank_number_width())
            } else {
                // o₀₁o₀₀p₀₃p₀₂p₀₁p₀₀a₁₂a₁₁a₁₀a₀₉a₀₈a₀₇a₀₆a₀₅a₀₄a₀₃a₀₂a₀₁a₀₀
                Segment::labeled('p', bank_sizes.inner_bank_number_width())
            };

        let outer_bank_segment = Segment::labeled('o', bank_sizes.outer_bank_number_width());

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
            inner_bank_width,
            raw_outer_bank_number: 0,
            raw_inner_bank_number: 0,
            reg_id: window.register_id(),
            work_ram_start_inner_bank_number,
        };
        assert!(address_template.total_width() <= MAX_WIDTH);

        address_template
    }

    pub const fn from_formatted(text: &'static str, work_ram_start_inner_bank_number: u16) -> Result<Self, &'static str> {
        Self::from_bit_template(BitTemplate::from_formatted(text)?, work_ram_start_inner_bank_number)
    }

    const fn from_bit_template(bit_template: BitTemplate, work_ram_start_inner_bank_number: u16) -> Result<Self, &'static str> {
        if bit_template.width() > 32 {
            return Err("AddressTemplate must not be longer than 32 bits.");
        }

        let base_address_width = bit_template.width_of('a');
        let inner_bank_ignored_low_count = bit_template.ignored_low_count_of('i');
        let inner_bank_width = base_address_width + inner_bank_ignored_low_count;

        let mut reg_id = None;
        let mut i = 0;
        while i < bit_template.segment_count() {
            match bit_template.label_at(i).map(Label::to_char) {
                Some(c@'p'..='y') => {
                    assert!(reg_id.is_none(), "Multiple inner bank segments not allowed in a single template.");
                    reg_id = Some(PrgBankRegisterId::from_char(c).expect("Bad inner bank label letter."));
                }
                None | Some(_) => {}
            }

            i += 1;
        }

        Ok(Self {
            bit_template,
            inner_bank_width,
            raw_outer_bank_number: 0,
            raw_inner_bank_number: 0,
            reg_id,
            work_ram_start_inner_bank_number,
        })
    }

    pub fn segment_constants(&self) -> Vec<u16> {
        (0..self.bit_template.segment_count())
            .map(|i| self.bit_template.constant_at(i).unwrap())
            .collect()
    }

    pub fn reduced(&self, bank_sizes: &BankSizes) -> Self {
        let mut result = self.clone();
        result.bit_template = result.bit_template.shortened(bank_sizes.full_width());
        result.inner_bank_width = bank_sizes.inner_bank_width();
        result
    }

    pub const fn total_width(&self) -> u8 {
        self.bit_template.width()
    }

    pub fn is_currently_resolving_to_save_ram(&self) -> bool {
        self.resolve_inner_bank_number() < self.work_ram_start_inner_bank_number
    }

    pub fn resolve_inner_bank_number(&self) -> u16 {
        self.bit_template.resolve_segment(INNER_BANK_SEGMENT, self.raw_inner_bank_number)
    }

    pub fn resolve_index(&self, addr: CpuAddress) -> u32 {
        self.bit_template.resolve(&[*addr, self.raw_inner_bank_number, self.raw_outer_bank_number])
    }

    pub fn formatted(&self) -> String {
        self.bit_template.formatted()
    }

    pub const fn segment_count(&self) -> u8 {
        self.bit_template.segment_count()
    }

    pub fn set_raw_outer_bank_number(&mut self, number: u16) {
        self.raw_outer_bank_number = number;
    }

    pub fn update_inner_bank_number(&mut self, regs: &PrgBankRegisters) {
        if let Some(reg_id) = self.reg_id {
            self.raw_inner_bank_number = regs.get(reg_id).index().unwrap().to_raw();
        }
    }
}

impl fmt::Display for AddressResolver {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.formatted())
    }
}
