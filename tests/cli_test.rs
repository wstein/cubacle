use std::process::Command;

const SOLUTION: &str = "B R L B L' U' D B D2 R2 F2 R2 L2 D R2 L2 U' L2 U' F2 R";

fn cubacle(args: &[&str]) -> (bool, String, String) {
    let output = Command::new(env!("CARGO_BIN_EXE_cubacle"))
        .args(args)
        .output()
        .expect("cubacle runs");
    (
        output.status.success(),
        String::from_utf8_lossy(&output.stdout).trim().to_owned(),
        String::from_utf8_lossy(&output.stderr).trim().to_owned(),
    )
}

#[test]
fn every_notation_of_one_cube_gets_the_same_solution() {
    let notations: [&[&str]; 5] = [
        &["A4ib8aejxj4Y"],
        &["FUFUUFFFFUUUURRURURRRFFRRFRBDBBDDBBBDLDDLLDDDLBLBBLLLL"],
        &[
            "FUFUUFFFF",
            "UUUURRURU",
            "RRRFFRRFR",
            "BDBBDDBBB",
            "DLDDLLDDD",
            "LBLBBLLLL",
        ],
        &["GWGWWGGGGWWWWRRWRWRRRGGRRGRBYBBYYBBBYOYYOOYYYOBOBBOOOO"],
        &[
            "GWGWWGGGG",
            "WWWWRRWRW",
            "RRRGGRRGR",
            "BYBBYYBBB",
            "YOYYOOYYY",
            "OBOBBOOOO",
        ],
    ];
    for cube in notations {
        let mut args = vec!["solve"];
        args.extend(cube);
        let (ok, solution, error) = cubacle(&args);
        assert!(ok, "{cube:?}: {error}");
        assert_eq!(solution, SOLUTION, "{cube:?}");
    }
}

#[test]
fn convert_reads_the_japanese_scheme() {
    let japanese = "WWWWWWWWWRRRRRRRRRGGGGGGGGGBBBBBBBBBOOOOOOOOOYYYYYYYYY";
    let solved = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
    assert_eq!(cubacle(&["convert", japanese]).1, solved);
    assert_eq!(
        cubacle(&["convert", "--scheme", "japanese-bl-wt", japanese]).1,
        solved
    );
    let (ok, _, error) = cubacle(&["convert", "--scheme", "western-boy", japanese]);
    assert!(!ok);
    assert!(error.contains("centres"), "{error}");
}

#[test]
fn unsupported_sizes_say_so() {
    let (ok, _, error) = cubacle(&["solve", "ASj79"]);
    assert!(!ok);
    assert!(error.contains("no engine for the 2x2x2"), "{error}");
}

#[test]
fn a_tight_move_limit_fails_quickly() {
    let (ok, _, error) = cubacle(&["solve", "-m", "3", "A4ib8aejxj4Y"]);
    assert!(!ok);
    assert!(error.contains("move limit"), "{error}");
}
