use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ABSENT),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
    ])
    .chr_rom_max_size(1048 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
    ])
    .build();

const MODES: [Mode; 2] = [Mode::Cnrom, Mode::Nina03];

// Maxi 15 multicart
#[derive(Default)]
pub struct Mapper234 {
    mode: Mode,
    rom_side: u8,
    outer_bank: u8,
    prg_inner_bank: u8,
    chr_inner_bank: u8,
}

impl Mapper for Mapper234 {
    fn has_bus_conflicts(&self) -> HasBusConflicts {
        HasBusConflicts::Yes
    }

    fn on_cpu_read(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        self.set_register(mem, addr, value);
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        self.set_register(mem, addr, value);
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper234 {
    fn set_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0xFF80..=0xFF9F => {
                let fields = splitbits!(value, "nmsdbbbb");
                mem.set_name_table_mirroring(fields.n as u8);
                self.mode = MODES[fields.m as usize];
                if fields.d {
                    self.rom_side = 0;
                } else {
                    self.rom_side = fields.s as u8;
                }

                self.outer_bank = fields.b;
                self.update_bank_registers(mem);
            }
            0xFFC0..=0xFFDF => { /* TODO: Record lockout defeat control value. */ }
            0xFFE8..=0xFFF7 => {
                (self.chr_inner_bank, self.prg_inner_bank) = splitbits_named!(min=u8, value, ".ccc...p");
                self.update_bank_registers(mem);
            }
            _ => { /* Do nothing. */}
        }
    }

    fn update_bank_registers(&self, mem: &mut Memory) {
        match self.mode {
            Mode::Cnrom => {
                mem.set_prg_register(P0, self.outer_bank);
                let chr_bank = (self.outer_bank << 2) | (self.chr_inner_bank & 0b11);
                mem.set_chr_register(C0, chr_bank);
            }
            Mode::Nina03 => {
                let outer_bank = self.outer_bank >> 1;
                let prg_bank = (outer_bank << 1) | self.prg_inner_bank;
                mem.set_prg_register(P0, prg_bank);
                let chr_bank = (outer_bank << 3) | self.chr_inner_bank;
                mem.set_chr_register(C0, chr_bank);
            }
        }
    }
}

#[derive(Clone, Copy, Default)]
enum Mode {
    #[default]
    Cnrom,
    Nina03,
}
