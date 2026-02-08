use std::ops::Index;

use crate::memory::address_template::bank_sizes::BankSizes;
use crate::memory::bank::bank_number::PrgBankRegisterId;
use crate::memory::window::PrgWindow;
use crate::util::const_vec::ConstVec;
use crate::util::unit::{KIBIBYTE, KIBIBYTE_U16};

#[derive(Clone)]
pub struct PrgLayouts {
    rom_max_bank_sizes: BankSizes,
    layouts: ConstVec<PrgLayout, 16>,
}

impl PrgLayouts {
    #[allow(clippy::large_types_passed_by_value)]
    pub const fn new(
        rom_size: u32,
        outer_bank_count: u8,
        layouts: ConstVec<PrgLayout, 16>,
    ) -> Self {
        let mut inner_bank_size = layouts.get(0).smallest_rom_window_size() as u32;
        let mut i = 1;
        while i < layouts.len() {
            let layout_min = layouts.get(i).smallest_rom_window_size() as u32;
            inner_bank_size = std::cmp::min(inner_bank_size, layout_min);
            i += 1;
        }

        inner_bank_size = std::cmp::max(inner_bank_size, 8 * KIBIBYTE);

        let outer_bank_size = outer_bank_count as u32 * inner_bank_size;
        let rom_max_bank_sizes =
            BankSizes::new(rom_size, outer_bank_size, inner_bank_size);
        PrgLayouts { rom_max_bank_sizes, layouts }
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

    pub fn iter(&self) -> impl DoubleEndedIterator<Item = PrgLayout> {
        self.layouts.as_iter()
    }
}

impl Index<u8> for PrgLayouts {
    type Output = PrgLayout;

    fn index(&self, index: u8) -> &Self::Output {
        self.layouts.get_ref(index)
    }
}

#[derive(Clone, Copy)]
pub struct PrgLayout(&'static [PrgWindow]);

impl PrgLayout {
    pub const fn new(windows: &'static [PrgWindow]) -> PrgLayout {
        assert!(!windows.is_empty(), "No PRG windows specified.");

        assert!(
            windows[0].start() <= 0x6000,
            "The first PRG window must start at 0x6000 at highest."
        );

        assert!(
            windows[windows.len() - 1].end().get() == 0xFFFF,
            "The last PRG window must end at 0xFFFF."
        );

        let mut has_rom = false;
        let mut i = 1;
        while i < windows.len() {
            assert!(
                windows[i].start() == windows[i - 1].end().get() + 1,
                "There must be no gaps nor overlap between PRG windows."
            );
            if windows[i].bank().is_rom() {
                has_rom = true;
            }

            i += 1;
        }

        assert!(has_rom, "Each PrgLayout must have a ROM window.");

        PrgLayout(windows)
    }

    pub const fn smallest_rom_window_size(&self) -> u16 {
        let mut i = 0;
        let mut smallest_size = 32 * KIBIBYTE_U16;
        while i < self.0.len() {
            let window = &self.0[i];
            if window.bank().is_rom() {
                smallest_size = std::cmp::min(smallest_size, window.size().to_raw());
            }

            i += 1;
        }

        smallest_size
    }

    pub const fn supports_ram(&self) -> bool {
        let mut i = 0;
        while i < self.0.len() {
            if self.0[i].bank().supports_ram() {
                return true;
            }

            i += 1;
        }

        false
    }

    pub fn windows(&self) -> &[PrgWindow] {
        self.0
    }

    pub fn active_register_ids(&self) -> Vec<PrgBankRegisterId> {
        self.0
            .iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}
