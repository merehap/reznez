use log::{Level, LevelFilter, Metadata, Record, SetLoggerError};

pub fn init(logger: Logger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
}

pub struct Logger {
    pub log_cpu_operations: bool,
    pub log_cpu_steps: bool,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        match metadata.target() {
            "" => true,
            "cpuoperation" => self.log_cpu_operations,
            "cpustep" => self.log_cpu_steps,
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

            println!("{}", record.args());
        }
    }

    fn flush(&self) {}
}
