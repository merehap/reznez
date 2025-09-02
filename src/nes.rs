use std::collections::VecDeque;
use std::fmt;
use std::fs::{DirBuilder, File};
use std::io::Read;
use std::path::Path;


use log::Level::Info;
use log::{info, log_enabled, warn};
use num_traits::FromPrimitive;

use crate::apu::apu::Apu;
use crate::apu::apu_registers::{ApuRegisters, FrameCounterWriteStatus};
use crate::cartridge::cartridge::Cartridge;
use crate::cartridge::cartridge_metadata::CartridgeMetadataBuilder;
use crate::cartridge::header_db::HeaderDb;
use crate::cartridge::resolved_metadata::{MetadataResolver, ResolvedMetadata};
use crate::config::Config;
use crate::controller::joypad::Joypad;
use crate::cpu::cpu::{Cpu, IrqStatus, NmiStatus, ResetStatus};
use crate::cpu::cpu_mode::CpuMode;
use crate::cpu::dmc_dma::{DmcDmaAction, DmcDmaState};
use crate::cpu::oam_dma::{OamDmaAction, OamDmaState};
use crate::cpu::step::Step;
use crate::gui::gui::Events;
use crate::logging::formatter;
use crate::logging::formatter::*;
use crate::mapper::{Mapper, MapperParams, NameTableMirroring, PrgBankRegisterId, ReadWriteStatus};
use crate::mapper_list;
use crate::memory::raw_memory::RawMemory;
use crate::memory::bank::bank_index::{BankLocation, ChrBankRegisterId};
use crate::memory::cpu::ports::Ports;
use crate::memory::memory::Memory;
use crate::ppu::clock::Clock;
use crate::ppu::ppu::Ppu;
use crate::ppu::render::frame::Frame;

pub struct Nes {
    cpu: Cpu,
    ppu: Ppu,
    apu: Apu,
    memory: Memory,
    mapper: Box<dyn Mapper>,
    resolved_metadata: ResolvedMetadata,
    metadata_resolver: MetadataResolver,
    frame: Frame,
    cycle: u64,

    log_formatter: Box<dyn Formatter>,
    snapshots: Snapshots,
    latest_values: LatestValues,
}

impl Nes {
    pub fn load_cartridge(path: &Path) -> Cartridge {
        info!("Loading ROM '{}'.", path.display());
        let mut raw_header_and_data = Vec::new();
        File::open(path).unwrap().read_to_end(&mut raw_header_and_data).unwrap();
        let raw_header_and_data = RawMemory::from_vec(raw_header_and_data);
        Cartridge::load(path, &raw_header_and_data).unwrap()
    }

    pub fn new(config: &Config, cartridge: Cartridge) -> Nes {
        let (mapper, mapper_params, metadata_resolver) = Nes::load_rom(config, cartridge);

        if let Err(err) = DirBuilder::new().recursive(true).create("saveram") {
            warn!("Failed to create saveram directory. {err}");
        }

        let (joypad1, joypad2) = (Joypad::new(), Joypad::new());

        let latest_values = LatestValues::new(&mapper_params);

        let ports = Ports::new(joypad1, joypad2);
        let mut memory = Memory::new(mapper_params, ports, config.ppu_clock, config.system_palette.clone());

        Nes {
            cpu: Cpu::new(&mut memory, config.starting_cpu_cycle, config.cpu_step_formatting),
            ppu: Ppu::new(&memory),
            apu: Apu::new(config.disable_audio),
            memory,
            mapper,
            resolved_metadata: metadata_resolver.resolve(),
            metadata_resolver,
            frame: Frame::new(),
            cycle: 0,

            log_formatter: Box::new(MesenFormatter),
            snapshots: Snapshots::new(),
            latest_values,
        }
    }

    pub fn cpu(&self) -> &Cpu {
        &self.cpu
    }

    pub fn ppu(&self) -> &Ppu {
        &self.ppu
    }

    pub fn memory(&self) -> &Memory {
        &self.memory
    }

    pub fn mapper(&self) -> &dyn Mapper {
        &*self.mapper
    }

    pub fn resolved_metadata(&self) -> &ResolvedMetadata {
        &self.resolved_metadata
    }

    pub fn metadata_resolver(&self) -> &MetadataResolver {
        &self.metadata_resolver
    }

    pub fn frame(&self) -> &Frame {
        &self.frame
    }

    pub fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    pub fn cycle(&self) -> u64 {
        self.cycle
    }

    pub fn stack_pointer(&self) -> u8 {
        self.memory.stack_pointer()
    }

