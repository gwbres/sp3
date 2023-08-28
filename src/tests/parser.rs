//! parser dedicated tests

#[cfg(test)]
mod test {
    use crate::prelude::SP3;
    use std::path::PathBuf;
    #[test]
    fn test_sp3_d() {
        let path = PathBuf::new()
            .join(env!("CARGO_MANIFEST_DIR"))
            .join("data")
            .join("sp3d.txt");
        let sp3 = SP3::from_file(&path.to_string_lossy());
        assert!(sp3.is_ok(), "failed to parse data/sp3d.txt");

        let sp3 = sp3.unwrap();
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
