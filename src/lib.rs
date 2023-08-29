//! SP3 precise orbit file parser.

use hifitime::{Duration, Epoch};
use itertools::Itertools;
use rinex::prelude::Sv;
use std::collections::BTreeMap;

use std::str::FromStr;
use thiserror::Error;

#[cfg(test)]
mod tests;

mod version;
use version::Version;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::version::Version;
    pub use crate::{DataType, OrbitType, SP3};
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

fn position_entry(content: &str) -> bool {
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

fn new_epoch(content: &str) -> bool {
    content.starts_with("*  ")
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
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
    type Err = Errors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else {
            Err(Errors::UnknownDataType(s.to_string()))
        }
    }
}

#[derive(Default, Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    BHN,
    HLM,
}

impl std::fmt::Display for OrbitType {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            Self::FIT => f.write_str("FIT"),
            Self::EXT => f.write_str("EXT"),
            Self::BCT => f.write_str("BCT"),
            Self::BHN => f.write_str("BHN"),
            Self::HLM => f.write_str("HLM"),
        }
    }
}

impl std::str::FromStr for OrbitType {
    type Err = Errors;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("FIT") {
            Ok(Self::FIT)
        } else if s.eq("EXT") {
            Ok(Self::EXT)
        } else if s.eq("BCT") {
            Ok(Self::BCT)
        } else if s.eq("BHN") {
            Ok(Self::BHN)
        } else if s.eq("HLM") {
            Ok(Self::HLM)
        } else {
            Err(Errors::UnknownOrbitType(s.to_string()))
        }
    }
}

type Position = (f64, f64, f64);

/*
 * Positions
 */
type PositionRecord = BTreeMap<Epoch, BTreeMap<Sv, Position>>;

/*
 * Clock estimates
 */
type ClockRecord = BTreeMap<Epoch, BTreeMap<Sv, f64>>;

/*
 * Velocity data
 */
type VelocityData = BTreeMap<Epoch, f64>;

/*
 * Comments contained in file
 */
type Comments = Vec<String>;

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3 {
    pub version: Version,
    pub data_type: DataType,
    pub coord_system: String,
    pub orbit_type: OrbitType,
    pub agency: String,
    pub week_counter: (u32, f64),
    pub mjd_start: (u32, f64),
    /// [`Epoch`]s where at least one position
    /// or one clock data is provided
    pub epoch: Vec<Epoch>,
    /// Returns sampling interval, ie., time between successive [`Epoch`]s.
    pub epoch_interval: Duration,
    /// Satellite Vehicles
    pub sv: Vec<Sv>,
    /// Positions expressed in km, with 1mm precision, per Epoch and Sv.
    pub position: PositionRecord,
    /// Clock estimates in microseconds, with 1E-12 precision per Epoch and Sv.
    pub clock: ClockRecord,
    /// Encountered comments, stored as is
    pub comments: Comments,
}

#[derive(Debug, Error)]
pub enum Errors {
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
    #[error("malformed header line #1")]
    MalformedH1,
    #[error("malformed header line #2")]
    MalformedH2,
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
    #[error("failed to parse sample rate from \"{0}\"")]
    EpochInterval(String),
    #[error("failed to parse mjd start \"{0}\"")]
    Mjd(String),
    #[error("failed to parse sv from \"{0}\"")]
    Sv(String),
    #[error("failed to parse (x, y, or z) coordinates from \"{0}\"")]
    Coordinates(String),
}

/*
 * Parses hifitime::Epoch from standard format
 */
fn parse_epoch(content: &str) -> Result<Epoch, ParsingError> {
    let y = u32::from_str(content[0..4].trim())
        .or(Err(ParsingError::EpochYear(content[0..4].to_string())))?;

    let m = u32::from_str(content[4..7].trim())
        .or(Err(ParsingError::EpochMonth(content[4..7].to_string())))?;

    let d = u32::from_str(content[7..10].trim())
        .or(Err(ParsingError::EpochDay(content[7..10].to_string())))?;

    let hh = u32::from_str(content[10..13].trim())
        .or(Err(ParsingError::EpochHours(content[10..13].to_string())))?;

    let mm = u32::from_str(content[13..16].trim())
        .or(Err(ParsingError::EpochMinutes(content[13..16].to_string())))?;

    let ss = u32::from_str(content[16..19].trim())
        .or(Err(ParsingError::EpochSeconds(content[16..19].to_string())))?;

    let ss_fract = f64::from_str(content[20..27].trim()).or(Err(
        ParsingError::EpochMilliSeconds(content[20..27].to_string()),
    ))?;

    Epoch::from_str(&format!(
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} UTC",
        y, m, d, hh, mm, ss
    ))
    .or(Err(ParsingError::Epoch))
}