    fn load_rom(config: &Config, cartridge: Cartridge) -> (Box<dyn Mapper>, MapperParams, MetadataResolver) {
        let header = cartridge.header();
        let header_db = HeaderDb::load();
        let cartridge_mapper_number = header.mapper_number().unwrap();
        let prg_rom_hash = header.prg_rom_hash().unwrap();
        let mut db_header = CartridgeMetadataBuilder::new().build();
        if let Some(db_cartridge_metadata) = header_db.header_from_db(
                header.full_hash().unwrap(), prg_rom_hash, cartridge_mapper_number, header.submapper_number()) {
            db_header = db_cartridge_metadata;
            if cartridge_mapper_number != db_header.mapper_number().unwrap() {
                warn!("Mapper number in ROM ({}) does not match the one in the DB ({}).",
                    cartridge_mapper_number, db_header.mapper_number().unwrap());
            }

            assert_eq!(header.prg_rom_size().unwrap(), db_header.prg_rom_size().unwrap());
            if header.chr_rom_size().unwrap() != db_header.chr_rom_size().unwrap_or(0) {
                warn!("CHR ROM size in cartridge did not match size in header DB.");
            }
        } else {
            warn!("ROM not found in header database.");
        }

        let mut hard_coded_overrides = CartridgeMetadataBuilder::new();
        if let Some((number, sub_number, full_hash, prg_hash)) =
                header_db.override_submapper_number(header.full_hash().unwrap(), prg_rom_hash) && cartridge_mapper_number == number {

            info!("Using override submapper {sub_number} for this ROM. Full hash: {full_hash:X} , PRG hash: {prg_hash:X}");
            hard_coded_overrides
                .mapper_and_submapper_number(number, Some(sub_number))
                .full_hash(full_hash)
                .prg_rom_hash(prg_hash);
        }

        let mut db_extension_metadata = CartridgeMetadataBuilder::new();
        if let Some((number, sub_number, full_hash, prg_hash)) =
                header_db.missing_submapper_number(header.full_hash().unwrap(), prg_rom_hash) && cartridge_mapper_number == number {

            info!("Using submapper {sub_number} from the database extension for this ROM. Full hash: {full_hash:X} , PRG hash: {prg_hash:X}");
            db_extension_metadata
                .mapper_and_submapper_number(number, Some(sub_number))
                .full_hash(full_hash)
                .prg_rom_hash(prg_hash);
        }

        let mut metadata_resolver = MetadataResolver {
            hard_coded_overrides: hard_coded_overrides.build(),
            cartridge: header.clone(),
            database: db_header,
            database_extension: db_extension_metadata.build(),
            // This can only be set correctly once the mapper has been looked up.
            layout_has_prg_ram: false,
        };

        let mapper = mapper_list::lookup_mapper(&metadata_resolver, &cartridge);

        let name_table_mirroring_index = usize::try_from(metadata_resolver.cartridge.name_table_mirroring_index().unwrap()).unwrap();
        if let Some(mirroring) = mapper.layout().cartridge_selection_name_table_mirrorings()[name_table_mirroring_index] {
            metadata_resolver.cartridge.set_name_table_mirroring(mirroring);
        }

        let metadata = metadata_resolver.resolve();
        let mut mapper_params = mapper.layout().make_mapper_params(&metadata, &cartridge, config.allow_saving);
        mapper.init_mapper_params(&mut mapper_params);

        metadata_resolver.layout_has_prg_ram = mapper.layout().has_prg_ram();
        let metadata = metadata_resolver.resolve();
        info!("ROM loaded.\n{metadata}");

        (mapper, mapper_params, metadata_resolver)
    }

    pub fn mute(&mut self) {
        self.apu.mute();
    }

    pub fn reset(&mut self) {
        self.cpu.reset();
        self.memory.apu_regs_mut().reset();
    }

    pub fn step_frame(&mut self) {
        loop {
            let step_result = self.step();
            if step_result.is_last_cycle_of_frame {
                if self.cpu.mode_state().is_jammed() {
                    info!("CPU is jammed!");
                }

                break;
            }
        }
    }

    pub fn step(&mut self) -> StepResult {
        let mut step = None;
        let is_last_cycle_of_frame;
        match self.cycle % 3 {
            0 => {
                self.apu_step();
                step = self.cpu_step_first_half();
                is_last_cycle_of_frame = self.ppu_step();
            }
            1 => {
                self.cpu_step_second_half();
                is_last_cycle_of_frame = self.ppu_step();
            }
            2 => {
                is_last_cycle_of_frame = self.ppu_step();
                self.snapshots.start_next();
            }
            _ => unreachable!(),
        }

        self.cycle += 1;

        StepResult {
            step,
            is_last_cycle_of_frame,
        }
    }

