use atomic_refcell::AtomicRefCell;
use log::{Level, Metadata, Record};
use std::fs::File;
use std::io::Write;
use std::ops::DerefMut;

pub struct SimpleLogger {
    file: AtomicRefCell<std::fs::File>,
}

impl SimpleLogger {
    pub fn new(path: String) -> Self {
        let file = File::create(path).expect("Error on log file creation");
        Self {
            file: AtomicRefCell::new(file),
        }
    }
}

impl log::Log for SimpleLogger {
    fn enabled(&self, metadata: &Metadata) -> bool {
        metadata.target() == "relay"
    }

    fn log(&self, record: &Record) {
        if self.enabled(record.metadata()) {
            if let Err(err) = self
                .file
                .borrow_mut()
                .deref_mut()
                .write_all(format!("{}: {}\n", record.level(), record.args()).as_bytes())
            {
                eprintln!("Error on flush: {}", err);
            }

            if record.metadata().level() <= Level::Warn {
                eprintln!("{}: {}", record.level(), record.args());
            } else {
                println!("{}: {}", record.level(), record.args());
            }
        }
    }

    fn flush(&self) {
        if let Err(err) = self.file.borrow_mut().deref_mut().flush() {
            eprintln!("Error on flush: {}", err);
        }
    }
}
