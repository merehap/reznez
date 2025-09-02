use std::fmt;

use num_derive::FromPrimitive;
use ux::u2;

use crate::mapper_list::MAPPERS_WITHOUT_SUBMAPPER_0;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CartridgeMetadata {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

    name_table_mirroring_index: Option<u2>,
    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: Option<bool>,

    full_hash: Option<u32>,
    prg_rom_hash: Option<u32>,
    chr_rom_hash: Option<u32>,
    trainer_hash: Option<u32>,

    prg_rom_size: Option<u32>,
    prg_work_ram_size: Option<u32>,
    prg_save_ram_size: Option<u32>,

    chr_rom_size: Option<u32>,
    chr_work_ram_size: Option<u32>,
    chr_save_ram_size: Option<u32>,

    console_type: Option<ConsoleType>,
    timing_mode: Option<TimingMode>,
    miscellaneous_rom_count: Option<u8>,
    default_expansion_device: Option<ExpansionDevice>,
    vs_hardware_type: Option<VsHardwareType>,
    vs_ppu_type: Option<VsPpuType>,
}

impl CartridgeMetadata {
    pub fn full_hash(&self) -> Option<u32> {
        self.full_hash
    }

    pub fn prg_rom_hash(&self) -> Option<u32> {
        self.prg_rom_hash
    }

    pub fn chr_rom_hash(&self) -> Option<u32> {
        self.chr_rom_hash
    }

    pub fn mapper_number(&self) -> Option<u16> {
        self.mapper_number
    }

    pub fn submapper_number(&self) -> Option<u8> {
        self.submapper_number
    }

    pub fn has_persistent_memory(&self) -> Option<bool> {
        self.has_persistent_memory
    }

    pub fn prg_rom_size(&self) -> Option<u32> {
        self.prg_rom_size
    }

    pub fn prg_work_ram_size(&self) -> Option<u32> {
        self.prg_work_ram_size
    }

    pub fn prg_save_ram_size(&self) -> Option<u32> {
        self.prg_save_ram_size
    }

    pub fn chr_rom_size(&self) -> Option<u32> {
        self.chr_rom_size
    }

    pub fn chr_work_ram_size(&self) -> Option<u32> {
        self.chr_work_ram_size
    }

    pub fn chr_save_ram_size(&self) -> Option<u32> {
        self.chr_save_ram_size
    }

    pub fn name_table_mirroring_index(&self) -> Option<u2> {
        self.name_table_mirroring_index
    }

    // FIXME: This returns None if there is no mirroring specified OR if the cartridge specifies FourScreen.
    pub fn name_table_mirroring(&self) -> Option<NameTableMirroring> {
        self.name_table_mirroring
    }

    pub fn console_type(&self) -> Option<ConsoleType> {
        self.console_type
    }

    pub fn timing_mode(&self) -> Option<TimingMode> {
        self.timing_mode
    }

    pub fn miscellaneous_rom_count(&self) -> Option<u8> {
        self.miscellaneous_rom_count
    }

    pub fn default_expansion_device(&self) -> Option<ExpansionDevice> {
        self.default_expansion_device
    }

    pub fn vs_hardware_type(&self) -> Option<VsHardwareType> {
        self.vs_hardware_type
    }

    pub fn vs_ppu_type(&self) -> Option<VsPpuType> {
        self.vs_ppu_type
    }

