use std::num::NonZeroI8;

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
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT0).switchable(N0).status_register(S0)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT1).switchable(N1).status_register(S0)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT2).switchable(N2).status_register(S0)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT3).switchable(N3).status_register(S0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT0).switchable(N0).status_register(S0)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT1).switchable(N1).status_register(S0)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT2).switchable(N2).status_register(S0)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT3).switchable(N3).status_register(S0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C0)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C2)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C4)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C6)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT0).switchable(N0).status_register(S0)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT1).switchable(N1).status_register(S0)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT2).switchable(N2).status_register(S0)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT3).switchable(N3).status_register(S0)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C0).status_register(S0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C1).status_register(S0)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C2).status_register(S0)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C3).status_register(S0)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C4).status_register(S0)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C5).status_register(S0)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C6).status_register(S0)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C7).status_register(S0)),
        ChrWindow::new(0x2000, 0x23FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT0).switchable(N0).status_register(S0)),
        ChrWindow::new(0x2400, 0x27FF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT1).switchable(N1).status_register(S0)),
        ChrWindow::new(0x2800, 0x2BFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT2).switchable(N2).status_register(S0)),
        ChrWindow::new(0x2C00, 0x2FFF, 1 * KIBIBYTE, ChrBank::with_switchable_source(NT3).switchable(N3).status_register(S0)),
    ])
    .name_table_mirrorings(&[
        NameTableMirroring::VERTICAL,
        NameTableMirroring::HORIZONTAL,
        NameTableMirroring::ONE_SCREEN_LEFT_BANK,
        NameTableMirroring::ONE_SCREEN_RIGHT_BANK,
    ])
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
    ])
    .build();

const IRQ_COUNTER: DirectlySetCounter = CounterBuilder::new()
    // This might be undefined at startup, since direction is set at the same time ticking is enabled.
    .step(1)
    .wraps(true)
    .full_range(0, 0xFF)
    .initial_count(0)
    .auto_trigger_when(AutoTriggerWhen::Wrapping)
    .when_disabled_prevent(WhenDisabledPrevent::CountingAndTriggering)
    .build_directly_set_counter();

const ONE: NonZeroI8 = NonZeroI8::new(1).unwrap();
const NEGATIVE_ONE: NonZeroI8 = NonZeroI8::new(-1).unwrap();

pub struct Mapper209 {
    irq_counter: DirectlySetCounter,
    irq_ticked_by: IrqTickedBy,
    irq_xor_value: u8,

    multiplicand: u8,
    multiplier: u8,
    multiplication_result: u16,

    extended_mode_mirroring_enabled: bool,
    extended_mirroring: NameTableMirroring,
    rom_name_table_mode: RomNameTableMode,
    ciram_selection_target: bool,
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
                let fields = splitbits!(value, "prrccppp");
                mem.prg_memory.set_layout(fields.p);
                mem.chr_memory.set_layout(fields.c);

