use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(32 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.fixed_index(0).status_register(S0)),
    ])
    .read_write_statuses(&[
        ReadWriteStatus::Disabled,
        ReadWriteStatus::ReadOnly,
    ])
    .build();

const DISABLED_CHR_ROM: u8 = 0;
const READABLE_CHR_ROM: u8 = 1;

// CNROM with CHR disable
#[derive(Default)]
pub struct Mapper185_0 {
    ppu_data_read_count: u8,
}

impl Mapper for Mapper185_0 {
    fn on_cpu_read(&mut self, params: &mut MapperParams, address: CpuAddress, _value: u8) {
        if address.to_raw() == 0x2007 {
            if self.ppu_data_read_count < 2 {
                params.set_read_write_status(S0, DISABLED_CHR_ROM);
                self.ppu_data_read_count += 1;
            } else {
                params.set_read_write_status(S0, READABLE_CHR_ROM);
            }
        }
    }

    fn write_register(&mut self, _params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
