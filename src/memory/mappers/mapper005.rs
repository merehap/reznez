use crate::apu::pulse_channel::PulseChannel;
use crate::ppu::palette::palette_index::PaletteIndex;
use crate::memory::mapper::*;
use crate::memory::memory::{NMI_VECTOR_LOW, NMI_VECTOR_HIGH};
use crate::memory::ppu::vram::VramSide;

const INITIAL_LAYOUT: InitialLayout = InitialLayout::builder()
    .prg_max_bank_count(128)
    .prg_bank_size(8 * KIBIBYTE)
    .prg_windows_by_board(&[(Board::Any, PRG_WINDOWS_MODE_3)])
    .chr_max_bank_count(1024)
    .chr_bank_size(1 * KIBIBYTE)
    .chr_windows(CHR_WINDOWS_MODE_0)
    .do_not_align_large_chr_windows()
    .name_table_mirroring_source(NameTableMirroringSource::Cartridge)
    .build();

const PRG_WINDOWS_MODE_0: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Banked(Ram,    BankIndex::Register(P0))),
    PrgWindow::new(0x8000, 0xFFFF, 32 * KIBIBYTE, PrgType::Banked(Rom,    BankIndex::Register(P4))),
];

const PRG_WINDOWS_MODE_1: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Banked(Ram,    BankIndex::Register(P0))),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P2))),
    PrgWindow::new(0xC000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom,    BankIndex::Register(P4))),
];

const PRG_WINDOWS_MODE_2: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Banked(Ram,    BankIndex::Register(P0))),
    PrgWindow::new(0x8000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P2))),
    PrgWindow::new(0xC000, 0xDFFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P3))),
    PrgWindow::new(0xE000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom,    BankIndex::Register(P4))),
];

const PRG_WINDOWS_MODE_3: &[PrgWindow] = &[
    PrgWindow::new(0x6000, 0x7FFF,  8 * KIBIBYTE, PrgType::Banked(Ram,    BankIndex::Register(P0))),
    PrgWindow::new(0x8000, 0x9FFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P1))),
    PrgWindow::new(0xA000, 0xBFFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P2))),
    PrgWindow::new(0xC000, 0xDFFF, 16 * KIBIBYTE, PrgType::Banked(RomRam, BankIndex::Register(P3))),
    PrgWindow::new(0xE000, 0xFFFF, 16 * KIBIBYTE, PrgType::Banked(Rom,    BankIndex::Register(P4))),
];

const CHR_WINDOWS_MODE_0: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x1FFF, 8 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C7))),
];

const CHR_WINDOWS_MODE_1: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x0FFF, 4 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C3))),
    ChrWindow::new(0x1000, 0x1FFF, 4 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C7))),
];

const CHR_WINDOWS_MODE_2: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x07FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C1))),
    ChrWindow::new(0x0800, 0x0FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C3))),
    ChrWindow::new(0x1000, 0x17FF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C5))),
    ChrWindow::new(0x1800, 0x1FFF, 2 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C7))),
];

const CHR_WINDOWS_MODE_3: &[ChrWindow] = &[
    ChrWindow::new(0x0000, 0x03FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C0))),
    ChrWindow::new(0x0400, 0x07FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C1))),
    ChrWindow::new(0x0800, 0x0BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C2))),
    ChrWindow::new(0x0C00, 0x0FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C3))),
    ChrWindow::new(0x1000, 0x13FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C4))),
    ChrWindow::new(0x1400, 0x17FF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C5))),
    ChrWindow::new(0x1800, 0x1BFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C6))),
    ChrWindow::new(0x1C00, 0x1FFF, 1 * KIBIBYTE, ChrType(Rom, BankIndex::Register(C7))),
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
    irq_in_frame: bool,
    previous_ppu_address_read: Option<PpuAddress>,
    consecutive_reads_of_same_address: u8,
    cpu_cycles_since_last_ppu_read: u8,
    ppu_read_occurred_since_last_cpu_cycle: bool,

    params: MapperParams,
}

