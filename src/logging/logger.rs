use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub fn init(logger: Logger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
}

pub struct Logger {
    pub log_frames: bool,
    pub log_cpu_instructions: bool,
    pub log_cpu_flow_control: bool,
    pub log_cpu_steps: bool,
    pub log_ppu_stages: bool,
    pub log_ppu_flags: bool,
    pub log_ppu_steps: bool,
    pub log_apu_cycles: bool,
    pub log_apu_events: bool,
    pub log_oam_addr: bool,
    pub log_timings: bool,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.target() {
            "" => true,
            "frames" => self.log_frames,
            "cpuinstructions" => self.log_cpu_instructions,
            "cpuflowcontrol" => self.log_cpu_flow_control,
            "cpustep" => self.log_cpu_steps,
            "ppustage" => self.log_ppu_stages,
            "ppuflags" => self.log_ppu_flags,
            "ppusteps" => self.log_ppu_steps,
            "apucycles" => self.log_apu_cycles,
            "apuevents" => self.log_apu_events,
            "oamaddr" => self.log_oam_addr,
            "timings" => self.log_timings,
            target => {
                let chunks: Vec<&str> = target.split("::").collect();
                match chunks[..] {
                    ["reznez", ..] => true,
                    ["winit", ..] => false,
                    ["wgpu_hal", ..] => false,
                    ["wgpu_core", ..] => false,
                    _ => panic!("Unexpected logger target: {target}"),
                }
            }
        }
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.level() < Level::Info {
                print!("{} - ", record.level());
            }

            match record.target() {
                "ppustage" => print!("PPU STAGE "),
                "ppuflags" => print!("PPU FLAGS "),
                "ppusteps" => print!("PPU STEPS "),
                _ => {}
            }

            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}
