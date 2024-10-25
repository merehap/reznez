use crate::apu::pulse_channel::PulseChannel;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::ppu::register::registers::ctrl::Ctrl;
use crate::ppu::sprite::sprite_height::SpriteHeight;
use crate::memory::mapper::*;
use crate::memory::memory::{NMI_VECTOR_LOW, NMI_VECTOR_HIGH};
use crate::memory::ppu::ppu_internal_ram::PpuInternalRam;
use crate::memory::ppu::vram::VramSide;

const ONE_32K_PRG_WINDOW: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_ram(P0)),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, Bank::switchable_rom(P4)),
]);

const TWO_16K_PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_ram(P0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_ram(P2).status_register(S0)),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, Bank::switchable_rom(P4)),
]);

const ONE_16K_AND_TWO_8K_PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_ram(P0)),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, Bank::switchable_ram(P2).status_register(S0)),
    PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, Bank::switchable_ram(P3).status_register(S0)),
    PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, Bank::switchable_rom(P4)),
]);

const FOUR_8K_PRG_WINDOWS: PrgLayout = PrgLayout::new(&[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, Bank::switchable_ram(P0)),
    PrgWindow::new(0x8000, 0x9FFF,  8 * KIBIBYTE, Bank::switchable_ram(P1).status_register(S0)),
    PrgWindow::new(0xA000, 0xBFFF,  8 * KIBIBYTE, Bank::switchable_ram(P2).status_register(S0)),
    PrgWindow::new(0xC000, 0xDFFF,  8 * KIBIBYTE, Bank::switchable_ram(P3).status_register(S0)),
    PrgWindow::new(0xE000, 0xFFFF,  8 * KIBIBYTE, Bank::switchable_rom(P4)),
]);

const ONE_8K_CHR_WINDOW: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C7)),
]);
const ONE_8K_CHR_WINDOW_ALTERNATE: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, Bank::switchable_rom(C11)),
]);

const TWO_4K_CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C7)),
]);
const TWO_4K_CHR_LAYOUT_ALTERNATE: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C11)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C11)),
]);

const FOUR_2K_CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::switchable_rom(C5)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::switchable_rom(C7)),
]);
const FOUR_2K_CHR_LAYOUT_ALTERNATE: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, Bank::switchable_rom(C9)),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, Bank::switchable_rom(C11)),
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, Bank::switchable_rom(C9)),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, Bank::switchable_rom(C11)),
]);

const EIGHT_1K_CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C0)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C1)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C2)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C3)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C4)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C5)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C6)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C7)),
]);
const EIGHT_1K_CHR_LAYOUT_ALTERNATE: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, Bank::switchable_rom(C8)),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, Bank::switchable_rom(C9)),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, Bank::switchable_rom(C10)),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, Bank::switchable_rom(C11)),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, Bank::switchable_rom(C8)),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, Bank::switchable_rom(C9)),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, Bank::switchable_rom(C10)),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, Bank::switchable_rom(C11)),
]);

const EXTENDED_ATTRIBUTES_CHR_LAYOUT: ChrLayout = ChrLayout::new(&[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, Bank::switchable_rom(C12)),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, Bank::switchable_rom(C12)),
]);

const SPRITE_PATTERN_FETCH_START: u8 = 64;
const BACKGROUND_PATTERN_FETCH_START: u8 = 81;

const PRG_REGISTER_IDS: [BankRegisterId; 5] =
    [P0,P1, P2, P3, P4];
const CHR_REGISTER_IDS: [BankRegisterId; 12] =
    [C0, C1, C2, C3, C4, C5, C6, C7, C8, C9, C10, C11];

const PRG_LAYOUTS: [PrgLayout; 4] = [
    ONE_32K_PRG_WINDOW,
    TWO_16K_PRG_WINDOWS,
    ONE_16K_AND_TWO_8K_PRG_WINDOWS,
    FOUR_8K_PRG_WINDOWS,
];
const CHR_WINDOW_MODES: [ChrWindowMode; 4] = [
    ChrWindowMode::One8K,
    ChrWindowMode::Two4K,
    ChrWindowMode::Four2K,
    ChrWindowMode::Eight1K,
];

const EXTENDED_RAM_MODES: [ExtendedRamMode; 4] = [
    ExtendedRamMode::WriteOnly,
    ExtendedRamMode::ExtendedAttributes,
    ExtendedRamMode::ReadWrite,
    ExtendedRamMode::ReadOnly,
];

const NAME_TABLE_SOURCES: [NameTableSource; 4] = [
    NameTableSource::CiramLeft,
    NameTableSource::CiramRight,
    NameTableSource::ExtendedRam,
    NameTableSource::Fill,
];

