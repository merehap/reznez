use crate::memory::bank::bank_index::BankRegisterId;
use crate::memory::window::Window;

#[derive(Clone, Copy)]
pub struct ChrLayout(&'static [Window]);

impl ChrLayout {
    pub const fn new(windows: &'static [Window]) -> ChrLayout {
        assert!(!windows.is_empty(), "No PRG layouts specified.");

        assert!(windows[0].start() == 0x0000, "The first CHR window must start at 0x0000.");

        assert!(windows[windows.len() - 1].end() >= 0x1FFF,
            "The last CHR window must end at 0x1FFF (or later, in rare cases).");

        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start() == windows[i - 1].end() + 1,
                    "There must be no gaps nor overlap between CHR layouts.");

            i += 1;
        }

        ChrLayout(windows)
    }

    pub fn windows(&self) -> &[Window] {
        self.0
    }

    // Usually 0x1FFF, but different for mapper 19, for example.
    pub fn max_window_index(&self) -> u16 {
        self.0[self.0.len() - 1].end()
    }

    pub fn active_register_ids(&self) -> Vec<BankRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }
}
