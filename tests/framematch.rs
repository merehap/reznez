extern crate reznez;

use std::collections::BTreeMap;
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use rayon::prelude::*;
use sscanf;
use walkdir::WalkDir;

use reznez::config::{Config, GuiType, Opt};
use reznez::nes::Nes;
use reznez::ppu::render::frame_rate::TargetFrameRate;
use reznez::ppu::render::ppm::Ppm;
use reznez::util::hash_util::calculate_hash;

#[test]
fn framematch() {
    let frames = Frames::load("tests/expected_frames").entries_by_rom_id;

    let failed = Arc::new(AtomicBool::new(false));
    frames.par_iter().for_each(|(_rom_id, frame_entries)| {
        let frame_directory = frame_entries[0].directory();
        let mut rom_path_vec: Vec<_> = frame_directory.into_iter().collect();
        rom_path_vec[1] = OsStr::new("roms");
        let mut rom_path: PathBuf = rom_path_vec.into_iter().collect();
        rom_path.set_extension("nes");
        if File::open(rom_path.clone()).is_err() {
            // Some ROMs aren't committed due to copyright.
            return;
        }

        let mut frame_hashes = BTreeMap::new();
        for FrameEntry { frame_index, ppm_hash, .. } in frame_entries {
            frame_hashes.insert(frame_index, ppm_hash);
        }

        if frame_hashes.is_empty() {
            return;
        }

        let opt = Opt {
            rom_path,
            gui: GuiType::NoGui,
            stop_frame: None,
            target_frame_rate: TargetFrameRate::Unbounded,
            disable_audio: true,
            log_frames: false,
            log_cpu_all: false,
            log_ppu_all: false,
            log_apu_all: false,
            log_cpu_instructions: false,
            log_cpu_flow_control: false,
            log_cpu_steps: false,
            log_ppu_stages: false,
            log_ppu_flags: false,
            log_ppu_steps: false,
            log_oam_addr: false,
            log_apu_cycles: false,
            log_apu_events: false,
            log_timings: false,
            frame_dump: false,
            analysis: false,
            disable_controllers: false,
        };

        let mut nes = Nes::new(&Config::new(&opt));
        nes.mute();
        let rom_name = frame_directory
            .file_stem()
            .unwrap()
            .to_str()
            .unwrap()
            .to_string();

        let max_frame_index = frame_hashes.keys().last().unwrap();
        for frame_index in 0..=**max_frame_index {
            nes.step_frame();
            if let Some(expected_hash) = frame_hashes.get(&frame_index) {
                let mask = nes.memory_mut().as_ppu_memory().regs().mask();
                let actual_ppm = &nes.frame().to_ppm(mask);
                let actual_hash = calculate_hash(&actual_ppm);
                if actual_hash != **expected_hash {
                    failed.store(true, Ordering::Relaxed);

                    let directory: PathBuf = frame_directory.components().skip(2).collect();
                    fs::create_dir_all(format!("tests/actual_frames/{}/", directory.display())).unwrap();
                    let actual_ppm_path =
                        format!("tests/actual_frames/{}/frame{:03}.ppm", directory.display(), frame_index);
                    fs::write(actual_ppm_path.clone(), actual_ppm.to_bytes()).unwrap();
                    println!(
                        "\t\tROM {} didn't match expected hash at frame {}. See '{}' .",
                        rom_name, frame_index, actual_ppm_path,
                    );
                }
            }
        }
    });

    assert!(!failed.load(Ordering::Relaxed));
}

struct Frames {
    entries_by_rom_id: BTreeMap<String, Vec<FrameEntry>>,
}

impl Frames {
    fn load(expected_frames_path: &str) -> Frames {
        let frame_paths = WalkDir::new(expected_frames_path)
            .into_iter()
            .map(|entry| entry.unwrap().path().to_path_buf())
            .filter(|path| path.extension() == Some(OsStr::new("ppm")));

        let mut entries_by_rom_id: BTreeMap<String, Vec<FrameEntry>> = BTreeMap::new();
        for frame_path in frame_paths {
            let entry = FrameEntry::new(frame_path);
            let rom_id = entry.directory();
            let rom_id: Vec<_> = rom_id.into_iter()
                .skip(2)
                .map(|id| id.to_str().unwrap())
                .collect();
            let rom_id = rom_id.join("/");
            if let Some(entries) = entries_by_rom_id.get_mut(&rom_id) {
                entries.push(entry);
            } else {
                entries_by_rom_id.insert(rom_id, vec![entry]);
            }
        }

        Frames { entries_by_rom_id }
    }
}

struct FrameEntry {
    full_path: PathBuf,
    frame_index: u32,
    ppm_hash: u64,
}

impl FrameEntry {
    fn new(full_path: PathBuf) -> FrameEntry {
        let file_name = full_path.file_name().unwrap().to_str().unwrap();
        let frame_index = sscanf::scanf!(file_name, "frame{}.ppm", u32)
            .expect("PPM frame must have a number in the file name");
        let ppm = Ppm::from_bytes(&fs::read(&full_path).unwrap()).unwrap();
        let ppm_hash = calculate_hash(&ppm);

        FrameEntry { full_path, frame_index, ppm_hash }
    }

    fn directory(&self) -> PathBuf {
        self.full_path.parent().unwrap().to_path_buf()
    }
}
