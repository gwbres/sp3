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

        let first_epoch = sp3.first_epoch().expect("failed to determine 1st epoch");

        let last_epoch = sp3.last_epoch().expect("failed to determine last epoch");

        let dt = sp3.epoch_interval;
        let total_epochs = sp3.epoch().count();

        //TODO: replace with max_error()
        for (order, max_error) in vec![(7, 1E-1_f64), (9, 1.0E-2_64), (11, 0.5E-3_f64)] {
            let tmin = first_epoch + (order / 2) * dt;
            let tmax = last_epoch - (order / 2) * dt;
            println!("running Interp({}) testbench..", order);
            //DEBUG
            for (index, epoch) in sp3.epoch().enumerate() {
                let feasible = epoch > tmin && epoch <= tmax;
                let interpolated = sp3.sv_position_interpolate(epoch, order as usize);
                let achieved = interpolated.len() > 0;
                //DEBUG
                //println!("tmin: {} | tmax: {} | epoch: {} | feasible : {} | achieved: {}", tmin, tmax, epoch, feasible, achieved);
                if feasible {
                    assert!(
                        achieved == feasible,
                        "interpolation should have been feasible @ epoch {}",
                        epoch,
                    );
                } else {
                    assert!(
                        achieved == feasible,
                        "interpolation should not have been feasible @ epoch {}",
                        epoch,
                    );
                }
                /*
                 * test interpolation errors
                 */
                for (sv, (x, y, z)) in interpolated {
                    let truth = sp3
                        .sv_position()
                        .find(|(t, svnn, (sv_x, sv_y, sv_z))| {
                            *svnn == sv && *t == epoch && x == *sv_x && y == *sv_y && z == *sv_z
                        })
                        .unwrap(); // infaillible
                    let err = (
                        (x - truth.2 .0).abs() * 1.0E3, // error in km
                        (y - truth.2 .1).abs() * 1.0E3,
                        (z - truth.2 .2).abs() * 1.0E3,
                    );
                    assert!(
                        err.0 < max_error,
                        "x error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                        err.0,
                        order,
                        sv,
                        index,
                        total_epochs,
                    );
                    assert!(
                        err.1 < max_error,
                        "y error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                        err.1,
                        order,
                        sv,
                        index,
                        total_epochs,
                    );
                    assert!(
                        err.2 < max_error,
                        "z error too large: {} for Interp({}) for {} @ Epoch {}/{}",
                        err.2,
                        order,
                        sv,
                        index,
                        total_epochs,
                    );
                }
            }
        }
    }
}
