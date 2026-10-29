//! Fixed-frame goldens from the pinned Cubistry print; replay uses independent Rodrigues geometry.

use cubacle::engines::two::fixed::{Accept, BuildLimits, Cell, FrameGoal, GoalTable, Rule};
use cubacle::puzzle::frame::{Print, Sticker};
use cubacle::puzzle::{Face, MoveSequence, Turn};

fn print() -> Print {
    let data: serde_json::Value =
        serde_json::from_str(include_str!("fixtures/fixed_frame/blank.print.json")).unwrap();
    let vector =
        |value: &serde_json::Value| std::array::from_fn(|i| value[i].as_i64().unwrap() as i8);
    Print::new(
        data.as_array()
            .unwrap()
            .iter()
            .map(|s| Sticker {
                id: s["id"].as_u64().unwrap() as u8,
                cubie: s["cubie"].as_u64().unwrap() as u8,
                p: vector(&s["p"]),
                n: vector(&s["n"]),
                up: vector(&s["up"]),
                image: s["image"].as_str().map(str::to_owned),
            })
            .collect(),
    )
    .unwrap()
}
fn display(blank: bool) -> Vec<Cell> {
    let mut cells = Vec::new();
    if blank {
        cells.push(Cell {
            p: [-1, 1, 1],
            n: [0, 0, 1],
            accept: vec![Rule {
                accept: Accept::Image("".into()),
                up: None,
            }],
        });
    }
    for (p, image) in [([1, 1, 1], "H"), ([-1, -1, 1], "3"), ([1, -1, 1], "4")] {
        cells.push(Cell {
            p,
            n: [0, 0, 1],
            accept: vec![Rule {
                accept: Accept::Image(image.into()),
                up: Some([0, 1, 0]),
            }],
        });
    }
    cells
}
fn independent_replay(mut stickers: [Sticker; 24], word: &MoveSequence) -> [Sticker; 24] {
    let rotate = |v: [i8; 3], a: [i8; 3]| {
        let dot = (0..3).map(|i| a[i] * v[i]).sum::<i8>();
        let cross = [
            a[1] * v[2] - a[2] * v[1],
            a[2] * v[0] - a[0] * v[2],
            a[0] * v[1] - a[1] * v[0],
        ];
        std::array::from_fn(|i| a[i] * dot - cross[i])
    };
    for mv in word.moves() {
        let axis = match mv.face {
            Face::U => [0, 1, 0],
            Face::R => [1, 0, 0],
            Face::F => [0, 0, 1],
            Face::D => [0, -1, 0],
            Face::L => [-1, 0, 0],
            Face::B => [0, 0, -1],
        };
        for sticker in &mut stickers {
            if (0..3).map(|i| sticker.p[i] * axis[i]).sum::<i8>() == 1 {
                for _ in 0..match mv.turn {
                    Turn::Clockwise => 1,
                    Turn::Half => 2,
                    Turn::CounterClockwise => 3,
                } {
                    sticker.p = rotate(sticker.p, axis);
                    sticker.n = rotate(sticker.n, axis);
                    sticker.up = rotate(sticker.up, axis);
                }
            }
        }
    }
    stickers
}

#[test]
fn golden_three_bodies() {
    let p = print();
    let table = GoalTable::build(
        FrameGoal::new(&p, display(false)).unwrap(),
        BuildLimits::default(),
    )
    .unwrap();
    assert_eq!(table.goal().projection().keys(), 9072);
    assert_eq!(table.goal_keys(), 1);
    assert_eq!(table.histogram(), &[1, 15, 177, 1397, 4981, 2497, 4]);
    assert_eq!(table.eccentricity(), Some(6));
    for word in ["", "R U2 F'", "L2 D B' U"] {
        let source = p
            .apply_moves(&p.reference(), &word.parse().unwrap())
            .unwrap();
        let answer = table.shortest_path(&source).unwrap();
        assert_eq!(answer.path.len(), answer.distance as usize);
        assert_eq!(
            independent_replay(p.stickers(&source).unwrap(), &answer.path),
            p.stickers(&answer.reached).unwrap()
        );
        assert!(table.goal().matches(&answer.reached).unwrap());
    }
}

#[test]
fn golden_blank_print_and_counterexample() {
    let p = print();
    let table = GoalTable::build(
        FrameGoal::new(&p, display(true)).unwrap(),
        BuildLimits::default(),
    )
    .unwrap();
    assert_eq!(table.goal().projection().bodies(), &[1, 3, 6, 7]);
    assert_eq!(table.goal().projection().keys(), 136080);
    assert_eq!(table.goal_keys(), 3);
    assert_eq!(table.histogram(), &[3, 45, 594, 6015, 39849, 79143, 10431]);
    assert_eq!(table.eccentricity(), Some(6));
    assert_eq!(
        table.sha256(),
        Some("0a3f73e6d95731fe778d47d5a8bdd092d925a3514f2936f60557899d38d81c1c")
    );
    let source = p
        .apply_moves(&p.reference(), &"U D2 B D'".parse().unwrap())
        .unwrap();
    let answer = table.shortest_path(&source).unwrap();
    assert_eq!(answer.key, 114083);
    assert_eq!(answer.distance, 6);
    assert_eq!(answer.path, "U B R2 B2 U' L".parse().unwrap());
    assert_eq!(answer.reached.poses(), &[2, 4, 14, 17, 18, 8, 23, 11]);
    assert_eq!(
        independent_replay(p.stickers(&source).unwrap(), &answer.path),
        p.stickers(&answer.reached).unwrap()
    );
    let alternate = "R F' D F2 R' D".parse().unwrap();
    assert!(table
        .goal()
        .matches(&p.apply_moves(&source, &alternate).unwrap())
        .unwrap());
    let reverse = independent_replay(p.stickers(&answer.reached).unwrap(), &answer.path.inverse());
    assert_eq!(reverse, p.stickers(&source).unwrap());
}
