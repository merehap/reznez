extern crate reznez;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::path::PathBuf;

use dashmap::DashMap;
use rayon::prelude::*;
use walkdir::WalkDir;

use reznez::config::{Config, GuiType, Opt};
use reznez::nes::Nes;
use reznez::ppu::render::frame_rate::TargetFrameRate;
use reznez::ppu::render::ppm::Ppm;
use reznez::util::hash_util::calculate_hash;

#[test]
fn framematch() {
    let expected_frames = ExpectedFrames::load("tests/expected_frames");
    let roms = Roms::load("tests/roms");
    let test_summary = TestSummary::load(roms, expected_frames);
    test_summary.print();
    assert!(test_summary.passed());
}

struct TestSummary {
    test_results: BTreeMap<RomId, TestStatus>,
}

impl TestSummary {
    fn load(roms: Roms, expected_frames: ExpectedFrames) -> Self {
        let test_results = DashMap::new();

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
                    ..Opt::new(rom_entry.path.clone())
                };

                let mut nes = Nes::new(&Config::new(&opt));
                nes.mute();
                *nes.frame_mut().show_overscan_mut() = true;

                let frame_directory = frame_entries[0].directory();
                let frame_entries: BTreeMap<_, _> = frame_entries.iter()
                    .map(|entry| (entry.frame_index, entry))
                    .collect();

                let max_frame_index = frame_entries.keys().last().unwrap();
                for frame_index in 0..=*max_frame_index {
                    nes.step_frame();
                    if let Some(frame_entry) = frame_entries.get(&frame_index) {
                        let expected_hash = frame_entry.ppm_hash;
                        let mask = nes.memory_mut().as_ppu_memory().regs().mask();
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
                                format!("tests/actual_frames/{}/frame{:03}.ppm", directory.display(), frame_index);
                            fs::write(actual_ppm_path.clone(), actual_ppm.to_bytes()).unwrap();
                            println!(
                                "\t\tROM {} didn't match expected hash at frame {}. See '{}'",
                                rom_id, frame_index, actual_ppm_path,
                            );
                        }
                    }
                }
            } else {
                test_results.insert(rom_id.clone(), TestStatus::ExpectedFramesMissing);
            };
        });

        for entry in expected_frames.iter() {
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
            println!("{:?}: {}", test_status, rom_id);
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
        let ppm = Ppm::from_bytes(&fs::read(&full_path).unwrap()).unwrap();
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
        Self { full_path, frame_index, tag, ppm_hash }
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