impl SP3 {
    pub fn from_file(fp: &str) -> Result<Self, Errors> {
        let content = std::fs::read_to_string(fp)?;

        let mut version = Version::default();
        let mut data_type = DataType::default();

        let mut start_epoch = Epoch::default();
        let mut nb_epochs = 0;

        let mut coord_system = String::from("Unknown");
        let mut orbit_type = OrbitType::default();
        let mut agency = String::from("Unknown");
        let mut week_counter = (0_u32, 0_f64);
        let mut epoch_interval = Duration::default();
        let mut mjd_start = (0_u32, 0_f64);

        let vehicles: Vec<Sv> = Vec::new();
        let mut position = PositionRecord::default();
        let mut clock = ClockRecord::default();
        let mut comments = Comments::new();

        let mut epoch = Epoch::default();
        let mut epochs: Vec<Epoch> = Vec::new();

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
                if line.len() != 60 {
                    return Err(Errors::ParsingError(ParsingError::MalformedH1));
                }

                version = Version::from_str(&line[1..2])?;
                data_type = DataType::from_str(&line[2..3])?;
                start_epoch = parse_epoch(&line[3..])?;

                nb_epochs = u32::from_str(line[31..39].trim())
                    .or(Err(ParsingError::NumberEpoch(line[31..39].to_string())))?;

                //= &line[39..45];

                coord_system = line[45..51].trim().to_string();

                orbit_type = OrbitType::from_str(line[51..55].trim())?;
                agency = line[55..].trim().to_string();
            }
            if header_line2(line) {
                if line.len() != 60 {
                    return Err(Errors::ParsingError(ParsingError::MalformedH2));
                }

                week_counter.0 = u32::from_str(line[2..7].trim())
                    .or(Err(ParsingError::WeekCounter(line[2..7].to_string())))?;

                week_counter.1 = f64::from_str(line[7..23].trim())
                    .or(Err(ParsingError::WeekCounter(line[7..23].to_string())))?;

                let dt = f64::from_str(line[24..38].trim())
                    .or(Err(ParsingError::EpochInterval(line[24..38].to_string())))?;
                epoch_interval = Duration::from_seconds(dt);

                mjd_start.0 = u32::from_str(line[38..44].trim())
                    .or(Err(ParsingError::Mjd(line[38..44].to_string())))?;

                mjd_start.1 = f64::from_str(line[44..].trim())
                    .or(Err(ParsingError::Mjd(line[44..].to_string())))?;
            }
            if position_entry(line) {
                if line.len() < 60 {
                    /*
                     * tolerate malformed positions
                     */
                    continue;
                }
                let sv = Sv::from_str(line[1..4].trim())
                    .or(Err(ParsingError::Sv(line[1..4].to_string())))?;

                let pos_x = f64::from_str(line[4..18].trim())
                    .or(Err(ParsingError::Coordinates(line[4..18].to_string())))?;
                let pos_y = f64::from_str(line[18..32].trim())
                    .or(Err(ParsingError::Coordinates(line[18..32].to_string())))?;
                let pos_z = f64::from_str(line[32..46].trim())
                    .or(Err(ParsingError::Coordinates(line[32..46].to_string())))?;

                if pos_x != 0.0_f64 && pos_y != 0.0_f64 && pos_z != 0.0_f64 {
                    /*
                     * Position vector is present & correct
                     */
                    if let Some(e) = position.get_mut(&epoch) {
                        e.insert(sv, (pos_x, pos_y, pos_z));
                    } else {
                        let mut map: BTreeMap<Sv, Position> = BTreeMap::new();
                        map.insert(sv, (pos_x, pos_y, pos_z));
                        position.insert(epoch, map);
                    }
                }

                if !line[46..53].trim().eq("999999.") {
                    /*
                     * Clock data is present & correct
                     */
                    let clk = f64::from_str(line[46..60].trim())
                        .or(Err(ParsingError::Coordinates(line[46..60].to_string())))?;

                    if let Some(e) = clock.get_mut(&epoch) {
                        e.insert(sv, clk);
                    } else {
                        let mut map: BTreeMap<Sv, f64> = BTreeMap::new();
                        map.insert(sv, clk);
                        clock.insert(epoch, map);
                    }
                }
            }
            if new_epoch(line) {
                epoch = parse_epoch(&line[3..])?;
                epochs.push(epoch);
            }
        }

        Ok(Self {
            version,
            data_type,
            epoch: epochs,
            coord_system,
            orbit_type,
            agency,
            week_counter,
            epoch_interval,
            mjd_start,
            sv: vehicles,
            position,
            clock,
            comments,
        })
    }
    /// Returns a unique Epoch iterator where either
    /// Position or Clock data is provided.
    pub fn epoch(&self) -> impl Iterator<Item = Epoch> + '_ {
        self.epoch.iter().copied()
    }
    /// Returns total number of epoch
    pub fn nb_epochs(&self) -> usize {
        self.epoch.len()
    }
    /// Returns first epoch
    pub fn first_epoch(&self) -> Option<Epoch> {
        self.epoch.get(0).copied()
    }
    /// Returns last epoch
    pub fn last_epoch(&self) -> Option<Epoch> {
        self.epoch.last().copied()
    }
    /// Returns a unique Sv iterator
    pub fn sv(&self) -> impl Iterator<Item = Sv> + '_ {
        self.sv.iter().copied()
    }
    /// Returns an Iterator over Sv position estimates, in km
    /// with 1mm precision.
    pub fn sv_position(&self) -> impl Iterator<Item = (Epoch, Sv, (f64, f64, f64))> + '_ {
        self.position
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, pos)| (*e, *sv, *pos)))
    }
    /// Returns an Iterator over Clock error estimates, in microseconds
    /// with 1E-12 precision.
    pub fn sv_clock(&self) -> impl Iterator<Item = (Epoch, Sv, f64)> + '_ {
        self.clock
            .iter()
            .flat_map(|(e, sv)| sv.iter().map(|(sv, clk)| (*e, *sv, *clk)))
    }
}
