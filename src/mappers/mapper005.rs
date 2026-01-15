use crate::mapper::*;

use crate::bus::Bus;
use crate::memory::bank::bank::{PrgSource, PrgSourceRegisterId};
use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::small_page::SmallPage;
use crate::mappers::mmc5::frame_state::FrameState;
use crate::ppu::constants::ATTRIBUTE_START_INDEX;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::sprite::sprite_height::SpriteHeight;

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
    fn init_mapper_params(&self, bus: &mut Bus) {
        bus.mapper_custom_pages.push(SmallPage::new("ExtRAM".to_owned(), ReadStatus::Enabled, WriteStatus::Enabled));
        bus.mapper_custom_pages.push(SmallPage::new("FillModeTile".to_owned(), ReadStatus::Enabled, WriteStatus::Disabled));
    }

    fn ppu_peek(&self, bus: &Bus, address: PpuAddress) -> PpuPeek {
        let should_substitute = self.substitutions_enabled
            && self.extended_ram_mode == ExtendedRamMode::ExtendedAttributes
            && !self.frame_state.sprite_fetching();

        match address.to_u16() {
            0x0000..=0x1FFF if should_substitute => {
                let lower_chr_bank_bits = Self::peek_ext_rom(bus, self.name_table_index) & 0b0011_1111;
                let pattern_bank = (self.upper_chr_bank_bits << 6) | lower_chr_bank_bits;
                let raw_chr_index = 4 * KIBIBYTE * u32::from(pattern_bank) * KIBIBYTE + u32::from(address.to_u16() % 0x1000);
                bus.chr_memory().peek_raw(raw_chr_index)
            }
            0x0000..=0x1FFF => bus.chr_memory().peek(&bus.ciram, &bus.mapper_custom_pages, address),
            0x2000..=0x3EFF => bus.peek_name_table_byte(address),
            0x3F00..=0x3FFF if should_substitute => {
                let palette = Self::peek_ext_rom(bus, self.name_table_index) >> 6;
                // The same palette is used for all 4 corners.
                let palette_byte = palette << 6 | palette << 4 | palette << 2 | palette;
                PpuPeek::new(palette_byte, EXT_RAM_PEEK_SOURCE)
            }
            0x3F00..=0x3FFF => bus.palette_ram.peek(address.to_palette_ram_index()),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn on_cpu_read(&mut self, bus: &mut Bus, addr: CpuAddress, _value: u8) {
        match *addr {
            0x5204 => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.frame_state.acknowledge_irq();
            }
            // NMI vector low and high
            0xFFFA | 0xFFFB => {
                bus.cpu_pinout.acknowledge_mapper_irq();
                self.frame_state.acknowledge_irq();
                self.frame_state.force_end_frame();
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_cpu_write(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            // PPU Ctrl
            0x2000 => {
                self.sprite_height = if (value & 0b0010_0000) == 0 { SpriteHeight::Normal } else { SpriteHeight::Tall };
                self.update_chr_layout(bus);
            }
            // PPU Mask
            0x2001 => {
                self.substitutions_enabled = value & 0b11 != 0;
            }
            _ => { /* Do nothing. */ }
        }
    }

    fn on_ppu_read(&mut self, bus: &mut Bus, addr: PpuAddress, _value: u8) {
        self.frame_state.sync_frame_status(addr);

        // Syncing the frame status may have switched in or out of special background banking mode.
        self.update_chr_layout(bus);

        if self.frame_state.irq_pending() {
            bus.cpu_pinout.assert_mapper_irq();
        }

        if addr.is_in_name_table_proper() {
            self.name_table_index = addr.to_u16() % 0x400;
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _bus: &mut Bus) {
        self.frame_state.maybe_end_frame();
    }

    fn peek_register(&self, bus: &Bus, addr: CpuAddress) -> ReadResult {
        match *addr {
            0x5204 => ReadResult::full(self.frame_state.to_status_byte()),
            0x5205 => ReadResult::full((u16::from(self.multiplicand) * u16::from(self.multiplier)) as u8),
            0x5206 => ReadResult::full(((u16::from(self.multiplicand) * u16::from(self.multiplier)) >> 8) as u8),
            0x5C00..=0x5FFF => ReadResult::full(Self::peek_ext_rom(bus, *addr - 0x5C00)),
            _ => ReadResult::OPEN_BUS,
        }
    }

    fn write_register(&mut self, bus: &mut Bus, addr: CpuAddress, value: u8) {
        match *addr {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5015 => { /* TODO: MMC5 audio */ }
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => bus.set_prg_layout(value & 0b11),
            0x5101 => self.set_chr_layout(bus, value),
            0x5102 => {
                self.ram_enabled_1 = value & 0b11 == 0b10;
                bus.set_writes_enabled(W1, self.ram_enabled_1 && self.ram_enabled_2);
            }
            0x5103 => {
                self.ram_enabled_2 = value & 0b11 == 0b01;
                bus.set_writes_enabled(W1, self.ram_enabled_1 && self.ram_enabled_2);
            }
            0x5104 => self.set_extended_ram_mode(bus, value),
            0x5105 => Self::set_name_table_mirroring(bus, value),
            0x5106 => Self::set_fill_mode_name_table_byte(bus, value),
            0x5107 => Self::set_fill_mode_attribute_table_byte(bus, value),
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113 => Self::set_prg_bank_register(bus, P0, None, value),
            0x5114 => Self::set_prg_bank_register(bus, P1, Some(PS0), value),
            0x5115 => Self::set_prg_bank_register(bus, P2, Some(PS1), value),
            0x5116 => Self::set_prg_bank_register(bus, P3, Some(PS2), value),
            0x5117 => Self::set_prg_bank_register(bus, P4, None, value),
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120 => self.set_chr_bank_register(bus, C0, value),
            0x5121 => self.set_chr_bank_register(bus, C1, value),
            0x5122 => self.set_chr_bank_register(bus, C2, value),
            0x5123 => self.set_chr_bank_register(bus, C3, value),
            0x5124 => self.set_chr_bank_register(bus, C4, value),
            0x5125 => self.set_chr_bank_register(bus, C5, value),
            0x5126 => self.set_chr_bank_register(bus, C6, value),
            0x5127 => self.set_chr_bank_register(bus, C7, value),
            0x5128 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(bus, C8, value);
            }
            0x5129 => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(bus, C9, value);
            }
            0x512A => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(bus, C10, value);
            }
            0x512B => {
                self.tall_sprite_background_enabled = true;
                self.set_chr_bank_register(bus, C11, value);
            }
            0x512C..=0x512F => { /* Do nothing. */ }
            0x5130 => self.upper_chr_bank_bits = value & 0b11,
            0x5131..=0x51FF => { /* Do nothing. */ }
            0x5200 => self.enable_vertical_split_mode(value),
            0x5201 => todo!("Vertical split scroll"),
            0x5202 => todo!("Vertical split bank"),
            0x5203 => self.frame_state.set_target_irq_scanline(value),
            0x5204 => self.enable_irq(bus, value),
            0x5205 => self.multiplicand = value,
            0x5206 => self.multiplier = value,
            0x5207..=0x5BFF => { /* Do nothing. */ }
            // TODO: ReadWriteStatus
            0x5C00..=0x5FFF => Self::write_ext_rom(bus, *addr - 0x5C00, value),
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
    fn set_chr_layout(&mut self, bus: &mut Bus, value: u8) {
        self.chr_window_mode = CHR_WINDOW_MODES[usize::from(value & 0b11)];
        self.update_chr_layout(bus);
    }

    // Write 0x5104
    fn set_extended_ram_mode(&mut self, bus: &mut Bus, value: u8) {
        self.extended_ram_mode = EXTENDED_RAM_MODES[usize::from(value & 0b11)];
        // FIXME: WriteOnly and ExtendedAttributes are only write-only during rendering.
        // They are supposed to cause corruption during VBlank.
        let (read_status, write_status) = match self.extended_ram_mode {
            ExtendedRamMode::ReadOnly => (ReadStatus::Enabled, WriteStatus::Disabled),
            ExtendedRamMode::WriteOnly | ExtendedRamMode::ExtendedAttributes => (ReadStatus::ReadOnlyZeros, WriteStatus::Enabled),
            ExtendedRamMode::ReadWrite => (ReadStatus::Enabled, WriteStatus::Enabled),
        };
        bus.mapper_custom_pages[EXT_RAM_PAGE_INDEX].set_read_status(read_status);
        bus.mapper_custom_pages[EXT_RAM_PAGE_INDEX].set_write_status(write_status);
    }

    // Write 0x5105
    fn set_name_table_mirroring(bus: &mut Bus, value: u8) {
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
        bus.set_name_table_quadrant_to_source(NameTableQuadrant::TopLeft, source(name_tables.a));
        bus.set_name_table_quadrant_to_source(NameTableQuadrant::TopRight, source(name_tables.b));
        bus.set_name_table_quadrant_to_source(NameTableQuadrant::BottomLeft, source(name_tables.c));
        bus.set_name_table_quadrant_to_source(NameTableQuadrant::BottomRight, source(name_tables.d));
    }

    // Write 0x5106
    fn set_fill_mode_name_table_byte(bus: &mut Bus, value: u8) {
        // The fill mode name table byte is not writeable except for right now.
        bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Enabled);
        // Set the fill-mode name table bytes but not the attribute table bytes.
        for i in 0..ATTRIBUTE_START_INDEX as u16 {
            bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].write(i, value);
        }

        bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Disabled);
    }

    // Write 0x5107
    fn set_fill_mode_attribute_table_byte(bus: &mut Bus, value: u8) {
        // The fill mode attribute table byte is not writeable except for right now.
        bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Enabled);

        let attribute = value & 0b11;
        let attribute_byte = (attribute << 6) | (attribute << 4) | (attribute << 2) | attribute;
        for i in ATTRIBUTE_START_INDEX as u16 .. 0x400 {
            bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].write(i, attribute_byte);
        }

        bus.mapper_custom_pages[FILL_MODE_TILE_PAGE_INDEX].set_write_status(WriteStatus::Disabled);
    }

    // Write 0x5113 through 0x5117
    fn set_prg_bank_register(
        bus: &mut Bus,
        id: PrgBankRegisterId,
        prg_source_reg_id: Option<PrgSourceRegisterId>,
        value: u8,
    ) {
        let fields = splitbits!(value, "mppppppp");
        bus.set_prg_register(id, fields.p);
        if let Some(prg_mode_reg_id) = prg_source_reg_id {
            let rom_ram_mode = [PrgSource::WorkRamOrRom, PrgSource::Rom][fields.m as usize];
            bus.set_rom_ram_mode(prg_mode_reg_id, rom_ram_mode);
        }
    }

    fn set_chr_bank_register(&mut self, bus: &mut Bus, id: ChrBankRegisterId, value: u8) {
        bus.set_chr_register(id, value);
        self.update_chr_layout(bus);
    }

    // Write 0x5200
    #[allow(clippy::unused_self)]
    fn enable_vertical_split_mode(&mut self, value: u8) {
        let fields = splitbits!(value, "es.ccccc");
        if fields.e {
            todo!("Vertical split mode");
        }
    }

    // Write 0x5204
    fn enable_irq(&mut self, bus: &mut Bus, value: u8) {
        let irq_enabled = value >> 7 == 1;
        self.frame_state.set_irq_enabled(irq_enabled);
        if !irq_enabled {
            bus.cpu_pinout.acknowledge_mapper_irq();
        } else if self.frame_state.irq_pending() {
            bus.cpu_pinout.assert_mapper_irq();
        }
    }

    fn update_chr_layout(&mut self, bus: &mut Bus) {
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

        bus.set_chr_layout(layout_index);
    }

    fn peek_ext_rom(bus: &Bus, index: u16) -> u8 {
        bus.mapper_custom_pages[EXT_RAM_PAGE_INDEX].peek(index).resolve(0)
    }

    fn write_ext_rom(bus: &mut Bus, index: u16, value: u8) {
        bus.mapper_custom_pages[EXT_RAM_PAGE_INDEX].write(index, value);
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
