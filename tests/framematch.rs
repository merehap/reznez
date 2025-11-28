#![feature(panic_update_hook)]

extern crate reznez;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;
use std::sync::{Arc, LazyLock, Mutex};

use dashmap::DashMap;
use rayon::prelude::*;
use reznez::cartridge::header_db::HeaderDb;
use reznez::controller::joypad::{Button, ButtonStatus};
use reznez::gui::gui::Events;
use walkdir::WalkDir;

use reznez::logging::logger;
use reznez::logging::logger::Logger;
use reznez::config::{Config, GuiType, Opt};
use reznez::nes::Nes;
use reznez::ppu::render::frame_rate::TargetFrameRate;
use reznez::ppu::render::ppm::Ppm;
use reznez::util::hash_util::calculate_hash;

type Crc = u32;
type FrameNumber = i64;

// FIXME
#[allow(clippy::type_complexity)]
static SCHEDULED_BUTTON_EVENTS: LazyLock<BTreeMap<Crc, BTreeMap<FrameNumber, (Button, ButtonStatus)>>> = LazyLock::new(|| {
    let mut presses_by_full_crc: BTreeMap<Crc, Vec<(FrameNumber, FrameNumber, Button)>> = BTreeMap::new();

    use Button::*;
    // Bio Miracle Bokutte Upa - [BROKEN] Status bar should be stationary.
    presses_by_full_crc.insert(0xE50AD737, vec![(100, 101, Start), (300, 301, Start)]);
    // Bio Miracle Bokutte Upa (Mario Baby FDS Hack) - [BROKEN] Flickering pixels.
    presses_by_full_crc.insert(0x04C94E4D, vec![(100, 101, Start), (300, 301, Start)]);
    // Crystalis - [BROKEN] Flickering pixels.
    presses_by_full_crc.insert(0x271C9FDD,
        vec![(147, 148, Start), (372, 373, Start), (453, 454, Start), (556, 557, Start), (768, 769, Start), (888, 889, Start),
             (999, 1000, Start), (1124, 1125, Start)]);
    // Fantastic Adventures of Dizzy - [BROKEN] Scanline lifts by one then returns, repeating.
    presses_by_full_crc.insert(0x59318584, vec![(364, 365, Start), (456, 457, Start), (570, 571, Start)]);
    // Super Fighter 3 - [BROKEN] Flickering scanline segment.
    presses_by_full_crc.insert(0x520C552E, vec![(690, 691, Start), (798, 799, Start)]);
    // Armadillo - [BROKEN] Flickering scanline.
    presses_by_full_crc.insert(0xAE73E0C2, vec![(41, 42,Start), (95, 96, Start), (141, 142, Start), (196, 197, Start)]);
    // Marble Madness - Probably nothing wrong, but the counter has a strange progression at the start,
    // and this game requires very precise timing for mid-scanline bank switches.
    presses_by_full_crc.insert(0xF9282F28,
        vec![(51, 64, Start), (107, 119, Start), (219, 229, A), (316, 322, Up), (361, 364, Down), (426, 429, Left),
             (467, 472, Right), (504, 513, A), (594, 602, A), (759, 767, Down), (1019, 1028, Down), (1078, 1116, Down)]);
    // Rollerblade Racer - [BROKEN] Status bar is rendered from the wrong bank.
    presses_by_full_crc.insert(0xFE780BE6,
        vec![(92, 103, Start), (151, 164, Start), (205, 215, Start), (302, 313, Start), (392, 402, Start)]);
    // Wizards & Warriors III - [BROKEN] One extra scanline scrolls.
    presses_by_full_crc.insert(0x4F505449, vec![(748, 761, Start), (1067, 1081, Start), (1215, 1261, Right)]);
    // Dragon Ball - Dai Maou Fukkatsu - [BROKEN] Bad tiles.
    presses_by_full_crc.insert(0xBEFF8C77, vec![(64, 74, Start), (119, 132, Start)]);
    // Silver Eagle - [BROKEN] Flickering scanline
    presses_by_full_crc.insert(0x16D7C36F,
        vec![(96, 107, Start), (247, 262, Start), (340, 352, Start), (533, 544, Start), (614, 627, Start)]);
    // World Heroes 2 - [BROKEN] Some sprites disappear every other frame. Much worse in Mesen.
    presses_by_full_crc.insert(0x7BAF8149, vec![(1763, 1764, Start)]);
    // Master Fighter VI' - [BROKEN] Wrong sprites rendered on character selection. Same in Mesen.
    presses_by_full_crc.insert(0xC2928549, vec![(52, 53, Start)]);
    // AccuracyCoin
    presses_by_full_crc.insert(0x86475E70, vec![(60, 61, Start)]);

    let mut all_events = BTreeMap::new();
    for (full_crc, presses) in presses_by_full_crc {
        let mut events: BTreeMap<i64, (Button, ButtonStatus)> = BTreeMap::new();
        for (press_frame_number, unpress_frame_number, button) in presses {
            events.insert(press_frame_number, (button, ButtonStatus::Pressed));
            events.insert(unpress_frame_number, (button, ButtonStatus::Unpressed));
        }

        all_events.insert(full_crc, events);
    }

    all_events
});

