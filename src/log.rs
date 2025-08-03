use std::cell::Cell;

use libretro_rs::{c_utf8::CUtf8, ffi::retro_log_level, retro::log::{LogInterface, PlatformLogger}};

#[allow(dead_code)]
#[derive(Clone, Copy)]
pub enum LogLevel {
    Info,
    Debug,
    Warn,
    Error,
}

impl Into<retro_log_level> for LogLevel {
    fn into(self) -> retro_log_level {
        match self {
            LogLevel::Info => retro_log_level::RETRO_LOG_INFO,
            LogLevel::Debug => retro_log_level::RETRO_LOG_DEBUG,
            LogLevel::Warn => retro_log_level::RETRO_LOG_WARN,
            LogLevel::Error => retro_log_level::RETRO_LOG_ERROR,
        }
    }
}

pub struct SnemLogger {
    logger: Cell<Option<PlatformLogger>>,
}

impl SnemLogger {
    pub fn new(logger: Option<PlatformLogger>) -> Self {
        SnemLogger {
            logger: Cell::new(logger),
        }
    }

    pub fn log(&self, level: LogLevel, message: &str) {
        if let Some(mut logger) = self.logger.get() {
            logger.log(
                level.into(), 
                // Safety: \0 char included manually in formatted str so from_str_unchecked is fine.
                unsafe { CUtf8::from_str_unchecked(format!("[Snemulator] {}\0", message).as_str()) }
            );

            self.logger.set(Some(logger));
        }
    }
}