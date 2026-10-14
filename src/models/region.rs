use std::fmt;
use std::str::FromStr;

use serde::{Deserialize, Serialize};

/// UK electricity distribution regions used by Octopus Energy.
/// Each region corresponds to a Distribution Network Operator (DNO) area.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
pub enum Region {
    A,
    B,
    #[default]
    C,
    D,
    E,
    F,
    G,
    H,
    J,
    K,
    L,
    M,
    N,
    P,
}

impl Region {
    /// All regions, in code order. Octopus does not use `I` or `O`.
    pub const ALL: [Self; 14] = [
        Self::A,
        Self::B,
        Self::C,
        Self::D,
        Self::E,
        Self::F,
        Self::G,
        Self::H,
        Self::J,
        Self::K,
        Self::L,
        Self::M,
        Self::N,
        Self::P,
    ];

    /// The single-letter code used in tariff codes and API URLs.
    pub const fn code(self) -> &'static str {
        match self {
            Self::A => "A",
            Self::B => "B",
            Self::C => "C",
            Self::D => "D",
            Self::E => "E",
            Self::F => "F",
            Self::G => "G",
            Self::H => "H",
            Self::J => "J",
            Self::K => "K",
            Self::L => "L",
            Self::M => "M",
            Self::N => "N",
            Self::P => "P",
        }
    }

    /// Human-readable name of the distribution area.
    pub const fn name(self) -> &'static str {
        match self {
            Self::A => "Eastern England",
            Self::B => "East Midlands",
            Self::C => "London",
            Self::D => "Merseyside and North Wales",
            Self::E => "West Midlands",
            Self::F => "North Eastern England",
            Self::G => "North Western England",
            Self::H => "Southern England",
            Self::J => "South Eastern England",
            Self::K => "Southern Wales",
            Self::L => "South Western England",
            Self::M => "Yorkshire",
            Self::N => "Southern Scotland",
            Self::P => "Northern Scotland",
        }
    }
}

impl fmt::Display for Region {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} ({})", self.name(), self.code())
    }
}

/// Returned when a string is not a known region code.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
#[error("invalid region code: {0:?}")]
pub struct ParseRegionError(String);

impl FromStr for Region {
    type Err = ParseRegionError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Self::ALL
            .into_iter()
            .find(|region| region.code().eq_ignore_ascii_case(s))
            .ok_or_else(|| ParseRegionError(s.to_owned()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_codes_case_insensitively() {
        assert_eq!("C".parse::<Region>(), Ok(Region::C));
        assert_eq!("m".parse::<Region>(), Ok(Region::M));
        assert!("X".parse::<Region>().is_err());
        assert!("I".parse::<Region>().is_err());
        assert!("".parse::<Region>().is_err());
    }

    #[test]
    fn every_region_round_trips_through_its_code() {
        for region in Region::ALL {
            assert_eq!(region.code().parse::<Region>(), Ok(region));
        }
    }

    #[test]
    fn displays_name_then_code() {
        assert_eq!(Region::C.to_string(), "London (C)");
    }
}
