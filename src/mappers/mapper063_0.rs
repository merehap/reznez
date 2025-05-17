use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(4096 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::EMPTY),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(8 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.fixed_index(0).status_register(S0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadWrite,
        ReadWriteStatus::ReadOnly,
    ])
    .build();

// TH2291-3 and CH-011
// TODO: Untested. Test ROM needed.
pub struct Mapper063_0;

impl Mapper for Mapper063_0 {
    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, _value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0xFFFF => {
                let fields = splitbits!(min=u8, cpu_address, ".... .rpp pppp pplm");
                params.set_read_write_status(S0, fields.r);
                params.set_prg_register(P0, fields.p);
                params.set_prg_layout(fields.l);
                params.set_name_table_mirroring(fields.m);
            }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}