    fn apu_step(&mut self) {
        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().apu_regs(self.memory.apu_regs());
        }

        let (apu_regs, dmc_dma) = self.memory.apu_regs_and_dmc_dma_mut();
        self.apu.step(apu_regs, dmc_dma);

        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().frame_irq(self.memory.apu_regs(), &self.cpu);
        }

        self.detect_changes();

        self.memory.apu_regs_mut().clock_mut().increment();
    }

    fn cpu_step_first_half(&mut self) -> Option<Step> {
        let cycle_parity = self.memory.apu_regs().clock().cycle_parity();
        self.memory.increment_cpu_cycle();

        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().instruction(self.cpu.mode_state().state_label());
        }
        let mut interrupt_text = String::new();
        if log_enabled!(target: "cpuinstructions", Info) {
            interrupt_text = formatter::interrupts(self);
        }

        let step = self.cpu.step_first_half(&mut *self.mapper, &mut self.memory, cycle_parity);
        if log_enabled!(target: "cpuinstructions", Info) &&
                let Some((current_instruction, start_address)) = self.cpu.mode_state().new_instruction_with_address() {

            let formatted_instruction = self.log_formatter.format_instruction(
                self,
                current_instruction,
                start_address,
                interrupt_text);
            info!("{formatted_instruction}");
        }

        if log_enabled!(target: "timings", Info) {
            if self.memory.apu_regs().frame_counter_write_status() == FrameCounterWriteStatus::Initialized {
                self.snapshots.start();
            }

            self.snapshots.current().cpu_cycle(self.memory.cpu_cycle());
            self.snapshots.current().irq_status(self.cpu.irq_status());
            self.snapshots.current().nmi_status(self.cpu.nmi_status());
        }

        self.detect_changes();
        step
    }

    fn cpu_step_second_half(&mut self) {
        self.cpu.step_second_half(&mut *self.mapper, &mut self.memory);
        self.detect_changes();
    }

    fn ppu_step(&mut self) -> bool {
        let rendering_enabled = self.memory.ppu_regs().rendering_enabled();
        let is_last_cycle_of_frame = self.memory.ppu_regs_mut().clock_mut().tick(rendering_enabled);
        if log_enabled!(target: "timings", Info) {
            self.snapshots.current().add_ppu_position(self.memory.ppu_regs().clock());
        }

        self.ppu.step(&mut *self.mapper, &mut self.memory, &mut self.frame);

        self.detect_changes();

        is_last_cycle_of_frame
    }

    fn detect_changes(&mut self) {
        if log_enabled!(target: "cpuflowcontrol", Info) {
            let apu_regs = self.memory.apu_regs();
            let mapper_params = self.memory.mapper_params();
            let latest = &mut self.latest_values;

            if latest.apu_frame_irq_pending != apu_regs.frame_irq_pending() {
                latest.apu_frame_irq_pending = apu_regs.frame_irq_pending();
                if latest.apu_frame_irq_pending {
                    info!("APU Frame IRQ pending. CPU cycle: {}", self.memory.cpu_cycle());
                }
            }

            if latest.dmc_irq_pending != apu_regs.dmc_irq_pending() {
                latest.dmc_irq_pending = apu_regs.dmc_irq_pending();
                if latest.dmc_irq_pending {
                    info!("DMC IRQ pending. CPU cycle: {}", self.memory.cpu_cycle());
                }
            }

            if latest.mapper_irq_pending != mapper_params.irq_pending() {
                latest.mapper_irq_pending = mapper_params.irq_pending();
                if latest.mapper_irq_pending {
                    info!("Mapper IRQ pending. CPU cycle: {}", self.memory.cpu_cycle());
                }
            }

            if latest.irq_status != self.cpu.irq_status() {
                latest.irq_status = self.cpu.irq_status();
                info!("IRQ status in CPU: {:?}. Cycle: {}", latest.irq_status, self.memory.cpu_cycle());
            }

            if latest.nmi_status != self.cpu.nmi_status() {
                latest.nmi_status = self.cpu.nmi_status();
                info!("NMI status: {:?}. Cycle: {}", latest.nmi_status, self.memory.cpu_cycle());
            }

            if latest.reset_status != self.cpu.reset_status() {
                latest.reset_status = self.cpu.reset_status();
                info!("RESET status: {:?}. Cycle: {}", latest.reset_status, self.memory.cpu_cycle());
            }

            if latest.dmc_dma_action != self.memory.dmc_dma.latest_action() {
                let previously_halted = latest.dmc_dma_action.cpu_should_be_halted();
                latest.dmc_dma_action = self.memory.dmc_dma.latest_action();
                let currently_halted = latest.dmc_dma_action.cpu_should_be_halted();
                if !previously_halted && currently_halted {
                    info!("CPU halted for DMC DMA transfer at {}.", self.memory.dmc_dma_address());
                }
            }

            if latest.extended_cpu_mode.oam_dma_action != self.memory.oam_dma.latest_action() {
                let previously_halted = latest.oam_dma_action.cpu_should_be_halted();
                latest.oam_dma_action = self.memory.oam_dma.latest_action();
                let currently_halted = latest.oam_dma_action.cpu_should_be_halted();
                if !previously_halted && currently_halted {
                    info!("CPU halted for OAM DMA transfer at {}.", self.memory.oam_dma.address());
                }
            }
        }

        assert!(!log_enabled!(target: "cpumode", Info) || !log_enabled!(target: "detailedcpumode", Info),
                "Either cpumode OR detailedcpumode can be specified, but not both.");

        if log_enabled!(target: "cpumode", Info) {
            let latest = &mut self.latest_values;
            let latest_extended_cpu_mode = ExtendedCpuMode {
                cpu_mode: self.cpu.mode_state().mode(),
                dmc_dma_state: self.memory.dmc_dma.state(),
                dmc_dma_action: self.memory.dmc_dma.latest_action(),
                oam_dma_state: self.memory.oam_dma.state(),
                oam_dma_action: self.memory.oam_dma.latest_action(),
            };

            if latest_extended_cpu_mode.coarse_change_occurred(&latest.extended_cpu_mode) {
                latest.extended_cpu_mode = latest_extended_cpu_mode.clone();
                info!("CPU Cycle: {:>7} *** CPU Mode = {:<11} ***",
                    self.memory.cpu_cycle(), latest_extended_cpu_mode.to_string());
            }
        }

        if log_enabled!(target: "detailedcpumode", Info) {
            let latest = &mut self.latest_values;
            let latest_extended_cpu_mode = ExtendedCpuMode {
                cpu_mode: self.cpu.mode_state().mode(),
                dmc_dma_state: self.memory.dmc_dma.state(),
                dmc_dma_action: self.memory.dmc_dma.latest_action(),
                oam_dma_state: self.memory.oam_dma.state(),
                oam_dma_action: self.memory.oam_dma.latest_action(),
            };

            let (mode_changed, dmc_changed, oam_changed) =
                latest_extended_cpu_mode.fine_change_occurred(&latest.extended_cpu_mode);
            if mode_changed || dmc_changed || oam_changed {
                let ExtendedCpuMode { dmc_dma_action, oam_dma_action, .. } = latest_extended_cpu_mode;
                let ExtendedCpuMode { dmc_dma_state, oam_dma_state, .. } = latest.extended_cpu_mode;
                let mode = if dmc_dma_action == DmcDmaAction::DoNothing && oam_dma_action == OamDmaAction::DoNothing {
                    if dmc_dma_state == DmcDmaState::Idle && oam_dma_state == OamDmaState::Idle {
                        latest_extended_cpu_mode.cpu_mode.to_string()
                    } else {
                        latest_extended_cpu_mode.cpu_mode.to_instruction_mode_string()
                    }
                } else {
                    "HALTED".to_owned()
                };
                let dmc_action = if dmc_changed || oam_changed {
                    let state = format!("{dmc_dma_state:?}");
                    let action = format!("{dmc_dma_action:?}");
                    format!("DMC = {state:<13} -> {action:<9} ")
                } else {
                    " ".repeat(33)
                };
                let oam_action = if dmc_changed || oam_changed {
                    let state = format!("{oam_dma_state:?}");
                    let action = format!("{oam_dma_action:?}");
                    format!(" | OAM = {state:<15} -> {action:<9}  ")
                } else {
                    " ".repeat(39)
                };

                latest.extended_cpu_mode = latest_extended_cpu_mode.clone();
                if !mode_changed &&
                        dmc_dma_state == DmcDmaState::Idle && dmc_dma_action == DmcDmaAction::DoNothing &&
                        oam_dma_state == OamDmaState::Idle && oam_dma_action == OamDmaAction::DoNothing {
                    info!("");
                } else {
                    info!("CPU Cycle: {:>7} *** {:<11} {dmc_action} {oam_action}", self.memory.cpu_cycle(), mode);
                }
            }
        }

        if log_enabled!(target: "mapperupdates", Info) {
            let mapper_params = self.memory.mapper_params();
            let prg_memory = mapper_params.prg_memory();
            let chr_memory = mapper_params.chr_memory();
            let latest = &mut self.latest_values;

            if latest.prg_layout_index != prg_memory.layout_index() {
                info!("PRG layout changed to index {}. Previously: {}.", prg_memory.layout_index(), latest.prg_layout_index);
                latest.prg_layout_index = prg_memory.layout_index();
            }

            if latest.chr_layout_index != chr_memory.layout_index() {
                info!("CHR layout changed to index {}. Previously: {}.", chr_memory.layout_index(), latest.chr_layout_index);
                latest.chr_layout_index = chr_memory.layout_index();
            }

            let prg_registers = mapper_params.prg_memory().bank_registers().registers();
            if &latest.prg_registers != prg_registers {
                for (i, latest_bank_location) in latest.prg_registers.iter_mut().enumerate() {
                    if *latest_bank_location != prg_registers[i] {
                        let id: PrgBankRegisterId = FromPrimitive::from_usize(i).unwrap();
                        match (prg_registers[i], *latest_bank_location) {
                            (BankLocation::Index(curr), BankLocation::Index(prev)) =>
                                info!("BankRegister {id:?} changed to {}. Previously: {}", curr.to_raw(), prev.to_raw()),
                            (BankLocation::Index(curr), BankLocation::Ciram(prev)) =>
                                info!("BankRegister {id:?} changed to {}. Previously: {prev:?}", curr.to_raw()),
                            (BankLocation::Ciram(curr), BankLocation::Index(prev)) =>
                                info!("BankRegister {id:?} changed to {curr:?}. Previously: {}", prev.to_raw()),
                            (BankLocation::Ciram(curr), BankLocation::Ciram(prev)) =>
                                info!("BankRegister {id:?} changed to Ciram{curr:?}. Previously: Ciram{prev:?}"),
                        }
                    }
                }

                latest.prg_registers = *prg_registers;
            }

            let chr_registers = mapper_params.chr_memory().bank_registers().registers();
            if &latest.chr_registers != chr_registers {
                for (i, latest_bank_location) in latest.prg_registers.iter_mut().enumerate() {
                    if *latest_bank_location != chr_registers[i] {
                        let id: ChrBankRegisterId = FromPrimitive::from_usize(i).unwrap();
                        match (chr_registers[i], *latest_bank_location) {
                            (BankLocation::Index(curr), BankLocation::Index(prev)) =>
                                info!("BankRegister {id:?} changed to {}. Previously: {}", curr.to_raw(), prev.to_raw()),
                            (BankLocation::Index(curr), BankLocation::Ciram(prev)) =>
                                info!("BankRegister {id:?} changed to {}. Previously: {prev:?}", curr.to_raw()),
                            (BankLocation::Ciram(curr), BankLocation::Index(prev)) =>
                                info!("BankRegister {id:?} changed to {curr:?}. Previously: {}", prev.to_raw()),
                            (BankLocation::Ciram(curr), BankLocation::Ciram(prev)) =>
                                info!("BankRegister {id:?} changed to Ciram{curr:?}. Previously: Ciram{prev:?}"),
                        }
                    }
                }

                latest.chr_registers = *chr_registers;
            }

            let meta_registers = mapper_params.chr_memory().bank_registers().meta_registers();
            if &latest.meta_registers != meta_registers {
                for (i, latest_bank_register_id) in latest.meta_registers.iter_mut().enumerate() {
                    if *latest_bank_register_id != meta_registers[i] {
                        let id: PrgBankRegisterId = FromPrimitive::from_usize(i).unwrap();
                        info!("MetaRegister {id:?} changed to {:?}. Previously: {latest_bank_register_id:?}.", meta_registers[i]);
                        *latest_bank_register_id = meta_registers[i];
                    }
                }
            }

            if latest.name_table_mirroring != mapper_params.name_table_mirroring() {
                info!("NameTableMirroring changed to {}. Previously: {}",
                    mapper_params.name_table_mirroring(), latest.name_table_mirroring);
                latest.name_table_mirroring = mapper_params.name_table_mirroring();
            }

            let prg_read_write_statuses = mapper_params.prg_memory.bank_registers().read_write_statuses();
            if &latest.read_write_statuses != prg_read_write_statuses {
                for (i, latest_read_write_status) in latest.read_write_statuses.iter_mut().enumerate() {
                    if *latest_read_write_status != prg_read_write_statuses[i] {
                        info!("RamStatus register S{i} changed to {:?}. Previously: {:?}",
                            prg_read_write_statuses[i],
                            *latest_read_write_status);
                        *latest_read_write_status = prg_read_write_statuses[i];
                    }
                }
            }
        }
    }

    #[inline]
    pub fn process_gui_events(&mut self, events: &Events) {
        for (button, status) in &events.joypad1_button_statuses {
            info!("Joypad 1: button {button:?} status is {status:?}");
            self.memory.ports_mut().joypad1.set_button_status(*button, *status);
        }

        for (button, status) in &events.joypad2_button_statuses {
            self.memory.ports_mut().joypad2.set_button_status(*button, *status);
        }
    }
}

