//! `py_float_repr` against CPython's `repr(float)` on the vectors `fixtures/py_float_repr.py`
//! writes: round-half-even ties and their neighbours, random bit patterns, subnormals,
//! integers, and every power of ten and of two. Each line is both the input (Python's repr
//! round-trips) and the expected text.
use flate2::read::GzDecoder;
use gai_core::py_float_repr;
use std::io::Read;

#[test]
fn float_repr_matches_cpython() {
    let mut text = String::new();
    GzDecoder::new(&include_bytes!("fixtures/py-float-repr.txt.gz")[..]).read_to_string(&mut text).unwrap();
    let lines: Vec<&str> = text.lines().collect();
    let bad: Vec<(&str, String)> = lines
        .iter()
        .map(|line| (*line, py_float_repr(line.parse().unwrap())))
        .filter(|(line, actual)| line != actual)
        .collect();
    assert_eq!(lines.len(), 29352, "fixture lines");
    assert!(bad.is_empty(), "{} of {} differ, e.g. {:?}", bad.len(), lines.len(), &bad[..bad.len().min(5)]);
}
