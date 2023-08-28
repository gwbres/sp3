//! sp3 precise orbit file data by IGS

use hifitime::{Duration, Epoch};
use rinex::prelude::Sv;
use std::collections::BTreeMap;

use std::str::FromStr;
use thiserror::Error;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::{DataType, OrbitType, Version, SP3};
    pub use hifitime::{Duration, Epoch, TimeScale};
}

fn header_line1(content: &str) -> bool {
    content.starts_with('#') && !header_line2(content)
}

fn header_line2(content: &str) -> bool {
    content.starts_with("##")
}

fn sv_identifier(content: &str) -> bool {
    content.starts_with('+') && !orbit_accuracy(content)
}

fn orbit_accuracy(content: &str) -> bool {
    content.starts_with("++")
}

fn sp3_comment(content: &str) -> bool {
    content.starts_with("/*")
}

fn end_of_file(content: &str) -> bool {
    content.eq("EOF")
}

fn position(content: &str) -> bool {
    content.starts_with('P')
}

fn possition_error(content: &str) -> bool {
    content.starts_with("EP")
}

fn velocity(content: &str) -> bool {
    content.starts_with('V')
}

fn velocity_error(content: &str) -> bool {
    content.starts_with("EV")
}

#[derive(Default, Clone, Debug, PartialEq, PartialOrd, Eq, Hash)]
pub enum Version {
    #[default]
    D,
}

impl std::fmt::Display for Version {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::D => f.write_str("d"),
        }
    }
}

impl std::str::FromStr for Version {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("d") {
            Ok(Self::D)
        } else {
            Err(Error::UnknownVersion(s.to_string()))
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum DataType {
    #[default]
    Position,
}

impl std::fmt::Display for DataType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::Position => f.write_str("P"),
        }
    }
}

impl std::str::FromStr for DataType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else {
            Err(Error::UnknownDataType(s.to_string()))
        }
    }
}

#[derive(Default, Clone, Debug, PartialEq, Eq, Hash)]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    HLM,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FIT => f.write_str("FIT"),
            Self::EXT => f.write_str("EXT"),
            Self::BCT => f.write_str("BCT"),
            Self::HLM => f.write_str("HLM"),
        }
    }
}

impl std::str::FromStr for OrbitType {
    type Err = Error;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FIT") {
            Ok(Self::FIT)
        } else if s.eq("EXT") {
            Ok(Self::EXT)
        } else if s.eq("BCT") {
            Ok(Self::BCT)
        } else if s.eq("HLM") {
            Ok(Self::HLM)
        } else {
            Err(Error::UnknownOrbitType(s.to_string()))
        }
    }
}

/*
 * Position + Clock data
 */
type PositionClockData = BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64, f64)>>;

/*
 * Velocity data
 */
type VelocityData = BTreeMap<Epoch, f64>;

/*
 * Comments contained in file
 */
type Comments = Vec<String>;

#[derive(Default, Clone, Debug)]
pub struct SP3 {
    pub version: Version,
    pub data_type: DataType,
    pub start_epoch: Epoch,
    pub nb_epochs: u32,
    pub coord_system: String,
    pub orbit_type: OrbitType,
    pub agency: String,
    pub week_counter: (u32, f64),
    pub epoch_interval: Duration,
    pub mjd_start: (u32, f64),
    /// Satellite Vehicles identifier
    pub sv: Vec<Sv>,
    /// Positions
    pub position: PositionClockData,
    /// Encountered comments, stored as is
    pub comments: Comments,
}

