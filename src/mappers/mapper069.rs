use crate::mapper::*;
use crate::memory::bank::bank_index::MemType;
use crate::memory::memory::Memory;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P0).status_register(S0).rom_ram_register(R0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .chr_rom_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::Disabled,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

const CHR_REGISTER_IDS: [ChrBankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];
// P0 is used by the ROM/RAM window, which gets special treatment.
const PRG_ROM_REGISTER_IDS: [PrgBankRegisterId; 3] = [P1, P2, P3];

// Sunsoft FME-7
pub struct Mapper069 {
    command: Command,

    irq_enabled: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper069 {
    fn on_end_of_cpu_cycle(&mut self, mem: &mut Memory) {
        if self.irq_counter_enabled {
            if self.irq_enabled && self.irq_counter == 0 {
                mem.mapper_params.set_irq_pending(true);
            }

            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }
    }

    fn write_register(&mut self, params: &mut MapperParams, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x7FFF => { /* Do nothing. */ }
            0x8000..=0x9FFF => self.set_command(value),
            0xA000..=0xBFFF => self.execute_command(params, value),
            0xC000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper069 {
    pub fn new() -> Self {
        Self {
            // TODO: Verify that this is the correct startup value.
            command: Command::ChrRomBank(C0),

            irq_enabled: false,
            irq_counter_enabled: false,
            irq_counter: 0,
        }
    }

    fn set_command(&mut self, value: u8) {
        let value = usize::from(value) & 0b0000_1111;
        self.command = match value {
            0x0..=0x7 => Command::ChrRomBank(CHR_REGISTER_IDS[value]),
            0x8       => Command::PrgRomRamBank,
            0x9..=0xB => Command::PrgRomBank(PRG_ROM_REGISTER_IDS[value - 0x9]),
            0xC       => Command::NameTableMirroring,
            0xD       => Command::IrqControl,
            0xE       => Command::IrqCounterLowByte,
            0xF       => Command::IrqCounterHighByte,                       
            _ => unreachable!(),
        }
    }

    fn execute_command(&mut self, params: &mut MapperParams, value: u8) {
        match self.command {
            Command::ChrRomBank(id) =>
                params.set_chr_register(id, value),
            Command::PrgRomRamBank => {
                let fields = splitbits!(value, "smpppppp");
                params.set_read_write_status(S0, fields.s as u8);
                let rom_ram_mode = [MemType::Rom, MemType::WorkRam][fields.m as usize];
                params.set_rom_ram_mode(R0, rom_ram_mode);
                params.set_prg_register(P0, fields.p);
            }
            Command::PrgRomBank(id) =>
                params.set_prg_register(id, value),
            Command::NameTableMirroring =>
                params.set_name_table_mirroring(value & 0b11),
            Command::IrqControl => {
                params.set_irq_pending(false);
                (self.irq_counter_enabled, self.irq_enabled) = splitbits_named!(value, "c......i");
            }
            Command::IrqCounterLowByte =>
                set_bits(&mut self.irq_counter, u16::from(value)     , 0b0000_0000_1111_1111),
            Command::IrqCounterHighByte =>
                set_bits(&mut self.irq_counter, u16::from(value) << 8, 0b1111_1111_0000_0000),
        }
    }
}

enum Command {
    ChrRomBank(ChrBankRegisterId),
    PrgRomRamBank,
    PrgRomBank(PrgBankRegisterId),
    NameTableMirroring,
    IrqControl,
    IrqCounterLowByte,
    IrqCounterHighByte,
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
