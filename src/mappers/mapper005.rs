use crate::memory::bank::bank::{PrgSource, PrgSourceRegisterId};
use crate::mapper::*;
use crate::mappers::mmc5::frame_state::FrameState;
use crate::memory::memory::{Memory, SmallPage};
use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::ppu::constants::ATTRIBUTE_START_INDEX;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::register::registers::ctrl::Ctrl;

const LAYOUT: Layout = Layout::builder()
    .prg_rom_max_size(1024 * KIBIBYTE)
    // Mode 0
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 1
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).write_status(W1).rom_ram_register(PS1)),
        PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 2
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).write_status(W1).rom_ram_register(PS1)),
        PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P3).write_status(W1).rom_ram_register(PS2)),
        PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    // Mode 3
    .prg_layout(&[
        PrgWindow::new(0x6000, 0x7FFF, 8 * KIBIBYTE, PrgBank::WORK_RAM.switchable(P0)),
        PrgWindow::new(0x8000, 0x9FFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P1).write_status(W1).rom_ram_register(PS0)),
        PrgWindow::new(0xA000, 0xBFFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P2).write_status(W1).rom_ram_register(PS1)),
        PrgWindow::new(0xC000, 0xDFFF, 8 * KIBIBYTE, PrgBank::ROM_RAM.switchable(P3).write_status(W1).rom_ram_register(PS2)),
        PrgWindow::new(0xE000, 0xFFFF, 8 * KIBIBYTE, PrgBank::ROM.switchable(P4)),
    ])
    .prg_layout_index(3)
    .override_prg_bank_register(P4, -1)

    .chr_rom_max_size(1024 * KIBIBYTE)
    // Normal sprite height layouts
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C0)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C1)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C2)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C3)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C4)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C5)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C6)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C7)),
    ])

    // Tall sprite height layouts
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
        ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C9)),
        ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
        ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C9)),
        ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
    ])
    .chr_layout(&[
        ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C8)),
        ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C9)),
        ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C10)),
        ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
        ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C8)),
        ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C9)),
        ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C10)),
        ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrBank::ROM_OR_RAM.switchable(C11)),
    ])
    .do_not_align_large_chr_windows()
    .complicated_name_table_mirroring()
    .build();

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

const EXT_RAM_PAGE_INDEX: usize = 0;
const FILL_MODE_TILE_PAGE_INDEX: usize = 1;
const EXT_RAM_PEEK_SOURCE: PeekSource = PeekSource::MapperCustom { page_number: EXT_RAM_PAGE_INDEX as u8 };

// MMC5
// TODO: Expansion Audio
// TODO: MMC5A registers
pub struct Mapper005 {
    ram_enabled_1: bool,
    ram_enabled_2: bool,

    extended_ram_mode: ExtendedRamMode,

    multiplicand: u8,
    multiplier: u8,

    chr_window_mode: ChrWindowMode,
    sprite_height: SpriteHeight,
    tall_sprite_background_enabled: bool,

    frame_state: FrameState,

    substitutions_enabled: bool,
    name_table_index: u16,
    upper_chr_bank_bits: u8,
}

impl Mapper for Mapper005 {
    fn init_mapper_params(&self, mem: &mut Memory) {
        mem.mapper_custom_pages.push(SmallPage::new("ExtRAM".to_owned(), ReadStatus::Enabled, WriteStatus::Enabled));
        mem.mapper_custom_pages.push(SmallPage::new("FillModeTile".to_owned(), ReadStatus::Enabled, WriteStatus::Disabled));
    }

