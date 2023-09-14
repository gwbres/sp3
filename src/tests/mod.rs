mod interpolation;
mod merge;
mod parser_3c;
mod parser_3d;
mod test_pool;

use crate::SP3;

fn print_or_panic(field: &str, a: String, b: String, panic: bool) {
    if panic {
        panic!("{} - A: {} B: {}", field, a, b);
    } else {
        println!("{}- A: {} B: {}", field, a, b);
    }
}

pub(crate) fn test_equality(file_a: SP3, file_b: SP3, panic: bool) {
    if file_a != file_b {
        println!("file a & b differ!");
        if file_a.version != file_b.version {
            print_or_panic(
                "version",
                file_a.version.to_string(),
                file_b.version.to_string(),
                panic,
            );
        }
        if file_a.data_used != file_b.data_used {
            print_or_panic(
                "data used",
                file_a.data_used.to_string(),
                file_b.data_used.to_string(),
                panic,
            );
        }
        if file_a.coord_system != file_b.coord_system {
            print_or_panic(
                "coordinates system",
                file_a.coord_system.to_string(),
                file_b.coord_system.to_string(),
                panic,
            );
        }
        if file_a.agency != file_b.agency {
            print_or_panic(
                "agency",
                file_a.agency.clone(),
                file_b.agency.clone(),
                panic,
            );
        }
        if file_a.constellation != file_b.constellation {
            print_or_panic(
                "constellation",
                file_a.constellation.to_string(),
                file_b.constellation.to_string(),
                panic,
            );
        }
        if file_a.time_system != file_b.time_system {
            print_or_panic(
                "time scale",
                file_a.time_system.to_string(),
                file_b.time_system.to_string(),
                panic,
            );
        }
        if file_a.week_counter != file_b.week_counter {
            print_or_panic(
                "week counter",
                format!("{:?}", file_a.week_counter),
                format!("{:?}", file_b.week_counter),
                panic,
            );
        }
        if file_a.mjd_start != file_b.mjd_start {
            print_or_panic(
                "mjd",
                format!("{:?}", file_a.mjd_start),
                format!("{:?}", file_b.mjd_start),
                panic,
            );
        }
        if file_a.epoch_interval != file_b.epoch_interval {
            print_or_panic(
                "duration",
                file_a.epoch_interval.to_string(),
                file_b.epoch_interval.to_string(),
                panic,
            );
        }
        for e_a in &file_a.epoch {
            if !file_b.epoch.contains(&e_a) {
                if panic {
                    panic!("B.epoch is missing {}", e_a);
                } else {
                    println!("B.epoch is missing {}", e_a);
                }
            }
        }
        for e_b in &file_b.epoch {
            if !file_a.epoch.contains(&e_b) {
                if panic {
                    panic!("B.epoch contains {}, but it should not", e_b);
                } else {
                    println!("B.epoch contains {}, but it should not", e_b);
                }
            }
        }
        for sv_a in file_a.sv() {
            assert!(
                file_b.sv().find(|sv| *sv == sv_a).is_some(),
                "b.sv() is missing {}",
                sv_a
            );
        }
        for sv_b in file_b.sv() {
            assert!(
                file_b.sv().find(|sv| *sv == sv_b).is_none(),
                "b.sv() contains {}, but it should not",
                sv_b
            );
        }
        for (epoch, svnn, (pos_x, pos_y, pos_z)) in file_a.sv_position() {
            assert!(
                file_b
                    .sv_position()
                    .find(|(e, sv, pos)| {
                        *e == epoch && *sv == svnn && *pos == (pos_x, pos_y, pos_z)
                    })
                    .is_some(),
                "a.sv_position() is missing {:?} for {} {}",
                (pos_x, pos_y, pos_z),
                svnn,
                epoch
            );
        }
        for (epoch, svnn, (pos_x, pos_y, pos_z)) in file_b.sv_position() {
            assert!(
                file_a
                    .sv_position()
                    .find(|(e, sv, pos)| {
                        *e == epoch && *sv == svnn && *pos == (pos_x, pos_y, pos_z)
                    })
                    .is_none(),
                "b.sv_position() contains {:?} for {} {}, but it should not",
                (pos_x, pos_y, pos_z),
                svnn,
                epoch
            );
        }
        for comment in file_a.comments() {
            assert!(
                file_b.comments().find(|c| *c == comment).is_some(),
                "b.comments is missing \"{}\"",
                comment
            );
        }
        for comment in file_b.comments() {
            assert!(
                file_a.comments().find(|c| *c == comment).is_none(),
                "b.comments contains \"{}\", but is should not",
                comment
            );
        }
    }
}
