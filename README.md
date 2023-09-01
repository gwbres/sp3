# SP3

[![crates.io](https://img.shields.io/crates/v/sp3.svg)](https://crates.io/crates/sp3)
[![Rust](https://github.com/gwbres/sp3/actions/workflows/rust.yml/badge.svg)](https://github.com/gwbres/sp3/actions/workflows/rust.yml)
[![crates.io](https://docs.rs/sp3/badge.svg)](https://docs.rs/sp3/)
[![crates.io](https://img.shields.io/crates/d/sp3.svg)](https://crates.io/crates/sp3)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/gwbres/sp3/blob/main/LICENSE-APACHE)
[![License](https://img.shields.io/badge/license-MIT-blue?style=flat-square)](https://github.com/gwbres/sp3/blob/main/LICENSE-MIT) 

SP3 Precise GNSS Orbit files parser. 

SP3 is specifid by [IGS](https://igs.org/products/#orbits_clocks).

The parser only supports Revisions C & D at the moment.

## Getting started

Add "sp3" to you cargo file

```toml
sp3 = "1"
```

Parse an SP3 file

```rust
use crate::prelude::*;
use rinex::prelude::Constellation;
use std::path::PathBuf;
use std::str::FromStr;
    
let path = PathBuf::new()
    .join(env!("CARGO_MANIFEST_DIR"))
    .join("data")
    .join("ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz");

let sp3 = SP3::from_file(&path.to_string_lossy());
assert!(
    sp3.is_ok(),
    "failed to parse ESA0OPSRAP_20232390000_01D_15M_ORB.SP3.gz : {:?}",
    sp3.err()
);

let sp3 = sp3.unwrap();

/*
 * Test general infos
 */
assert_eq!(sp3.version, Version::C);
assert_eq!(sp3.data_type, DataType::Position);

assert_eq!(
    sp3.first_epoch(),
    Some(Epoch::from_str("2023-08-27T00:00:00 GPST").unwrap())
);

assert_eq!(sp3.nb_epochs(), 96, "bad number of epochs");
assert_eq!(sp3.coord_system, "ITRF2");
assert_eq!(sp3.orbit_type, OrbitType::BHN);
assert_eq!(sp3.time_system, TimeScale::GPST);
assert_eq!(sp3.constellation, Constellation::Mixed);
assert_eq!(sp3.agency, "ESOC");

assert_eq!(sp3.week_counter, (2277, 0.0_f64));
assert_eq!(sp3.epoch_interval, Duration::from_seconds(900.0_f64));

// browse/iterate
for (epoch, svnn, (x, y, z)) in sp3.sv_position() {

}
```

## File Merge

## Position Vector Interpolation

The idea when interpolation 3D position state vectors,
is to preserve the +/- 1mm precision of the SP3 file.  

This library proposes a Lagrangian interpolation method that is designed to do just that.  
For an evenly sampled SP3, one can interpolate using order N, between [tmin, tmax] both included :

- tmin is T0 + (N +1)/2 *dt, where T0 is the initial Epoch
- tmax is TN - (N +1)/2 *dt, where TN is the last Epoch encountered

Refer to the online API

