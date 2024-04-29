use bigbrain::table::{Output, Table};

fn main() {
    let a_segment: Table<4> = Table {
        outputs: [
            Output::One,
            Output::Zero,
            Output::One,
            Output::One,
            Output::Zero,
            Output::One,
            Output::One,
            Output::One,
            Output::One,
            Output::One,
            Output::DontCare,
            Output::DontCare,
            Output::DontCare,
            Output::DontCare,
            Output::DontCare,
            Output::DontCare,
        ],
    };

    for implicant in a_segment.minimize() {
        println!("{implicant}");
    }
}
