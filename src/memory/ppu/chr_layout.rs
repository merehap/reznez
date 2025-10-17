use crate::memory::bank::bank_number::{ChrBankRegisterId, ChrBankRegisters};
use crate::memory::window::{ReadWriteStatusInfo, ChrWindow};

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

    pub fn force_rom(&self) -> Self {
        let windows: Vec<ChrWindow> = self.initial_windows.iter()
            .map(|window| window.force_rom())
            .collect();
        Self { initial_windows: Box::leak(Box::new(windows)) }
    }

    pub fn force_ram(&self) -> Self {
        let windows: Vec<ChrWindow> = self.initial_windows.iter()
            .map(|window| window.force_ram())
            .collect();
        Self { initial_windows: Box::leak(Box::new(windows)) }
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

    pub fn active_read_write_status_register_ids(&self) -> Vec<ReadWriteStatusInfo> {
        self.initial_windows.iter()
            .map(|window| window.read_write_status_info())
            .collect()
    }
}
