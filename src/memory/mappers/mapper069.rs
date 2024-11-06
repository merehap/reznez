use crate::memory::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(512 * KIBIBYTE)
    .prg_layout(&[
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::RAM.switchable(P0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::ROM.switchable(P1)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P2)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P3)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.fixed_index(-1)),
    ])
    .chr_max_size(256 * KIBIBYTE)
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C7)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::Vertical,
        NameTableMirroring::Horizontal,
        NameTableMirroring::OneScreenLeftBank,
        NameTableMirroring::OneScreenRightBank,
    ])
    .ram_statuses(&[
        RamStatus::ReadOnly,
        RamStatus::Disabled,
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
    ])
    .build();

const CHR_REGISTER_IDS: [BankRegisterId; 8] = [C0, C1, C2, C3, C4, C5, C6, C7];
// P0 is used by the ROM/RAM window, which gets special treatment.
const PRG_ROM_REGISTER_IDS: [BankRegisterId; 3] = [P1, P2, P3];

// Sunsoft FME-7
pub struct Mapper069 {
    command: Command,

    irq_pending: bool,
    irq_enabled: bool,
    irq_counter_enabled: bool,
    irq_counter: u16,
}

impl Mapper for Mapper069 {
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

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
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
                params.set_bank_register(id, value),
            Command::PrgRomRamBank => {
                let fields = splitbits!(value, "rrpppppp");
                params.set_ram_status(S0, fields.r);
                params.set_bank_register(P0, fields.p);
            }
            Command::PrgRomBank(id) =>
                params.set_bank_register(id, value),
            Command::NameTableMirroring =>
                params.set_name_table_mirroring(value & 0b11),
            Command::IrqControl => {
                self.irq_pending = false;
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
    ChrRomBank(BankRegisterId),
    PrgRomRamBank,
    PrgRomBank(BankRegisterId),
    NameTableMirroring,
    IrqControl,
    IrqCounterLowByte,
    IrqCounterHighByte,
}

fn set_bits(value: &mut u16, new_bits: u16, mask: u16) {
    *value = (*value & !mask) | (new_bits & mask);
}
