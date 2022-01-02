extern crate reznez;

use std::collections::BTreeMap;
use std::fs;
use std::fs::File;
use std::path::PathBuf;

use sscanf;

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
    let mut frame_hash_data = Vec::new();
    for entry in fs::read_dir("tests/expected_frames").unwrap() {
        let frame_directory = entry.unwrap().path();
        if frame_directory.is_dir() {
            let rom_name: String = frame_directory
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_string();
            let rom_path = PathBuf::from(format!("tests/roms/{}.nes", rom_name.clone()));
            if File::open(rom_path.clone()).is_err() {
                continue;
            }

            let mut frame_hashes = BTreeMap::new();
            for ppm_entry in fs::read_dir(frame_directory).unwrap() {
                let ppm_path = ppm_entry.unwrap().path();
                println!("PPM Path: {}", ppm_path.to_str().unwrap());
                let ppm_file_name = ppm_path.file_name().unwrap().to_str().unwrap();
                let frame_index = sscanf::scanf!(ppm_file_name, "frame{}.ppm", u16);

                if let Some(frame_index) = frame_index {
                    let ppm = Ppm::from_bytes(&fs::read(ppm_path).unwrap()).unwrap();
                    let hash = calculate_hash(&ppm);
                    println!("{} hash {}: {}", rom_name, frame_index, hash);
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

            let nes = Nes::new(Config::new(&opt));
            frame_hash_data.push(FrameHashData {rom_name, nes, frame_hashes});
        }
    }

    for FrameHashData {rom_name, mut nes, frame_hashes} in frame_hash_data {
        let mut gui = Box::new(NoGui::initialize()) as Box<dyn Gui>;
        println!("FRAMEMATCH TEST: TESTING EXPECTED FRAMES FOR {}.", rom_name);
        let max_frame_index = frame_hashes.keys().last().unwrap();
        for frame_index in 0..=*max_frame_index {
            nes.step_frame(&mut *gui);
            if let Some(expected_hash) = frame_hashes.get(&frame_index) {
                println!(
                    "\tCHECKING ACTUAL HASH VS EXPECTED HASH FOR FRAME {}.",
                    frame_index,
                );
                let actual_ppm = &gui.frame_mut().to_ppm();
                let actual_hash = calculate_hash(&actual_ppm);
                if actual_hash != *expected_hash {
                    let actual_ppm_path = format!(
                        "{}_actual_frame_{:03}.ppm",
                        rom_name,
                        frame_index,
                    );
                    fs::write(
                        actual_ppm_path.clone(),
                        gui.frame_mut().to_ppm().to_bytes(),
                    ).unwrap();
                    panic!(
                        "Actual hash {} didn't match expected hash {}. See {}.",
                        actual_hash,
                        expected_hash,
                        actual_ppm_path,
                    );
                }
                assert_eq!(*expected_hash, actual_hash);
            }
        }
    }
}

struct FrameHashData {
    rom_name: String,
    nes: Nes,
    frame_hashes: BTreeMap<u16, u64>,
}
