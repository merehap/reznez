use crate::apu::apu::Apu;
use crate::apu::apu_clock::ApuClock;
use crate::apu::apu_registers::ApuRegisters;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::Cpu;
use crate::cpu::dmc_dma::DmcDma;
use crate::cpu::oam_dma::OamDma;
use crate::master_clock::MasterClock;
use crate::memory::bank::bank::{ChrSource, ChrSourceRegisterId, PrgSource, ReadStatusRegisterId, PrgSourceRegisterId, WriteStatusRegisterId};
use crate::memory::bank::bank_number::{ReadStatus, WriteStatus};
use crate::memory::cpu::cpu_address::{CpuAddress, FriendlyCpuAddress};
use crate::memory::cpu::cpu_internal_ram::CpuInternalRam;
use crate::memory::cpu::cpu_pinout::CpuPinout;
use crate::mapper::{ChrBankRegisterId, ChrMemory, CiramSide, KIBIBYTE, Mapper, MetaRegisterId, NameTableMirroring, NameTableQuadrant, NameTableSource, PpuAddress, PrgBankRegisterId, PrgMemory, ReadResult};
use crate::memory::ppu::chr_memory::{PeekSource, PpuPeek};
use crate::memory::ppu::palette_ram::PaletteRam;
use crate::memory::ppu::ciram::Ciram;
use crate::memory::ppu::ppu_pinout::PpuPinout;
use crate::memory::small_page::SmallPage;
use crate::ppu::ppu_clock::PpuClock;
use crate::ppu::palette::palette_table::PaletteTable;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::ppu::Ppu;
use crate::ppu::register::ppu_registers::{PpuRegisters, WriteToggle};
use crate::ppu::sprite::oam::Oam;

pub const NMI_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFA);
pub const NMI_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFB);
pub const RESET_VECTOR_LOW: CpuAddress   = CpuAddress::new(0xFFFC);
pub const RESET_VECTOR_HIGH: CpuAddress  = CpuAddress::new(0xFFFD);
pub const IRQ_VECTOR_LOW: CpuAddress     = CpuAddress::new(0xFFFE);
pub const IRQ_VECTOR_HIGH: CpuAddress    = CpuAddress::new(0xFFFF);

pub struct Bus {
    // Primary devices
    pub cpu: Cpu,
    pub ppu: Ppu,
    pub apu: Apu,

    // Other devices
    pub master_clock: MasterClock,
    pub dmc_dma: DmcDma,
    pub oam_dma: OamDma,
    pub joypad1: Joypad,
    pub joypad2: Joypad,

    // Registers
    pub ppu_regs: PpuRegisters,
    pub apu_regs: ApuRegisters,

    // Memory
    pub cpu_internal_ram: CpuInternalRam,
    pub ciram: Ciram,
    pub palette_ram: PaletteRam,
    pub oam: Oam,
    pub prg_memory: PrgMemory,
    pub chr_memory: ChrMemory,
    pub mapper_custom_pages: Vec<SmallPage>,

    // Pinouts
    pub cpu_pinout: CpuPinout,
    pub ppu_pinout: PpuPinout,
    pub oam_dma_address_bus: CpuAddress,
    pub dmc_dma_address_bus: CpuAddress,

    // Miscellaneous
    pub name_table_mirrorings: &'static [NameTableMirroring], // TODO: Move into ChrMemory.
    pub dip_switch: u8,
    system_palette: SystemPalette,
}

