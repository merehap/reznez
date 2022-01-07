use std::collections::BTreeMap;
use std::io::{Write, ErrorKind};
use std::fs;
use std::fs::File;

use crate::gui::gui::{Gui, Events};
use crate::ppu::render::frame::Frame;

const FRAME_DUMP_DIRECTORY: &str = "frame_dump";

pub struct FrameDumpGui {
    frame: Frame,
}

impl Gui for FrameDumpGui {
    fn initialize() -> FrameDumpGui {
        if let Err(err) = fs::create_dir(FRAME_DUMP_DIRECTORY) {
            assert!(err.kind() == ErrorKind::AlreadyExists, "{:?}", err.kind());
        }
        
        FrameDumpGui {
            frame: Frame::new(),
        }
    }

    #[inline]
    fn events(&mut self) -> Events {
        Events {
            should_quit: false,
            joypad_1_button_statuses: BTreeMap::new(),
            joypad_2_button_statuses: BTreeMap::new(),
        }
    }

    fn frame_mut(&mut self) -> &mut Frame {
        &mut self.frame
    }

    fn display_frame(&mut self, frame_index: u64) {
        let file_name = format!(
            "{}/frame{:03}.ppm",
            FRAME_DUMP_DIRECTORY,
            frame_index,
        );
        let mut file = File::create(file_name).unwrap();
        file.write_all(&self.frame.to_ppm().to_bytes()).unwrap();
    }
}
