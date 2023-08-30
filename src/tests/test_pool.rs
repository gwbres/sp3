#[cfg(test)]
mod test {
    use crate::prelude::*;
    use std::path::PathBuf;
    #[test]
    fn data_folder() {
        let prefix = PathBuf::new().join(env!("CARGO_MANIFEST_DIR")).join("data");

        for file in vec![
            "EMR0OPSULT_20232391800_02D_15M_ORB.SP3",
            "ESA0OPSULT_20232320600_02D_15M_ORB.SP3",
            "COD0MGXFIN_20230500000_01D_05M_ORB.SP3",
            "ESA0OPSRAP_20232390000_01D_15M_ORB.SP3",
            "co108870.sp3",
            "em108871.sp3",
            //"emr08874.sp3",
            //"sio06492.sp3",
            "sp3d.txt",
        ] {
            let file_path = prefix.clone().join(file);
            println!("Parsing file \"{}\"", file_path.to_string_lossy());
            let sp3 = SP3::from_file(&file_path.to_string_lossy());
            assert!(
                sp3.is_ok(),
                "failed to parse data/{}, error: {:?}",
                file,
                sp3.err()
            );
        }
    }
}
