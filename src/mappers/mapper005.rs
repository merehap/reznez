use crate::memory::bank::bank::RomRamModeRegisterId;
use crate::memory::bank::bank_index::MemoryType;
use crate::mapper::*;
use crate::mappers::mmc5::frame_state::FrameState;
use crate::memory::raw_memory::RawMemoryArray;
use crate::ppu::name_table::name_table;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::register::registers::ctrl::Ctrl;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    // Mode 0
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 1
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).status_register(S1).rom_ram_register(R1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 2
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).status_register(S1).rom_ram_register(R1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P3).status_register(S1).rom_ram_register(R2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 3
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P1).status_register(S1).rom_ram_register(R0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).status_register(S1).rom_ram_register(R1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P3).status_register(S1).rom_ram_register(R2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    .prg_layout_index(3)
    .override_prg_bank_register(P4, -1)

    .chr_rom_max_size(1024 * KIBIBYTE)
    // Normal sprite height layouts
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::RAM.switchable(C7)),
    ])

    // Tall sprite height layouts
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C9)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C9)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C8)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C9)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C10)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C8)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C9)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C10)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM.switchable(C11)),
    ])
    .do_not_align_large_chr_windows()
    .read_write_statuses(&[
        ReadWriteStatus::ReadOnly,
        ReadWriteStatus::ReadWrite,
        // Write-only is only used by Extended RAM (S0).
        ReadWriteStatus::WriteOnly,
    ])
    .build();

// Indexes into the above RAM statuses.
const READ_ONLY: u8 = 0;
const READ_WRITE: u8 = 1;
const WRITE_ONLY: u8 = 2;

const EXTENDED_RAM_MODES: [ExtendedRamMode; 4] = [
    ExtendedRamMode::WriteOnly,
    ExtendedRamMode::ExtendedAttributes,
    ExtendedRamMode::ReadWrite,
    ExtendedRamMode::ReadOnly,
];

const CHR_WINDOW_MODES: [ChrWindowMode; 4] = [
    ChrWindowMode::One8K,
    ChrWindowMode::Two4K,
    ChrWindowMode::Four2K,
    ChrWindowMode::Eight1K,
];

// MMC5
// TODO: Expansion Audio
// TODO: MMC5A registers
pub struct Mapper005 {
    ram_enabled_1: bool,
    ram_enabled_2: bool,

    extended_ram_mode: ExtendedRamMode,

    multiplicand: u8,
    multiplier: u8,

    // In fill mode, all of the name table values are the same so this is an inefficient
    // representation. However, it's necessary in order to work with the NameTable type that
    // requires stores a slice.
    fill_mode_name_table: [u8; KIBIBYTE as usize],

    chr_window_mode: ChrWindowMode,
    sprite_height: SpriteHeight,
    tall_sprite_background_enabled: bool,

    irq_enabled: bool,
    frame_state: FrameState,

    extended_ram: RawMemoryArray<KIBIBYTE>,
}

