use std::io::{Write, ErrorKind};
use std::fs;
use std::fs::File;

use crate::gui::gui::{Gui, Events};
use crate::ppu::render::frame::Frame;

const FRAME_DUMP_DIRECTORY: &str = "frame_dump";

pub struct FrameDumpGui {
    inner: Box<dyn Gui>,
}

impl FrameDumpGui {
    pub fn new(inner: Box<dyn Gui>) -> FrameDumpGui {
        if let Err(err) = fs::create_dir(FRAME_DUMP_DIRECTORY) {
            assert!(err.kind() == ErrorKind::AlreadyExists, "{:?}", err.kind());
        }
        
        FrameDumpGui {inner}
    }
}

impl Gui for FrameDumpGui {
    #[inline]
    fn events(&mut self) -> Events {
        self.inner.events()
    }

    fn display_frame(&mut self, frame: &Frame, frame_index: u64) {
        let file_name = format!(
            "{}/frame{:03}.ppm",
            FRAME_DUMP_DIRECTORY,
            frame_index,
        );
        let mut file = File::create(file_name).unwrap();
        file.write_all(&frame.to_ppm().to_bytes()).unwrap();

        self.inner.display_frame(frame, frame_index);
    }
}