struct LatestValues {
    apu_frame_irq_pending: bool,
    dmc_irq_pending: bool,
    mapper_irq_pending: bool,

    irq_status: IrqStatus,
    nmi_status: NmiStatus,
    reset_status: ResetStatus,

    extended_cpu_mode: ExtendedCpuMode,
    dmc_dma_action: DmcDmaAction,
    oam_dma_action: OamDmaAction,

    prg_layout_index: u8,
    chr_layout_index: u8,
    prg_registers: [BankLocation; 5],
    chr_registers: [BankLocation; 18],
    meta_registers: [ChrBankRegisterId; 2],
    name_table_mirroring: NameTableMirroring,
    read_write_statuses: [ReadWriteStatus; 15],
}

impl LatestValues {
    fn new(initial_params: &MapperParams) -> Self {
        Self {
            apu_frame_irq_pending: false,
            dmc_irq_pending: false,
            mapper_irq_pending: false,

            irq_status: IrqStatus::Inactive,
            nmi_status: NmiStatus::Inactive,
            reset_status: ResetStatus::Inactive,

            extended_cpu_mode: ExtendedCpuMode::new(),
            dmc_dma_action: DmcDmaAction::DoNothing,
            oam_dma_action: OamDmaAction::DoNothing,

            prg_layout_index: initial_params.prg_memory.layout_index(),
            chr_layout_index: initial_params.chr_memory.layout_index(),
            prg_registers: *initial_params.prg_memory().bank_registers().registers(),
            chr_registers: *initial_params.chr_memory().bank_registers().registers(),
            meta_registers: *initial_params.chr_memory().bank_registers().meta_registers(),
            name_table_mirroring: initial_params.chr_memory().name_table_mirroring(),
            read_write_statuses: *initial_params.prg_memory().bank_registers().read_write_statuses(),
        }
    }
}

