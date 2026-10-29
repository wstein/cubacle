use std::io::{BufReader, Cursor};

use super::{parse, read_bounded_line, read_line};

#[test]
fn duplicate_keys_at_every_depth_are_rejected() {
    for input in [
        r#"{"a":1,"a":2}"#,
        r#"{"a":{"b":1,"b":2}}"#,
        r#"[{"a":1,"\u0061":2}]"#,
        r#"{"extra":{"ignored":1,"ignored":2}}"#,
    ] {
        assert!(parse(input.as_bytes()).unwrap_err().contains("duplicate"));
    }
    assert!(parse(br#"{"a":{"key":1},"b":{"key":2}}"#).is_ok());
}

#[test]
fn integer_lexemes_keep_the_full_u64_range() {
    let value = parse(br#"[-9223372036854775808,18446744073709551615,-0]"#).unwrap();
    assert_eq!(value[0].as_i64(), Some(i64::MIN));
    assert_eq!(value[1].as_u64(), Some(u64::MAX));
    assert_eq!(value[2].as_i64(), Some(0));
    for input in [
        "1.0",
        "1e0",
        "0E+0",
        "NaN",
        "Infinity",
        "18446744073709551616",
    ] {
        assert!(parse(input.as_bytes()).is_err(), "accepted {input}");
    }
}

#[test]
fn numeric_token_limit_is_checked_outside_strings() {
    assert!(parse(format!("[{}]", "9".repeat(129)).as_bytes())
        .unwrap_err()
        .contains("128 bytes"));
    // A huge but syntactically bounded number fails range validation, not the token bound.
    assert!(!parse("9".repeat(128).as_bytes())
        .unwrap_err()
        .contains("128 bytes"));
    assert!(parse(format!(r#"{{"text":"{}1.0\\\"2e0"}}"#, "9".repeat(200)).as_bytes()).is_ok());
}

#[test]
fn depth_64_passes_and_depth_65_fails() {
    let nested = |depth| format!("{}0{}", "[".repeat(depth), "]".repeat(depth));
    assert!(parse(nested(64).as_bytes()).is_ok());
    assert!(parse(nested(65).as_bytes())
        .unwrap_err()
        .contains("64 containers"));
}

#[test]
fn malformed_utf8_surrogates_bom_and_trailing_values_fail() {
    for input in [
        &b"{\"bad\":\"\xff\"}"[..],
        br#"{"s":"\ud800"}"#,
        br#"{"s":"\udc00"}"#,
        b"\xef\xbb\xbf{}",
        b"{} {}",
        b"{\"x\":01}",
        b"{\"x\":+1}",
    ] {
        assert!(parse(input).is_err(), "accepted {input:?}");
    }
    assert_eq!(parse(br#""\ud83d\ude00""#).unwrap().as_str(), Some("😀"));
}

#[test]
fn line_reader_enforces_objects_and_lf_and_reports_eof() {
    let mut input = Cursor::new(b"{}\n{\"n\":2}\n");
    assert!(read_line(&mut input).unwrap().unwrap().is_object());
    assert_eq!(read_line(&mut input).unwrap().unwrap()["n"], 2);
    assert!(read_line(&mut input).unwrap().is_none());
    for bad in [&b"{}"[..], b"{}\r\n", b"\n", b"[]\n", b"null\n"] {
        assert!(
            read_line(&mut Cursor::new(bad)).is_err(),
            "accepted {bad:?}"
        );
    }
}

#[test]
fn oversized_lines_are_bounded_and_drained_before_continuing() {
    let mut input = BufReader::with_capacity(3, Cursor::new(b"{\"x\":12345}\n{}\n"));
    assert!(read_bounded_line(&mut input, 8)
        .unwrap_err()
        .contains("8 bytes"));
    assert!(read_bounded_line(&mut input, 8)
        .unwrap()
        .unwrap()
        .is_object());
    assert!(read_bounded_line(&mut Cursor::new(b"{}\n"), 2).is_ok());
    assert!(read_bounded_line(&mut Cursor::new(b"{ }\n"), 2).is_err());
}
