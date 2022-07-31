use log::{Record, Level, Metadata};

pub struct SimpleLogger;

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target() == "relay"
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if record.metadata().level() <= Level::Warn { 
                eprintln!("{}: {}", record.level(), record.args());
            } else {
                println!("{}: {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {}
}