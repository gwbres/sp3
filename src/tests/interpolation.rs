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
        let mut fact = (order + 1) as f64;
        for i in 0..order {
            fact *= i as f64;
        }
        q.abs() / fact // TODO f^(n+1)[x]
    }
    #[cfg(feature = "flate2")]
    #[test]
    fn interp_5() {
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
        for (epoch_desc, feasible, epoch_exists) in vec![
            ("2023-08-27T18:00:05 GPST", false, false),
            ("2023-08-27T18:00:15 GPST", false, false),
            ("2023-08-27T18:00:30 GPST", false, false),
            ("2023-08-27T18:00:45 GPST", false, false),
            ("2023-08-27T18:15:00 GPST", false, false),
            ("2023-08-27T18:30:00 GPST", true, false),
            ("2023-08-27T18:30:10 GPST", true, false),
            ("2023-08-27T18:45:00 GPST", true, false),
            ("2023-08-27T19:00:00 GPST", true, false),
            ("2023-08-27T19:15:00 GPST", true, true),
            ("2023-08-27T19:30:00 GPST", true, true),
            ("2023-08-27T19:45:00 GPST", true, true),
            ("2023-08-27T20:00:00 GPST", true, true),
        ] {
            let epoch = Epoch::from_str(epoch_desc).unwrap();
            let position = sp3.interpolate(e, sv!("G01"), 5);
            assert_eq!(
                position.is_some(),
                feasible,
                "failed for \"{}\"",
                epoch_desc
            );
            if feasible {
                let vector3d = position.unwrap();
                if epoch_exists {
                    let position = sp3
                        .sv_position()
                        .filter_map(|(e, svnn, position)| {
                            if e == epoch && svnn == sv!("G01") {
                                Some(position)
                            } else {
                                None
                            }
                        })
                        .unique()
                        .collect();
                    let err = (
                        (position.0 - vector3d.0).abs(),
                        (position.1 - vector3d.2).abs(),
                        (position.2 - vector3d.1).abs(),
                    );
                    // error in km,
                    // maintain +/- 1mm precision
                    //TODO: use max_error() here
                    assert!(err.0 * 1.0E3 < 1.0E-3, "error x too large: {}", err.0);
                    assert!(err.1 * 1.0E3 < 1.0E-3, "error y too large: {}", err.1);
                    assert!(err.2 * 1.0E3 < 1.0E-3, "error z too large: {}", err.2);
                }
            }
        }
    }
}