#[test]
fn framematch() {
    let expected_frames = ExpectedFrames::load("tests/expected_frames");
    let roms = Roms::load("tests/roms");
    let test_summary = TestSummary::load(&roms, &expected_frames);
    test_summary.print();
    assert!(test_summary.passed());
}

struct TestSummary {
    test_results: BTreeMap<RomId, TestStatus>,
}

impl TestSummary {
    fn load(roms: &Roms, expected_frames: &ExpectedFrames) -> Self {
        // Log nothing by default, but if debugging is needed, then logging can be enabled.
        logger::init(Logger {
            disable_all: true,
            buffer: Arc::new(Mutex::new(String::new())),
            .. Logger::default()
        }).unwrap();

        let test_results = DashMap::new();

        let header_db = HeaderDb::load();
        let expected_frames: DashMap<RomId, Vec<FrameEntry>> =
            expected_frames.entries_by_rom_id.clone().into_iter().collect();
        roms.entries_by_rom_id.par_iter().for_each(|(rom_id, rom_entry)| {
            if rom_entry.is_ignored() {
                test_results.insert(rom_id.clone(), TestStatus::RomIgnored);
            } else if let Some((rom_id, frame_entries)) = expected_frames.remove(rom_id) {
                assert!(!frame_entries.is_empty());

                let opt = Opt {
                    gui: GuiType::NoGui,
                    target_frame_rate: TargetFrameRate::Unbounded,
                    disable_audio: true,
                    prevent_saving: true,
                    ..Opt::new(Some(rom_entry.path.clone()))
                };

                let config = Config::new(&opt);
                let cartridge = Nes::load_cartridge(&opt.rom_path.unwrap()).unwrap();
                let scheduled_button_events = SCHEDULED_BUTTON_EVENTS.get(&cartridge.header().full_hash().unwrap())
                    .cloned()
                    .unwrap_or(BTreeMap::new());

                let mut nes = Nes::new(&header_db, &config, &cartridge).unwrap();
                nes.mute();
                *nes.frame_mut().show_overscan_mut() = true;

                std::panic::update_hook(|prev, info| {
                    log::logger().flush();
                    prev(info);
                });

                let frame_directory = frame_entries[0].directory();
                let frame_entries: BTreeMap<_, _> = frame_entries.iter()
                    .map(|entry| (entry.frame_index, entry))
                    .collect();

                let max_frame_number = frame_entries.keys().last().unwrap();
                for frame_number in 0..=*max_frame_number {
                    let mut joypad1_button_statuses = BTreeMap::new();
                    if let Some((button, button_status)) = scheduled_button_events.get(&(frame_number as i64)) {
                        joypad1_button_statuses.insert(*button, *button_status);
                    }

                    let events = Events { should_quit: false, joypad1_button_statuses, joypad2_button_statuses: BTreeMap::new() };
                    nes.process_gui_events(&events);
                    nes.step_frame();
                    if let Some(frame_entry) = frame_entries.get(&frame_number) {
                        let expected_hash = frame_entry.ppm_hash;
                        let mask = nes.memory().ppu_regs.mask();
                        let actual_ppm = &nes.frame().to_ppm(mask);
                        let actual_hash = calculate_hash(&actual_ppm);
                        if actual_hash == expected_hash {
                            // FIXME: Hack until we allow different TestStatuses per frame for a
                            // single ROM.
                            if test_results.get(&rom_id.clone()).is_none() {
                                if frame_entry.is_known_bad() {
                                    test_results.insert(rom_id.clone(), TestStatus::KnownBad);
                                } else {
                                    test_results.insert(rom_id.clone(), TestStatus::Pass);
                                }
                            }
                        } else {
                            test_results.insert(rom_id.clone(), TestStatus::Fail);
                            let directory: PathBuf = frame_directory.components().skip(2).collect();
                            fs::create_dir_all(format!("tests/actual_frames/{}/", directory.display())).unwrap();
                            let actual_ppm_path =
                                format!("tests/actual_frames/{}/frame{:03}.ppm", directory.display(), frame_number);
                            fs::write(actual_ppm_path.clone(), actual_ppm.to_bytes()).unwrap();
                            println!("\t\tROM {rom_id} didn't match expected hash at frame {frame_number}. See '{actual_ppm_path}'");
                        }
                    }
                }
            } else {
                test_results.insert(rom_id.clone(), TestStatus::ExpectedFramesMissing);
            }
        });

        log::logger().flush();

        for entry in &expected_frames {
            test_results.insert(entry.key().clone(), TestStatus::RomMissing);
        }

        let test_results = test_results.into_iter().collect();
        TestSummary { test_results }
    }

    fn passed(&self) -> bool {
        !self.test_results.iter()
            .any(|(_, status)| *status == TestStatus::Fail || *status == TestStatus::ExpectedFramesMissing)
    }

    fn print(&self) {
        for (rom_id, test_status) in &self.test_results {
            println!("{test_status:?}: {rom_id}");
        }
    }
}

