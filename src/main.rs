use std::time::Instant;

use bigbrain::table::{Output, Table};

fn main() {
    let table: Table<3> = Table {
        outputs: [
            Output::One,
            Output::One,
            Output::One,
            Output::Zero,
            Output::Zero,
            Output::One,
            Output::One,
            Output::One,
        ],
    };

    let instant = Instant::now();

    let implicants = table.minimize();

    println!("took: {}us", instant.elapsed().as_micros());

    for implicant in implicants {
        println!("{implicant}");
    }
}
