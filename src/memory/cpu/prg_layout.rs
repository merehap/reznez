use crate::memory::bank::bank_index::BankRegisterId;
use crate::memory::window::{RamStatusInfo, Window};

#[derive(Clone, Copy)]
pub struct PrgLayout(&'static [Window]);

impl PrgLayout {
    pub const fn new(windows: &'static [Window]) -> PrgLayout {
        assert!(!windows.is_empty(), "No PRG windows specified.");

        assert!(windows[0].start() <= 0x6000,
            "The first PRG window must start at 0x6000 at highest.");

        assert!(windows[windows.len() - 1].end() == 0xFFFF,
                "The last PRG window must end at 0xFFFF.");

        let mut i = 1;
        while i < windows.len() {
            assert!(windows[i].start() == windows[i - 1].end() + 1,
                "There must be no gaps nor overlap between PRG windows.");

            i += 1;
        }

        PrgLayout(windows)
    }

    pub fn windows(&self) -> &[Window] {
        self.0
    }

    pub fn active_register_ids(&self) -> Vec<BankRegisterId> {
        self.0.iter()
            .filter_map(|window| window.register_id())
            .collect()
    }

    pub fn ram_status_infos(&self) -> Vec<RamStatusInfo> {
        self.0.iter()
            .map(|window| window.ram_status_info())
            .collect()
    }
}
