use crate::memory::mapper::*;
use crate::memory::mappers::mmc5::scanline_detector::ScanlineDetector;
use crate::ppu::name_table::name_table;
use crate::ppu::name_table::name_table_quadrant::NameTableQuadrant;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::ppu::register::registers::ctrl::Ctrl;

const LAYOUT: Layout = Layout::builder()
    .prg_max_size(1024 * KIBIBYTE)
    // Mode 0
    .prg_layout(&[
        Window::new(0x5C00, 0x5FFF,  1 * KIBIBYTE, Bank::EXTENDED_RAM.status_register(S0)),
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::RAM.switchable(P0)),
        Window::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::ROM.switchable(P4)),
    ])
    // Mode 1
    .prg_layout(&[
        Window::new(0x5C00, 0x5FFF,  1 * KIBIBYTE, Bank::EXTENDED_RAM.status_register(S0)),
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::RAM.switchable(P0)),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::RAM.switchable(P2).status_register(S1)),
        Window::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::ROM.switchable(P4)),
    ])
    // Mode 2
    .prg_layout(&[
        Window::new(0x5C00, 0x5FFF,  1 * KIBIBYTE, Bank::EXTENDED_RAM.status_register(S0)),
        Window::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::RAM.switchable(P0)),
        Window::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::RAM.switchable(P2).status_register(S1)),
        Window::new(0xC000, 0xDFFF,  8 * KIBIBYTE, Bank::RAM.switchable(P3).status_register(S1)),
        Window::new(0xE000, 0xFFFF,  8 * KIBIBYTE, Bank::ROM.switchable(P4)),
    ])
    // Mode 3
    .prg_layout(&[
        Window::new(0x5C00, 0x5FFF, 1 * KIBIBYTE, Bank::EXTENDED_RAM.status_register(S0)),
        Window::new(0x6000, 0x7FFF, 8 * KIBIBYTE, Bank::RAM.switchable(P0)),
        Window::new(0x8000, 0x9FFF, 8 * KIBIBYTE, Bank::RAM.switchable(P1).status_register(S1)),
        Window::new(0xA000, 0xBFFF, 8 * KIBIBYTE, Bank::RAM.switchable(P2).status_register(S1)),
        Window::new(0xC000, 0xDFFF, 8 * KIBIBYTE, Bank::RAM.switchable(P3).status_register(S1)),
        Window::new(0xE000, 0xFFFF, 8 * KIBIBYTE, Bank::ROM.switchable(P4)),
    ])
    .prg_layout_index(3)
    .override_bank_register(P4, -1)

    .chr_max_size(1024 * KIBIBYTE)
    // Normal sprite height layouts
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::RAM.switchable(C3)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::RAM.switchable(C1)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::RAM.switchable(C3)),
        Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::RAM.switchable(C5)),
        Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::RAM.switchable(C7)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::RAM.switchable(C0)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::RAM.switchable(C1)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C2)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C3)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::RAM.switchable(C4)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::RAM.switchable(C5)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::RAM.switchable(C6)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::RAM.switchable(C7)),
    ])

    // Tall sprite height layouts
    .chr_layout(&[
        Window::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C11)),
        Window::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::ROM.switchable(C9)),
        Window::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C11)),
        Window::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::ROM.switchable(C9)),
        Window::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::ROM.switchable(C11)),
    ])
    .chr_layout(&[
        Window::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::ROM.switchable(C8)),
        Window::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::ROM.switchable(C9)),
        Window::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C10)),
        Window::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C11)),
        Window::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::ROM.switchable(C8)),
        Window::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::ROM.switchable(C9)),
        Window::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::ROM.switchable(C10)),
        Window::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::ROM.switchable(C11)),
    ])
    .do_not_align_large_chr_windows()
    .ram_statuses(&[
        RamStatus::ReadOnly,
        RamStatus::ReadWrite,
        // Write-only is only used by Extended RAM (S0).
        RamStatus::WriteOnly,
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

    irq_enabled: bool,
    frame_state: FrameState,
}

impl Mapper for Mapper005 {
    fn peek_cartridge_space(&self, params: &MapperParams, cpu_addr: u16) -> ReadResult {
        match cpu_addr {
            0x0000..=0x401F => unreachable!(),
            0x5204 => {
                // TODO: Move this formatting to a FrameState method.
                let mut status = 0;
                if self.frame_state.irq_pending() {
                    status |= 0b1000_0000;
                }

                if self.frame_state.in_frame() {
                    status |= 0b0100_0000;
                }

                // TODO: Should the last 6 bits be open bus?
                ReadResult::full(status)
            }
            0x5205 => ReadResult::full((u16::from(self.multiplicand) * u16::from(self.multiplier)) as u8),
            0x5206 => ReadResult::full(((u16::from(self.multiplicand) * u16::from(self.multiplier)) >> 8) as u8),
            0x4020..=0x5BFF => ReadResult::OPEN_BUS,
            0x5C00..=0xFFFF => params.peek_prg(cpu_addr),
        }
    }

    fn on_cpu_read(&mut self, params: &mut MapperParams, addr: CpuAddress) {
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
                self.update_chr_layout(params, false);
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
        self.update_chr_layout(params, true);

        if self.irq_enabled && self.frame_state.irq_pending() {
            params.set_irq_pending(true);
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _params: &mut MapperParams, _cycle: i64) {
        self.frame_state.maybe_end_frame();
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, cpu_address: u16, value: u8) {
        match cpu_address {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000..=0x5015 => { /* TODO: MMC5 audio */ }
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => params.set_prg_layout(value & 0b11),
            0x5101 => {
                self.chr_window_mode = CHR_WINDOW_MODES[usize::from(value & 0b11)];
                self.update_chr_layout(params, false);
            }
            0x5102 => {
                self.ram_enabled_1 = value & 0b11 == 0b10;
                if !self.ram_enabled_1 {
                    params.set_ram_status(S1, READ_ONLY);
                }
            }
            0x5103 => {
                self.ram_enabled_2 = value & 0b11 == 0b01;
                if !self.ram_enabled_2 {
                    params.set_ram_status(S1, READ_ONLY);
                }
            }
            0x5104 => {
                self.extended_ram_mode = EXTENDED_RAM_MODES[usize::from(value & 0b11)];
                let ram_status = match self.extended_ram_mode {
                    // FIXME: These are only write-only during rendering. They are supposed to
                    // cause corruption during VBlank.
                    ExtendedRamMode::WriteOnly | ExtendedRamMode::ExtendedAttributes => WRITE_ONLY,
                    ExtendedRamMode::ReadWrite => READ_WRITE,
                    ExtendedRamMode::ReadOnly => READ_ONLY,
                };
                params.set_ram_status(S0, ram_status);
            }
            0x5105 => {
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
                params.name_table_mirroring_mut().set_quadrant_to_source(
                    NameTableQuadrant::TopLeft, source(name_tables.a));
                params.name_table_mirroring_mut().set_quadrant_to_source(
                    NameTableQuadrant::TopRight, source(name_tables.b));
                params.name_table_mirroring_mut().set_quadrant_to_source(
                    NameTableQuadrant::BottomLeft, source(name_tables.c));
                params.name_table_mirroring_mut().set_quadrant_to_source(
                    NameTableQuadrant::BottomRight, source(name_tables.d));
            }
            0x5106 => {
                // Set the fill-mode name table bytes but not the attribute table bytes.
                for i in 0..name_table::ATTRIBUTE_START_INDEX as usize {
                    self.fill_mode_name_table[i] = value;
                }
            }
            0x5107 => {
                // Set the fill-mode attribute table bytes.
                let attribute = value & 0b11;
                let attribute_byte = (attribute << 6) | (attribute << 4) | (attribute << 2) | attribute;
                for i in name_table::ATTRIBUTE_START_INDEX as usize..self.fill_mode_name_table.len() {
                    self.fill_mode_name_table[i] = attribute_byte;
                }
            }
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113 => {
                let prg_bank = splitbits_named!(value, ".ppppppp");
                params.set_bank_register(P0, prg_bank);
            }
            0x5114 => {
                let (ram_writable, prg_bank) = splitbits_named!(min=u8, value, "wppppppp");
                params.set_bank_register(P1, prg_bank);
                if self.ram_enabled_1 && self.ram_enabled_2 {
                    params.set_ram_status(S1, ram_writable);
                }
            }
            0x5115 => {
                let (ram_writable, prg_bank) = splitbits_named!(min=u8, value, "wppppppp");
                params.set_bank_register(P2, prg_bank);
                if self.ram_enabled_1 && self.ram_enabled_2 {
                    params.set_ram_status(S1, ram_writable);
                }
            }
            0x5116 => {
                let (ram_writable, prg_bank) = splitbits_named!(min=u8, value, "wppppppp");
                params.set_bank_register(P3, prg_bank);
                if self.ram_enabled_1 && self.ram_enabled_2 {
                    params.set_ram_status(S1, ram_writable);
                }
            }
            0x5117 => {
                let prg_bank = splitbits_named!(value, ".ppppppp");
                params.set_bank_register(P4, prg_bank);
            }
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120 => params.set_bank_register(C0, value),
            0x5121 => params.set_bank_register(C1, value),
            0x5122 => params.set_bank_register(C2, value),
            0x5123 => params.set_bank_register(C3, value),
            0x5124 => params.set_bank_register(C4, value),
            0x5125 => params.set_bank_register(C5, value),
            0x5126 => params.set_bank_register(C6, value),
            0x5127 => params.set_bank_register(C7, value),
            0x5128 => {
                if self.sprite_height == SpriteHeight::Tall {
                    params.set_bank_register(C8, value);
                }
            }
            0x5129 => {
                if self.sprite_height == SpriteHeight::Tall {
                    params.set_bank_register(C9, value);
                }
            }
            0x512A => {
                if self.sprite_height == SpriteHeight::Tall {
                    params.set_bank_register(C10, value);
                }
            }
            0x512B => {
                if self.sprite_height == SpriteHeight::Tall {
                    params.set_bank_register(C11, value);
                }
            }
            0x512C..=0x512F => { /* Do nothing. */ }
            0x5130 => { /* TODO. No official game relies on Upper CHR Bank bits, but a few initialize them. */ }
            0x5131..=0x51FF => { /* Do nothing. */ }
            0x5200 => {
                let fields = splitbits!(value, "es.ccccc");
                if fields.e {
                    todo!("Vertical split mode");
                }
            }
            0x5201 => todo!("Vertical split scroll"),
            0x5202 => todo!("Vertical split bank"),
            0x5203 => self.frame_state.set_target_irq_scanline(value),
            0x5204 => {
                self.irq_enabled = value >> 7 == 1;
                if !self.irq_enabled {
                    params.set_irq_pending(false);
                } else if self.frame_state.irq_pending() {
                    params.set_irq_pending(true);
                }

            }
            0x5205 => self.multiplicand = value,
            0x5206 => self.multiplier = value,
            0x5207..=0xFFFF => { /* Do nothing. */ }
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

            irq_enabled: false,
            frame_state: FrameState::new(),
        }
    }


    fn update_chr_layout(&mut self, params: &mut MapperParams, dedup_logging: bool) {
        let mut layout_index = self.chr_window_mode as u8;
        let special_background_mode =
            self.sprite_height == SpriteHeight::Tall && !self.frame_state.sprite_fetching();
        if special_background_mode {
            layout_index |= 0b100;
        }

        if dedup_logging {
            params.set_chr_layout_dedup_logging(layout_index);
        } else {
            params.set_chr_layout(layout_index);
        }
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

const SPRITE_PATTERN_FETCH_START: u8 = 64;
const BACKGROUND_PATTERN_FETCH_START: u8 = 81;

struct FrameState {
    stage: FrameStage,
    scanline_detector: ScanlineDetector,
    ppu_is_reading: bool,
    scanline: u8,
    irq_target_scanline: u8,
    irq_pending: bool,

    pattern_fetch_count: u8,
}

impl FrameState {
    fn new() -> Self {
        Self {
            stage: FrameStage::OutOfFrame,
            scanline_detector: ScanlineDetector::new(),
            ppu_is_reading: false,
            scanline: 0,
            // A target of 0 means IRQs are disabled (unless one was already pending).
            irq_target_scanline: 0,
            irq_pending: false,

            pattern_fetch_count: 0,
        }
    }

    fn in_frame(&self) -> bool {
        self.stage != FrameStage::OutOfFrame
    }

    fn irq_pending(&self) -> bool {
        self.irq_pending
    }

    fn sprite_fetching(&self) -> bool {
        (SPRITE_PATTERN_FETCH_START..BACKGROUND_PATTERN_FETCH_START)
            .contains(&self.pattern_fetch_count)
    }

    // Called every PPU read.
    fn sync_frame_status(&mut self, addr: PpuAddress) {
        self.ppu_is_reading = true;

        if self.in_frame() || self.scanline_detector.scanline_detected() {
            self.stage = FrameStage::InFrame0;
        }

        let new_scanline_detected = self.scanline_detector.step(addr);
        if new_scanline_detected {
            if self.in_frame() {
                self.scanline += 1;
                if self.scanline == self.irq_target_scanline {
                    self.irq_pending = true;
                }
            } else {
                self.scanline = 0;
                self.irq_pending = false;
            }

            self.pattern_fetch_count = 0;
        }

        if addr.to_u16() < 0x2000 {
            self.pattern_fetch_count += 1;
        }
    }

    // Called every CPU cycle.
    fn maybe_end_frame(&mut self) {
        if !self.ppu_is_reading {
            return;
        }

        use FrameStage::*;
        self.stage = match self.stage {
            // Advance the stage.
            InFrame0 => InFrame1,
            InFrame1 => InFrame2,
            // No PPU reads occurred for 3 PPU cycles, rendering must have been disabled.
            InFrame2 => OutOfFrame,
            OutOfFrame => OutOfFrame,
        };

        self.ppu_is_reading = false;
    }

    // Called on PPU mask (0x2001) write, and on NMI vector (0xFFFA or 0xFFFB) read.
    fn force_end_frame(&mut self) {
        self.stage = FrameStage::OutOfFrame
    }

    // Called on 0x5203 write.
    fn set_target_irq_scanline(&mut self, target: u8) {
        self.irq_target_scanline = target;
    }

    // Called on 0x5204 read, and on NMI vector (0xFFFA or 0xFFFB) read.
    fn acknowledge_irq(&mut self) {
        self.irq_pending = false;
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum FrameStage {
    InFrame0,
    InFrame1,
    InFrame2,
    OutOfFrame,
}