#[derive(Clone, Debug)]
struct ExtendedCpuMode {
    cpu_mode: CpuMode,
    dmc_dma_state: DmcDmaState,
    dmc_dma_action: DmcDmaAction,
    oam_dma_state: OamDmaState,
    oam_dma_action: OamDmaAction,
}

impl ExtendedCpuMode {
    fn new() -> Self {
        Self {
            cpu_mode: CpuMode::StartNext,
            dmc_dma_state: DmcDmaState::Idle,
            dmc_dma_action: DmcDmaAction::DoNothing,
            oam_dma_state: OamDmaState::Idle,
            oam_dma_action: OamDmaAction::DoNothing,
        }
    }

    fn coarse_change_occurred(&self, prev: &ExtendedCpuMode) -> bool {
        if (prev.dmc_dma_action == DmcDmaAction::DoNothing && self.dmc_dma_action != DmcDmaAction::DoNothing) ||
                (prev.dmc_dma_action != DmcDmaAction::DoNothing && self.dmc_dma_action == DmcDmaAction::DoNothing) ||
                (prev.oam_dma_action == OamDmaAction::DoNothing && self.oam_dma_action != OamDmaAction::DoNothing) ||
                (prev.oam_dma_action != OamDmaAction::DoNothing && self.oam_dma_action == OamDmaAction::DoNothing) {
            return true;
        }

        if self.dmc_dma_action.cpu_should_be_halted() || self.oam_dma_action.cpu_should_be_halted() {
            return false;
        }

        match (prev.cpu_mode, self.cpu_mode) {
            (_, CpuMode::StartNext) => false,
            (prev, curr) if prev == curr => false,
            (CpuMode::Instruction(_, _), CpuMode::Instruction(_, _)) => false,
            (_, _) => true,
        }
    }