    pub fn set_name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring) {
        self.name_table_mirroring = Some(name_table_mirroring);
    }

    pub fn set_prg_rom_hash(&mut self, prg_rom_hash: u32) {
        self.prg_rom_hash = Some(prg_rom_hash);
    }

    pub fn set_chr_rom_hash(&mut self, chr_rom_hash: u32) {
        self.chr_rom_hash = Some(chr_rom_hash);
    }

    pub const fn into_builder(self) -> CartridgeMetadataBuilder {
        CartridgeMetadataBuilder {
            mapper_number: self.mapper_number,
            submapper_number: self.submapper_number,

            name_table_mirroring_index: self.name_table_mirroring_index,
            name_table_mirroring: self.name_table_mirroring,
            has_persistent_memory: self.has_persistent_memory,

            full_hash: self.full_hash,
            prg_rom_hash: self.prg_rom_hash,
            chr_rom_hash: self.chr_rom_hash,
            trainer_hash: self.trainer_hash,

            prg_rom_size: self.prg_rom_size,
            prg_work_ram_size: self.prg_work_ram_size,
            prg_save_ram_size: self.prg_save_ram_size,

            chr_rom_size: self.chr_rom_size,
            chr_work_ram_size: self.chr_work_ram_size,
            chr_save_ram_size: self.chr_save_ram_size,

            console_type: self.console_type,
            timing_mode: self.timing_mode,
            miscellaneous_rom_count: self.miscellaneous_rom_count,
            default_expansion_device: self.default_expansion_device,
            vs_hardware_type: self.vs_hardware_type,
            vs_ppu_type: self.vs_ppu_type,
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub struct CartridgeMetadataBuilder {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

    name_table_mirroring_index: Option<u2>,
    name_table_mirroring: Option<NameTableMirroring>,
    has_persistent_memory: Option<bool>,

    full_hash: Option<u32>,
    prg_rom_hash: Option<u32>,
    chr_rom_hash: Option<u32>,
    trainer_hash: Option<u32>,

    prg_rom_size: Option<u32>,
    prg_work_ram_size: Option<u32>,
    prg_save_ram_size: Option<u32>,

    chr_rom_size: Option<u32>,
    chr_work_ram_size: Option<u32>,
    chr_save_ram_size: Option<u32>,

    console_type: Option<ConsoleType>,
    timing_mode: Option<TimingMode>,
    miscellaneous_rom_count: Option<u8>,
    default_expansion_device: Option<ExpansionDevice>,
    vs_hardware_type: Option<VsHardwareType>,
    vs_ppu_type: Option<VsPpuType>,
}

impl CartridgeMetadataBuilder {
    pub const fn new() -> Self {
        Self {
            mapper_number: None,
            submapper_number: None,

            name_table_mirroring_index: None,
            name_table_mirroring: None,
            has_persistent_memory: None,

            full_hash: None,
            prg_rom_hash: None,
            chr_rom_hash: None,
            trainer_hash: None,

            prg_rom_size: None,
            prg_work_ram_size: None,
            prg_save_ram_size: None,

            chr_rom_size: None,
            chr_work_ram_size: None,
            chr_save_ram_size: None,

            console_type: None,
            timing_mode: None,
            miscellaneous_rom_count: None,
            default_expansion_device: None,
            vs_hardware_type: None,
            vs_ppu_type: None,
        }
    }

    pub fn mapper_and_submapper_number(&mut self, mapper_number: u16, submapper_number: Option<u8>) -> &mut Self {
        assert!(self.mapper_number.is_none(), "Can't set mapper number twice.");

        self.mapper_number = Some(mapper_number);
        if MAPPERS_WITHOUT_SUBMAPPER_0.contains(&mapper_number) && submapper_number == Some(0) {
            self.submapper_number = None;
        } else {
            self.submapper_number = submapper_number;
        }

        self
    }

    pub const fn name_table_mirroring_index(&mut self, index: u2) -> &mut Self {
        self.name_table_mirroring_index = Some(index);
        self
    }

    pub const fn name_table_mirroring(&mut self, name_table_mirroring: NameTableMirroring) -> &mut Self {
        self.name_table_mirroring = Some(name_table_mirroring);
        self
    }

    pub const fn has_persistent_memory(&mut self, has_persistent_memory: bool) -> &mut Self {
        self.has_persistent_memory = Some(has_persistent_memory);
        self
    }

    pub const fn full_hash(&mut self, full_hash: u32) -> &mut Self {
        self.full_hash = Some(full_hash);
        self
    }

    pub const fn prg_rom_hash(&mut self, prg_rom_hash: u32) -> &mut Self {
        self.prg_rom_hash = Some(prg_rom_hash);
        self
    }

    pub const fn chr_rom_hash(&mut self, chr_rom_hash: u32) -> &mut Self {
        self.chr_rom_hash = Some(chr_rom_hash);
        self
    }

    pub const fn prg_rom_size(&mut self, prg_rom_size: u32) -> &mut Self {
        self.prg_rom_size = Some(prg_rom_size);
        self
    }

    pub const fn prg_work_ram_size(&mut self, prg_work_ram_size: u32) -> &mut Self {
        self.prg_work_ram_size = Some(prg_work_ram_size);
        self
    }

    pub const fn prg_save_ram_size(&mut self, prg_save_ram_size: u32) -> &mut Self {
        self.prg_save_ram_size = Some(prg_save_ram_size);
        self
    }

    pub const fn chr_rom_size(&mut self, chr_rom_size: u32) -> &mut Self {
        self.chr_rom_size = Some(chr_rom_size);
        self
    }

    pub const fn chr_work_ram_size(&mut self, chr_work_ram_size: u32) -> &mut Self {
        self.chr_work_ram_size = Some(chr_work_ram_size);
        self
    }

    pub const fn chr_save_ram_size(&mut self, chr_save_ram_size: u32) -> &mut Self {
        self.chr_save_ram_size = Some(chr_save_ram_size);
        self
    }

    pub const fn console_type(&mut self, console_type: ConsoleType) -> &mut Self {
        self.console_type = Some(console_type);
        self
    }

    pub const fn timing_mode(&mut self, timing_mode: TimingMode) -> &mut Self {
        self.timing_mode = Some(timing_mode);
        self
    }

    pub const fn miscellaneous_rom_count(&mut self, miscellaneous_rom_count: u8) -> &mut Self {
        self.miscellaneous_rom_count = Some(miscellaneous_rom_count);
        self
    }

    pub const fn default_expansion_device(&mut self, default_expansion_device: ExpansionDevice) -> &mut Self {
        self.default_expansion_device = Some(default_expansion_device);
        self
    }

    pub const fn vs_hardware_type(&mut self, vs_hardware_type: VsHardwareType) -> &mut Self {
        self.vs_hardware_type = Some(vs_hardware_type);
        self
    }

    pub const fn vs_ppu_type(&mut self, vs_ppu_type: VsPpuType) -> &mut Self {
        self.vs_ppu_type = Some(vs_ppu_type);
        self
    }

    pub const fn build(&mut self) -> CartridgeMetadata {
        CartridgeMetadata {
            mapper_number: self.mapper_number,
            submapper_number: self.submapper_number,
            name_table_mirroring_index: self.name_table_mirroring_index,
            name_table_mirroring: self.name_table_mirroring,
            has_persistent_memory: self.has_persistent_memory,
            full_hash: self.full_hash,
            prg_rom_hash: self.prg_rom_hash,
            chr_rom_hash: self.chr_rom_hash,
            trainer_hash: self.trainer_hash,
            prg_rom_size: self.prg_rom_size,
            prg_work_ram_size: self.prg_work_ram_size,
            prg_save_ram_size: self.prg_save_ram_size,
            chr_rom_size: self.chr_rom_size,
            chr_work_ram_size: self.chr_work_ram_size,
            chr_save_ram_size: self.chr_save_ram_size,
            console_type: self.console_type,
            timing_mode: self.timing_mode,
            miscellaneous_rom_count: self.miscellaneous_rom_count,
            default_expansion_device: self.default_expansion_device,
            vs_hardware_type: self.vs_hardware_type,
            vs_ppu_type: self.vs_ppu_type,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default, FromPrimitive)]
pub enum ConsoleType {
    #[default]
    NesFamiconDendy,
    Vs,
    PlayChoice10,
    DecimalModeFamiclone,
    NesFamiconWithEpsm,
    Vt01,
    Vt02,
    Vt03,
    Vt09,
    Vt32,
    Vt369,
    UmcUm6578,
    FamiconNetworkSystem,
}

impl ConsoleType {
    pub fn basic(basic_console_type: u8) -> Self {
        assert!(basic_console_type < 3);
        Self::from_u8(basic_console_type)
    }

    pub fn extended(basic_console_type: u8, extended_console_type: u8) -> Self {
        match basic_console_type {
            0..=2 => Self::from_u8(basic_console_type),
            3 => {
                assert!(extended_console_type > 3);
                Self::from_u8(extended_console_type)
            }
            _ => panic!("Basic console type must be less than 4."),
        }
    }

    fn from_u8(value: u8) -> Self {
        let console_type = match value {
            0x0 => ConsoleType::NesFamiconDendy,
            0x1 => ConsoleType::Vs,
            0x2 => ConsoleType::PlayChoice10,
            0x3 => ConsoleType::DecimalModeFamiclone,
            0x4 => ConsoleType::NesFamiconWithEpsm,
            0x5 => ConsoleType::Vt01,
            0x6 => ConsoleType::Vt02,
            0x7 => ConsoleType::Vt03,
            0x8 => ConsoleType::Vt09,
            0x9 => ConsoleType::Vt32,
            0xA => ConsoleType::Vt369,
            0xB => ConsoleType::UmcUm6578,
            0xC => ConsoleType::FamiconNetworkSystem,
            0xD..=0xF => panic!("Reserved"),
            _ => unreachable!(),
        };

        assert_eq!(console_type, ConsoleType::NesFamiconDendy);
        console_type
    }
}

impl fmt::Display for ConsoleType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let text = match self {
            ConsoleType::NesFamiconDendy => "NES/Famicon/Dendy",
            ConsoleType::Vs => "VS",
            ConsoleType::PlayChoice10 => "Play Choice 10",
            ConsoleType::DecimalModeFamiclone => "Famicon with Decimal Mode CPU",
            ConsoleType::NesFamiconWithEpsm => "NES/Famicon with EPSM module",
            ConsoleType::Vt01 => "V.R. Technology VT01",
            ConsoleType::Vt02 => "V.R. Technology VT02",
            ConsoleType::Vt03 => "V.R. Technology VT03",
            ConsoleType::Vt09 => "V.R. Technology VT09",
            ConsoleType::Vt32 => "V.R. Technology VT32",
            ConsoleType::Vt369 => "V.R. Technology VT369",
            ConsoleType::UmcUm6578 => "V.R. Technology VT01",
            ConsoleType::FamiconNetworkSystem => "Famicon Network System",
        };

        write!(f, "{text}")
    }
}

#[derive(Clone, Copy, Debug, Default, FromPrimitive)]
pub enum TimingMode {
    #[default]
    Ntsc,
    Pal,
    MultiRegion,
    Dendy,
}

#[derive(Clone, Copy, Debug, Default, FromPrimitive)]
pub enum VsHardwareType {
    #[default]
    Unisystem,
    UnisystemRbiBaseballProtection,
    UnisystemTkoBoxingProtection,
    UnisystemSuperXeviousProtection,
    UnisystemIceClimberProtection,
    DualSystem,
    DualSystemRaidOnBungelingBayProtection,
}

#[derive(Clone, Copy, Debug, Default, FromPrimitive)]
pub enum VsPpuType {
    #[default]
    Rp2c03Rc2c03 = 0,
    // 1 is reserved
    Rp2c04_0001 = 2,
    Rp2c04_0002,
    Rp2c04_0003,
    Rp2c04_0004,
    // 6 is supposed to be reserved per the wiki, but Stroke & Golf uses it in nes20db.xml.
    StrokeAndGolf,
    // 7 is reserved
    Rc2c05_01 = 8,
    Rc2c05_02,
    Rc2c05_03,
    Rc2c05_04,
}

#[derive(Clone, Copy, Debug, Default, FromPrimitive)]
pub enum ExpansionDevice {
    Unspecified,
    #[default]
    StandardNesFamicomControllers,
    NesFourScoreSatellite,
    FamicomFourPlayersAdapter,
    VsSystem4016,
    VsSystem4017,
    // The wiki says 0x06 is reserved, but nes20db.xml has a VS Pinball entry that uses this.
    VsPinballController,
    VsZapper,
    Zapper4017,
    TwoZappers,
    BandaiHyperShotLightgun,
    PowerPadSideA,
    PowerPadSideB,
    FamilyTrainerSideA,
    FamilyTrainerSideB,
    ArkanoidVausControllerNes,
    ArkanoidVausControllerFamicom,
    TwoVausControllersPlusFamicomDataRecorder,
    KonamiHyperShotController,
    CoconutsPachinkoController,
    ExcitingBoxingPunchingBagBlowupDoll,
    JissenMahjongController,
    PartyTap,
    OekaKidsTablet,
    SunsoftBarcodeBattler,
    MiraclePianoKeyboard,
    PokkunMoguraa,
    TopRider,
    DoubleFisted,
    Famicom3DSystem,
    DoremikkoKeyboard,
    RobGyroSet,
    FamicomDataRecorder,
    AsciiTurboFile,
    IgsStorageBattleBox,
    FamilyBasicKeyboardPlusFamicomDataRecorder,
    PecKeyboard,
    Bit79Keyboard,
    SuborKeyboard,
    SuborKeyboardPlusMacroWinnersMouse,
    SuborKeyboardPlusSuborMouse,
    SnesMouse4016,
    Multicart,
    TwoSnesControllers,
    RacerMateBicycle,
    UForce,
    RobStackUp,
    CityPatrolmanLightgun,
    SharpC1CassetteInterface,
    StandardControllerWithSwappedButtons,
    ExcaliburSudokuPad,
    AblPinball,
    GoldenNuggetCasinoExtraButtons,
    KedaKeyboard,
    SuborKeyboardPlusSuborMouse4017,
    PortTestController,
    BandaiMultiGamePlayerGamepadButtons,
    VenomTVDanceMat,
    LgTvRemoteControl,
    FamicomNetworkController,
    KingFishingController,
    CroakyKaraokeController,
    KingwonKeyboard,
    ZechengKeyboard,
    SuborKeyboardPlusL90RotatedPs2Mouse4017,
    Ps2KeyboardPlusPs2Mouse4017,
    Ps2Mouse,
    YuxingMouse4016,
    SuborKeyboardPlusYuxingMouse4016,
    GigggleTvPump,
    BbkKeyboardPlusPs2Mouse4017,
    MagicalCooking,
    SnesMouse4017,
    Zapper4016,
    ArkanoidVausControllerPrototype,
    TVMahjongGameController,
    MahjongGekitouDensetsuController,
    SuborKeyboardPlusPs2Mouse4017,
    IbmPcXtKeyboard,
}