//! sp3 precise orbit file data by IGS

use rinex::prelude::Sv;
use hifitime::{Epoch, Duration};
use std::collections::BTreeMap;

use thiserror::Error;

#[cfg(test)]
mod tests;

pub mod prelude {
    pub use crate::SP3;
    pub use hifitime::{Duration, Epoch, TimeScale};
}

fn header_line1(content: &str) -> bool {
    content.starts_with("#") && !header_line2(content)
}

fn header_line2(content: &str) -> bool {
    content.starts_with("##")
}

fn sv_identifier(content: &str) -> bool {
    content.starts_with("+") && !orbit_accuracy(content)
}

fn orbit_accuracy(content: &str) -> bool {
    content.starts_with("++")
}

fn sp3_comment(content: &str) -> bool {
    content.starts_with("/*")
}

fn end_of_file(content: &str) -> bool {
    content.trim().eq("EOF")
}

fn position(content: &str) -> bool {
    content.starts_with("P")
}

fn possition_error(content: &str) -> bool {
    content.starts_with("EP")
}

fn velocity(content: &str) -> bool {
    content.starts_with("V")
}

fn velocity_error(content: &str) -> bool {
    content.starts_with("EV")
}

#[derive(Default, Clone, Debug)]
pub enum Version {
    #[default]
    D,
}

#[derive(Default, Clone, Debug)]
pub enum DataType {
    #[default]
    Position,
}

#[derive(Default, Clone, Debug)]
pub enum OrbitType {
    #[default]
    FIT,
    EXT,
    BCT,
    HLM,
}

/*
 * Position + Clock data
 */
type PositionClockData = BTreeMap<Epoch, BTreeMap<Sv, (f64, f64, f64, f64)>>;

/*
 * Velocity data
 */
type VelocityData =  BTreeMap<Epoch, f64>;

/*
 * Comments contained in file
 */
type Comments = Vec<String>;

#[derive(Default, Clone, Debug)]
pub struct SP3 {
    pub version: Version,
    pub data_type: DataType,
    pub start_epoch: Epoch,
    pub nb_epochs: usize,
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
    #[error("failed to read provided file")]
    DataParsingError(#[from] std::io::Error), 
}

impl SP3 {
    pub fn from_file(fp: &str) -> Result<Self, Error> {
        let content = std::fs::read_to_string(fp)?;

        let version = Version::default();
        let data_type = DataType::default();
        let start_epoch = Epoch::default();
        let epoch_interval = Duration::default();
        let nb_epochs = 0;
        let sv: Vec<Sv> = Vec::new();
        let position = PositionClockData::default();
        let mjd_start = (0_u32, 0_f64);
        let week_counter = (0_u32, 0_f64);
        let agency = String::from("Unknown");
        let coord_system = String::from("Unknown");
        let orbit_type = OrbitType::default();
        let mut comments = Comments::new(); 

        for line in content.lines() {
            let line = line.trim();
            if sp3_comment(line) {
                comments.push(line[3..].to_string());
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
