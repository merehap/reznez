use crate::mapper::*;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P5)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P6)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])

    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P5)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P6)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P7)),
    ])

    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P9)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P8)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P5)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P6)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.fixed_index(-1)),
    ])

    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P9)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P8)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
    ])
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P3)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P5)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P6)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P7)),
    ])

    .chr_rom_max_size(1024 * KIBIBYTE)
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
    ])
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
    .build();

pub struct Mapper209 {
    multiplicand: u8,
    multiplier: u8,
    multiplication_result: u16,
}

impl Mapper for Mapper209 {
    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        if matches!(*addr, 0x5000 | 0x5400 | 0x5C00) {
            todo!("Jumper Register");
        }

        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x5000 | 0x5400 | 0x5C00 => todo!("Jumper register"),
            0x6000..=0xFFFF => mem.peek_prg(addr),
            _ => match *addr & 0xF803 {
                0x5800 => ReadResult::full(self.multiplication_result as u8),
                0x5801 => ReadResult::full((self.multiplication_result >> 8) as u8),
                0x5802 => todo!("Read Accumulator"),
                0x5803 => todo!("Read Test Register"),
                _ => ReadResult::OPEN_BUS,
            }
        }
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr & 0xF803 {
            0x5800 => self.multiplicand = value,
            0x5801 => {
                self.multiplier = value;
                // FIXME: This is supposed to be calculated over 6 CPU cycles, with the intermediate results being visible.
                self.multiplication_result = u16::from(self.multiplicand) * u16::from(self.multiplier);
            }
            0x5802 => todo!("Increase accumulator"),
            0x5803 => todo!("Reset accumulator"),
            0x8000 => {
                mem.set_prg_register(P0, value & 0b0111_1111);
                mem.set_prg_register(P4, reverse_lower_seven_bits(value));
            }
            0x8001 => {
                mem.set_prg_register(P1, value & 0b0111_1111);
                mem.set_prg_register(P5, reverse_lower_seven_bits(value));
            }
            0x8002 => {
                mem.set_prg_register(P2, value & 0b0111_1111);
                mem.set_prg_register(P6, reverse_lower_seven_bits(value));
            }
            0x8003 => {
                mem.set_prg_register(P3, value & 0b0111_1111);
                mem.set_prg_register(P7, reverse_lower_seven_bits(value));
                mem.set_prg_register(P8, (value << 1) | 0b1);
                mem.set_prg_register(P9, (value << 2) | 0b11);
                //mem.set_prg_register(P9, reverse_lower_seven_bits(value));
            }
            0xD000 => {
                let fields = splitbits!(value, "pgnccppp");
                mem.prg_memory.set_layout(fields.p);
                mem.chr_memory.set_layout(fields.c);
                assert_eq!(fields.g, false, "ROM name tables not supported yet");
                assert_eq!(fields.n, false, "ROM name tables not supported yet");
            }
            0xD001 => {
                let fields = splitbits!(value, "....e.mm");
                assert!(!fields.e, "Extended mode mirroring isn't supported yet.");
                mem.set_name_table_mirroring(fields.m);
            }
            0xD002 => todo!("PPU Address Space"),
            0xD003 => todo!("Outer Bank"),
            _ => { /* Do nothing multiplication, PRG bank select, or mode related. */ }
        }

        match *addr & 0xF807 {
            0x9000 => mem.set_chr_bank_register_bits(C0, u16::from(value), 0b0000_0000_1111_1111),
            0x9001 => mem.set_chr_bank_register_bits(C1, u16::from(value), 0b0000_0000_1111_1111),
            0x9002 => mem.set_chr_bank_register_bits(C2, u16::from(value), 0b0000_0000_1111_1111),
            0x9003 => mem.set_chr_bank_register_bits(C3, u16::from(value), 0b0000_0000_1111_1111),
            0xA000 => mem.set_chr_bank_register_bits(C4, u16::from(value) << 8, 0b1111_1111_0000_0000),
            0xA001 => mem.set_chr_bank_register_bits(C5, u16::from(value) << 8, 0b1111_1111_0000_0000),
            0xA002 => mem.set_chr_bank_register_bits(C6, u16::from(value) << 8, 0b1111_1111_0000_0000),
            0xA003 => mem.set_chr_bank_register_bits(C7, u16::from(value) << 8, 0b1111_1111_0000_0000),
            0xB000..=0xB003 => todo!("NameTable LSB Bank Select"),
            0xB004..=0xB007 => todo!("NameTable MSB Bank Select"),
            _ => { /* Do nothing CHR related. */}
        }

        match *addr & 0xF007 {
            0xC000 => todo!("IRQ Enable/Disable"),
            0xC001 => todo!("IRQ Mode Select"),
            0xC002 => todo!("IRQ Disable"),
            0xC003 => todo!("IRQ Enable"),
            0xC004 => todo!("Prescaler"),
            0xC005 => todo!("Counter"),
            0xC006 => todo!("XOR"),
            0xC007 => todo!("Unknown mode"),
            _ => { /* Do nothing IRQ related. */}
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper209 {
    pub fn new() -> Self {
        Self {
            multiplicand: 0,
            multiplier: 0,
            multiplication_result: 0,
        }
    }
}

fn reverse_lower_seven_bits(mut value: u8) -> u8 {
    // Drop the top bit, since we're only reversing the bottom 7 bits.
    value <<= 1;

    let mut result = 0;
    for i in 0..7 {
        result |= (value >> 7) << i;
        value <<= 1;
    }

    result
}