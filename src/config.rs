use std::fs::File;
use std::io::Read;
use std::path::{Path, PathBuf};
use std::str::FromStr;

use log::{info, warn};
use structopt::StructOpt;

use crate::cartridge::cartridge::Cartridge;
use crate::cartridge::cartridge_metadata::{CartridgeMetadata, CartridgeMetadataBuilder};
use crate::cartridge::header_db::HeaderDb;
use crate::cartridge::resolved_metadata::MetadataResolver;
use crate::gui::egui_gui::EguiGui;
use crate::gui::gui::Gui;
use crate::gui::no_gui::NoGui;
use crate::mapper::{Mapper, MapperParams};
use crate::mapper_list;
use crate::memory::raw_memory::RawMemory;
use crate::ppu::clock::Clock;
use crate::ppu::palette::system_palette::SystemPalette;
use crate::ppu::render::frame_rate::{FrameRate, TargetFrameRate};

pub struct Config {
    pub cartridge: Cartridge,
    pub metadata_resolver: MetadataResolver,
    pub starting_cpu_cycle: i64,
    pub ppu_clock: Clock,
    pub system_palette: SystemPalette,
    pub target_frame_rate: TargetFrameRate,
    pub disable_audio: bool,
    pub stop_frame: Option<i64>,
    pub frame_dump: bool,
    pub cpu_step_formatting: CpuStepFormatting,
}

impl Config {
    pub fn new(opt: &Opt) -> (Config, Box<dyn Mapper>, MapperParams) {
        let (cartridge, mapper, mapper_params, metadata_resolver) = Config::load_rom(&opt.rom_path, !opt.prevent_saving);
        let system_palette =
            SystemPalette::parse(include_str!("../palettes/2C02.pal")).unwrap();

        let config = Config {
            cartridge,
            metadata_resolver,
            starting_cpu_cycle: 0,
            ppu_clock: Clock::mesen_compatible(),
            system_palette,
            target_frame_rate: opt.target_frame_rate,
            disable_audio: opt.disable_audio,
            stop_frame: opt.stop_frame,
            frame_dump: opt.frame_dump,
            cpu_step_formatting: opt.cpu_step_formatting,
        };

        (config, mapper, mapper_params)
    }

    pub fn gui(opt: &Opt) -> Box<dyn Gui> {
        match opt.gui {
            GuiType::NoGui => Box::new(NoGui::new()) as Box<dyn Gui>,
            GuiType::Egui => Box::new(EguiGui),
        }
    }

    pub fn load_rom(path: &Path, allow_saving: bool) -> (Cartridge, Box<dyn Mapper>, MapperParams, MetadataResolver) {
        info!("Loading ROM '{}'.", path.display());
        let mut raw_header_and_data = Vec::new();
        File::open(path).unwrap().read_to_end(&mut raw_header_and_data).unwrap();
        let raw_header_and_data = RawMemory::from_vec(raw_header_and_data);
        let (mut header, cartridge_selected_mirroring) = CartridgeMetadata::parse(&raw_header_and_data).unwrap();
        let cartridge = Cartridge::load(path, &header, &raw_header_and_data, allow_saving).unwrap();
        let prg_rom_hash = crc32fast::hash(cartridge.prg_rom().as_slice());
        header.set_prg_rom_hash(prg_rom_hash);

        let header_db = HeaderDb::load();
        let cartridge_mapper_number = header.mapper_number().unwrap();
        let mut db_header = CartridgeMetadataBuilder::new().build();
        if let Some(db_cartridge_metadata) = header_db.header_from_db(header.full_hash().unwrap(), prg_rom_hash, cartridge_mapper_number, header.submapper_number()) {
            db_header = db_cartridge_metadata;
            if cartridge_mapper_number != db_header.mapper_number().unwrap() {
                warn!("Mapper number in ROM ({}) does not match the one in the DB ({}).",
                    cartridge_mapper_number, header.mapper_number().unwrap());
            }

            assert_eq!(header.prg_rom_size().unwrap(), db_header.prg_rom_size().unwrap());
            if header.chr_rom_size().unwrap() != db_header.chr_rom_size().unwrap_or(0) {
                warn!("CHR ROM size in cartridge did not match size in header DB.");
            }
        } else {
            warn!("ROM not found in header database.");
        }

        let mut hard_coded_overrides = CartridgeMetadataBuilder::new();
        if let Some((number, sub_number, data_hash, prg_hash)) =
                header_db.override_submapper_number(header.full_hash().unwrap(), prg_rom_hash) && cartridge_mapper_number == number {

            info!("Using override submapper {sub_number} for this ROM. Full hash: {data_hash} , PRG hash: {prg_hash}");
            hard_coded_overrides.mapper_and_submapper_number(number, Some(sub_number));
        }

        let mut db_extension_metadata = CartridgeMetadataBuilder::new();
        if let Some((number, sub_number, data_hash, prg_hash)) =
                header_db.missing_submapper_number(header.full_hash().unwrap(), prg_rom_hash) && cartridge_mapper_number == number {

            info!("Using override submapper {sub_number} for this ROM. Full hash: {data_hash} , PRG hash: {prg_hash}");
            db_extension_metadata.mapper_and_submapper_number(number, Some(sub_number));
        }

        let mut metadata_resolver = MetadataResolver {
            hard_coded_overrides: hard_coded_overrides.build(),
            cartridge: header,
            // Metadata from the mapper is populated a little later.
            mapper: CartridgeMetadataBuilder::new().build(),
            database: db_header,
            database_extension: db_extension_metadata.build(),
            layout_has_prg_ram: false,
        };

        let mapper = mapper_list::lookup_mapper(&metadata_resolver, &cartridge);
        if let Some(mirroring) = mapper.layout().cartridge_selection_name_table_mirrorings()[cartridge_selected_mirroring as usize] {
            metadata_resolver.cartridge.set_name_table_mirroring(mirroring);
        }

        let metadata = metadata_resolver.resolve();
        let mut mapper_params = mapper.layout().make_mapper_params(&metadata, &cartridge);
        mapper.init_mapper_params(&mut mapper_params);

        metadata_resolver.mapper = mapper.layout().cartridge_metadata_override();
        metadata_resolver.layout_has_prg_ram = mapper.layout().has_prg_ram();
        let metadata = metadata_resolver.resolve();
        info!("ROM loaded.\n{metadata}");

        (cartridge, mapper, mapper_params, metadata_resolver)
    }
}

