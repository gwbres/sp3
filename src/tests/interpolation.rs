//! SP3 interpolation specific tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use rinex::prelude::Sv;
    use rinex::sv;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[cfg(feature = "flate2")]
    #[test]
    fn interp_feasibility() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz");
        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(
            sp3.is_ok(),
            "failed to parse EMR0OPSULT_20232391800_02D_15M_ORB.SP3.gz"
        );
        let sp3 = sp3.unwrap();
        for (epoch, is_some) in vec![
            ("2023-08-27T18:00:05 GPST", false),
            ("2023-08-27T18:00:15 GPST", false),
            ("2023-08-27T18:00:30 GPST", false),
            ("2023-08-27T18:00:45 GPST", false),
            ("2023-08-27T18:15:00 GPST", false),
            ("2023-08-27T18:30:00 GPST", true),
            ("2023-08-27T18:30:10 GPST", true),
            ("2023-08-27T18:45:00 GPST", true),
            ("2023-08-27T19:00:00 GPST", true),
        ] {
            let e = Epoch::from_str(epoch).unwrap();
            let position = sp3.interpolate(e, sv!("G01"), 5);
            assert_eq!(position.is_some(), is_some, "failed for \"{}\"", epoch);
        }
    }
}