    fn peek_cartridge_space(&self, mem: &Memory, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x5204 => ReadResult::full(self.frame_state.to_status_byte()),
            0x5205 => ReadResult::full((u16::from(self.multiplicand) * u16::from(self.multiplier)) as u8),
            0x5206 => ReadResult::full(((u16::from(self.multiplicand) * u16::from(self.multiplier)) >> 8) as u8),
            0x4020..=0x5BFF => ReadResult::OPEN_BUS,
            // TODO: ReadWriteStatus
            0x5C00..=0x5FFF => ReadResult::full(self.peek_ext_rom(mem, *addr - 0x5C00)),
            0x6000..=0xFFFF => mem.peek_prg(addr),
        }
    }

    fn ppu_peek(&self, mem: &Memory, address: PpuAddress) -> PpuPeek {
        let should_substitute = self.substitutions_enabled
            && self.extended_ram_mode == ExtendedRamMode::ExtendedAttributes
            && !self.frame_state.sprite_fetching();

        match address.to_u16() {
            0x0000..=0x1FFF if should_substitute => {
                let lower_chr_bank_bits = self.peek_ext_rom(mem, self.name_table_index) & 0b0011_1111;
                let pattern_bank = (self.upper_chr_bank_bits << 6) | lower_chr_bank_bits;
                let raw_chr_index = 4 * KIBIBYTE * u32::from(pattern_bank) * KIBIBYTE + u32::from(address.to_u16() % 0x1000);
                mem.chr_memory().peek_raw(raw_chr_index)
            }
            0x0000..=0x1FFF => mem.chr_memory().peek(&mem.ciram, &mem.mapper_custom_pages, address),
            0x2000..=0x3EFF => self.peek_name_table_byte(&mem, &mem.ciram, address),
            0x3F00..=0x3FFF if should_substitute => {
                let palette = self.peek_ext_rom(mem, self.name_table_index) >> 6;
                // The same palette is used for all 4 corners.
                let palette_byte = palette << 6 | palette << 4 | palette << 2 | palette;
                PpuPeek::new(palette_byte, EXT_RAM_PEEK_SOURCE)
            }
            0x3F00..=0x3FFF => self.peek_palette_table_byte(&mem.palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn on_cpu_read(&mut self, mem: &mut Memory, addr: CpuAddress, _value: u8) {
        match *addr {
            0x5204 => {
                mem.cpu_pinout.acknowledge_mapper_irq();
                self.frame_state.acknowledge_irq();
            }
            // NMI vector low and high
            0xFFFA | 0xFFFB => {
                mem.cpu_pinout.acknowledge_mapper_irq();
                self.frame_state.acknowledge_irq();
                self.frame_state.force_end_frame();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_cpu_write(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            // PPU Ctrl
            0x2000 => {
                self.sprite_height = Ctrl::from_u8(value).sprite_height();
                self.update_chr_layout(mem);
            }
            // PPU Mask
            0x2001 => {
                self.substitutions_enabled = value & 0b11 != 0;
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_ppu_read(&mut self, mem: &mut Memory, addr: PpuAddress, _value: u8) {
        self.frame_state.sync_frame_status(addr);

        // Syncing the frame status may have switched in or out of special background banking mode.
        self.update_chr_layout(mem);

        if self.frame_state.irq_pending() {
            mem.cpu_pinout.assert_mapper_irq();
        }

        if addr.is_in_name_table_proper() {
            self.name_table_index = addr.to_u16() % 0x400;
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _mem: &mut Memory) {
        self.frame_state.maybe_end_frame();
    }

    fn write_register(&mut self, mem: &mut Memory, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5015 => { /* TODO: MMC5 audio */ }
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => mem.set_prg_layout(value & 0b11),
            0x5101 => self.set_chr_layout(mem, value),
            0x5102 => {
                self.ram_enabled_1 = value & 0b11 == 0b10;
                mem.set_writes_enabled(W1, self.ram_enabled_1 && self.ram_enabled_2);
            }
            0x5103 => {
                self.ram_enabled_2 = value & 0b11 == 0b01;
                mem.set_writes_enabled(W1, self.ram_enabled_1 && self.ram_enabled_2);
            }
            0x5104 => self.set_extended_ram_mode(mem, value),
            0x5105 => Self::set_name_table_mirroring(mem, value),
            0x5106 => self.set_fill_mode_name_table_byte(mem, value),
            0x5107 => self.set_fill_mode_attribute_table_byte(mem, value),
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113 => self.set_prg_bank_register(mem, P0, None, value),
            0x5114 => self.set_prg_bank_register(mem, P1, Some(PS0), value),
            0x5115 => self.set_prg_bank_register(mem, P2, Some(PS1), value),
            0x5116 => self.set_prg_bank_register(mem, P3, Some(PS2), value),
            0x5117 => self.set_prg_bank_register(mem, P4, None, value),
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120 => self.set_chr_bank_register(mem, C0, value),
            0x5121 => self.set_chr_bank_register(mem, C1, value),
            0x5122 => self.set_chr_bank_register(mem, C2, value),
            0x5123 => self.set_chr_bank_register(mem, C3, value),
            0x5124 => self.set_chr_bank_register(mem, C4, value),
            0x5125 => self.set_chr_bank_register(mem, C5, value),
            0x5126 => self.set_chr_bank_register(mem, C6, value),
            0x5127 => self.set_chr_bank_register(mem, C7, value),
            0x5128 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(mem, C8, value);
            }
            0x5129 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(mem, C9, value);
            }
            0x512A => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(mem, C10, value);
            }
            0x512B => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(mem, C11, value);
            }
            0x512C..=0x512F => { /* Do nothing. */ }
            0x5130 => self.upper_chr_bank_bits = value & 0b11,
            0x5131..=0x51FF => { /* Do nothing. */ }
            0x5200 => self.enable_vertical_split_mode(value),
            0x5201 => todo!("Vertical split scroll"),
            0x5202 => todo!("Vertical split bank"),
            0x5203 => self.frame_state.set_target_irq_scanline(value),
            0x5204 => self.enable_irq(mem, value),
            0x5205 => self.multiplicand = value,
            0x5206 => self.multiplier = value,
            0x5207..=0x5BFF => { /* Do nothing. */ }
            // TODO: ReadWriteStatus
            0x5C00..=0x5FFF => self.write_ext_rom(mem, *addr - 0x5C00, value),
            0x6000..=0xFFFF => { /* Do nothing. */ }
        }
    }

    fn irq_counter_info(&self) -> Option<IrqCounterInfo> {
        Some(self.frame_state.to_irq_counter_info())
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

            chr_window_mode: ChrWindowMode::One8K,
            sprite_height: SpriteHeight::Normal,
            tall_sprite_background_enabled: false,

            frame_state: FrameState::new(),

            substitutions_enabled: false,
            name_table_index: 0,
            upper_chr_bank_bits: 0b0000_0000,
        }
    }

    // Write 0x5101
    fn set_chr_layout(&mut self, mem: &mut Memory, value: u8) {
        self.chr_window_mode = CHR_WINDOW_MODES[usize::from(value & 0b11)];
        self.update_chr_layout(mem);
    }

    // Write 0x5104
    fn set_extended_ram_mode(&mut self, mem: &mut Memory, value: u8) {
        self.extended_ram_mode = EXTENDED_RAM_MODES[usize::from(value & 0b11)];
        // FIXME: WriteOnly and ExtendedAttributes are only write-only during rendering.
        // They are supposed to cause corruption during VBlank.
        let (read_status, write_status) = match self.extended_ram_mode {
            ExtendedRamMode::ReadOnly => (ReadStatus::Enabled, WriteStatus::Disabled),
            ExtendedRamMode::WriteOnly | ExtendedRamMode::ExtendedAttributes => (ReadStatus::ReadOnlyZeros, WriteStatus::Enabled),
            ExtendedRamMode::ReadWrite => (ReadStatus::Enabled, WriteStatus::Enabled),
        };
        mem.mapper_custom_pages[EXT_RAM_PAGE_INDEX].set_read_status(read_status);
        mem.mapper_custom_pages[EXT_RAM_PAGE_INDEX].set_write_status(write_status);
    }

    // Write 0x5105
    fn set_name_table_mirroring(mem: &mut Memory, value: u8) {
        fn source(raw: u8) -> NameTableSource {
            match raw {
                0 => NameTableSource::Ciram(CiramSide::Left),
                1 => NameTableSource::Ciram(CiramSide::Right),
                2 => NameTableSource::MapperCustom { page_number: EXT_RAM_PAGE_INDEX as u8 },
                3 => NameTableSource::MapperCustom { page_number: FILL_MODE_TILE_PAGE_INDEX as u8 },
                _ => unreachable!(),
            }
        }

        let name_tables = splitbits!(value, "ddccbbaa");
        mem.set_name_table_quadrant_to_source(NameTableQuadrant::TopLeft, source(name_tables.a));
        mem.set_name_table_quadrant_to_source(NameTableQuadrant::TopRight, source(name_tables.b));
        mem.set_name_table_quadrant_to_source(NameTableQuadrant::BottomLeft, source(name_tables.c));
        mem.set_name_table_quadrant_to_source(NameTableQuadrant::BottomRight, source(name_tables.d));
    }

    // Write 0x5106
    fn set_fill_mode_name_table_byte(&self, mem: &mut Memory, value: u8) {
        // The fill mode name table byte is not writeable except for right now.
        mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Enabled);
        // Set the fill-mode name table bytes but not the attribute table bytes.
        for i in 0..ATTRIBUTE_START_INDEX as u16 {
            mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].write(i, value);
        }

        mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Disabled);
    }

    // Write 0x5107
    fn set_fill_mode_attribute_table_byte(&mut self, mem: &mut Memory, value: u8) {
        // The fill mode attribute table byte is not writeable except for right now.
        mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Enabled);

        let attribute = value & 0b11;
        let attribute_byte = (attribute << 6) | (attribute << 4) | (attribute << 2) | attribute;
        for i in ATTRIBUTE_START_INDEX as u16 .. 0x400 {
            mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].write(i, attribute_byte);
        }

        mem.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Disabled);
    }

    // Write 0x5113 through 0x5117
    fn set_prg_bank_register(
        &self,
        mem: &mut Memory,
        id: PrgBankRegisterId,
        prg_source_reg_id: Option<PrgSourceRegisterId>,
        value: u8,
    ) {
        let fields = splitbits!(value, "mppppppp");
        mem.set_prg_register(id, fields.p);
        if let Some(prg_mode_reg_id) = prg_source_reg_id {
            let rom_ram_mode = [PrgSource::WorkRamOrRom, PrgSource::Rom][fields.m as usize];
            mem.set_rom_ram_mode(prg_mode_reg_id, rom_ram_mode);
        }
    }

    fn set_chr_bank_register(&mut self, mem: &mut Memory, id: ChrBankRegisterId, value: u8) {
        mem.set_chr_register(id, value);
        self.update_chr_layout(mem);
    }

    // Write 0x5200
    fn enable_vertical_split_mode(&mut self, value: u8) {
        let fields = splitbits!(value, "es.ccccc");
        if fields.e {
            todo!("Vertical split mode");
        }
    }

    // Write 0x5204
    fn enable_irq(&mut self, mem: &mut Memory, value: u8) {
        let irq_enabled = value >> 7 == 1;
        self.frame_state.set_irq_enabled(irq_enabled);
        if !irq_enabled {
            mem.cpu_pinout.acknowledge_mapper_irq();
        } else if self.frame_state.irq_pending() {
            mem.cpu_pinout.assert_mapper_irq();
        }
    }

    fn update_chr_layout(&mut self, mem: &mut Memory) {
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

        mem.set_chr_layout(layout_index);
    }

    fn peek_ext_rom(&self, mem: &Memory, index: u16) -> u8 {
        mem.mapper_custom_pages[EXT_RAM_PAGE_INDEX].peek(index).resolve(0).0
    }

    fn write_ext_rom(&self, mem: &mut Memory, index: u16, value: u8) {
        mem.mapper_custom_pages[EXT_RAM_PAGE_INDEX].write(index, value)
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
