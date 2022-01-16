extern crate reznez;

use std::collections::{BTreeMap, BTreeSet};
use std::ffi::OsStr;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use sscanf;
use walkdir::WalkDir;

use reznez::config::{Config, Opt, GuiType};
use reznez::gui::gui::Gui;
use reznez::gui::no_gui::NoGui;
use reznez::nes::Nes;
use reznez::ppu::render::frame_rate::TargetFrameRate;
use reznez::ppu::render::ppm::Ppm;
use reznez::util::hash_util::calculate_hash;

#[test]
fn framematch() {
    println!("FRAMEMATCH TEST: LOADING EXPECTED_FRAMES PPMS AND ROMS.");
    let frame_directories: BTreeSet<_> = WalkDir::new("tests/expected_frames")
        .into_iter()
        .map(|entry| entry.unwrap().path().to_path_buf())
        .filter(|path| path.extension() == Some(OsStr::new("ppm")))
        .map(|path| path.parent().unwrap().to_path_buf())
        .collect();

    let mut frame_hash_data = Vec::new();
    for frame_directory in frame_directories {
        println!("Frame directory: {}", frame_directory.as_path().display());
        let mut rom_path_vec: Vec<_> = frame_directory.into_iter().collect();
        rom_path_vec[1] = OsStr::new("roms");
        let mut rom_path: PathBuf = rom_path_vec.into_iter().collect();
        rom_path.set_extension("nes");
        if File::open(rom_path.clone()).is_err() {
            // Some ROMs aren't committed due to copyright.
            continue;
        }

        let mut frame_hashes = BTreeMap::new();
        for ppm_entry in fs::read_dir(frame_directory.clone()).unwrap() {
            let ppm_path = ppm_entry.unwrap().path();
            println!("\tPPM Path: {}", ppm_path.to_str().unwrap());
            let ppm_file_name = ppm_path.file_name().unwrap().to_str().unwrap();
            let frame_index = sscanf::scanf!(ppm_file_name, "frame{}.ppm", u16);

            if let Some(frame_index) = frame_index {
                let ppm = Ppm::from_bytes(&fs::read(ppm_path).unwrap()).unwrap();
                let hash = calculate_hash(&ppm);
                frame_hashes.insert(frame_index, hash);
            }
        }

        if frame_hashes.is_empty() {
            continue;
        }

        let opt = Opt {
            rom_path,
            gui: GuiType::NoGui,
            stop_frame: None,
            target_frame_rate: TargetFrameRate::Unbounded,
            override_program_counter: None,
            log_cpu: false,
        };

        let nes = Nes::new(&Config::new(&opt));
        let rom_name = frame_directory.file_stem().unwrap().to_str().unwrap().to_string();
        frame_hash_data.push(FrameHashData {rom_name, nes, frame_hashes});
    }

    println!();

    let mut failed = false;
    for FrameHashData {rom_name, mut nes, frame_hashes} in frame_hash_data {
        let mut gui = Box::new(NoGui::initialize()) as Box<dyn Gui>;
        println!("FRAMEMATCH TEST: testing against expected frames for {}.", rom_name);
        let max_frame_index = frame_hashes.keys().last().unwrap();
        for frame_index in 0..=*max_frame_index {
            nes.step_frame(&mut *gui);
            if let Some(expected_hash) = frame_hashes.get(&frame_index) {
                println!(
                    "\tChecking actual hash vs expected hash for frame {}.",
                    frame_index,
                );
                let actual_ppm = &gui.frame_mut().to_ppm();
                let actual_hash = calculate_hash(&actual_ppm);
                if actual_hash != *expected_hash {
                    failed = true;

                    let actual_ppm_path = format!(
                        "{}_actual_frame_{:03}.ppm",
                        rom_name,
                        frame_index,
                    );
                    fs::write(
                        actual_ppm_path.clone(),
                        gui.frame_mut().to_ppm().to_bytes(),
                    ).unwrap();
                    println!(
                        "\t\tActual hash {} didn't match expected hash {}. See {}.",
                        actual_hash,
                        expected_hash,
                        actual_ppm_path,
                    );
                }
            }
        }
    }

    assert!(!failed);
}

struct FrameHashData {
    rom_name: String,
    nes: Nes,
    frame_hashes: BTreeMap<u16, u64>,
}
