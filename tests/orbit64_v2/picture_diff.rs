//! The Rust side of the picture facelet differential: the answer of the
//! picture reader to mutated facelet strings, one line per case, in the form
//! the Flix reference (`Orbit64.V2.Facelets`, flix-orbit64, Apache-2.0, Werner
//! Stein) writes for the same case.
//!
//! The cases and the answer line come from
//! `tools/orbit64_v2_picture_diff/cases.rs`, which the example
//! `orbit64_v2_picture_diff` uses as well. The test reads the committed sample
//! (`tests/fixtures/orbit64_v2_picture_diff_cases.txt`, 200 cases) and checks
//! what holds without the reference: the sample is what the generator writes
//! for its seed, every case is answered, an accepted string is a token whose
//! decoding reads back as the same cube. The comparison with Flix is
//! `cargo run --release --example orbit64_v2_picture_diff -- run --flix-dir ...`.

#[path = "../../tools/orbit64_v2_picture_diff/cases.rs"]
#[allow(dead_code)]
mod cases;

use cases::{answer, cases_text, read_cases, SAMPLE, SAMPLE_COUNT, SAMPLE_SEED};
use cubacle::notation::orbit64::v2::facelets::{decode, from_facelets};
use cubacle::notation::orbit64::v2::{entry, Model};

#[test]
fn the_sample_is_the_output_of_the_generator() {
    assert!(
        SAMPLE == cases_text(SAMPLE_SEED, SAMPLE_COUNT),
        "regenerate with: cargo run --release --example orbit64_v2_picture_diff -- cases --seed {SAMPLE_SEED} --count {SAMPLE_COUNT}"
    );
}

#[test]
fn the_reader_answers_every_case() {
    let cases = read_cases(SAMPLE);
    assert_eq!(cases.len(), SAMPLE_COUNT);
    let (mut accepted, mut refused) = (0, 0);
    for (index, id, facelets) in cases {
        let line = answer(index, id, facelets);
        let fields: Vec<&str> = line.splitn(4, '|').collect();
        match fields[1] {
            "ok" => {
                accepted += 1;
                // The decoded picture shows the cube that was read.
                let e = entry(id).unwrap();
                let back = from_facelets(e.n, Model::Picture, e.pose, fields[3]).unwrap();
                let read = from_facelets(e.n, Model::Picture, e.pose, facelets).unwrap();
                assert_eq!(decode(fields[2]).unwrap().0.id, id, "{line}");
                assert_eq!(back, read, "{line}");
            }
            _ => refused += 1,
        }
    }
    // The sample holds both outcomes, or it would not test the reader.
    assert!(accepted >= 10 && refused >= 100, "{accepted} {refused}");
}
