mod extract;
mod join;
mod vcs;

pub use extract::*;
pub use join::*;
pub use vcs::*;

use std::{fmt, str::FromStr};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum DurationOrPercent {
    Seconds(f32),
    Percent(f32),
}

impl DurationOrPercent {
    pub fn to_secs(self, total_s: f32) -> f32 {
        match self {
            Self::Seconds(s) => s,
            Self::Percent(p) => total_s * p * 0.01,
        }
    }
}

impl FromStr for DurationOrPercent {
    type Err = anyhow::Error;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        use DurationOrPercent::{Percent, Seconds};

        let v = v.trim();

        if let Some(percent) = v.strip_suffix('%') {
            return Ok(Percent(percent.parse::<f32>()?));
        }

        Ok(Seconds(humantime::parse_duration(v)?.as_secs_f32()))
    }
}

impl fmt::Display for DurationOrPercent {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Seconds(s) => write!(f, "{s}s"),
            Self::Percent(p) => write!(f, "{p}%"),
        }
    }
}

impl Default for DurationOrPercent {
    fn default() -> Self {
        Self::Seconds(0.0)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq)]
pub struct HumanDuration {
    pub seconds: f32,
}

impl FromStr for HumanDuration {
    type Err = anyhow::Error;

    fn from_str(v: &str) -> Result<Self, Self::Err> {
        Ok(Self {
            seconds: humantime::parse_duration(v.trim())?.as_secs_f32(),
        })
    }
}

impl fmt::Display for HumanDuration {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}s", self.seconds)
    }
}