                self.rom_name_table_mode = match fields.r {
                    0 | 2 => RomNameTableMode::Disabled,
                    1 => RomNameTableMode::SelectionsEnabled,
                    3 => RomNameTableMode::GloballyEnabled,
                    _ => unreachable!(),
                };
            }
            0xD001 => {
                let mirroring;
                (self.extended_mode_mirroring_enabled, mirroring) = splitbits_named!(value, "....e.mm");

                if self.extended_mode_mirroring_enabled {
                    mem.set_name_table_mirroring_directly(self.extended_mirroring);
                } else if self.rom_name_table_mode == RomNameTableMode::Disabled {
                    mem.set_name_table_mirroring(mirroring);
                }
            }
            0xD002 => {
                let chr_writes_enabled;
                (self.ciram_selection_target, chr_writes_enabled) = splitbits_named!(value, "nw......");
                mem.set_read_write_status(S0, chr_writes_enabled as u8);
            }
            0xD003 => todo!("Outer Bank"),
            _ => { /* Do nothing multiplication, PRG bank select, or mode related. */ }
        }

        match *addr & 0xF807 {
            0x9000 => mem.set_chr_register_low_byte(C0, value),
            0x9001 => mem.set_chr_register_low_byte(C1, value),
            0x9002 => mem.set_chr_register_low_byte(C2, value),
            0x9003 => mem.set_chr_register_low_byte(C3, value),
            0x9004 => mem.set_chr_register_low_byte(C4, value),
            0x9005 => mem.set_chr_register_low_byte(C5, value),
            0x9006 => mem.set_chr_register_low_byte(C6, value),
            0x9007 => mem.set_chr_register_low_byte(C7, value),
            0xA000 => mem.set_chr_register_high_byte(C0, value),
            0xA001 => mem.set_chr_register_high_byte(C1, value),
            0xA002 => mem.set_chr_register_high_byte(C2, value),
            0xA003 => mem.set_chr_register_high_byte(C3, value),
            0xA004 => mem.set_chr_register_high_byte(C4, value),
            0xA005 => mem.set_chr_register_high_byte(C5, value),
            0xA006 => mem.set_chr_register_high_byte(C6, value),
            0xA007 => mem.set_chr_register_high_byte(C7, value),
            0xB000..=0xB003 => {
                let quadrant = NameTableQuadrant::ALL[usize::from(*addr & 0b11)];
                let ciram_side = [CiramSide::Left, CiramSide::Right][usize::from(value & 1)];
                // TODO: Determine if extended mode mirroring takes precedence over ROM name tables, or vis-a-versa.
                if self.extended_mode_mirroring_enabled {
                    self.extended_mirroring.set_quadrant(quadrant, ciram_side);
                    mem.set_name_table_mirroring_directly(self.extended_mirroring);
                } else {
                    match self.rom_name_table_mode {
                        RomNameTableMode::Disabled => { /* Do nothing. */ }
                        RomNameTableMode::GloballyEnabled => {}
                        RomNameTableMode::SelectionsEnabled => {
                            let ciram_selection = (value >> 7) == 1;
                            if ciram_selection == self.ciram_selection_target {
                                self.extended_mirroring.set_quadrant(quadrant, ciram_side);
                                mem.set_name_table_mirroring_directly(self.extended_mirroring);
                            } else {
                                let reg_id = [N0, N1, N2, N3][usize::from(*addr & 0b11)];
                                // TODO: Actually switch to ROM/RAM source.
                                mem.set_chr_register_low_byte(reg_id, value);
                            }
                        }
                    }
                }
            }
            0xB004..=0xB007 => {
                if !self.extended_mode_mirroring_enabled {
                    match self.rom_name_table_mode {
                        RomNameTableMode::Disabled => { /* Do nothing. */ }
                        RomNameTableMode::GloballyEnabled => {}
                        RomNameTableMode::SelectionsEnabled => {
                            let ciram_selection = (value >> 7) == 1;
                            if ciram_selection != self.ciram_selection_target {
                                let reg_id = [N0, N1, N2, N3][usize::from(*addr & 0b11)];
                                mem.set_chr_register_high_byte(reg_id, value);
                            }
                        }
                    }
                }
            }
            _ => { /* Do nothing CHR related. */ }
        }

        match *addr & 0xF007 {
            0xC000 => {
                if value & 1 == 0 {
                    self.irq_counter.enable_triggering();
                } else {
                    self.irq_counter.disable_triggering();
                    mem.cpu_pinout.acknowledge_mapper_irq();
                }
            }
            0xC001 => {
                // IRQ mode
                let (counting_mode, unknown, use_prescaler_mask, irq_ticked_by) = splitbits_named!(value, "cc..upss");
                assert!(!unknown, "IRQ Unknown Mode Configuration is not supported yet.");

                let new_step = match counting_mode {
                    1 => Some(ONE),
                    2 => Some(NEGATIVE_ONE),
                    _ => None,
                };
                if let Some(new_step) = new_step {
                    self.irq_counter.enable_counting();
                    self.irq_counter.set_step(new_step);
                    self.irq_counter.set_prescaler_step(new_step);
                } else {
                    self.irq_counter.disable_counting();
                }

                let prescaler_mask = if use_prescaler_mask { 0x07 } else { 0xFF };
                self.irq_counter.set_prescaler_mask(prescaler_mask);

                self.irq_ticked_by = match irq_ticked_by {
                    0 => IrqTickedBy::CpuCycle,
                    1 => IrqTickedBy::PpuCycle,
                    2 => IrqTickedBy::PpuRead,
                    3 => IrqTickedBy::CpuWrite,
                    _ => unreachable!(),
                };
            }
            0xC002 => {
                self.irq_counter.disable_triggering();
                mem.cpu_pinout.acknowledge_mapper_irq();
            }
            0xC003 => self.irq_counter.enable_triggering(),
            0xC004 => self.irq_counter.set_prescaler_count(value & self.irq_xor_value),
            0xC005 => self.irq_counter.set_count(value & self.irq_xor_value),
            0xC006 => self.irq_xor_value = value,
            0xC007 => todo!("Unknown mode"),
            _ => { /* Do nothing IRQ related. */ }
        }
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper209 {
    pub fn new() -> Self {
        Self {
            irq_counter: IRQ_COUNTER,
            // Unused starting value.
            irq_ticked_by: IrqTickedBy::CpuCycle,
            irq_xor_value: 0,

            multiplicand: 0,
            multiplier: 0,
            multiplication_result: 0,

            extended_mode_mirroring_enabled: false,
            extended_mirroring: NameTableMirroring::ONE_SCREEN_LEFT_BANK,
            rom_name_table_mode: RomNameTableMode::Disabled,
            ciram_selection_target: false,
        }
    }
}

#[derive(PartialEq, Eq)]
enum IrqTickedBy {
    CpuCycle,
    PpuCycle,
    PpuRead,
    CpuWrite,
}

#[derive(PartialEq, Eq)]
enum RomNameTableMode {
    Disabled,
    SelectionsEnabled,
    GloballyEnabled,
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