    fn fine_change_occurred(&self, prev: &ExtendedCpuMode) -> (bool, bool, bool) {
        let fine_cpu_mode_changed = match (prev.cpu_mode, self.cpu_mode) {
            (_, CpuMode::StartNext) => false,
            (prev, curr) if prev == curr => false,
            (CpuMode::Instruction(_, prev_instr_mode), CpuMode::Instruction(_, curr_instr_mode))
                if prev_instr_mode == curr_instr_mode => false,
            (_, _) => true,
        };

        let mode_changed = if matches!((prev.cpu_mode, self.cpu_mode), (CpuMode::Instruction(..), CpuMode::Instruction(..))) {
            false
        } else {
            fine_cpu_mode_changed
        };

        let dmc_changed = prev.dmc_dma_state != self.dmc_dma_state || prev.dmc_dma_action != self.dmc_dma_action;
        let oam_changed = prev.oam_dma_state != self.oam_dma_state || prev.oam_dma_action != self.oam_dma_action;

        (mode_changed, dmc_changed, oam_changed)
    }
}

impl fmt::Display for ExtendedCpuMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.dmc_dma_action.cpu_should_be_halted(), self.oam_dma_action.cpu_should_be_halted()) {
            (false, false) => write!(f, "{}", self.cpu_mode),
            (false, true ) => write!(f, "OAM DMA"),
            (true , false) => write!(f, "DMC DMA"),
            (true , true ) => write!(f, "DMC and OAM DMA"),
        }
    }
}