#[derive(Clone, Debug, StructOpt)]
#[structopt(name = "REZNEZ", about = "The ultra-accurate NES emulator.")]
pub struct Opt {
    #[structopt(name = "ROM", parse(from_os_str))]
    pub rom_path: PathBuf,

    #[structopt(short, long, default_value = "egui")]
    pub gui: GuiType,

    #[structopt(name = "targetframerate", long, default_value = "ntsc")]
    pub target_frame_rate: TargetFrameRate,

    #[structopt(name = "stopframe", long)]
    pub stop_frame: Option<i64>,

    #[structopt(name = "disableaudio", long)]
    pub disable_audio: bool,

    #[structopt(name = "logcpuall", long)]
    pub log_cpu_all: bool,

    #[structopt(name = "logcpuinstructions", long)]
    pub log_cpu_instructions: bool,

    #[structopt(name = "logcpuflowcontrol", long)]
    pub log_cpu_flow_control: bool,

    #[structopt(name = "logcpumode", long)]
    pub log_cpu_mode: bool,

    #[structopt(name = "logdetailedcpumode", long)]
    pub log_detailed_cpu_mode: bool,

    #[structopt(name = "logframes", long)]
    pub log_frames: bool,

    #[structopt(name = "logcpusteps", long)]
    pub log_cpu_steps: bool,

    #[structopt(name = "logppuall", long)]
    pub log_ppu_all: bool,

    #[structopt(name = "logppustages", long)]
    pub log_ppu_stages: bool,

    #[structopt(name = "logppuflags", long)]
    pub log_ppu_flags: bool,

    #[structopt(name = "logppusteps", long)]
    pub log_ppu_steps: bool,

    #[structopt(name = "logapuall", long)]
    pub log_apu_all: bool,

    #[structopt(name = "logapucycles", long)]
    pub log_apu_cycles: bool,

    #[structopt(name = "logapuevents", long)]
    pub log_apu_events: bool,

    #[structopt(name = "logoamaddr", long)]
    pub log_oam_addr: bool,

    #[structopt(name = "logmapperupdates", long)]
    pub log_mapper_updates: bool,

    #[structopt(name = "logtimings", long)]
    pub log_timings: bool,

    #[structopt(name = "cpustepformatting", long, default_value = "data")]
    pub cpu_step_formatting: CpuStepFormatting,

    #[structopt(name = "framedump", long)]
    pub frame_dump: bool,

    #[structopt(long)]
    pub analysis: bool,

    #[structopt(name = "preventsaving", long)]
    pub prevent_saving: bool,
}

impl Opt {
    pub fn new(rom_path: PathBuf) -> Self {
        Self {
            rom_path,
            gui: GuiType::Egui,
            stop_frame: None,
            target_frame_rate: TargetFrameRate::Value(FrameRate::NTSC),
            disable_audio: false,
            log_frames: false,
            log_cpu_all: false,
            log_ppu_all: false,
            log_apu_all: false,
            log_cpu_instructions: false,
            log_cpu_flow_control: false,
            log_cpu_mode: false,
            log_detailed_cpu_mode: false,
            log_cpu_steps: false,
            log_ppu_stages: false,
            log_ppu_flags: false,
            log_ppu_steps: false,
            log_oam_addr: false,
            log_apu_cycles: false,
            log_apu_events: false,
            log_mapper_updates: false,
            log_timings: false,
            cpu_step_formatting: CpuStepFormatting::Data,
            frame_dump: false,
            analysis: false,
            prevent_saving: false,
        }
    }
}

#[derive(Clone, Debug)]
pub enum GuiType {
    NoGui,
    Egui,
}

impl FromStr for GuiType {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "nogui" => Ok(GuiType::NoGui),
            "egui" => Ok(GuiType::Egui),
            _ => Err(format!("Invalid gui type: {value}")),
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum CpuStepFormatting {
    NoData,
    Data,
}

impl FromStr for CpuStepFormatting {
    type Err = String;

    fn from_str(value: &str) -> Result<Self, String> {
        match value.to_lowercase().as_str() {
            "nodata" => Ok(CpuStepFormatting::NoData),
            "data" => Ok(CpuStepFormatting::Data),
            _ => Err(format!("Invalid cpu step formatting: {value}")),
        }
    }
}
