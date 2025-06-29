use std::io::Write;
use std::sync::{Arc, Mutex};

use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub fn init(logger: Logger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
}

#[derive(Default)]
pub struct Logger {
    pub log_frames: bool,
    pub log_cpu_instructions: bool,
    pub log_cpu_flow_control: bool,
    pub log_cpu_mode: bool,
    pub log_detailed_cpu_mode: bool,
    pub log_cpu_steps: bool,
    pub log_ppu_stages: bool,
    pub log_ppu_flags: bool,
    pub log_ppu_steps: bool,
    pub log_apu_cycles: bool,
    pub log_apu_events: bool,
    pub log_oam_addr: bool,
    pub log_mapper_updates: bool,
    pub log_timings: bool,

    pub buffer: Arc<Mutex<String>>,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.target() {
            "" => true,
            "frames" => self.log_frames,
            "cpuinstructions" => self.log_cpu_instructions,
            "cpuflowcontrol" => self.log_cpu_flow_control,
            "cpumode" => self.log_cpu_mode,
            "detailedcpumode" => self.log_detailed_cpu_mode,
            "cpustep" => self.log_cpu_steps,
            "ppustage" => self.log_ppu_stages,
            "ppuflags" => self.log_ppu_flags,
            "ppusteps" => self.log_ppu_steps,
            "apucycles" => self.log_apu_cycles,
            "apuevents" => self.log_apu_events,
            "oamaddr" => self.log_oam_addr,
            "mapperupdates" => self.log_mapper_updates,
            "timings" => self.log_timings,
            target => {
                let chunks: Vec<&str> = target.split("::").collect();
                match chunks[..] {
                    ["reznez", ..] => true,
                    ["nestest", ..] => true,
                    ["framematch", ..] => true,
                    ["winit", ..] => false,
                    ["wgpu_hal", ..] => false,
                    ["wgpu_core", ..] => false,
                    ["gilrs_core", ..] => true,
                    _ => panic!("Unexpected logger target: {target}"),
                }
            }
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            let mut buffer = self.buffer.lock().unwrap();
            if record.level() < Level::Info {
                buffer.push_str(&record.level().to_string());
                buffer.push_str(" - ");
            }

            match record.target() {
                "ppustage" => buffer.push_str("PPU STAGE "),
                "ppuflags" => buffer.push_str("PPU FLAGS "),
                "ppusteps" => buffer.push_str("PPU STEPS "),
                _ => {}
            }

            buffer.push_str(&record.args().to_string());
            buffer.push('\n');
        }
    }

    fn flush(&self) {
        let mut buffer = self.buffer.lock().unwrap();
        print!("{}", buffer);
        std::io::stdout().flush().unwrap();
        buffer.clear();
    }
}