impl Bus {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        master_clock: MasterClock,
        cpu: Cpu,
        ppu: Ppu,
        apu: Apu,
        prg_memory: PrgMemory,
        chr_memory: ChrMemory,
        name_table_mirrorings: &'static [NameTableMirroring],
        dip_switch: u8,
        system_palette: SystemPalette,
    ) -> Self {
        Self {
            cpu,
            ppu,
            apu,

            master_clock,
            dmc_dma: DmcDma::IDLE,
            oam_dma: OamDma::IDLE,
            joypad1: Joypad::new(),
            joypad2: Joypad::new(),

            ppu_regs: PpuRegisters::new(),
            apu_regs: ApuRegisters::new(),

            cpu_internal_ram: CpuInternalRam::new(),
            ciram: Ciram::new(),
            palette_ram: PaletteRam::new(),
            oam: Oam::new(),
            prg_memory,
            chr_memory,
            mapper_custom_pages: Vec::new(),

            cpu_pinout: CpuPinout::new(),
            ppu_pinout: PpuPinout::new(),
            oam_dma_address_bus: CpuAddress::ZERO,
            dmc_dma_address_bus: CpuAddress::ZERO,

            name_table_mirrorings,
            dip_switch,
            system_palette,
        }
    }

    pub fn ciram(&self) -> &Ciram {
        &self.ciram
    }

    pub fn master_clock(&self) -> &MasterClock {
        &self.master_clock
    }

    pub(in crate) fn master_clock_mut(&mut self) -> &mut MasterClock {
        &mut self.master_clock
    }

    pub fn cpu_cycle(&self) -> i64 {
        self.master_clock.cpu_cycle()
    }

    pub fn ppu_clock(&self) -> &PpuClock {
        self.master_clock.ppu_clock()
    }

    pub fn apu_clock(&self) -> &ApuClock {
        self.master_clock.apu_clock()
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

    pub fn chr_rom_bank_count(&self) -> u16 {
        self.chr_memory.rom_bank_count()
    }

    pub fn chr_ram_bank_count(&self) -> u16 {
        self.chr_memory.ram_bank_count()
    }

    #[inline]
    pub fn palette_table(&self) -> PaletteTable {
        PaletteTable::new(self.palette_ram.to_slice(), &self.system_palette, self.ppu_regs.mask())
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

    #[inline]
    pub fn peek_name_table_byte(&self, address: PpuAddress) -> PpuPeek {
        let (name_table_quadrant, index) = address.to_name_table_index();
        let value = self.raw_name_table(name_table_quadrant)[index as usize];
        PpuPeek::new(value, PeekSource::from_name_table_source(self.name_table_mirroring().name_table_source_in_quadrant(name_table_quadrant)))
    }

    #[inline]
    pub fn write_name_table_byte(&mut self, address: PpuAddress, value: u8) {
        let (quadrant, index) = address.to_name_table_index();
        match self.name_table_mirroring().name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) =>
                self.ciram.write(side, index, value),
            NameTableSource::Rom {..} => { /* ROM is read-only. */}
            // FIXME: This currently ignores whether RAM writes are enabled. It shouldn't be possible to do that.
            NameTableSource::Ram { bank_number } =>
                self.chr_memory.work_ram_1kib_page_mut(0x400 * u32::from(bank_number.to_raw()))[index as usize] = value,
            NameTableSource::MapperCustom { page_number, .. } => {
                if let Some(page) = self.mapper_custom_pages[page_number as usize].to_raw_ref_mut() {
                    // This page must be writeable.
                    page[index as usize] = value;
                }
            }
        }
    }

    #[inline]
    pub fn raw_name_table(&self, quadrant: NameTableQuadrant) -> &[u8; KIBIBYTE as usize] {
        match self.name_table_mirroring().name_table_source_in_quadrant(quadrant) {
            NameTableSource::Ciram(side) => self.ciram.side(side),
            // FIXME: Hack
            NameTableSource::Rom { bank_number } => self.chr_memory.rom_1kib_page(0x400 * u32::from(bank_number.to_raw())),
            // FIXME: Hack
            NameTableSource::Ram { bank_number } => self.chr_memory.work_ram_1kib_page(0x400 * u32::from(bank_number.to_raw())),
            NameTableSource::MapperCustom { page_number, .. } => self.mapper_custom_pages[page_number as usize].to_raw_ref(),
        }
    }

    pub fn set_chr_layout(&mut self, index: u8) {
        self.chr_memory.set_layout(index);
    }

    pub fn cpu_peek(&self, mapper: &dyn Mapper, address_bus_type: AddressBusType, addr: CpuAddress) -> u8 {
        self.cpu_peek_unresolved(mapper, address_bus_type, addr).resolve(self.cpu_pinout.data_bus)
    }

    pub fn cpu_peek_unresolved(&self, mapper: &dyn Mapper, _address_bus_type: AddressBusType, addr: CpuAddress) -> ReadResult {
        use FriendlyCpuAddress as Addr;
        let normal_peek_value = match addr.to_friendly() {
            Addr::CpuInternalRam(index) => self.cpu_internal_ram().peek(index),
            Addr::PpuStatus             => self.ppu_regs.peek_status(),
            Addr::OamData               => self.ppu_regs.peek_oam_data(&self.oam),
            Addr::PpuData => {
                let old_value = mapper.ppu_peek(self, self.ppu_regs.current_address).value();
                self.ppu_regs.peek_ppu_data(old_value)
            }
            Addr::PpuControl | Addr::PpuMask | Addr::OamAddress | Addr::PpuScroll | Addr::PpuAddress => {
                // Write-only PPU registers.
                self.ppu_regs.peek_from_write_only_register()
            }
            Addr::MapperRegisters => {
                match *addr {
                    0x4020..=0x5FFF => mapper.peek_register(self, addr),
                    0x6000..=0xFFFF => mapper.peek_prg(self, addr),
                    _ => unreachable!(),
                }
            }

            // Write-only registers and unused register addresses result in a read of Open BUS.
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            _ => ReadResult::OPEN_BUS,
        };

        let mut should_apu_read_dominate_normal_read = false;
        let apu_peek_value = if self.apu_registers_active() {
            let addr = CpuAddress::new(0x4000 + *addr % 0x20);
            use FriendlyCpuAddress as Addr;
            match addr.to_friendly() {
                Addr::ApuStatus => {
                    should_apu_read_dominate_normal_read = true;
                    self.apu_regs.peek_status(&self.cpu_pinout, &self.dmc_dma)
                }
                Addr::Controller1AndStrobe       => self.joypad1.peek_status(),
                Addr::Controller2AndFrameCounter => self.joypad2.peek_status(),

                // APU channel registers and OAM DMA are write-only. CPU Test Mode is not yet supported.
                _ if addr.is_in_apu_register_range() => ReadResult::OPEN_BUS,
                _ => unreachable!()
            }
        } else {
            ReadResult::OPEN_BUS
        };

        if should_apu_read_dominate_normal_read {
            apu_peek_value.dominate(normal_peek_value)
        } else {
            normal_peek_value.dominate(apu_peek_value)
        }
    }

    #[inline]
    #[rustfmt::skip]
    pub fn cpu_read(&mut self, mapper: &mut dyn Mapper, address_bus_type: AddressBusType) -> u8 {
        let addr = self.cpu_address_bus(address_bus_type);
        use FriendlyCpuAddress as Addr;
        let normal_read_value = match addr.to_friendly() {
            Addr::CpuInternalRam(index) => self.cpu_internal_ram().peek(index),
            Addr::PpuStatus             => self.ppu_regs.read_status(self.master_clock.ppu_clock()),
            Addr::OamData               => self.ppu_regs.read_oam_data(&self.oam),
            Addr::PpuData => {
                self.set_ppu_address_bus(mapper, self.ppu_regs.current_address);
                // TODO: Instead of peeking the old data, it must be available as part of some register.
                let old_data = mapper.ppu_peek(self, self.ppu_pinout.address()).value();
                let new_data = self.ppu_regs.read_ppu_data(old_data);

                let pending_data_source = self.ppu_regs.current_address.to_pending_data_source();
                let buffered_data = mapper.ppu_peek(self, pending_data_source).value();
                mapper.on_ppu_read(self, pending_data_source, buffered_data);
                self.ppu_regs.set_ppu_read_buffer_and_advance(buffered_data);
                self.set_ppu_address_bus(mapper, self.ppu_regs.current_address);

                new_data
            }
            Addr::PpuControl | Addr::PpuMask | Addr::OamAddress | Addr::PpuScroll | Addr::PpuAddress => {
                self.ppu_regs.peek_from_write_only_register()
            }
            Addr::MapperRegisters => {
                match *addr {
                    0x0000..=0x401F => unreachable!(),
                    0x4020..=0x5FFF => mapper.peek_register(self, addr),
                    0x6000..=0xFFFF => mapper.peek_prg(self, addr),
                }
            }

            // Write-only registers and unused register addresses result in a read of Open BUS.
            // APU registers can only be read if the current address bus AND the CPU address bus are in the correct range.
            _ => ReadResult::OPEN_BUS,
        };

        let mut should_apu_read_dominate_normal_read = false;
        let mut should_apu_read_update_data_bus = true;
        let apu_read_value = if self.apu_registers_active() {
            let addr = CpuAddress::new(0x4000 + *addr % 0x20);
            use FriendlyCpuAddress as Addr;
            match addr.to_friendly() {
                Addr::ApuStatus => {
                    // APU status reads only use the data bus when using a DMA address bus.
                    should_apu_read_dominate_normal_read = true;
                    should_apu_read_update_data_bus = address_bus_type != AddressBusType::Cpu;
                    self.apu_regs.read_status(self.master_clock.apu_clock(), &self.cpu_pinout, &self.dmc_dma)
                }
                Addr::Controller1AndStrobe       => self.joypad1.read_status(),
                Addr::Controller2AndFrameCounter => self.joypad2.read_status(),
                // Most APU registers and OAM DMA are write-only. CPU Test Mode is not yet supported.
                _ if addr.is_in_apu_register_range() => ReadResult::OPEN_BUS,
                _ => unreachable!(),
            }
        } else {
            ReadResult::OPEN_BUS
        };

        let value = if should_apu_read_dominate_normal_read {
            apu_read_value.dominate(normal_read_value).resolve(self.cpu_pinout.data_bus)
        } else {
            normal_read_value.dominate(apu_read_value).resolve(self.cpu_pinout.data_bus)
        };

        self.cpu_pinout.data_bus = if should_apu_read_update_data_bus {
            value
        } else {
            normal_read_value.resolve(self.cpu_pinout.data_bus)
        };

        mapper.on_cpu_read(self, addr, value);

        value
    }

    // TODO: APU register mirroring probably affects writes (at least for $2004/$4004), so implement it.
    #[inline]
    #[rustfmt::skip]
    pub fn cpu_write(&mut self, mapper: &mut dyn Mapper, address_bus_type: AddressBusType) {
        let addr = self.cpu_address_bus(address_bus_type);

        use FriendlyCpuAddress as Addr;
        match addr.to_friendly() {
            Addr::CpuInternalRam(index) => self.cpu_internal_ram.write(index, self.cpu_pinout.data_bus),

            // PPU registers.
            Addr::PpuControl => self.ppu_regs.write_ctrl(self.cpu_pinout.data_bus),
            Addr::PpuMask    => self.ppu_regs.write_mask(self.cpu_pinout.data_bus),
            Addr::PpuStatus  => self.ppu_regs.write_status(self.cpu_pinout.data_bus), // Read-only
            Addr::OamAddress => self.ppu_regs.write_oam_addr(self.cpu_pinout.data_bus),
            Addr::OamData    => self.ppu_regs.write_oam_data(&mut self.oam, self.cpu_pinout.data_bus),
            Addr::PpuScroll  => self.ppu_regs.write_scroll(self.cpu_pinout.data_bus),
            Addr::PpuAddress => {
                self.ppu_regs.write_ppu_addr(self.cpu_pinout.data_bus);
                if self.ppu_regs.write_toggle() == WriteToggle::FirstByte {
                    self.set_ppu_address_bus(mapper, self.ppu_regs.current_address);
                }
            }
            Addr::PpuData => {
                // FIXME: The ordering of these three statements seems wrong. Surely the address bus should be set first?
                // This seems likely to be related to how the PPU data bus and address bus have shared bits.
                self.ppu_write(self.ppu_regs.current_address, self.cpu_pinout.data_bus);
                self.ppu_regs.write_ppu_data(self.cpu_pinout.data_bus);
                self.set_ppu_address_bus(mapper, self.ppu_regs.current_address);
            }

            // APU channel registers.
            Addr::Pulse1Control   => self.apu_regs.pulse_1.set_control(self.cpu_pinout.data_bus),
            Addr::Pulse1Sweep     => self.apu_regs.pulse_1.set_sweep(self.cpu_pinout.data_bus),
            Addr::Pulse1Period    => self.apu_regs.pulse_1.set_period_low(self.cpu_pinout.data_bus),
            Addr::Pulse1Length    => self.apu_regs.pulse_1.set_length_and_period_high(self.cpu_pinout.data_bus),
            Addr::Pulse2Control   => self.apu_regs.pulse_2.set_control(self.cpu_pinout.data_bus),
            Addr::Pulse2Sweep     => self.apu_regs.pulse_2.set_sweep(self.cpu_pinout.data_bus),
            Addr::Pulse2Period    => self.apu_regs.pulse_2.set_period_low(self.cpu_pinout.data_bus),
            Addr::Pulse2Length    => self.apu_regs.pulse_2.set_length_and_period_high(self.cpu_pinout.data_bus),
            Addr::TriangleControl => self.apu_regs.triangle.set_control_and_linear(self.cpu_pinout.data_bus),
            Addr::TrianglePeriod  => self.apu_regs.triangle.set_timer_low(self.cpu_pinout.data_bus),
            Addr::TriangleLength  => self.apu_regs.triangle.set_length_and_timer_high(self.cpu_pinout.data_bus),
            Addr::NoiseControl    => self.apu_regs.noise.set_control(self.cpu_pinout.data_bus),
            Addr::NoisePeriod     => self.apu_regs.noise.set_loop_and_period(self.cpu_pinout.data_bus),
            Addr::NoiseLength     => self.apu_regs.noise.set_length(self.cpu_pinout.data_bus),
            Addr::DmcControl      => self.apu_regs.dmc.write_control_byte(&mut self.cpu_pinout),
            Addr::DmcVolume       => self.apu_regs.dmc.write_volume(self.cpu_pinout.data_bus),
            Addr::DmcAddress      => self.apu_regs.dmc.write_sample_start_address(self.cpu_pinout.data_bus),
            Addr::DmcLength       => self.dmc_dma.write_sample_length(self.cpu_pinout.data_bus),

            // Miscellaneous registers.
            Addr::OamDma          => self.oam_dma.prepare_to_start(self.cpu_pinout.data_bus),
            Addr::ApuStatus       => self.apu_regs.write_status(self.master_clock.apu_clock(), &mut self.cpu_pinout, &mut self.dmc_dma),
            Addr::Controller2AndFrameCounter => self.apu_regs.write_frame_counter(self.master_clock.apu_clock(), &mut self.cpu_pinout),
            Addr::Controller1AndStrobe => {
                self.joypad1.change_strobe(self.cpu_pinout.data_bus);
                self.joypad2.change_strobe(self.cpu_pinout.data_bus);
            }
            Addr::MapperRegisters => {
                if matches!(*addr, 0x6000..=0xFFFF) {
                    // TODO: Verify if bus conflicts only occur for address >= 0x6000.
                    if mapper.has_bus_conflicts() {
                        let rom_value = self.cpu_peek_unresolved(mapper, address_bus_type, addr);
                        self.cpu_pinout.data_bus = rom_value.bus_conflict(self.cpu_pinout.data_bus);
                    }

                    self.prg_memory.write(addr, self.cpu_pinout.data_bus);
                }

                mapper.write_register(self, addr, self.cpu_pinout.data_bus);
            }
            Addr::Unused => { /* Do nothing for unused register addresses. */ }
        }

        mapper.on_cpu_write(self, addr, self.cpu_pinout.data_bus);
    }

    pub fn ppu_peek(&self, address: PpuAddress) -> PpuPeek {
        match address.to_u16() {
            0x0000..=0x1FFF => self.peek_chr(address),
            0x2000..=0x3EFF => self.peek_name_table_byte(address),
            0x3F00..=0x3FFF => self.palette_ram.peek(address.to_palette_ram_index()),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    pub fn ppu_internal_read(&mut self, mapper: &mut dyn Mapper) -> PpuPeek {
        let result = mapper.ppu_peek(self, self.ppu_pinout.address());
        mapper.on_ppu_read(self, self.ppu_pinout.address(), result.value());
        result
    }

    #[inline]
    pub fn ppu_write(&mut self, addr: PpuAddress, value: u8) {
        match addr.to_u16() {
            0x0000..=0x1FFF => self.chr_memory.write(&mut self.ciram, &mut self.mapper_custom_pages, addr, value),
            0x2000..=0x3EFF => self.write_name_table_byte(addr, value),
            0x3F00..=0x3FFF => self.palette_ram.write(addr.to_palette_ram_index(), value),
            0x4000..=0xFFFF => unreachable!(),
        }
    }

    pub fn set_ppu_address_bus(&mut self, mapper: &mut dyn Mapper, addr: PpuAddress) {
        let address_changed = self.ppu_pinout.set_address_bus(addr);
        if address_changed {
            mapper.on_ppu_address_change(self, addr);
        }
    }

    pub fn set_ppu_data_bus(&mut self, mapper: &mut dyn Mapper, data: u8) {
        let address_changed = self.ppu_pinout.set_data_bus(data);
        if address_changed {
            mapper.on_ppu_address_change(self, self.ppu_pinout.address());
        }
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

    // See "APU Register Activation" in the README and asm file here: https://github.com/100thCoin/AccuracyCoin
    pub fn apu_registers_active(&self) -> bool {
        matches!(*self.cpu_pinout.address_bus, 0x4000..=0x401F)
    }
}

#[derive(PartialEq, Eq, Clone, Copy, Debug)]
pub enum AddressBusType {
    Cpu,
    OamDma,
    DmcDma,
}