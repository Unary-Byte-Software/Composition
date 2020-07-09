// logger.rs
// author: Garen Tyler
// description:
//   A global logger for Composition.
//   The Logger struct makes it easy to create useful server logs.

extern crate chrono; // Used because std::time sucks.
extern crate toml; // Colorful console logging is fun.

use chrono::prelude::*;
use colorful::{Color, Colorful};
use std::fs::File;
use std::fs::OpenOptions;
use std::io::prelude::*;

// Uses chrono::Local to get a timestamp in YYYY-MM-DD HH:MM:SS format, in local time.
pub fn get_timestring() -> String {
    Local::now().format("%Y-%m-%d %H:%M:%S").to_string()
}
// Just a helper function to avoid repeating myself.
pub fn log(logtype: &str, logmessage: &str) -> String {
    format!("{} [{}] - {}", get_timestring(), logtype, logmessage)
}

#[derive(Clone)]
pub struct Logger {
    pub logfile: String,
}
impl Logger {
    pub fn new(logfile: &str) -> Logger {
        Logger {
            logfile: logfile.to_owned(),
        }
    }
    // For logging something important, like the server port.
    pub fn important(&self, s: &str) {
        let l = log("IMPORTANT", s);
        println!("{}", l.clone().color(Color::Green));
        self.append(&l);
    }
    // For everything that doesn't fit into any of the other categories.
    pub fn info(&self, s: &str) {
        let l = log("INFO", s);
        // Not sure whether I want normal logs to be user controlled color or white.
        // println!("{}", l.clone().color(Color::White));
        println!("{}", l.clone());
        self.append(&l);
    }
    // For warnings.
    pub fn warn(&self, s: &str) {
        let l = log("WARN", s);
        println!("{}", l.clone().color(Color::LightYellow));
        self.append(&l);
    }
    // For errors.
    pub fn error(&self, s: &str) {
        let l = log("ERROR", s);
        println!("{}", l.clone().color(Color::LightRed));
        self.append(&l);
    }
    // Append to the logfile.
    pub fn append(&self, s: &str) {
        let a = || -> std::io::Result<()> {
            let mut file = OpenOptions::new()
                .create(true)
                .write(true)
                .append(true)
                .open(&self.logfile)?;
            writeln!(file, "{}", s)?;
            Ok(())
        };
        if a().is_err() {
            self.logger_error(&format!("Could not write to log file {}", self.logfile));
        }
    }
    // Clear the logfile. Adds an important note to the file that it was cleared.
    pub fn clear(&self) {
        if std::fs::remove_file(&self.logfile).is_err() {
            self.logger_error(&format!("Could not write to log file {}", self.logfile));
        }
        self.important(&format!("Cleared log file {}", self.logfile));
    }
    // Read the logfile into a String.
    pub fn read(&self) -> std::io::Result<String> {
        let mut f = File::open(&self.logfile)?;
        let mut buffer = String::new();
        f.read_to_string(&mut buffer)?;
        Ok(buffer)
    }
    // For a critical logger error. This doesn't append to the logfile.
    pub fn logger_error(&self, error_message: &str) {
        let l = log("ERROR", error_message);
        println!("{}", l.clone().color(Color::LightRed));
    }
}
