//! parser dedicated tests

#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::str::FromStr;
    use std::path::PathBuf;
    #[test]
    fn test_sp3_d() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("sp3d.txt");
        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(sp3.is_ok(), "failed to parse data/sp3d.txt: {:?}", sp3);

        let sp3 = sp3.unwrap();

        /*
         * Test general infos
         */
        assert_eq!(sp3.version, Version::D);
        assert_eq!(sp3.data_type, DataType::Position);
        assert_eq!(sp3.start_epoch, Epoch::from_str("2019-10-27T00:00:00 UTC").unwrap());

        /*
         * Test file comments
         */
        assert_eq!(sp3.comments.len(), 4, "failed to parse files comment correctly");
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