#[derive(PartialEq, Eq, Debug)]
enum TestStatus {
    // The actual frame matched the expected frame, and the expected frame wasn't marked known bad.
    Pass,
    // The actual frame did not match the expected frame.
    Fail,
    // The actual frame matched the expected frame, but was marked known bad.
    KnownBad,
    // ROM exists, but there are not expected frames to match the actual frames against.
    ExpectedFramesMissing,
    // Expected frames exist for a ROM, but the ROM itself doesn't exist. May indicate a
    // copyrighted ROM that won't be committed.
    RomMissing,
    // ROM was intentionally marked as ignored. Either it requires joypad input, or it tests a
    // mapper that hasn't been implemented yet.
    RomIgnored,
}

struct Roms {
    entries_by_rom_id: BTreeMap<RomId, RomEntry>,
}

impl Roms {
    fn load(roms_path: &str) -> Roms {
        let entries_by_rom_id: BTreeMap<RomId, RomEntry> = WalkDir::new(roms_path)
            .into_iter()
            .map(|entry| entry.unwrap().path().to_path_buf())
            .filter(|path| path.extension() == Some(OsStr::new("nes")))
            .map(|path| {
                let entry = RomEntry::new(path.clone());
                (entry.rom_id(), entry)
            })
            .collect();

        Roms { entries_by_rom_id }
    }
}

struct RomEntry {
    path: PathBuf,
    tag: Option<String>,
}

impl RomEntry {
    fn new(path: PathBuf) -> Self {
        let file_stem = path.file_stem().unwrap();
        let mut stem_segments = file_stem.to_str().unwrap().split('#');
        let tag = match (stem_segments.next(), stem_segments.next(), stem_segments.next()) {
            (Some(_), None          , None   ) => None,
            (Some(_), Some(file_tag), None   ) => Some(file_tag.to_string()),
            (Some(_), Some(_)       , Some(_)) => panic!("There should only be one tag ('#' character) per rom name."),
            _ => unreachable!(),
        };

        Self { path, tag }
    }

    fn rom_id(&self) -> RomId {
        let path = self.path.with_extension("");
        let rom_id: Vec<_> = path.iter()
            .skip(2)
            .map(|id| id.to_str().unwrap())
            .collect();
        rom_id.join("/")
    }

    // If the current ROM should be ignored for frame matching.
    fn is_ignored(&self) -> bool {
        self.tag == Some("ignored".to_string())
    }
}

#[derive(Clone)]
struct ExpectedFrames {
    entries_by_rom_id: BTreeMap<RomId, Vec<FrameEntry>>,
}

impl ExpectedFrames {
    fn load(expected_frames_path: &str) -> Self {
        let frame_paths = WalkDir::new(expected_frames_path)
            .into_iter()
            .map(|entry| entry.unwrap().path().to_path_buf())
            .filter(|path| path.extension() == Some(OsStr::new("ppm")));

        let mut entries_by_rom_id: BTreeMap<String, Vec<FrameEntry>> = BTreeMap::new();
        for frame_path in frame_paths {
            let entry = FrameEntry::new(frame_path);
            let rom_id = entry.rom_id();
            if let Some(entries) = entries_by_rom_id.get_mut(&rom_id) {
                entries.push(entry);
            } else {
                entries_by_rom_id.insert(rom_id, vec![entry]);
            }
        }

        Self { entries_by_rom_id }
    }
}

#[derive(Clone)]
struct FrameEntry {
    full_path: PathBuf,
    tag: Option<String>,
    frame_index: u32,
    ppm_hash: u64,
}

impl FrameEntry {
    fn new(full_path: PathBuf) -> Self {
        let ppm = Ppm::from_bytes(&fs::read(&full_path).unwrap())
            .unwrap_or_else(|err| panic!("Failed to open PPM {}. {err}", full_path.display()));
        let ppm_hash = calculate_hash(&ppm);

        let file_stem = full_path.file_stem().unwrap();
        let mut stem_segments = file_stem.to_str().unwrap().split('#');
        let (start, tag) = match (stem_segments.next(), stem_segments.next(), stem_segments.next()) {
            (Some(start), None          , None   ) => (start, None),
            (Some(start), Some(file_tag), None   ) => (start, Some(file_tag.to_string())),
            (Some(_)    , Some(_)       , Some(_)) => panic!("There should only be one tag ('#' character) per test frame."),
            _ => unreachable!(),
        };

        let frame_index = sscanf::scanf!(start, "frame{}", u32)
            .expect("PPM frame must have a number in the file name");
        Self { full_path, tag, frame_index, ppm_hash }
    }

    fn directory(&self) -> PathBuf {
        self.full_path.parent().unwrap().to_path_buf()
    }

    fn rom_id(&self) -> RomId {
        let rom_id = self.directory();
        let rom_id: Vec<_> = rom_id.iter()
            .skip(2)
            .map(|id| id.to_str().unwrap())
            .collect();
        rom_id.join("/")
    }

    // If the current frame is known to be a non-success as the result of a known reznez bug.
    fn is_known_bad(&self) -> bool {
        self.tag == Some("bad".to_string())
    }
}

type RomId = String;
