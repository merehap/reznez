use crate::memory::bank::bank_number::PrgBankRegisterId;
use crate::memory::window::PrgWindow;
use crate::util::unit::KIBIBYTE_U16;

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

    pub fn supports_ram(&self) -> bool {
        self.0.iter().any(|window| window.bank().supports_ram())
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