#[derive(Debug, Error)]
pub enum Error {
    #[error("parsing error")]
    ParsingError(#[from] ParsingError),
    #[error("unknown or non supported revision \"{0}\"")]
    UnknownVersion(String),
    #[error("unknown data type \"{0}\"")]
    UnknownDataType(String),
    #[error("unknown orbit type \"{0}\"")]
    UnknownOrbitType(String),
    #[error("file i/o error")]
    DataParsingError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("failed to parse epoch year from \"{0}\"")]
    EpochYear(String),
    #[error("failed to parse epoch month from \"{0}\"")]
    EpochMonth(String),
    #[error("failed to parse epoch day from \"{0}\"")]
    EpochDay(String),
    #[error("failed to parse epoch hours from \"{0}\"")]
    EpochHours(String),
    #[error("failed to parse epoch minutes from \"{0}\"")]
    EpochMinutes(String),
    #[error("failed to parse epoch seconds from \"{0}\"")]
    EpochSeconds(String),
    #[error("failed to parse epoch milliseconds from \"{0}\"")]
    EpochMilliSeconds(String),
    #[error("failed to parse number of epochs \"{0}\"")]
    NumberEpoch(String),
    #[error("failed to parse week counter")]
    WeekCounter(String),
    #[error("failed to parse hifitime::Epoch")]
    Epoch,
}

impl SP3 {
    pub fn from_file(fp: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(fp)?;

        let mut version = Version::default();
        let mut data_type = DataType::default();
        let mut start_epoch = Epoch::default();
        let mut nb_epochs = 0;
        let mut coord_system = String::from("Unknown");
        let mut orbit_type = OrbitType::default();
        let mut agency = String::from("Unknown");
        let mut week_counter = (0_u32, 0_f64);

        let epoch_interval = Duration::default();
        let sv: Vec<Sv> = Vec::new();
        let position = PositionClockData::default();
        let mjd_start = (0_u32, 0_f64);
        let mut comments = Comments::new();

        for line in content.lines() {
            let line = line.trim();
            if sp3_comment(line) {
                comments.push(line[3..].to_string());
                continue;
            }
            if end_of_file(line) {
                break;
            }
            if header_line1(line) {
                version = Version::from_str(&line[1..2])?;
                data_type = DataType::from_str(&line[2..3])?;

                let y = u32::from_str(line[3..7].trim())
                    .or(Err(ParsingError::EpochYear(line[3..7].to_string())))?;

                let m = u32::from_str(line[7..10].trim())
                    .or(Err(ParsingError::EpochMonth(line[7..10].to_string())))?;

                let d = u32::from_str(line[10..13].trim())
                    .or(Err(ParsingError::EpochDay(line[10..13].to_string())))?;

                let hh = u32::from_str(line[13..16].trim())
                    .or(Err(ParsingError::EpochHours(line[13..16].to_string())))?;

                let mm = u32::from_str(line[16..19].trim())
                    .or(Err(ParsingError::EpochMinutes(line[16..19].to_string())))?;

                let ss = u32::from_str(line[19..22].trim())
                    .or(Err(ParsingError::EpochSeconds(line[19..22].to_string())))?;

                let ss_fract = f64::from_str(line[23..30].trim()).or(Err(
                    ParsingError::EpochMilliSeconds(line[23..30].to_string()),
                ))?;

                start_epoch = Epoch::from_str(&format!(
                    "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} UTC",
                    y, m, d, hh, mm, ss
                ))
                .or(Err(ParsingError::Epoch))?;

                nb_epochs = u32::from_str(line[31..39].trim())
                    .or(Err(ParsingError::NumberEpoch(line[31..39].to_string())))?;

                //= &line[39..45];

                coord_system = line[45..51].trim().to_string();

                orbit_type = OrbitType::from_str(line[51..55].trim())?;
                agency = line[55..].trim().to_string();
                continue;
            }
            if header_line2(line) {
                week_counter.0 = u32::from_str(line[2..7].trim())
                    .or(Err(ParsingError::WeekCounter(line[2..7].to_string())))?;

                continue;
            }
        }

        Ok(Self {
            version,
            data_type,
            start_epoch,
            nb_epochs,
            coord_system,
            orbit_type,
            agency,
            week_counter,
            epoch_interval,
            mjd_start,
            sv,
            position,
            comments,
        })
    }
}
