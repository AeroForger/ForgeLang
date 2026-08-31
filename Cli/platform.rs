use std::fmt;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    Linux,
}

impl Platform {
    pub fn parse(s: &str) -> Result<Self, String> {
        match s.to_lowercase().as_str() {
            "linux" => Ok(Platform::Linux),
            other => Err(format!(
                "unsupported target platform '{}'. Currently supported platforms: linux",
                other
            )),
        }
    }

    pub fn linker_name(&self) -> &'static str {
        match self {
            Platform::Linux => "cc",
        }
    }

    pub fn default_linker_flags(&self) -> &'static [&'static str] {
        match self {
            Platform::Linux => &["-lm"],
        }
    }
}

impl fmt::Display for Platform {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Platform::Linux => write!(f, "linux"),
        }
    }
}
