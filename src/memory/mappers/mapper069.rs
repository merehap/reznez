use crate::memory::mapper::*;

const PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P0)),
    PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P1)),
    PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P2)),
    PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::Switchable(Rom, P3)),
    PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::Fixed(Rom, BankIndex::LAST)),
]);

const CHR_WINDOWS: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::Switchable(Rom, C7)),
]);

const CHR_REGISTER_IDS: [BankIndexRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];
// P0 is used by the ROM/RAM window, which gets special treatment.
const PRG_ROM_REGISTER_IDS: [BankIndexRegisterId; 3] = [P1, P2, P3];

const NAME_TABLE_MIRRORINGS: [NameTableMirroring; 4] = [
    NameTableMirroring::Vertical,
    NameTableMirroring::Horizontal,
    NameTableMirroring::OneScreenLeftBank,
    NameTableMirroring::OneScreenRightBank,
];

// Sunsoft FME-7
// TODO: PRG RAM functionality
pub struct Mapper069 {
    command: Command,

    irq_pending: bool,
    irq_enabled: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper069 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(64)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_windows(PRG_WINDOWS)
            .chr_max_bank_count(256)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_windows(CHR_WINDOWS)
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .build()
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.irq_counter_enabled {
            if self.irq_enabled && self.irq_counter == 0 {
                self.irq_pending = true;
            }

            self.irq_counter = self.irq_counter.wrapping_sub(1);
        }
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5FFF => { /* Do nothing. */ }
            0x6000..=0x7FFF => todo!("Mapper 69 Work RAM writes."),
            0x8000..=0x9FFF => self.set_command(value),
            0xA000..=0xBFFF => self.execute_command(params, value),
            0xC000..=0xFFFF => { /* Do nothing. */ }
        }
    }
}

impl Mapper069 {
    pub fn new() -> Self {
        Self {
            // TODO: Verify that this is the correct startup value.
            command: Command::ChrRomBank(C0),

            irq_pending: false,
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
                params.set_bank_index_register(id, value),
            Command::PrgRomRamBank => {
                if value & 0b1100_0000 != 0 {
                    todo!("PRG RAM toggle");
                }
                params.set_bank_index_register(P0, value & 0b0011_1111);
            }
            Command::PrgRomBank(id) =>
                params.set_bank_index_register(id, value),
            Command::NameTableMirroring => {
                let mirroring = NAME_TABLE_MIRRORINGS[usize::from(value) & 0b11];
                params.set_name_table_mirroring(mirroring);
            }
            Command::IrqControl => {
                self.irq_pending = false;
                self.irq_counter_enabled = value & 0b1000_0000 != 0;
                self.irq_enabled         = value & 0b0000_0001 != 0;
            }
            Command::IrqCounterLowByte =>
                set_bits(&mut self.irq_counter, u16::from(value)     , 0b0000_0000_1111_1111),
            Command::IrqCounterHighByte =>
                set_bits(&mut self.irq_counter, u16::from(value) << 8, 0b1111_1111_0000_0000),
        }
    }
}

enum Command {
    ChrRomBank(BankIndexRegisterId),
    PrgRomRamBank,
    PrgRomBank(BankIndexRegisterId),
    NameTableMirroring,
    IrqControl,
    IrqCounterLowByte,
    IrqCounterHighByte,
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
