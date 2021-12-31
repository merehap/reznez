use log::{Record, Level, Metadata, SetLoggerError, LevelFilter};

pub fn init(logger: Logger) -> Result<(), SetLoggerError> {
    log::set_boxed_logger(Box::new(logger))
        .map(|()| log::set_max_level(LevelFilter::Info))
}

pub struct Logger {
    pub log_cpu: bool,
}

impl log::Log for Logger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        if !self.log_cpu && metadata.target() == "cpu" {
            return false;
        }

        true
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
