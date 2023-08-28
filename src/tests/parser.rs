//! parser dedicated tests

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;
    use std::str::FromStr;
    #[test]
    fn test_sp3_d() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("sp3d.txt");
        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(
            sp3.is_ok(),
            "failed to parse data/sp3d.txt: {:?}",
            sp3.err()
        );

        let sp3 = sp3.unwrap();

        /*
         * Test general infos
         */
        assert_eq!(sp3.version, Version::D);
        assert_eq!(sp3.data_type, DataType::Position);
        assert_eq!(
            sp3.start_epoch,
            Epoch::from_str("2019-10-27T00:00:00 UTC").unwrap()
        );
        assert_eq!(sp3.nb_epochs, 288, "bad number of epochs");
        assert_eq!(sp3.coord_system, "IGS14");
        assert_eq!(sp3.orbit_type, OrbitType::FIT);
        assert_eq!(sp3.agency, "IGS");
        assert_eq!(sp3.week_counter, (2077, 0.0_f64));
        assert_eq!(sp3.epoch_interval, Duration::from_seconds(300.0_f64));
        assert_eq!(sp3.mjd_start, (58783, 0.0_f64));

        let position: Vec<_> = sp3.sv_position().collect();
        println!("{:?}", position);

        let mut clk: Vec<_> = sp3.sv_clock().collect();
        println!("{:?}", clk);

        /*
         * Test file comments
         */
        assert_eq!(
            sp3.comments.len(),
            4,
            "failed to parse files comment correctly"
        );
        assert_eq!(
            sp3.comments,
            vec![
                "PCV:IGS14_2074 OL/AL:FES2004  NONE     YN CLK:CoN ORB:CoN",
                "THIS EXAMPLE OF SP3 FILE IS PART OF THE gLAB TOOL SUITE",
                "FILE PREPARED BY: MOWEN LI",
                "PLEASE EMAIL ANY COMMENT OR REQUEST TO glab.gage @upc.edu",
            ],
        );
    }
}
