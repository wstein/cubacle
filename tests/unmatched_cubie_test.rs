use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Face, Facelets, Failure, PuzzleKind, SolveOptions, Solver};

const EXAMPLES: [&str; 3] = [
    "ULUDUULRLDFRFRLUBLFBBDFRDBFFLRFDUBUFLRDULBBDRDDBFBLURR",
    // U9 <-> R1 on the solved cube
    "UUUUUUUURURRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB",
    // U1 <-> L1 on the solved cube
    "LUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDULLLLLLLLBBBBBBBBB",
];

struct XorShift(u64);

impl XorShift {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[test]
fn unmatched_pieces_are_invalid_input_with_a_sentence() {
    let solver = Min2PhaseSolver::shared();
    for text in EXAMPLES {
        let cube = Facelets::parse(text).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        assert_eq!(result.outcome, Err(Failure::InvalidInput), "{text}");
        let message = result.message.expect("a message");
        assert!(message.starts_with("the "), "{message}");
        assert!(!message.contains("Error"), "{message}");
    }
}

#[test]
fn swapped_stickers_never_give_a_wrong_solution() {
    let solver = Min2PhaseSolver::shared();
    let moves = ["U", "D", "L", "R", "F", "B"];
    let mut rng = XorShift(0x9e37_79b9_7f4a_7c15);
    let (mut ok, mut invalid) = (0, 0);
    for _ in 0..3000 {
        let scramble: Vec<String> = (0..25)
            .map(|_| moves[(rng.next() % 6) as usize].to_owned())
            .collect();
        let valid = Facelets::solved(PuzzleKind::Cube3)
            .apply(&scramble.join(" ").parse().unwrap())
            .unwrap();
        // The unchanged cube still solves.
        let result = solver.solve(&valid, &SolveOptions::default());
        assert!(valid.apply(result.solution().unwrap()).unwrap().is_solved());

        let mut stickers: Vec<Face> = valid.stickers().to_vec();
        let (a, b) = ((rng.next() % 54) as usize, (rng.next() % 54) as usize);
        stickers.swap(a, b);
        let cube = Facelets::from_stickers(stickers).unwrap();
        let result = solver.solve(&cube, &SolveOptions::default());
        match result.solution() {
            Some(solution) => {
                assert!(cube.apply(solution).unwrap().is_solved(), "{cube}");
                ok += 1;
            }
            None => {
                assert_eq!(result.outcome, Err(Failure::InvalidInput), "{cube}");
                invalid += 1;
            }
        }
    }
    println!("ok {ok}, invalid {invalid}");
}