// MMC5 (ExROM)
pub struct Mapper005 {
    pulse_2: PulseChannel,
    pulse_3: PulseChannel,

    prg_ram_enabled_1: bool,
    prg_ram_enabled_2: bool,

    extended_ram: [u8; 1 * KIBIBYTE],
    extended_ram_mode: ExtendedRamMode,
    name_table_sources: [NameTableSource; 4],
    fill_mode_tile: u8,
    fill_mode_palette_index: Option<PaletteIndex>,

    scanline_irq_enabled: bool,
    irq_scanline: u8,
    current_scanline: u8,
    irq_pending: bool,
    in_frame: bool,
    previous_ppu_address_read: Option<PpuAddress>,
    consecutive_reads_of_same_address: u8,
    cpu_cycles_since_last_ppu_read: u8,
    ppu_read_occurred_since_last_cpu_cycle: bool,

    chr_window_mode: ChrWindowMode,
    sprite_height: SpriteHeight,
    pattern_fetch_count: u8,
    upper_chr_bank_bits: u8,

    multiplicand: u8,
    multiplier: u8,
}

impl Mapper for Mapper005 {
    fn initial_layout(&self) -> InitialLayout {
        InitialLayout::builder()
            .prg_max_bank_count(128)
            .prg_bank_size(8 * KIBIBYTE)
            .prg_layout(FOUR_8K_PRG_WINDOWS)
            .chr_max_bank_count(1024)
            .chr_bank_size(1 * KIBIBYTE)
            .chr_layout(ONE_8K_CHR_WINDOW)
            .do_not_align_large_chr_layout()
            .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
            .override_bank_register(P4, BankIndex::LAST)
            .build()
    }

