//! A fixed pilot series for `recover`. `generate` writes the data file once:
//! `cargo run --release --example recover_pilot -- generate > tests/fixtures/recover_pilot.txt`.

mod gen;
mod rng;
mod solvers;

fn main() {
    if std::env::args().nth(1).as_deref() == Some("generate") {
        gen::generate();
    } else {
        eprintln!("usage: recover_pilot generate > tests/fixtures/recover_pilot.txt");
    }
}