struct Snapshots {
    active: bool,
    snapshots: Vec<Snapshot>,
    builder: SnapshotBuilder,
    max_count: usize,
}

impl Snapshots {
    fn new() -> Snapshots {
        Snapshots {
            active: false,
            snapshots: Vec::new(),
            builder: SnapshotBuilder::new(),
            max_count: 29832 + 10,
        }
    }

    fn start(&mut self) {
        self.snapshots = Vec::new();
        self.active = true;
    }

    fn clear(&mut self) {
        self.snapshots = Vec::new();
        self.builder = SnapshotBuilder::new();
    }

    fn count(&self) -> usize {
        self.snapshots.len()
    }

    fn current(&mut self) -> &mut SnapshotBuilder {
        &mut self.builder
    }

    fn start_next(&mut self) {
        if !self.active {
            return;
        }

        let snapshot = std::mem::take(&mut self.builder).build();
        self.snapshots.push(snapshot);

        if self.count() >= self.max_count {
            self.active = false;
            info!("{}", self.format());
            info!("");
            self.clear();
        }
    }

    fn format(&self) -> String {
        let mut cpu_cycle   = "CPU Cycle   ".to_string();
        let mut apu_cycle   = "APU Cycle   ".to_string();
        let mut cycle_count = "Cycle Offset".to_string();
        let mut apu_parity  = "APU Parity  ".to_string();
        let mut instr       = "CPU         ".to_string();
        let mut fcw_status  = "FRM Count   ".to_string();
        let mut nmi_status  = "NMI Status  ".to_string();
        let mut irq_status  = "IRQ Status  ".to_string();
        let mut frame_irq   = "FRM         ".to_string();
        let mut ppu_vpos    = "PPU VPOS    ".to_string();
        let mut ppu_hpos    = "PPU HPOS    ".to_string();

        let mut append_cycle = |index, skip| {
            let snapshot: &Snapshot = &self.snapshots[index];
            append(&mut cpu_cycle, &center(&snapshot.cpu_cycle.to_string()), true, skip);
            append(&mut apu_cycle, &center(&snapshot.apu_cycle.to_string()), true, skip);
            append(&mut cycle_count, &center(&(snapshot.cpu_cycle - self.snapshots[0].cpu_cycle).to_string()), true, skip);
            append(&mut apu_parity, &center(&snapshot.apu_parity), true, skip);

            let mut vpos = String::new();
            let mut hpos = String::new();
            for (v, h) in snapshot.ppu_pos {
                vpos.push_str(&center_n(3, &v.to_string()));
                hpos.push_str(&center_n(3, &h.to_string()));
            }

            append(&mut ppu_vpos, &vpos, true, skip);
            append(&mut ppu_hpos, &hpos, true, skip);

            append(&mut instr, &center(&snapshot.instruction.to_string()), true, skip);
            append(&mut fcw_status, &center(&format!("{:?}", snapshot.frame_counter_write_status)),
                snapshot.frame_counter_write_status != FrameCounterWriteStatus::Inactive, skip);
            append(&mut nmi_status, &center(&format!("{:?}", snapshot.nmi_status)), snapshot.nmi_status != NmiStatus::Inactive, skip);
            append(&mut irq_status, &center(&format!("{:?}", snapshot.irq_status)), snapshot.irq_status != IrqStatus::Inactive, skip);
            append(&mut frame_irq, &center("Raise IRQ"), snapshot.frame_irq, skip);
        };

        append_cycle(0, false);
        append_cycle(1, true);

        let len = self.snapshots.len();
        for index in len - 13..len {
            append_cycle(index, false);
        }

        vec![cpu_cycle, apu_cycle, cycle_count, apu_parity, instr,
             nmi_status, irq_status, frame_irq, /*fcw_status, */ppu_vpos, ppu_hpos].join("\n")
    }
}