impl Mapper for Mapper005 {
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_addr: u16) -> ReadResult {
        match cpu_addr {
            0x0000..=0x401F => unreachable!(),
            0x5204 => ReadResult::full(self.frame_state.to_status_byte()),
            0x5205 => ReadResult::full((u16::from(self.multiplicand) * u16::from(self.multiplier)) as u8),
            0x5206 => ReadResult::full(((u16::from(self.multiplicand) * u16::from(self.multiplier)) >> 8) as u8),
            0x4020..=0x5BFF => ReadResult::OPEN_BUS,
            // TODO: ReadWriteStatus
            0x5C00..=0x5FFF => ReadResult::full(self.extended_ram[u32::from(cpu_addr - 0x5C00)]),
            0x6000..=0xFFFF => params.peek_prg(cpu_addr),
        }
    }

    fn on_cpu_read(&mut self, params: &mut MapperParams, addr: CpuAddress, _value: u8) {
        match addr.to_raw() {
            0x5204 => {
                params.set_irq_pending(false);
                self.frame_state.acknowledge_irq();
            }
            // NMI vector low and high
            0xFFFA | 0xFFFB => {
                params.set_irq_pending(false);
                self.frame_state.acknowledge_irq();
                self.frame_state.force_end_frame();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_cpu_write(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            // PPU Ctrl
            0x2000 => {
                self.sprite_height = Ctrl::from_u8(value).sprite_height();
                self.update_chr_layout(params);
            }
            // PPU Mask
            0x2001 => {
                // TODO: Disable substitutions.
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, addr: PpuAddress, _value: u8) {
        self.frame_state.sync_frame_status(addr);

        // Syncing the frame status may have switched in or out of special background banking mode.
        self.update_chr_layout(params);

        if self.irq_enabled && self.frame_state.irq_pending() {
            params.set_irq_pending(true);
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _params: &mut MapperParams, _cycle: i64) {
        self.frame_state.maybe_end_frame();
    }

    fn write_register(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5015 => { /* TODO: MMC5 audio */ }
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => params.set_prg_layout(value & 0b11),
            0x5101 => self.set_chr_layout(params, value),
            0x5102 => {
                self.ram_enabled_1 = value & 0b11 == 0b10;
                let status = if self.ram_enabled_1 && self.ram_enabled_2 {
                    READ_WRITE
                } else {
                    READ_ONLY
                };
                params.set_read_write_status(S1, status);
            }
            0x5103 => {
                self.ram_enabled_2 = value & 0b11 == 0b01;
                let status = if self.ram_enabled_1 && self.ram_enabled_2 {
                    READ_WRITE
                } else {
                    READ_ONLY
                };
                params.set_read_write_status(S1, status);
            }
            0x5104 => self.set_extended_ram_mode(params, value),
            0x5105 => Self::set_name_table_mirroring(params, value),
            0x5106 => self.set_fill_mode_name_table_byte(value),
            0x5107 => self.set_fill_mode_attribute_table_byte(value),
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113 => self.set_prg_bank_register(params, P0, None, value),
            0x5114 => self.set_prg_bank_register(params, P1, Some(R0), value),
            0x5115 => self.set_prg_bank_register(params, P2, Some(R1), value),
            0x5116 => self.set_prg_bank_register(params, P3, Some(R2), value),
            0x5117 => self.set_prg_bank_register(params, P4, None, value),
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120 => self.set_chr_bank_register(params, C0, value),
            0x5121 => self.set_chr_bank_register(params, C1, value),
            0x5122 => self.set_chr_bank_register(params, C2, value),
            0x5123 => self.set_chr_bank_register(params, C3, value),
            0x5124 => self.set_chr_bank_register(params, C4, value),
            0x5125 => self.set_chr_bank_register(params, C5, value),
            0x5126 => self.set_chr_bank_register(params, C6, value),
            0x5127 => self.set_chr_bank_register(params, C7, value),
            0x5128 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(params, C8, value);
            }
            0x5129 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(params, C9, value);
            }
            0x512A => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(params, C10, value);
            }
            0x512B => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(params, C11, value);
            }
            0x512C..=0x512F => { /* Do nothing. */ }
            0x5130 => { /* TODO. No official game relies on Upper CHR Bank bits, but a few initialize them. */ }
            0x5131..=0x51FF => { /* Do nothing. */ }
            0x5200 => self.enable_vertical_split_mode(value),
            0x5201 => todo!("Vertical split scroll"),
            0x5202 => todo!("Vertical split bank"),
            0x5203 => self.frame_state.set_target_irq_scanline(value),
            0x5204 => self.enable_irq(params, value),
            0x5205 => self.multiplicand = value,
            0x5206 => self.multiplier = value,
            0x5207..=0x5BFF => { /* Do nothing. */ }
            // TODO: ReadWriteStatus
            0x5C00..=0x5FFF => self.extended_ram[u32::from(cpu_address - 0x5C00)] = value,
            0x6000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn fill_mode_name_table(&self) -> &[u8; KIBIBYTE as usize] {
        &self.fill_mode_name_table
    }

    fn layout(&self) -> Layout {
        LAYOUT
    }
}

impl Mapper005 {
    pub fn new() -> Self {
         Self {
            ram_enabled_1: false,
            ram_enabled_2: false,

            extended_ram_mode: ExtendedRamMode::WriteOnly,

            multiplicand: 0xFF,
            multiplier: 0xFF,

            fill_mode_name_table: [0; KIBIBYTE as usize],

            chr_window_mode: ChrWindowMode::One8K,
            sprite_height: SpriteHeight::Normal,
            tall_sprite_background_enabled: false,

            irq_enabled: false,
            frame_state: FrameState::new(),

            extended_ram: RawMemoryArray::new(),
        }
    }


    // Write 0x5101
    fn set_chr_layout(&mut self, params: &mut MapperParams, value: u8) {
        self.chr_window_mode = CHR_WINDOW_MODES[usize::from(value & 0b11)];
        self.update_chr_layout(params);
    }

    // Write 0x5104
    fn set_extended_ram_mode(&mut self, params: &mut MapperParams, value: u8) {
        self.extended_ram_mode = EXTENDED_RAM_MODES[usize::from(value & 0b11)];
        let read_write_status = match self.extended_ram_mode {
            // FIXME: These are only write-only during rendering. They are supposed to
            // cause corruption during VBlank.
            ExtendedRamMode::WriteOnly | ExtendedRamMode::ExtendedAttributes => WRITE_ONLY,
            ExtendedRamMode::ReadWrite => READ_WRITE,
            ExtendedRamMode::ReadOnly => READ_ONLY,
        };
        params.set_read_write_status(S0, read_write_status);
    }

    // Write 0x5105
    fn set_name_table_mirroring(params: &mut MapperParams, value: u8) {
        fn source(raw: u8) -> NameTableSource {
            match raw {
                0 => NameTableSource::Ciram(CiramSide::Left),
                1 => NameTableSource::Ciram(CiramSide::Right),
                2 => NameTableSource::ExtendedRam,
                3 => NameTableSource::FillModeTile,
                _ => unreachable!(),
            }
        }

        let name_tables = splitbits!(value, "ddccbbaa");
        params.set_name_table_quadrant_to_source(NameTableQuadrant::TopLeft, source(name_tables.a));
        params.set_name_table_quadrant_to_source(NameTableQuadrant::TopRight, source(name_tables.b));
        params.set_name_table_quadrant_to_source(NameTableQuadrant::BottomLeft, source(name_tables.c));
        params.set_name_table_quadrant_to_source(NameTableQuadrant::BottomRight, source(name_tables.d));
    }

    // Write 0x5106
    fn set_fill_mode_name_table_byte(&mut self, value: u8) {
        // Set the fill-mode name table bytes but not the attribute table bytes.
        for i in 0..name_table::ATTRIBUTE_START_INDEX as usize {
            self.fill_mode_name_table[i] = value;
        }
    }

    // Write 0x5107
    fn set_fill_mode_attribute_table_byte(&mut self, value: u8) {
        let attribute = value & 0b11;
        let attribute_byte = (attribute << 6) | (attribute << 4) | (attribute << 2) | attribute;
        for i in name_table::ATTRIBUTE_START_INDEX as usize..self.fill_mode_name_table.len() {
            self.fill_mode_name_table[i] = attribute_byte;
        }
    }

    // Write 0x5113 through 0x5117
    fn set_prg_bank_register(
        &self,
        params: &mut MapperParams,
        id: PrgBankRegisterId,
        mode_reg_id: Option<RomRamModeRegisterId>,
        value: u8,
    ) {
        let (is_rom_mode, prg_bank) = splitbits_named!(value, "mppppppp");
        params.set_prg_register(id, prg_bank);
        if let Some(mode_reg_id) = mode_reg_id {
            let rom_ram_mode = if is_rom_mode { MemoryType::Rom } else { MemoryType::Ram };
            params.set_rom_ram_mode(mode_reg_id, rom_ram_mode);
        }
    }

    fn set_chr_bank_register(&mut self, params: &mut MapperParams, id: ChrBankRegisterId, value: u8) {
        params.set_chr_register(id, value);
        self.update_chr_layout(params);
    }

    // Write 0x5200
    fn enable_vertical_split_mode(&mut self, value: u8) {
        let fields = splitbits!(value, "es.ccccc");
        if fields.e {
            todo!("Vertical split mode");
        }
    }

    // Write 0x5204
    fn enable_irq(&mut self, params: &mut MapperParams, value: u8) {
        self.irq_enabled = value >> 7 == 1;
        if !self.irq_enabled {
            params.set_irq_pending(false);
        } else if self.frame_state.irq_pending() {
            params.set_irq_pending(true);
        }

    }

    fn update_chr_layout(&mut self, params: &mut MapperParams) {
        if self.sprite_height == SpriteHeight::Normal {
            self.tall_sprite_background_enabled = false;
        }

        let normal_background_mode =
            self.sprite_height == SpriteHeight::Normal ||
            self.frame_state.sprite_fetching() ||
            (!self.frame_state.in_frame() && !self.tall_sprite_background_enabled);

        let mut layout_index = self.chr_window_mode as u8;
        if !normal_background_mode {
            layout_index |= 0b100;
        }

        params.set_chr_layout(layout_index);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
enum ExtendedRamMode {
    WriteOnly,
    ExtendedAttributes,
    ReadWrite,
    ReadOnly,
}

#[derive(Clone, Copy)]
enum ChrWindowMode {
    One8K = 0,
    Two4K = 1,
    Four2K = 2,
    Eight1K = 3,
}
