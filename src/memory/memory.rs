use crate::apu::apu_registers::ApuRegisters;
use crate::controller::joypad::Joypad;
use crate::cpu::dmc_dma::DmcDma;
use crate::cpu::oam_dma::OamDma;
use crate::memory::bank::bank::{ChrSource, ChrSourceRegisterId, PrgSource, ReadStatusRegisterId, PrgSourceRegisterId, WriteStatusRegisterId};
use crate::memory::bank::bank_number::{ReadStatus, WriteStatus};
use crate::memory::cpu::cpu_address::CpuAddress;
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::memory::cpu::stack::Stack;
use crate::mapper::{ChrBankRegisterId, ChrMemory, CiramSide, MetaRegisterId, NameTableMirroring, NameTableQuadrant, NameTableSource, PpuAddress, PrgBankRegisterId, PrgMemory, ReadResult};
use crate::memory::ppu::chr_memory::PpuPeek;
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::ppu::ppu_pinout::PpuPinout;
use crate::ppu::clock::Clock;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::register::ppu_registers::PpuRegisters;
use crate::ppu::sprite::oam::Oam;

pub const NMI_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFA);
pub const NMI_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFB);
pub const RESET_VECTOR_LOW: CpuAddress   = CpuAddress::new(0xFFFC);
pub const RESET_VECTOR_HIGH: CpuAddress  = CpuAddress::new(0xFFFD);
pub const IRQ_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFE);
pub const IRQ_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFF);

pub struct Memory {
    pub cpu_internal_ram: CpuInternalRam,
    pub ciram: Ciram,
    pub palette_ram: PaletteRam,
    pub oam: Oam,
    pub joypad1: Joypad,
    pub joypad2: Joypad,
    pub ppu_regs: PpuRegisters,
    pub apu_regs: ApuRegisters,
    system_palette: SystemPalette,
    pub dmc_dma: DmcDma,
    pub oam_dma: OamDma,
    pub cpu_pinout: CpuPinout,
    pub ppu_pinout: PpuPinout,
    pub oam_dma_address_bus: CpuAddress,
    pub dmc_dma_address_bus: CpuAddress,
    cpu_cycle: i64,

    pub prg_memory: PrgMemory,
    pub chr_memory: ChrMemory,
    pub name_table_mirrorings: &'static [NameTableMirroring],
    pub mapper_custom_pages: Vec<SmallPage>,

    pub dip_switch: u8,
}

