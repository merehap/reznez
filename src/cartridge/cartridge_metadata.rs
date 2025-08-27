use std::fmt;
use std::path::Path;

use num_derive::FromPrimitive;
use num_traits::FromPrimitive;
use splitbits::{combinebits, splitbits};

use crate::mapper_list::MAPPERS_WITHOUT_SUBMAPPER_0;
use crate::memory::raw_memory::RawMemory;
use crate::ppu::name_table::name_table_mirroring::NameTableMirroring;
use crate::util::unit::KIBIBYTE;

pub const PRG_ROM_CHUNK_LENGTH: u32 = 16 * KIBIBYTE;
pub const CHR_ROM_CHUNK_LENGTH: u32 = 8 * KIBIBYTE;
const INES_HEADER_CONSTANT: u32 = u32::from_be_bytes([b'N', b'E', b'S', 0x1A]);
const NES2_0_HEADER_CONSTANT: u8 = 0b10;

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub struct CartridgeMetadata {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

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
    vs_hardware_type: Option<VsHardwareType>,
    vs_ppu_type: Option<VsPpuType>,
    default_expansion_device: Option<ExpansionDevice>,
}

impl CartridgeMetadata {
    pub fn parse(path: &Path, raw_header_and_data: &RawMemory) -> Result<(CartridgeMetadata, MirroringSelection), String> {
        let Some(low_header) = raw_header_and_data.peek_u64(0..=7) else {
            return Err(format!("ROM file should have a 16 byte header. ROM: {}", path.display()));
        };
        let low_header = splitbits!(low_header, "iiiiiiii iiiiiiii iiiiiiii iiiiiiii pppppppp cccccccc llllntbn mmmmvvxx");
        if low_header.i != INES_HEADER_CONSTANT {
            return Err(format!("Cannot load non-iNES ROM. Found {:08X} but need {INES_HEADER_CONSTANT:08X}.", low_header.i));
        }

        if low_header.t {
            return Err(format!("Trainer isn't implemented yet. ROM: {}", path.display()));
        }

        let mut builder = CartridgeMetadataBuilder::new();
        builder
            .has_persistent_memory(low_header.b)
            .full_hash(crc32fast::hash(raw_header_and_data.as_slice()));

        if low_header.v == NES2_0_HEADER_CONSTANT {
            // NES2.0 fields
            let Some(high_header) = raw_header_and_data.peek_u64(8..=15) else {
                return Err(format!("ROM file should have a 16 byte header. ROM: {}", path.display()));
            };
            let high_header = splitbits!(high_header, "ssssmmmm ccccpppp ffffgggg hhhhiiii ......tt vvvvxxxx ......rr ..dddddd");
            assert!(high_header.c != 0xF, "CHR exponent notation not yet supported.");
            assert!(high_header.p != 0xF, "PRG exponent notation not yet supported.");

            let mapper_number = combinebits!(high_header.m, low_header.m, low_header.l, "0000hhhh mmmmllll");
            builder
                .mapper_and_submapper_number(mapper_number, Some(high_header.s))
                .prg_rom_size(combinebits!(high_header.p, low_header.p, "000000hh hhllllll ll000000 00000000"))
                .chr_rom_size(combinebits!(high_header.c, low_header.c, "0000000h hhhlllll lll00000 00000000"))
                .prg_save_ram_size(if high_header.f == 0 { 0 } else { 64 << high_header.f })
                .prg_work_ram_size(if high_header.g == 0 { 0 } else { 64 << high_header.g })
                .chr_save_ram_size(if high_header.h == 0 { 0 } else { 64 << high_header.h })
                .chr_work_ram_size(if high_header.i == 0 { 0 } else { 64 << high_header.i })
                .console_type(ConsoleType::extended(low_header.x, high_header.x))
                .timing_mode(FromPrimitive::from_u8(high_header.t).unwrap())
                .default_expansion_device(FromPrimitive::from_u8(high_header.d).unwrap());

                if low_header.x == ConsoleType::Vs as u8 {
                    builder
                        .vs_hardware_type(FromPrimitive::from_u8(high_header.v).unwrap())
                        .vs_ppu_type(FromPrimitive::from_u8(high_header.x).unwrap());
                }
        } else {
            // iNES only (*no* NES2.0 fields)
            let mapper_number = combinebits!(low_header.m, low_header.l, "00000000 mmmmllll");
            builder
                .mapper_and_submapper_number(mapper_number, None)
                .prg_rom_size(u32::from(low_header.p) * PRG_ROM_CHUNK_LENGTH)
                .chr_rom_size(u32::from(low_header.c) * CHR_ROM_CHUNK_LENGTH)
                .console_type(ConsoleType::basic(low_header.x));
        }

        let name_table_mirroring_selection = low_header.n;
        Ok((builder.build(), name_table_mirroring_selection as usize))
    }

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