    fn peek_cartridge_space(&self, params: &MapperParams, address: CpuAddress) -> ReadResult {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x500F => ReadResult::OPEN_BUS,
            0x5010 => /* TODO: Implement properly. */ ReadResult::full(0x01),
            0x5011..=0x5014 => ReadResult::OPEN_BUS,
            0x5015 => todo!(),
            0x5016..=0x5203 => ReadResult::OPEN_BUS,
            0x5204 => ReadResult::full(self.scanline_irq_status()),
            0x5205 => ReadResult::full((u16::from(self.multiplicand) * u16::from(self.multiplier)) as u8),
            0x5206 => ReadResult::full(((u16::from(self.multiplicand) * u16::from(self.multiplier)) >> 8) as u8),
            0x5207..=0x5BFF => ReadResult::OPEN_BUS,
            0x5C00..=0x5FFF => self.peek_from_extended_ram(address),
            0x6000..=0xFFFF => params.peek_prg(address),
        }
    }

    fn read_from_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress) -> ReadResult {
        let result = self.peek_cartridge_space(params, address);
        // TODO: Replace with ifs?
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5203 => {}
            0x5204 => self.irq_pending = false,
            0x5205..=0xFFFF => {}
            // FIXME: Shouldn't we have this here?
            // 0x6000..=0xFFFF => params.read_prg(address),
        }

        result
    }

    fn write_to_cartridge_space(&mut self, params: &mut MapperParams, address: CpuAddress, value: u8) {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x4FFF => { /* Do nothing. */ }
            0x5000 => self.pulse_2.write_control_byte(value),
            0x5001 => { /* Do nothing: MMC5 pulse channels have no sweep unit. */ }
            0x5002 => self.pulse_2.write_timer_low_byte(value),
            0x5003 => self.pulse_2.write_length_and_timer_high_byte(value),
            0x5004 => self.pulse_3.write_control_byte(value),
            0x5005 => { /* Do nothing: MMC5 pulse channels have no sweep unit. */ }
            0x5006 => self.pulse_3.write_timer_low_byte(value),
            0x5007 => self.pulse_3.write_length_and_timer_high_byte(value),
            0x5008..=0x500F => { /* Do nothing. */ }
            0x5010 => Mapper005::write_pcm_info(value),
            0x5011 => Mapper005::write_raw_pcm(value),
            0x5012..=0x5014 => { /* Do nothing. */ }
            0x5015 => Mapper005::write_apu_status(value),
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => params.set_prg_layout(PRG_LAYOUTS[usize::from(value & 0b0000_0011)]),
            0x5101 => self.set_chr_banking_mode(params, value),
            0x5102 => self.prg_ram_protect_1(params, value),
            0x5103 => self.prg_ram_protect_2(params, value),
            0x5104 => self.extended_ram_mode(params, value),
            0x5105 => self.set_name_table_mapping(value),
            0x5106 => self.set_fill_mode_tile(value),
            0x5107 => self.set_fill_mode_palette_index(value),
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113..=0x5117 => Mapper005::prg_bank_switching(params, address.to_raw(), value),
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120..=0x512B => Mapper005::chr_bank_switching(params, address.to_raw(), value),
            0x512C..=0x512F => { /* Do nothing. */ }
            0x5130 => self.set_upper_chr_bank_bits(value),
            0x5131..=0x51FF => { /* Do nothing */ }
            0x5200 => self.vertical_split_mode(value),
            0x5201 => self.vertical_split_scroll(value),
            0x5202 => self.vertical_split_bank(value),
            0x5203 => self.set_target_irq_scanline(value),
            0x5204 => self.enable_or_disable_scanline_irq(value),
            0x5205 => self.set_multiplicand(value),
            0x5206 => self.set_multiplier(value),
            0x5207..=0x520A => { /* Do nothing yet. MMC5A registers. */ }
            0x520B..=0x57FF => { /* Do nothing. */ }
            0x5800..=0x5BFF => { /* Do nothing yet. MMC5A registers. */ }
            0x5C00..=0x5FFF => self.write_to_extended_ram(address, value),
            0x6000..=0xFFFF => params.write_prg(address, value),
        }
    }

    fn ppu_peek(
        &self,
        params: &MapperParams, 
        ppu_internal_ram: &PpuInternalRam,
        address: PpuAddress,
    ) -> u8 {
        match address.to_u16() {
            0x0000..=0x1FFF => params.peek_chr(address),
            0x2000..=0x3EFF
                if address.is_in_attribute_table() && self.extended_attribute_mode_enabled() => {
                    let (_, index) = address.name_table_location().unwrap();
                    let attribute = self.extended_ram[index as usize] >> 6;
                    (attribute << 6) | (attribute << 4) | (attribute << 2) | (attribute << 0)
                }
            0x2000..=0x3EFF => {
                let (name_table_quadrant, index) = address.name_table_location().unwrap();
                match self.name_table_sources[name_table_quadrant as usize] {
                    NameTableSource::CiramLeft =>
                        ppu_internal_ram.vram.side(VramSide::Left)[index as usize],
                    NameTableSource::CiramRight =>
                        ppu_internal_ram.vram.side(VramSide::Right)[index as usize],
                    NameTableSource::ExtendedRam =>
                        self.extended_ram[index as usize],
                    NameTableSource::Fill =>
                        self.fill_mode_tile,
                }
            }
            0x3F00..=0x3FFF => self.peek_palette_table_byte(&ppu_internal_ram.palette_ram, address),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    fn on_end_of_cpu_cycle(&mut self, _cycle: i64) {
        if self.ppu_read_occurred_since_last_cpu_cycle {
            self.cpu_cycles_since_last_ppu_read = 0;
        } else {
            self.cpu_cycles_since_last_ppu_read += 1;
            if self.cpu_cycles_since_last_ppu_read == 3 {
                self.in_frame = false;
                self.previous_ppu_address_read = None;
            }
        }

        self.ppu_read_occurred_since_last_cpu_cycle = false;
    }

    fn on_cpu_read(&mut self, address: CpuAddress) {
        if address == NMI_VECTOR_LOW || address == NMI_VECTOR_HIGH {
            self.in_frame = false;
            self.previous_ppu_address_read = None;
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
            0x2001 if value & 0b0001_1000 == 0 => {
                self.in_frame = false;
                self.previous_ppu_address_read = None;
            }
            _ => {}
        }
    }

    fn on_ppu_read(&mut self, params: &mut MapperParams, address: PpuAddress, _value: u8) {
        let sprite_fetching =
            (SPRITE_PATTERN_FETCH_START..BACKGROUND_PATTERN_FETCH_START)
            .contains(&self.pattern_fetch_count);
        if !sprite_fetching && self.extended_attribute_mode_enabled() && address.is_in_name_table_proper() {
            // TODO: Verify if this is correct. Potential bugs:
            // * Is it right to cache the value? A write will overwrite the original exram value.
            // * Does any PPU read trigger this? Or just actual scheduled rendering reads?
            // If this value isn't cached, then some ugly hack to get the value into C12
            // just-in-time may be necessary.
            let raw_bank_index = (self.upper_chr_bank_bits << 6) |
                (self.extended_ram[usize::from(address.to_u16() % 0x400)] & 0b0011_1111);
            println!("{address} is in name table proper. Raw bank index: {raw_bank_index}. Pattern Fetch: {}",
                self.pattern_fetch_count);
            params.set_bank_register(C12, raw_bank_index);
        }

        if (0x0000..=0x1FFF).contains(&address.to_u16()) {
            self.pattern_fetch_count += 1;
            if self.pattern_fetch_count == SPRITE_PATTERN_FETCH_START
                || self.pattern_fetch_count == BACKGROUND_PATTERN_FETCH_START {
                self.update_chr_layout(params);
            }
        } else if (0x2000..=0x2FFF).contains(&address.to_u16())
            && self.previous_ppu_address_read == Some(address) {

            self.consecutive_reads_of_same_address += 1;
            if self.consecutive_reads_of_same_address == 2 {
                if self.in_frame {
                    self.current_scanline += 1;
                    if self.current_scanline == self.irq_scanline {
                        self.irq_pending = true;
                    }
                } else {
                    // Starting new frame.
                    self.in_frame = true;
                    self.current_scanline = 0;
                }

                self.pattern_fetch_count = 0;
            }
        } else {
            self.consecutive_reads_of_same_address = 0;
        }

        self.previous_ppu_address_read = Some(address);
        self.ppu_read_occurred_since_last_cpu_cycle = true;
    }

    fn irq_pending(&self) -> bool {
        self.scanline_irq_enabled && self.irq_pending
    }
}

impl Mapper005 {
    pub fn new() -> Self {
        Mapper005 {
            pulse_2: PulseChannel::default(),
            pulse_3: PulseChannel::default(),

            prg_ram_enabled_1: false,
            prg_ram_enabled_2: false,

            extended_ram: [0; 1 * KIBIBYTE],
            extended_ram_mode: ExtendedRamMode::WriteOnly,
            name_table_sources: [NameTableSource::CiramLeft; 4],
            fill_mode_tile: 0,
            fill_mode_palette_index: None,

            scanline_irq_enabled: false,
            irq_scanline: 0,
            current_scanline: 0,
            irq_pending: false,
            in_frame: false,
            previous_ppu_address_read: None,
            consecutive_reads_of_same_address: 0,
            cpu_cycles_since_last_ppu_read: 0,
            ppu_read_occurred_since_last_cpu_cycle: false,

            chr_window_mode: ChrWindowMode::One8K,
            sprite_height: SpriteHeight::Normal,
            pattern_fetch_count: 0,
            upper_chr_bank_bits: 0b0000_0000,

            multiplicand: 0xFF,
            multiplier: 0xFF,
        }
    }

    fn write_pcm_info(_value: u8) {}
    fn write_raw_pcm(_value: u8) {}
    fn write_apu_status(_value: u8) {}

    fn set_chr_banking_mode(&mut self, params: &mut MapperParams, value: u8) {
        self.chr_window_mode = CHR_WINDOW_MODES[usize::from(value & 0b0000_0011)];
        self.update_chr_layout(params);
    }

    fn prg_ram_protect_1(&mut self, params: &mut MapperParams, value: u8) {
        self.prg_ram_enabled_1 = value & 0b0000_0011 == 0b0000_0010;
        let status = if self.prg_ram_enabled_1 && self.prg_ram_enabled_2 {
            RamStatus::ReadWrite
        } else {
            RamStatus::ReadOnly
        };
        params.set_ram_status(S0, status);
    }

    fn prg_ram_protect_2(&mut self, params: &mut MapperParams, value: u8) {
        self.prg_ram_enabled_2 = value & 0b0000_0011 == 0b0000_0001;
        let status = if self.prg_ram_enabled_1 && self.prg_ram_enabled_2 {
            RamStatus::ReadWrite
        } else {
            RamStatus::ReadOnly
        };
        params.set_ram_status(S0, status);
    }

    fn extended_ram_mode(&mut self, params: &mut MapperParams, value: u8) {
        self.extended_ram_mode = EXTENDED_RAM_MODES[usize::from(value & 0b11)];
        self.update_chr_layout(params);
    }

    fn set_name_table_mapping(&mut self, value: u8) {
        for (i, source) in self.name_table_sources.iter_mut().enumerate() {
            *source = NAME_TABLE_SOURCES[usize::from((value >> (2 * i)) & 0b11)];
        }
    }

    fn set_fill_mode_tile(&mut self, value: u8) {
        self.fill_mode_tile = value;
    }

    fn set_fill_mode_palette_index(&mut self, value: u8) {
        self.fill_mode_palette_index = PaletteIndex::from_two_low_bits(value);
    }

    fn prg_bank_switching(params: &mut MapperParams, address: u16, value: u8) {
        let register_id = PRG_REGISTER_IDS[(address - 0x5113) as usize];
        params.set_bank_register(register_id, value);
    }

    fn chr_bank_switching(params: &mut MapperParams, address: u16, value: u8) {
        let register_id = CHR_REGISTER_IDS[(address - 0x5120) as usize];
        params.set_bank_register(register_id, value);
    }

    fn set_upper_chr_bank_bits(&mut self, value: u8) {
        self.upper_chr_bank_bits = value;
    }

    fn vertical_split_mode(&mut self, value: u8) {
        if value & 0b1000_0000 != 0 {
            todo!("Vertical split mode");
        }
    }

    fn vertical_split_scroll(&mut self, _value: u8) {
        todo!("Vertical split scroll");
    }

    fn vertical_split_bank(&mut self, _value: u8) {
        todo!("Vertical split bank");
    }

    fn set_target_irq_scanline(&mut self, value: u8) {
        self.irq_scanline = value;
    }

    fn enable_or_disable_scanline_irq(&mut self, value: u8) {
        self.scanline_irq_enabled = value & 0b1000_0000 != 0;
    }

    fn set_multiplicand(&mut self, value: u8) {
        self.multiplicand = value;
    }

    fn set_multiplier(&mut self, value: u8) {
        self.multiplier = value;
    }

    fn peek_from_extended_ram(&self, address: CpuAddress) -> ReadResult {
        if self.extended_ram_mode.is_readable() {
            ReadResult::full(self.extended_ram[usize::from(address.to_raw() - 0x5C00)])
        } else {
            ReadResult::OPEN_BUS
        }
    }

    fn write_to_extended_ram(&mut self, address: CpuAddress, value: u8) {
        // TODO: Write zeros if rendering is disabled.
        self.extended_ram[usize::from(address.to_raw() - 0x5C00)] = value;
    }

    fn scanline_irq_status(&self) -> u8 {
        let mut result = 0;
        if self.irq_pending {
            result |= 0b1000_0000;
        }

        if self.in_frame {
            result |= 0b0100_0000;
        }

        result
    }

    fn update_chr_layout(&mut self, params: &mut MapperParams) {
        let sprite_fetching =
            (SPRITE_PATTERN_FETCH_START..BACKGROUND_PATTERN_FETCH_START)
            .contains(&self.pattern_fetch_count);
        if !sprite_fetching && self.extended_attribute_mode_enabled() {
            params.set_chr_layout(EXTENDED_ATTRIBUTES_CHR_LAYOUT);
            return;
        }

        let sprite_fetching =
            (SPRITE_PATTERN_FETCH_START..BACKGROUND_PATTERN_FETCH_START)
            .contains(&self.pattern_fetch_count);
        let normal_mode = self.sprite_height == SpriteHeight::Normal || sprite_fetching;
        let windows = match (self.chr_window_mode, normal_mode) {
            (ChrWindowMode::One8K, true) => ONE_8K_CHR_WINDOW,
            (ChrWindowMode::One8K, false) => ONE_8K_CHR_WINDOW_ALTERNATE,
            (ChrWindowMode::Two4K, true) => TWO_4K_CHR_LAYOUT,
            (ChrWindowMode::Two4K, false) => TWO_4K_CHR_LAYOUT_ALTERNATE,
            (ChrWindowMode::Four2K, true) => FOUR_2K_CHR_LAYOUT,
            (ChrWindowMode::Four2K, false) => FOUR_2K_CHR_LAYOUT_ALTERNATE,
            (ChrWindowMode::Eight1K, true) => EIGHT_1K_CHR_LAYOUT,
            (ChrWindowMode::Eight1K, false) => EIGHT_1K_CHR_LAYOUT_ALTERNATE,
        };

        params.set_chr_layout(windows);
    }

    fn extended_attribute_mode_enabled(&self) -> bool {
        self.extended_ram_mode == ExtendedRamMode::ExtendedAttributes
    }
}

#[derive(Clone, Copy)]
enum ChrWindowMode {
    One8K,
    Two4K,
    Four2K,
    Eight1K,
}

#[derive(Clone, Copy)]
enum NameTableSource {
    CiramLeft,
    CiramRight,
    ExtendedRam,
    Fill,
}

#[derive(PartialEq, Eq, Clone, Copy)]
enum ExtendedRamMode {
    WriteOnly,
    ExtendedAttributes,
    ReadWrite,
    ReadOnly,
}

impl ExtendedRamMode {
    fn is_readable(self) -> bool {
        use ExtendedRamMode::*;
        match self {
            ReadWrite | ReadOnly => true,
            WriteOnly | ExtendedAttributes => false,
        }
    }
}
