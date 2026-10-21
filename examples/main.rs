use cubacle::{from_moves, random_cube, random_moves, solve};

fn main() {
    println!("=== Testing orbit64: A_VAs734dSkb ===");
    let moves =
        "R' U' F D U R2 U' R2 F2 U L2 R2 F R' B2 D2 R' F2 L F' D' R2 U2 R' U' F".to_string();
    let cube = from_moves(&moves).expect("valid test moves");
    let solution = solve(&cube, 100);
    println!("Solution: {}", solution);
    let solution = solve(&cube, 17);
    println!("Solution: {}", solution);

    println!("=== Superflip ===");
    let superflip_moves = "U R2 F B R B2 R U2 L B2 R U' D' R2 F R' L B2 U2 F2".to_string();
    println!("Scramble: {}", superflip_moves);
    let superflip_cube = from_moves(&superflip_moves).expect("valid superflip moves");
    println!("Facelet:  {}", superflip_cube);
    println!("Solving superflip in <= 20 moves...");
    let superflip_solution = solve(&superflip_cube, 21);
    println!("Solution: {}", superflip_solution);

    println!("\n=== Random Cube ===");
    let cube: String = random_cube();
    println!("Generated: {}", cube);
    println!("Solve it in 21 moves...");
    let solution: String = solve(&cube, 21);
    println!("Result: {}", solution);

    println!("\n=== Random Moves ===");
    let moves: String = random_moves(30);
    println!("Generated: {}", moves);
    println!("Get scrambled cube:");
    let cube: Option<String> = from_moves(&moves);
    if let Some(cube) = cube {
        println!("Generated: {}", cube);
        println!("Solve it in 21 moves...");
        let solution = solve(&cube, 21);
        println!("Result: {}", solution);
    } else {
        println!("Parse scramble error: {}", moves);
    }
}
