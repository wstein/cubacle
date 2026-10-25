use super::*;

/// Variables a1 a2 a3 b1 b2 b3 c1 c2 c3, in this order.
fn names() -> Vec<String> {
    ["a", "b", "c"]
        .iter()
        .flat_map(|letter| (1..=3).map(move |i| format!("{letter}{i}")))
        .collect()
}

fn holds(guard: &str, colors: [u8; 9]) -> bool {
    Guard::parse(guard, &names())
        .unwrap_or_else(|e| panic!("{guard:?}: {e}"))
        .holds(&colors)
}

fn error(guard: &str) -> String {
    Guard::parse(guard, &names()).unwrap_err().to_string()
}

#[test]
fn blank_guard_always_holds() {
    assert!(holds("", [0; 9]));
    assert!(holds("  \n ", [0; 9]));
}

#[test]
fn unique_equal_and_count() {
    let colors = [0, 0, 0, 1, 1, 1, 2, 2, 2];
    assert!(holds("unique(a1, b1, c1)", colors));
    assert!(!holds("unique(a1, a2)", colors));
    assert!(holds("equal(a1, a2, a3)", colors));
    assert!(!holds("equal(a1, b1)", colors));
    assert!(holds("count(a1, a2, b1) == 2", colors));
    assert!(holds("count(a1, b1, c1) >= 3", colors));
    assert!(holds("count(a1, a2) < 2", colors));
    assert!(holds("count(a1, a2) <= 1 && 1 <= count(c1)", colors));
}

#[test]
fn bare_variables_are_operands() {
    let colors = [0, 1, 2, 3, 3, 3, 5, 5, 5];
    assert!(holds("a1 != a2", colors));
    assert!(holds("a1 < a2 && a2 < a3", colors));
    assert!(holds("b1 == b2", colors));
    assert!(holds("c1 > 4", colors));
}

#[test]
fn operator_spellings_and_precedence() {
    let colors = [0, 1, 2, 3, 3, 3, 5, 5, 5];
    for guard in [
        "a1 == 9 || a2 == 1 && a3 == 2",
        "a1 == 9 OR a2 == 1 AND a3 == 2",
        "a1 == 9 or a2 == 1 and a3 == 2",
        "NOT a1 == 9 && a2 == 1",
        "not (a1 == 9) && a2 == 1",
        "!(a1 == 9) && a2 == 1",
        "! ! a1 == 0",
    ] {
        assert!(holds(guard, colors), "{guard}");
    }
    assert!(!holds("(a1 == 9 || a2 == 1) && a3 == 9", colors));
    assert!(!holds("a1 == 9 || a2 == 9 && a3 == 2", colors));
}

#[test]
fn star_is_a_conjunction_over_the_indices() {
    // Index 1 has a, b and c differing; index 3 does not.
    let good = [0, 4, 5, 1, 1, 1, 2, 2, 2];
    assert!(holds("unique(a*, b*, c*)", good));
    let bad = [0, 4, 5, 1, 1, 5, 2, 2, 2];
    assert!(!holds("unique(a*, b*, c*)", bad));
    // Equal values at different indices are allowed: a is not unique to itself.
    assert!(holds("unique(a*, b*)", good));
    assert!(holds("equal(a*, b3)", [1, 1, 1, 0, 0, 1, 0, 0, 0]));
    assert!(!holds("equal(a*, b3)", [1, 1, 0, 0, 0, 1, 0, 0, 0]));
    assert!(holds("count(a*, b*, c*) == 3", good));
    assert!(!holds("count(a*, b*, c*) == 3", bad));
    assert!(holds("a* != b*", good));
}

#[test]
fn star_skips_arguments_without_that_index() {
    let names: Vec<String> = ["a1", "a2", "b1"].map(String::from).to_vec();
    let guard = Guard::parse("unique(a*, b*)", &names).unwrap();
    assert!(guard.holds(&[0, 0, 1]));
    assert!(!guard.holds(&[1, 0, 1]));
}

#[test]
fn errors_name_line_column_and_expectation() {
    assert_eq!(
        error("unique(a1, b1"),
        "line 1, column 14: expected \")\" or \",\", found the end of the guard"
    );
    assert_eq!(
        error("unique(a1 b1)"),
        "line 1, column 11: expected \")\" or \",\", found \"b1\""
    );
    assert_eq!(
        error("a1 == "),
        "line 1, column 7: expected a variable, a number, a call or \"(\", found the end of the guard"
    );
    assert_eq!(
        error("unique(z9, a1)"),
        "line 1, column 8: variable \"z9\" does not occur in the schema"
    );
    assert_eq!(
        error("unique(z*)"),
        "line 1, column 8: no variable of the schema matches \"z*\""
    );
    assert_eq!(
        error("a1 == a2\n && (b1 == b2"),
        "line 2, column 14: expected \")\", found the end of the guard"
    );
    assert_eq!(
        error("a1 == a2 b1"),
        "line 1, column 10: expected \"&&\", \"||\" or the end of the guard, found \"b1\""
    );
    assert_eq!(
        error("a1"),
        "line 1, column 3: expected a comparison (\"==\", \"!=\", \"<\", \"<=\", \">\" or \">=\"), found the end of the guard"
    );
    assert_eq!(
        error("unique(a1) == 1"),
        "line 1, column 12: unique(...), equal(...) and parenthesised conditions cannot be compared"
    );
    assert_eq!(
        error("unique()"),
        "line 1, column 8: expected a variable, found \")\""
    );
    assert_eq!(
        error("count"),
        "line 1, column 6: expected \"(\", found the end of the guard"
    );
}

#[test]
fn deep_nesting_is_refused_without_overflowing_the_stack() {
    let deep = format!("{}a1 == a2{}", "(".repeat(5000), ")".repeat(5000));
    assert!(error(&deep).contains("nests deeper than 64 levels"));
    let nots = format!("{}a1 == a2", "!".repeat(5000));
    assert!(error(&nots).contains("nests deeper than 64 levels"));
}

#[test]
fn expansion_is_limited() {
    let names: Vec<String> = (1..=300).map(|i| format!("a{i}")).collect();
    let args = vec!["a*"; 400].join(", ");
    let err = Guard::parse(&format!("unique({args})"), &names).unwrap_err();
    assert!(
        err.message().contains("expands to more than 100000"),
        "{err}"
    );
}

#[test]
fn long_guards_are_refused() {
    let text = "a1 == a1 && ".repeat(10_000);
    assert!(error(&text).contains("longer than 100000 characters"));
}
