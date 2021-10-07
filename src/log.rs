use serde::Deserialize;

#[derive(Debug, PartialEq, PartialOrd, Eq, Ord, Clone, Deserialize)]
pub enum LogLevel {
    Quiet,
    Error,
    Info,
}

impl Default for LogLevel {
    fn default() -> Self {
        Self::Info
    }
}

impl LogLevel {
    pub fn is_writable(&self, level: &Self) -> bool {
        use std::cmp::Ordering;
        matches!(self.cmp(level), Ordering::Greater | Ordering::Equal)
    }

    pub fn write(&self, level: &Self) -> Box<dyn std::io::Write> {
        if self.is_writable(level) {
            match level {
                Self::Error => Box::from(std::io::stderr()),
                _ => Box::from(std::io::stdout()),
            }
        } else {
            Box::from(std::io::sink())
        }
    }
}

impl From<LogLevel> for &'static str {
    fn from(log_level: LogLevel) -> &'static str {
        match log_level {
            LogLevel::Quiet => "quiet",
            LogLevel::Info => "info",
            LogLevel::Error => "error",
        }
    }
}

impl std::str::FromStr for LogLevel {
    type Err = String;

    fn from_str(s: &str) -> Result<LogLevel, Self::Err> {
        match s {
            "quiet" => Ok(Self::Quiet),
            "info" | "all" => Ok(Self::Info),
            "error" => Ok(Self::Error),
            level => Err(format!("I don't know the log level of {:?}", level)),
        }
    }
}

#[macro_export]
macro_rules! outln {
    ($config:ident#$level:path, $($expr:expr),+) => {{
        use $crate::log::LogLevel::*;
        writeln!($config.log_level.write(&$level), $($expr),+).expect("Can't write output");
    }}
}

#[macro_export]
macro_rules! debug {
    ($($expr:expr),+) => {
        #[cfg(debug_assertions)]
        {
            use std::io::{Write};
            use std::fs::OpenOptions;
            let mut file = OpenOptions::new()
                .write(true)
                .create(true)
                .append(true)
                .open("gobang.log")
                .unwrap();
            writeln!(file, $($expr),+).expect("Can't write output");
        }
    }
}
