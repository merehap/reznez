use std::ops::Index;

use crate::memory::bank::bank_number::PrgBankRegisterId;
use crate::memory::window::PrgWindow;
use crate::util::const_vec::ConstVec;
use crate::util::unit::KIBIBYTE_U16;

#[derive(Clone)]
pub struct PrgLayouts {
    layouts: ConstVec<PrgLayout, 16>,
}

impl PrgLayouts {
    pub const fn new(layouts: ConstVec<PrgLayout, 16>) -> Self {
        PrgLayouts { layouts }
    }

    pub const fn rom_inner_bank_size(&self) -> u32 {
        let mut bank_size = self.layouts.get(0).smallest_rom_window_size() as u32;
        let mut i = 1;
        while i < self.layouts.len() {
            let layout_min = self.layouts.get(i).smallest_rom_window_size() as u32;
            bank_size = std::cmp::min(bank_size, layout_min);
            i += 1;
        }

        bank_size
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

        assert!(windows[0].start() <= 0x6000,
            "The first PRG window must start at 0x6000 at highest.");

        assert!(windows[windows.len() - 1].end().get() == 0xFFFF,
                "The last PRG window must end at 0xFFFF.");

        let mut has_rom = false;
        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start() == windows[i - 1].end().get() + 1,
                "There must be no gaps nor overlap between PRG windows.");
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
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}
