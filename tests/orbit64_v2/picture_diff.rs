//! The Rust side of `tools/orbit64_v2_picture_diff.py`: the answer of the
//! picture reader to mutated facelet strings, one line per case, in the form
//! the Flix reference (`Orbit64.V2.Facelets`, flix-orbit64, Apache-2.0, Werner
//! Stein) writes for the same case.
//!
//! Without environment variables the test reads the committed sample
//! (`tests/fixtures/orbit64_v2_picture_diff_cases.txt`, 200 cases) and checks
//! what holds without the reference: every case is answered, an accepted string
//! is a token whose decoding reads back as the same cube. The comparison with
//! Flix is `tools/orbit64_v2_picture_diff.py run --flix-dir ...`, which sets
//! `ORBIT64_PICTURE_DIFF_CASES` and `ORBIT64_PICTURE_DIFF_OUT`.

use cubacle::notation::orbit64::v2::facelets::{decode, encode, from_facelets};
use cubacle::notation::orbit64::v2::{entry, Model};

const SAMPLE: &str = include_str!("../fixtures/orbit64_v2_picture_diff_cases.txt");

/// `index|ok|token|facelets` or `index|err|text`.
fn answer(index: &str, id: usize, text: &str) -> String {
    match encode(id, text) {
        Err(why) => format!("{index}|err|{why}"),
        Ok(token) => match decode(&token) {
            Ok((_, back)) => format!("{index}|ok|{token}|{back}"),
            Err(why) => format!("{index}|err|{why}"),
        },
    }
}

fn cases(text: &str) -> Vec<(&str, usize, &str)> {
    text.lines()
        .filter(|line| !line.starts_with('#') && !line.is_empty())
        .map(|line| {
            let mut f = line.splitn(3, '|');
            let (index, id, facelets) = (f.next().unwrap(), f.next().unwrap(), f.next().unwrap());
            (index, id.parse().unwrap(), facelets)
        })
        .collect()
}

#[test]
fn the_reader_answers_every_case() {
    let external = std::env::var("ORBIT64_PICTURE_DIFF_CASES").ok();
    let text = match &external {
        Some(path) => std::fs::read_to_string(path).unwrap(),
        None => SAMPLE.to_string(),
    };
    let cases = cases(&text);
    assert!(!cases.is_empty());
    let mut out = String::new();
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
        out.push_str(&line);
        out.push('\n');
    }
    match std::env::var("ORBIT64_PICTURE_DIFF_OUT") {
        Ok(path) => std::fs::write(path, out).unwrap(),
        // The sample holds both outcomes, or it would not test the reader.
        Err(_) => assert!(accepted >= 10 && refused >= 100, "{accepted} {refused}"),
    }
}