    pub fn vs_hardware_type(&self) -> Option<VsHardwareType> {
        self.vs_hardware_type
    }

    pub fn vs_ppu_type(&self) -> Option<VsPpuType> {
        self.vs_ppu_type
    }

    pub fn default_expansion_device(&self) -> Option<ExpansionDevice> {
        self.default_expansion_device
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
            vs_hardware_type: self.vs_hardware_type,
            vs_ppu_type: self.vs_ppu_type,
            default_expansion_device: self.default_expansion_device,
        }
    }
}

type MirroringSelection = usize;

#[derive(Clone, Copy, Debug)]
pub struct CartridgeMetadataBuilder {
    mapper_number: Option<u16>,
    submapper_number: Option<u8>,

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
    vs_hardware_type: Option<VsHardwareType>,
    vs_ppu_type: Option<VsPpuType>,
    default_expansion_device: Option<ExpansionDevice>,
}

impl CartridgeMetadataBuilder {
    pub const fn new() -> Self {
        Self {
            mapper_number: None,
            submapper_number: None,

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
            vs_hardware_type: None,
            vs_ppu_type: None,
            default_expansion_device: None,
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

    pub const fn vs_hardware_type(&mut self, vs_hardware_type: VsHardwareType) -> &mut Self {
        self.vs_hardware_type = Some(vs_hardware_type);
        self
    }

    pub const fn vs_ppu_type(&mut self, vs_ppu_type: VsPpuType) -> &mut Self {
        self.vs_ppu_type = Some(vs_ppu_type);
        self
    }

    pub const fn default_expansion_device(&mut self, default_expansion_device: ExpansionDevice) -> &mut Self {
        self.default_expansion_device = Some(default_expansion_device);
        self
    }

    pub const fn build(&mut self) -> CartridgeMetadata {
        CartridgeMetadata {
            mapper_number: self.mapper_number,
            submapper_number: self.submapper_number,
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
            vs_hardware_type: self.vs_hardware_type,
            vs_ppu_type: self.vs_ppu_type,
            default_expansion_device: self.default_expansion_device,
        }
    }
}

#[allow(dead_code)]
#[derive(PartialEq, Eq, Clone, Copy, Debug, Default)]
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
    fn basic(basic_console_type: u8) -> Self {
        assert!(basic_console_type < 3);
        Self::from_u8(basic_console_type)
    }

    fn extended(basic_console_type: u8, extended_console_type: u8) -> Self {
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

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum TimingMode {
    Ntsc,
    Pal,
    MultiRegion,
    Dendy,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum VsHardwareType {
    Unisystem,
    UnisystemRbiBaseballProtection,
    UnisystemTkoBoxingProtection,
    UnisystemSuperXeviousProtection,
    UnisystemIceClimberProtection,
    DualSystem,
    DualSystemRaidOnBungelingBayProtection,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum VsPpuType {
    Rp2c03Rc2c03 = 0,
    // 1 is reserved
    Rp2c04_0001 = 2,
    Rp2c04_0002,
    Rp2c04_0003,
    Rp2c04_0004,
    // 6 and 7 are reserved
    Rc2c05_01 = 8,
    Rc2c05_02,
    Rc2c05_03,
    Rc2c05_04,
}

#[derive(Clone, Copy, Debug, FromPrimitive)]
pub enum ExpansionDevice {
    Unspecified,
    StandardNesFamicomControllers,
    NesFourScoreSatellite,
    FamicomFourPlayersAdapter,
    VsSystem4016,
    VsSystem4017,
    // 0x06 is reserved
    VsZapper = 0x07,
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