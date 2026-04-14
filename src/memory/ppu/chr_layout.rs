use std::ops::Index;

use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::bank::bank_number::{ChrBankRegisterId, ChrBankRegisters};
use crate::memory::window::ChrWindow;
use crate::util::const_vec::ConstVec;
use crate::util::unit::KIBIBYTE_U16;

#[derive(Clone)]
pub struct ChrLayouts {
    rom_max_bank_sizes: BankSizes,
    layouts: ConstVec<ChrLayout, 16>,
}

impl ChrLayouts {
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(
        rom_size: u32,
        outer_bank_count: u16,
        inner_bank_size: Option<u32>,
        layouts: ConstVec<ChrLayout, 16>,
    ) -> Self {
        let mut inferred_inner_bank_size = layouts.get(0).smallest_rom_window_size() as u32;
        let mut i = 1;
        while i < layouts.len() {
            let layout_min = layouts.get(i).smallest_rom_window_size() as u32;
            inferred_inner_bank_size = std::cmp::min(inferred_inner_bank_size, layout_min);
            i += 1;
        }

        let inner_bank_size = inner_bank_size.unwrap_or(inferred_inner_bank_size);
        let outer_bank_size = outer_bank_count as u32 * inner_bank_size;
        let rom_max_bank_sizes = BankSizes::new(rom_size, outer_bank_size, inner_bank_size);
        Self { rom_max_bank_sizes, layouts }
    }

    pub const fn ram_supported(&self) -> bool {
        let mut i = 0;
        while i < self.layouts.len() {
            if self.layouts.get(i).supports_ram() {
                return true;
            }

            i += 1;
        }

        false
    }

    pub fn rom_max_bank_sizes(&self) -> &BankSizes {
        &self.rom_max_bank_sizes
    }

    pub fn count(&self) -> u8 {
        self.layouts.len()
    }

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = ChrLayout> {
        self.layouts.as_iter()
    }
}

impl Index<u8> for ChrLayouts {
    type Output = ChrLayout;

    fn index(&self, index: u8) -> &Self::Output {
        self.layouts.get_ref(index)
    }
}
#[derive(Clone, Copy)]
pub struct ChrLayout {
    initial_windows: &'static [ChrWindow],
}

impl ChrLayout {
    pub const fn new(initial_windows: &'static [ChrWindow]) -> ChrLayout {
        assert!(!initial_windows.is_empty(), "No CHR windows specified.");

        assert!(initial_windows[0].start() == 0x0000, "The first CHR window must start at 0x0000.");

        assert!(initial_windows.last().unwrap().end().get() >= 0x1FFF,
            "The last CHR window must end at 0x1FFF (or later, in rare cases).");

        let mut i = 1;
        while i < initial_windows.len() {
            assert!(initial_windows[i].start() == initial_windows[i - 1].end().get() + 1,
                    "There must be no gaps nor overlap between CHR layouts.");

            i += 1;
        }

        ChrLayout {
            initial_windows,
        }
    }

    pub const fn smallest_rom_window_size(&self) -> u16 {
        let mut i = 0;
        let mut smallest_size = 32 * KIBIBYTE_U16;
        while i < self.initial_windows.len() {
            let window = &self.initial_windows[i];
            if window.bank().is_rom() {
                smallest_size = std::cmp::min(smallest_size, window.size().to_raw());
            }

            i += 1;
        }

        smallest_size
    }

    pub const fn supports_ram(&self) -> bool {
        let mut i = 0;
        while i < self.initial_windows.len() {
            if self.initial_windows[i].bank().supports_ram() {
                return true;
            }

            i += 1;
        }

        false
    }

    pub fn windows(&self) -> &[ChrWindow] {
        self.initial_windows
    }

    // Usually 0x1FFF, but different for mapper 19, for example.
    pub fn max_window_index(&self) -> u16 {
        self.initial_windows.last().unwrap().end().get()
    }

    pub fn active_register_ids(&self, regs: &ChrBankRegisters) -> Vec<ChrBankRegisterId> {
        self.initial_windows.iter()
            .filter_map(|window| window.register_id(regs))
            .collect()
    }
}
