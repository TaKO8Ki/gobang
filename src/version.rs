use std::{env, fmt};

#[derive(Default)]
pub struct Version {
    major: u32,
    minor: u32,
    patch: u32,
    pre: Option<String>,
}

impl Version {
    /// read version at compile time from env variables
    pub fn new() -> Self {
        let mut res = Self::default();
        let major_str = env!("CARGO_PKG_VERSION_MAJOR");
        if let Ok(major) = major_str.parse::<u32>() {
            res.major = major;
        }
        let minor_str = env!("CARGO_PKG_VERSION_MINOR");
        if let Ok(minor) = minor_str.parse::<u32>() {
            res.minor = minor;
        }
        let patch_str = env!("CARGO_PKG_VERSION_PATCH");
        if let Ok(patch) = patch_str.parse::<u32>() {
            res.patch = patch;
        }
        let pre_str = env!("CARGO_PKG_VERSION_PRE");
        res.pre = if pre_str.is_empty() {
            None
        } else {
            Some(pre_str.to_string())
        };
        res
    }
}

impl fmt::Display for Version {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "v{}.{}.{}{}",
            self.major,
            self.minor,
            self.patch,
            self.pre
                .as_ref()
                .map_or(String::new(), |pre| format!("-{}", pre.to_string()))
        )
    }
}
