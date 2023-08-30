//! SP3 precise orbit file parser.
#![cfg_attr(docrs, feature(doc_cfg))]

use std::collections::BTreeMap;
use rinex::prelude::{Constellation, Sv};
use hifitime::{Duration, Epoch, TimeScale};

use thiserror::Error;
use std::str::FromStr;

#[cfg(test)]
mod tests;

mod header;
mod merge;
mod reader;
mod version;

use header::{
    line1::{is_header_line1, Line1},
    line2::{is_header_line2, Line2},
};

use reader::BufferedReader;
use std::io::BufRead;
use version::Version;

#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};

pub mod prelude {
    pub use crate::version::Version;
    //pub use rinex::{Sv, Constellation};
    pub use crate::{DataType, OrbitType, SP3};
    pub use hifitime::{Duration, Epoch, TimeScale};
}

pub use merge::Merge;

fn file_descriptor(content: &str) -> bool {
    content.starts_with("%c")
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

// fn possition_error(content: &str) -> bool {
//     content.starts_with("EP")
// }

// fn velocity(content: &str) -> bool {
//     content.starts_with('V')
// }

// fn velocity_error(content: &str) -> bool {
//     content.starts_with("EV")
// }

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
    type Err = ParsingError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.eq("P") {
            Ok(Self::Position)
        } else {
            Err(ParsingError::UnknownDataType(s.to_string()))
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
    type Err = ParsingError;
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
            Err(ParsingError::UnknownOrbitType(s.to_string()))
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
 * type VelocityData = BTreeMap<Epoch, f64>;
 */

/*
 * Comments contained in file
 */
type Comments = Vec<String>;

#[derive(Default, Clone, Debug)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub struct SP3 {
    /// File revision
    pub version: Version,
    pub data_type: DataType,
    pub coord_system: String,
    pub orbit_type: OrbitType,
    /// Agency providing this data
    pub agency: String,
    /// Type of constellations encountered in this file.
    /// For example "GPS" means only GPS vehicles are present.
    pub constellation: Constellation,
    /// File original time system,
    /// either UTC or time source from which we converted to UTC.
    pub time_system: TimeScale,
    /// Initial week counter, in time_system
    pub week_counter: (u32, f64),
    /// Initial MJD, in time_system
    pub mjd_start: (u32, f64),
    /// [`Epoch`]s where at least one position
    /// or one clock data is provided. Epochs are expressed UTC time,
    /// either directly if provided as such, or internally converted.
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
    #[error("hifitime parsing error")]
    HifitimeParsingError(#[from] hifitime::Errors),
    #[error("constellation parsing error")]
    ConstellationParsingError(#[from] rinex::constellation::Error),
    #[error("file i/o error")]
    DataParsingError(#[from] std::io::Error),
}

#[derive(Debug, Error)]
pub enum ParsingError {
    #[error("unknown or non supported revision \"{0}\"")]
    UnknownVersion(String),
    #[error("unknown data type \"{0}\"")]
    UnknownDataType(String),
    #[error("unknown orbit type \"{0}\"")]
    UnknownOrbitType(String),
    #[error("malformed header line #1")]
    MalformedH1,
    #[error("malformed header line #2")]
    MalformedH2,
    #[error("malformed %c line \"{0}\"")]
    MalformedDescriptor(String),
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
fn parse_epoch(content: &str, time_scale: TimeScale) -> Result<Epoch, ParsingError> {
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
        "{:04}-{:02}-{:02}T{:02}:{:02}:{:02} {}",
        y, m, d, hh, mm, ss, time_scale,
    ))
    .or(Err(ParsingError::Epoch))
}

impl SP3 {
    /// Parses given SP3 file, with possible seamless
    /// .gz decompression, if compiled with the "flate2" feature.
    pub fn from_file(path: &str) -> Result<Self, Errors> {
        let mut reader = BufferedReader::new(path)?;

        let mut version = Version::default();
        let mut data_type = DataType::default();

        let mut pc_count = 0_u8;
        let mut time_system = TimeScale::default();
        let mut constellation = Constellation::default();

        //let mut start_epoch = Epoch::default();
        //let mut nb_epochs = 0;

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

        for line in reader.lines() {
            let line = line.unwrap();
            let line = line.trim();

            if sp3_comment(line) {
                comments.push(line[3..].to_string());
                continue;
            }
            if end_of_file(line) {
                break;
            }
            if is_header_line1(line) && !is_header_line2(line) {
                let l1 = Line1::from_str(line)?;
                (version, data_type, coord_system, orbit_type, agency) = l1.to_parts();
            }
            if is_header_line2(line) {
                let l2 = Line2::from_str(line)?;
                (week_counter, epoch_interval, mjd_start) = l2.to_parts();
            }
            if file_descriptor(line) {
                if line.len() < 60 {
                    return Err(Errors::ParsingError(ParsingError::MalformedDescriptor(
                        line.to_string(),
                    )));
                }

                if pc_count == 0 {
                    constellation = Constellation::from_str(line[3..4].trim())?;
                    time_system = TimeScale::from_str(line[9..12].trim())?;
                }

                pc_count += 1;
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
                epoch = parse_epoch(&line[3..], time_system)?;
                epochs.push(epoch);
            }
        }

        Ok(Self {
            version,
            data_type,
            epoch: epochs,
            time_system,
            constellation,
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
    /// Fit Lagrangian polynomial of desired oreder, to interpolate data at desired Epoch.
    /// Only Odd orders are currently supported currently !
    pub fn interpolate(&self, epoch: Epoch, sv: Sv, order: usize) -> Option<(f64, f64, f64)> {
        let x = epoch;
        if order % 2 > 0 {
            /*
             * Only odd orders currently supported
             */
            return None;
        }
        let before: Vec<(Epoch, f64)> = self
            .sv_position()
            .filter_map(|(e, svnn, (x, _y, _z))| {
                if e <= epoch && svnn == sv {
                    Some((e, x))
                } else {
                    None
                }
            })
            .collect();
        let after: Vec<(Epoch, f64)> = self
            .sv_position()
            .filter_map(|(e, svnn, (x, _y, _z))| {
                if e > epoch && svnn == sv {
                    Some((e, x))
                } else {
                    None
                }
            })
            .collect();

        if before.len() < order / 2 && after.len() < order / 2 {
            return None; // not enough data in this window
        }

        let n = before.len();
        let mut lagrangians: Vec<f64> = Vec::with_capacity(order);
        let mut polynomials: Vec<f64> = Vec::with_capacity(order);
        for i in 0..order {
            let mut prod = 1.0_f64;
            for j in 0..order {
                if i == j {
                    continue;
                }
                if j > order / 2 {
                    prod *= (x - after[j].0).to_seconds();
                    prod /= (after[i].0 - after[j].0).to_seconds();
                } else {
                    prod *= (x - before[n - j].0).to_seconds();
                    prod /= (before[n - i].0 - before[n - j].0).to_seconds();
                }
            }
            lagrangians[i] = prod;
        }
        for i in 0..order {
            if i > order / 2 {
                polynomials[i] += after[i].1 * lagrangians[i];
            } else {
                polynomials[i] += before[i].1 * lagrangians[i];
            }
        }
        Some((0.0_f64, 0.0_f64, 0.0_f64))
    }
}

use merge::MergeError;

impl Merge for SP3 {
    fn merge(&self, rhs: &Self) -> Result<Self, MergeError> {
        let mut s = self.clone();
        s.merge_mut(rhs)?;
        Ok(s)
    }
    fn merge_mut(&mut self, rhs: &Self) -> Result<(), MergeError> {
        if self.agency != rhs.agency {
            return Err(MergeError::DataProvider);
        }
        if self.time_system != rhs.time_system {
            return Err(MergeError::TimeScale);
        }
        if self.coord_system != rhs.coord_system {
            return Err(MergeError::CoordSystem);
        }
        if self.constellation != rhs.constellation {
            /*
             * Convert self to Mixed constellation
             */
            self.constellation = Constellation::Mixed;
        }
        // adjust revision
        if rhs.version > self.version {
            self.version = rhs.version;
        }
        // Adjust MJD start
        if rhs.mjd_start.0 < self.mjd_start.0 {
            self.mjd_start.0 = rhs.mjd_start.0;
        }
        if rhs.mjd_start.1 < self.mjd_start.1 {
            self.mjd_start.1 = rhs.mjd_start.1;
        }
        // Adjust week counter
        if rhs.week_counter.0 < self.week_counter.0 {
            self.week_counter.0 = rhs.week_counter.0;
        }
        if rhs.week_counter.1 < self.week_counter.1 {
            self.week_counter.1 = rhs.week_counter.1;
        }
        // update Sv table
        for sv in &rhs.sv {
            if !self.sv.contains(sv) {
                self.sv.push(*sv);
            }
        }
        // update sampling interval (pessimistic)
        self.epoch_interval = std::cmp::max(self.epoch_interval, rhs.epoch_interval);

        for (epoch, svnn) in &rhs.position {
            if let Some(lhs_sv) = self.position.get_mut(epoch) {
                for (sv, position) in svnn {
                    lhs_sv.insert(*sv, *position);
                }
            } else {
                // introduce new epoch
                self.epoch.push(*epoch);
                self.position.insert(*epoch, svnn.clone());
            }
        }

        for (epoch, svnn) in &rhs.clock {
            if let Some(lhs_sv) = self.clock.get_mut(epoch) {
                for (sv, clock) in svnn {
                    lhs_sv.insert(*sv, *clock);
                }
            } else {
                // introduce new epoch : in clock record
                self.clock.insert(*epoch, svnn.clone());
                // introduce new epoch : if not contained in positions
                let mut found = false;
                for e in &self.epoch {
                    found |= *e == *epoch;
                    if found {
                        break;
                    }
                }
                if !found {
                    self.epoch.push(*epoch);
                }
            }
        }

        // maintain Epochs in correct order
        self.epoch.sort();
        Ok(())
    }
}