impl Memory {
    pub fn new(
        prg_memory: PrgMemory,
        chr_memory: ChrMemory,
        name_table_mirrorings: &'static [NameTableMirroring],
        ppu_clock: Clock,
        dip_switch: u8,
        system_palette: SystemPalette,
    ) -> Memory {
        Memory {
            cpu_internal_ram: CpuInternalRam::new(),
            ciram: Ciram::new(),
            palette_ram: PaletteRam::new(),
            oam: Oam::new(),
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),
            ppu_regs: PpuRegisters::new(ppu_clock),
            apu_regs: ApuRegisters::new(),
            system_palette,
            dmc_dma: DmcDma::IDLE,
            oam_dma: OamDma::IDLE,
            cpu_pinout: CpuPinout::new(),
            ppu_pinout: PpuPinout::new(),
            oam_dma_address_bus: CpuAddress::ZERO,
            dmc_dma_address_bus: CpuAddress::ZERO,
            cpu_cycle: 0,

            prg_memory,
            chr_memory,
            name_table_mirrorings,
            mapper_custom_pages: Vec::new(),

            dip_switch,
        }
    }

    pub fn stack_pointer(&self) -> u8 {
        self.cpu_internal_ram.stack_pointer
    }

    pub fn ciram(&self) -> &Ciram {
        &self.ciram
    }

    pub fn cpu_cycle(&self) -> i64 {
        self.cpu_cycle
    }

    pub fn cpu_internal_ram(&self) -> &CpuInternalRam {
        &self.cpu_internal_ram
    }

    pub fn cpu_address_bus(&self, address_bus_type: AddressBusType) -> CpuAddress {
        match address_bus_type {
            AddressBusType::Cpu => self.cpu_pinout.address_bus,
            AddressBusType::OamDma => self.oam_dma_address_bus,
            AddressBusType::DmcDma => self.dmc_dma_address_bus,
        }
    }

    pub fn set_cpu_address_bus(&mut self, address_bus_type: AddressBusType, address: CpuAddress) {
        match address_bus_type {
            AddressBusType::Cpu => self.cpu_pinout.address_bus = address,
            AddressBusType::OamDma => self.oam_dma_address_bus = address,
            AddressBusType::DmcDma => self.dmc_dma_address_bus = address,
        }
    }

    pub fn dmc_dma_address(&self) -> CpuAddress {
        self.apu_regs.dmc.dma_sample_address()
    }

    pub fn set_dmc_sample_buffer(&mut self, value: u8) {
        self.apu_regs.dmc.set_sample_buffer(&mut self.cpu_pinout, value);
    }

    #[inline]
    pub fn cpu_stack<'a>(&'a mut self) -> Stack<'a> {
        self.cpu_internal_ram.stack()
    }

    #[inline]
    pub fn cpu_stack_pointer_mut(&mut self) -> &mut u8 {
        &mut self.cpu_internal_ram.stack_pointer
    }

    pub fn cpu_stack_pointer_address(&self) -> CpuAddress {
        CpuAddress::from_low_high(self.stack_pointer(), 0x01)
    }

    pub fn increment_cpu_cycle(&mut self) {
        self.cpu_cycle += 1;
    }

    pub fn set_cpu_cycle(&mut self, cycle: i64) {
        self.cpu_cycle = cycle;
    }

    pub fn chr_rom_bank_count(&self) -> u16 {
        self.chr_memory.rom_bank_count()
    }

    pub fn chr_ram_bank_count(&self) -> u16 {
        self.chr_memory.ram_bank_count()
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(
            self.palette_ram.to_slice(),
            &self.system_palette,
            self.ppu_regs.mask(),
        )
    }

    pub fn name_table_mirroring(&self) -> NameTableMirroring {
        self.chr_memory().name_table_mirroring()
    }

    pub fn set_name_table_mirroring(&mut self, mirroring_index: u8) {
        let mirroring = *self.name_table_mirrorings.get(usize::from(mirroring_index))
            .expect("NameTableMirroring cannot be selected from the list since the list is empty.");
        self.chr_memory.set_name_table_mirroring(mirroring);
    }

    // Almost all use-cases should use set_name_table_mirroring instead of this.
    pub fn set_name_table_mirroring_directly(&mut self, mirroring: NameTableMirroring) {
        self.chr_memory.set_name_table_mirroring(mirroring);
    }

    pub fn set_name_table_quadrant(&mut self, quadrant: NameTableQuadrant, ciram_side: CiramSide) {
        self.chr_memory.set_name_table_quadrant(quadrant, NameTableSource::Ciram(ciram_side));
    }

    pub fn set_name_table_quadrant_to_source(&mut self, quadrant: NameTableQuadrant, source: NameTableSource) {
        self.chr_memory.set_name_table_quadrant(quadrant, source);
    }

    pub fn prg_memory(&self) -> &PrgMemory {
        &self.prg_memory
    }

    pub fn set_prg_layout(&mut self, index: u8) {
        self.prg_memory.set_layout(index);
    }

    pub fn set_prg_rom_outer_bank_number(&mut self, index: u8) {
        self.prg_memory.set_prg_rom_outer_bank_number(index);
    }

    // TODO: Remove this.
    pub fn write_prg(&mut self, addr: CpuAddress, value: u8) {
        self.prg_memory.write(addr, value);
    }

    pub fn set_reads_enabled(&mut self, id: ReadStatusRegisterId, enabled: bool) {
        let status = if enabled { ReadStatus::Enabled } else { ReadStatus::Disabled };
        self.prg_memory.set_read_status(id, status);
        self.chr_memory.set_read_status(id, status);
    }

    pub fn set_read_status(&mut self, id: ReadStatusRegisterId, read_status: ReadStatus) {
        self.prg_memory.set_read_status(id, read_status);
        self.chr_memory.set_read_status(id, read_status);
    }

    pub fn set_read_zeroes(&mut self, id: ReadStatusRegisterId) {
        self.prg_memory.set_read_status(id, ReadStatus::ReadOnlyZeros);
        self.chr_memory.set_read_status(id, ReadStatus::ReadOnlyZeros);
    }

    pub fn set_writes_enabled(&mut self, id: WriteStatusRegisterId, enabled: bool) {
        let status = if enabled { WriteStatus::Enabled } else { WriteStatus::Disabled };
        self.prg_memory.set_write_status(id, status);
        self.chr_memory.set_write_status(id, status);
    }

    pub fn set_rom_ram_mode(&mut self, id: PrgSourceRegisterId, rom_ram_mode: PrgSource) {
        self.prg_memory.set_rom_ram_mode(id, rom_ram_mode);
    }

    pub fn set_chr_source(&mut self, id: ChrSourceRegisterId, chr_source: ChrSource) {
        self.chr_memory.set_chr_source(id, chr_source);
    }

    pub fn chr_memory(&self) -> &ChrMemory {
        &self.chr_memory
    }

    pub fn set_chr_layout(&mut self, index: u8) {
        self.chr_memory.set_layout(index);
    }

    pub fn peek_chr(&self, address: PpuAddress) -> PpuPeek {
        self.chr_memory.peek(&self.ciram, &self.mapper_custom_pages, address)
    }

    pub fn set_chr_rom_outer_bank_number(&mut self, index: u8) {
        self.chr_memory.set_chr_rom_outer_bank_number(index);
    }

    pub fn set_prg_register<INDEX: Into<u16>>(&mut self, id: PrgBankRegisterId, value: INDEX) {
        self.prg_memory.set_bank_register(id, value.into());
    }

    pub fn set_prg_bank_register_bits(&mut self, id: PrgBankRegisterId, new_value: u16, mask: u16) {
        self.prg_memory.set_bank_register_bits(id, new_value, mask);
    }

    pub fn update_prg_register(&mut self, id: PrgBankRegisterId, updater: &dyn Fn(u16) -> u16) {
        self.prg_memory.update_bank_register(id, updater);
    }

    pub fn set_chr_register<INDEX: Into<u16>>(&mut self, id: ChrBankRegisterId, value: INDEX) {
        self.chr_memory.set_bank_register(id, value);
    }

    pub fn set_chr_register_low_byte(&mut self, id: ChrBankRegisterId, low_byte_value: u8) {
        self.set_chr_bank_register_bits(id, u16::from(low_byte_value), 0b0000_0000_1111_1111);
    }

    pub fn set_chr_register_high_byte(&mut self, id: ChrBankRegisterId, high_byte_value: u8) {
        self.set_chr_bank_register_bits(id, u16::from(high_byte_value) << 8, 0b1111_1111_0000_0000);
    }

    pub fn set_chr_bank_register_bits(&mut self, id: ChrBankRegisterId, new_value: u16, mask: u16) {
        self.chr_memory.set_bank_register_bits(id, new_value, mask);
    }

    pub fn set_chr_meta_register(&mut self, id: MetaRegisterId, value: ChrBankRegisterId) {
        self.chr_memory.set_meta_register(id, value);
    }

    pub fn update_chr_register(&mut self, id: ChrBankRegisterId, updater: &dyn Fn(u16) -> u16) {
        self.chr_memory.update_bank_register(id, updater);
    }

    pub fn set_chr_bank_register_to_ciram_side(&mut self, id: ChrSourceRegisterId, ciram_side: CiramSide) {
        self.chr_memory.set_chr_bank_register_to_ciram_side(id, ciram_side);
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressBusType {
    Cpu,
    OamDma,
    DmcDma,
}

pub struct SmallPage {
    _name: String,
    page: [u8; 0x400],
    read_status: ReadStatus,
    write_status: WriteStatus,
}

impl SmallPage {
    pub fn new(_name: String, read_status: ReadStatus, write_status: WriteStatus) -> Self {
        Self {
            _name,
            page: [0; 0x400],
            read_status,
            write_status,
        }
    }

    pub fn peek(&self, index: u16) -> ReadResult {
        match self.read_status {
            ReadStatus::Disabled => ReadResult::OPEN_BUS,
            ReadStatus::ReadOnlyZeros => ReadResult::full(0),
            ReadStatus::Enabled => ReadResult::full(self.page[index as usize]),
        }
    }

    pub fn write(&mut self, index: u16, value: u8) {
        match self.write_status {
            WriteStatus::Disabled => { /* Do nothing. */ }
            WriteStatus::Enabled => self.page[index as usize] = value,
        }
    }

    pub fn set_read_status(&mut self, read_status: ReadStatus) {
        self.read_status = read_status;
    }

    pub fn set_write_status(&mut self, write_status: WriteStatus) {
        self.write_status = write_status;
    }

    pub fn to_raw_ref(&self) -> &[u8; 0x400] {
        &self.page
    }

    pub fn to_raw_ref_mut(&mut self) -> Option<&mut [u8; 0x400]> {
        match self.write_status {
            WriteStatus::Disabled => None,
            WriteStatus::Enabled => Some(&mut self.page),
        }
    }
}