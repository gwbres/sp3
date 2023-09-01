//! SP3 interpolation specific tests
#[cfg(test)]
mod test {
    use crate::prelude::*;
    use rinex::prelude::Sv;
    use rinex::sv;
    use std::path::PathBuf;
    use std::str::FromStr;
    /*
     * Theoretical maximal error of a Lagrangian interpolation
     * over a given Dataset for specified interpolation order
     */
    fn max_error(values: Vec<(Epoch, f64)>, epoch: Epoch, order: usize) -> f64 {
        let mut q = 1.0_f64;
        for (e, _) in values {
            q *= (epoch - e).to_seconds();
        }
        let factorial: usize = (1..=order + 1).product();
        q.abs() / factorial as f64 // TODO f^(n+1)[x]
    }
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
        for (epoch_desc, feasible) in vec![
            ("2023-08-27T18:00:00 GPST", false),
            ("2023-08-27T18:00:05 GPST", false),
            ("2023-08-27T18:00:15 GPST", false),
            ("2023-08-27T18:00:30 GPST", false),
            ("2023-08-27T18:00:45 GPST", false),
            ("2023-08-27T18:15:00 GPST", false),
            ("2023-08-27T18:30:00 GPST", false),
            ("2023-08-27T18:30:10 GPST", false),
            ("2023-08-27T18:45:00 GPST", true),
            ("2023-08-27T18:45:01 GPST", true),
            ("2023-08-27T18:45:05 GPST", true),
            ("2023-08-27T18:45:10 GPST", true),
            ("2023-08-27T19:00:00 GPST", true),
            ("2023-08-27T19:00:01 GPST", true),
            ("2023-08-27T19:00:05 GPST", true),
            ("2023-08-27T19:15:00 GPST", true),
            ("2023-08-27T19:30:00 GPST", true),
            ("2023-08-27T19:45:00 GPST", true),
            ("2023-08-27T20:00:00 GPST", true),
            ("2023-08-27T20:15:00 GPST", true),
            ("2023-08-27T20:30:00 GPST", true),
            ("2023-08-27T20:45:00 GPST", true),
        ] {
            let epoch = Epoch::from_str(epoch_desc).unwrap();
            let interpolated = sp3.interpolate(epoch, sv!("G01"), 5);
            assert_eq!(
                interpolated.is_some(),
                feasible,
                "interpolation feasibility should be : {} for \"{}\"",
                feasible,
                epoch_desc
            );
        }
        for (epoch_desc, feasible) in vec![
            ("2023-08-27T18:00:00 GPST", false),
            ("2023-08-27T18:00:05 GPST", false),
            ("2023-08-27T18:00:15 GPST", false),
            ("2023-08-27T18:00:30 GPST", false),
            ("2023-08-27T18:00:45 GPST", false),
            ("2023-08-27T18:15:00 GPST", false),
            ("2023-08-27T18:30:00 GPST", false),
            ("2023-08-27T18:30:10 GPST", false),
            ("2023-08-27T18:45:00 GPST", false),
            ("2023-08-27T18:45:01 GPST", false),
            ("2023-08-27T18:45:05 GPST", false),
            ("2023-08-27T18:45:10 GPST", false),
            ("2023-08-27T19:00:00 GPST", false),
            ("2023-08-27T19:00:01 GPST", false),
            ("2023-08-27T19:00:05 GPST", false),
            ("2023-08-27T19:15:00 GPST", true),
            ("2023-08-27T19:30:00 GPST", true),
            ("2023-08-27T19:45:00 GPST", true),
            ("2023-08-27T20:00:00 GPST", true),
            ("2023-08-27T20:15:00 GPST", true),
            ("2023-08-27T20:30:00 GPST", true),
            ("2023-08-27T20:45:00 GPST", true),
        ] {
            let epoch = Epoch::from_str(epoch_desc).unwrap();
            let interpolated = sp3.interpolate(epoch, sv!("G01"), 9);
            assert_eq!(
                interpolated.is_some(),
                feasible,
                "interpolation feasibility should be : {} for \"{}\"",
                feasible,
                epoch_desc
            );
        }
    }
    #[cfg(feature = "flate2")]
    #[test]
    fn interp() {
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
        //TODO: replace with max_error()
        for (order, max_error) in vec![(9, 1.0E-2_64)] {
            for (epoch, sv, position) in sp3.sv_position() {
                let interpolated = sp3.interpolate(epoch, sv, order);
                if let Some(interpolated) = interpolated {
                    //println!("{} : Truth {:?} Interp {:?}", epoch, position, interpolated);
                    let err = (
                        (interpolated.0 - position.0).abs() * 1.0E3, // error in km
                        (interpolated.1 - position.1).abs() * 1.0E3, // test:
                        (interpolated.2 - position.2).abs() * 1.0E3, // maintain +/- 1mm precision
                    );
                    let epoch_index = sp3
                        .epoch()
                        .enumerate()
                        .filter_map(|(index, e)| if e == epoch { Some(index) } else { None })
                        .reduce(|acc, e| e)
                        .unwrap();
                    let total_epoch = sp3.epoch().count();
                    assert!(
                        err.0 < max_error,
                        "error x too large: {} for Interp({}) @ Epoch {}/{})",
                        err.0,
                        order,
                        epoch_index,
                        total_epoch
                    );
                }
            }
        }
    }
}