impl Mapper for Mapper005 {
    fn peek_from_cartridge_space(&self, address: CpuAddress) -> Option<u8> {
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x500F => None,
            0x5010 => /* TODO: Implement properly. */ Some(0x01),
            0x5011..=0x5014 => None,
            0x5015 => todo!(),
            0x5016..=0x5203 => None,
            0x5204 => Some(self.scanline_irq_status()),
            0x5205..=0x5206 => todo!(),
            0x5007..=0x5BFF => None,
            0x5C00..=0x5FFF => self.peek_from_extended_ram(address),
            0x6000..=0xFFFF => self.prg_memory().peek(address),
        }
    }

    fn read_from_cartridge_space(&mut self, address: CpuAddress) -> Option<u8> {
        let result = self.peek_from_cartridge_space(address);
        match address.to_raw() {
            0x0000..=0x401F => unreachable!(),
            0x4020..=0x5203 => {}
            0x5204 => self.irq_pending = false,
            0x5205..=0xFFFF => {}
        }
        result
    }

    fn write_to_cartridge_space(&mut self, address: CpuAddress, value: u8) {
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
            0x5010 => self.write_pcm_info(value),
            0x5011 => self.write_raw_pcm(value),
            0x5012..=0x5014 => { /* Do nothing. */ }
            0x5015 => self.write_apu_status(value),
            0x5016..=0x50FF => { /* Do nothing. */ }
            0x5100 => self.set_prg_banking_mode(value),
            0x5101 => self.set_chr_banking_mode(value),
            0x5102 => self.prg_ram_protect_1(value),
            0x5103 => self.prg_ram_protect_2(value),
            0x5104 => self.extended_ram_mode(value),
            0x5105 => self.set_name_table_mapping(value),
            0x5106 => self.set_fill_mode_tile(value),
            0x5107 => self.set_fill_mode_palette_index(value),
            0x5108..=0x5112 => { /* Do nothing. */ }
            0x5113..=0x5117 => self.prg_bank_switching(address.to_raw(), value),
            0x5118..=0x511F => { /* Do nothing. */ }
            0x5120..=0x512B => self.chr_bank_switching(address.to_raw(), value),
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
            0x6000..=0xFFFF => self.prg_memory_mut().write(address, value),
        }
    }

    fn custom_ppu_peek(&self, address: PpuAddress) -> CustomPpuPeekResult {
        if let Some((name_table_quadrant, index)) = address.name_table_location() {
            match self.name_table_sources[name_table_quadrant as usize] {
                NameTableSource::CiramLeft =>
                    CustomPpuPeekResult::InternalRam(VramSide::Left, index),
                NameTableSource::CiramRight =>
                    CustomPpuPeekResult::InternalRam(VramSide::Right, index),
                NameTableSource::ExtendedRam =>
                    CustomPpuPeekResult::Value(self.extended_ram[index as usize]),
                NameTableSource::Fill =>
                    CustomPpuPeekResult::Value(self.fill_mode_tile),
            }
        } else {
            CustomPpuPeekResult::NoOverride
        }
    }

    fn on_end_of_cpu_cycle(&mut self) {
        if self.ppu_read_occurred_since_last_cpu_cycle {
            self.cpu_cycles_since_last_ppu_read = 0;
        } else {
            self.cpu_cycles_since_last_ppu_read += 1;
            if self.cpu_cycles_since_last_ppu_read == 3 {
                self.irq_in_frame = false;
                self.previous_ppu_address_read = None;
            }
        }

        self.ppu_read_occurred_since_last_cpu_cycle = false;
    }

    fn on_cpu_read(&mut self, address: CpuAddress) {
        if address == NMI_VECTOR_LOW || address == NMI_VECTOR_HIGH {
            self.irq_in_frame = false;
            self.previous_ppu_address_read = None;
        }
    }

    fn on_cpu_write(&mut self, address: CpuAddress, value: u8) {
        if address.to_raw() == 0x2001 && value & 0b0001_1000 == 0 {
            self.irq_in_frame = false;
            self.previous_ppu_address_read = None;
        }
    }

    fn on_ppu_read(&mut self, address: PpuAddress) {
        if address.to_u16() >= 0x2000
            && address.to_u16() < 0x3000
            && self.previous_ppu_address_read == Some(address) {

            self.consecutive_reads_of_same_address += 1;
            if self.consecutive_reads_of_same_address == 2 {
                if self.irq_in_frame {
                    self.current_scanline += 1;
                    if self.current_scanline == self.irq_scanline {
                        self.irq_pending = true;
                    }
                } else {
                    self.irq_in_frame = true;
                    self.current_scanline = 0;
                }
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

    /*
    fn process_end_of_ppu_cycle(&mut self) {
    }

    fn process_current_ppu_address(&mut self, address: PpuAddress) {
    }
    */

    fn params(&self) -> &MapperParams { &self.params }
    fn params_mut(&mut self) -> &mut MapperParams { &mut self.params }
}

impl Mapper005 {
    pub fn new(cartridge: &Cartridge) -> Result<Mapper005, String> {
        let mut mapper = Mapper005 {
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
            irq_in_frame: false,
            previous_ppu_address_read: None,
            consecutive_reads_of_same_address: 0,
            cpu_cycles_since_last_ppu_read: 0,
            ppu_read_occurred_since_last_cpu_cycle: false,

            params: INITIAL_LAYOUT.make_mapper_params(cartridge, Board::Any),
        };
        let last_bank_index = mapper.prg_memory().last_bank_index();
        mapper.prg_memory_mut().set_bank_index_register(P4, last_bank_index);
        Ok(mapper)
    }

    fn write_pcm_info(&mut self, _value: u8) {}
    fn write_raw_pcm(&mut self, _value: u8) {}
    fn write_apu_status(&mut self, _value: u8) {}

    fn set_prg_banking_mode(&mut self, value: u8) {
        let windows = match value & 0b0000_0011 {
            0 => PRG_WINDOWS_MODE_0,
            1 => PRG_WINDOWS_MODE_1,
            2 => PRG_WINDOWS_MODE_2,
            3 => PRG_WINDOWS_MODE_3,
            _ => unreachable!(),
        };
        self.prg_memory_mut().set_windows(windows);
    }

    fn set_chr_banking_mode(&mut self, value: u8) {
        let windows = match value & 0b0000_0011 {
            0 => CHR_WINDOWS_MODE_0,
            1 => CHR_WINDOWS_MODE_1,
            2 => CHR_WINDOWS_MODE_2,
            3 => CHR_WINDOWS_MODE_3,
            _ => unreachable!(),
        };
        self.chr_memory_mut().set_windows(windows);
    }

    fn prg_ram_protect_1(&mut self, value: u8) {
        self.prg_ram_enabled_1 = value & 0b0000_0011 == 0b0000_0010;
    }

    fn prg_ram_protect_2(&mut self, value: u8) {
        self.prg_ram_enabled_2 = value & 0b0000_0011 == 0b0000_0001;
    }

    fn extended_ram_mode(&mut self, value: u8) {
        self.extended_ram_mode = match value & 0b11 {
            0b00 => ExtendedRamMode::WriteOnly,
            0b01 => ExtendedRamMode::ExtendedAttributes,
            0b10 => ExtendedRamMode::ReadWrite,
            0b11 => ExtendedRamMode::ReadOnly,
            _ => unreachable!(),
        }
    }

    fn set_name_table_mapping(&mut self, value: u8) {
        for (i, source) in self.name_table_sources.iter_mut().enumerate() {
            *source = match (value >> (2 * i)) & 0b11 {
                0b00 => NameTableSource::CiramLeft,
                0b01 => NameTableSource::CiramRight,
                0b10 => NameTableSource::ExtendedRam,
                0b11 => NameTableSource::Fill,
                _ => unreachable!(),
            };
        }
    }

    fn set_fill_mode_tile(&mut self, value: u8) {
        self.fill_mode_tile = value;
    }

    fn set_fill_mode_palette_index(&mut self, value: u8) {
        self.fill_mode_palette_index = PaletteIndex::from_two_low_bits(value);
    }

    fn prg_bank_switching(&mut self, address: u16, value: u8) {
        let register_id = match address {
            0x5113 => P0,
            0x5114 => P1,
            0x5115 => P2,
            0x5116 => P3,
            0x5117 => P4,
            _ => unreachable!(),
        };
        self.prg_memory_mut().set_bank_index_register(register_id, value);
    }

    fn chr_bank_switching(&mut self, address: u16, value: u8) {
        let (first_reg_id, maybe_second_reg_id) = match address {
            0x5120 => (C0, None),
            0x5121 => (C1, None),
            0x5122 => (C2, None),
            0x5123 => (C3, None),
            0x5124 => (C4, None),
            0x5125 => (C5, None),
            0x5126 => (C6, None),
            0x5127 => (C7, None),
            0x5128 => (C0, Some(C4)),
            0x5129 => (C1, Some(C5)),
            0x512A => (C2, Some(C6)),
            0x512B => (C3, Some(C7)),
            _ => unreachable!(),
        };
        self.chr_memory_mut().set_bank_index_register(first_reg_id, value);
        if let Some(second_reg_id) = maybe_second_reg_id {
            self.chr_memory_mut().set_bank_index_register(second_reg_id, value);
        }
    }

    fn set_upper_chr_bank_bits(&mut self, _value: u8) {
        todo!("Upper CHR Bank bits. No commercial game uses them.");
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

    fn set_multiplicand(&mut self, _value: u8) {
        todo!("Multiplicand");
    }

    fn set_multiplier(&mut self, _value: u8) {
        todo!("Multiplier");
    }

    fn peek_from_extended_ram(&self, address: CpuAddress) -> Option<u8> {
        if self.extended_ram_mode.is_readable() {
            Some(self.extended_ram[usize::from(address.to_raw() - 0x5C00)])
        } else {
            None
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

        if self.irq_in_frame {
            result |= 0b0100_0000;
        }

        result
    }
}

#[derive(Clone, Copy)]
enum NameTableSource {
    CiramLeft,
    CiramRight,
    ExtendedRam,
    Fill,
}

#[derive(Clone, Copy)]
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