fn append(field: &mut String, value: &str, active: bool, skip: bool) {
    let result = if skip {
        "........"
    } else if active {
        value
    } else {
        "               "
    };

    field.push_str(result);
}

fn center(text: &str) -> String {
    center_n(13, text)
}

fn center_n(n: usize, text: &str) -> String {
    assert!(n >= 2);

    let text: String = text.chars().take(n).collect();
    let back = (n - text.len()) / 2;
    let front = n - text.len() - back;

    let mut result = "[".to_string();
    result.push_str(&String::from_utf8(vec![b' '; front]).unwrap());
    result.push_str(&text);
    result.push_str(&String::from_utf8(vec![b' '; back]).unwrap());
    result.push(']');
    result
}

struct Snapshot {
    cpu_cycle: i64,
    apu_cycle: u16,
    apu_parity: String,
    instruction: String,
    frame_counter_write_status: FrameCounterWriteStatus,
    frame_irq: bool,
    irq_status: IrqStatus,
    nmi_status: NmiStatus,
    ppu_pos: [(u16, u16); 3],
}

#[derive(Default)]
struct SnapshotBuilder {
    cpu_cycle: Option<i64>,
    apu_cycle: Option<u16>,
    apu_parity: Option<String>,
    instruction: String,
    frame_counter_write_status: Option<FrameCounterWriteStatus>,
    frame_irq: Option<bool>,
    irq_status: Option<IrqStatus>,
    nmi_status: Option<NmiStatus>,
    ppu_pos: VecDeque<(u16, u16)>,
}

impl SnapshotBuilder {
    fn new() -> Self {
        Self::default()
    }

    fn cpu_cycle(&mut self, value: i64) {
        self.cpu_cycle = Some(value);
    }

    fn apu_regs(&mut self, regs: &ApuRegisters) {
        let clock = regs.clock();
        self.apu_cycle = Some(clock.cycle());
        self.apu_parity = Some(clock.cycle_parity().to_string());
        self.frame_counter_write_status = Some(regs.frame_counter_write_status());
    }

    fn frame_irq(&mut self, regs: &ApuRegisters, cpu: &Cpu) {
        self.frame_irq = Some(regs.frame_irq_pending() && !cpu.status().interrupts_disabled);
    }

    fn add_ppu_position(&mut self, clock: &Clock) {
        assert!(self.ppu_pos.len() < 4);
        if self.ppu_pos.len() == 3 {
            self.ppu_pos.pop_front();
        }

        self.ppu_pos.push_back((clock.scanline(), clock.cycle()));
    }

    fn instruction(&mut self, value: String) {
        self.instruction = value;
    }

    fn irq_status(&mut self, irq_status: IrqStatus) {
        self.irq_status = Some(irq_status);
    }

    fn nmi_status(&mut self, nmi_status: NmiStatus) {
        self.nmi_status = Some(nmi_status);
    }

    fn build(self) -> Snapshot {
        assert_eq!(self.ppu_pos.len(), 3);
        Snapshot {
            cpu_cycle: self.cpu_cycle.unwrap(),
            apu_cycle: self.apu_cycle.unwrap(),
            apu_parity: self.apu_parity.unwrap(),
            instruction: self.instruction,
            frame_counter_write_status: self.frame_counter_write_status.unwrap(),
            frame_irq: self.frame_irq.unwrap(),
            irq_status: self.irq_status.unwrap(),
            nmi_status: self.nmi_status.unwrap(),
            ppu_pos: [self.ppu_pos[0], self.ppu_pos[1], self.ppu_pos[2]],
        }
    }
}

pub struct StepResult {
    pub step: Option<Step>,
    pub is_last_cycle_of_frame: bool,
}
