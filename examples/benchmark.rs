use cubacle::engines::three::Min2PhaseSolver;
use cubacle::puzzle::{Facelets, MoveSequence, PuzzleKind, SolveOptions, SolveResult, Solver};
use std::env;
use std::io;
use std::time::{Duration, Instant};

const MAX_LENGTH: u8 = 20;
const SCRAMBLES: [&str; 8] = [
    "R U R' U' F2 D B2 L2 U2 R2 F' D2",
    "F R U R' U' F' L U2 L' B2 D R2 F2 U' L2 D' B",
    "U2 R2 F2 D2 L2 B2 U R' F D' L B' U2 R F2 D",
    "L2 D R2 U' B2 L F' U2 R D2 F B' L2 U R' D",
    "B U2 L' D2 R F2 U' B2 L D R' F U2 L2 D' B'",
    "R2 F U2 L' B D2 R U F2 L2 D' B' R2 U' F L",
    "D2 B2 R' U F2 L D' R2 U2 B L' F2 D R U' B'",
    "L U2 F' R2 D B2 L' U R F2 D' B R2 U' L2 F",
];

fn percentile(sorted: &[Duration], numerator: usize, denominator: usize) -> Duration {
    let index = (sorted.len() * numerator)
        .div_ceil(denominator)
        .saturating_sub(1);
    sorted[index]
}

fn validate_solution(cube: &Facelets, result: &SolveResult) -> io::Result<()> {
    let invalid = |message| io::Error::new(io::ErrorKind::InvalidData, message);
    let solution = result
        .solution()
        .ok_or_else(|| invalid("solver found no solution"))?;
    let solved = cube
        .apply(solution)
        .map_err(|_| invalid("solver returned invalid moves"))?;
    if !solved.is_solved() {
        return Err(invalid("solver result did not solve the cube"));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let mut parallel = false;
    let mut iteration_arg = None;
    for argument in env::args().skip(1) {
        if argument == "--parallel" {
            parallel = true;
        } else if iteration_arg.replace(argument).is_some() {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "usage: benchmark [iterations] [--parallel]",
            ));
        }
    }
    let iterations = match iteration_arg {
        Some(value) => value.parse::<u32>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "iterations must be a positive integer",
            )
        })?,
        None => 10,
    };
    if iterations == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "iterations must be a positive integer",
        ));
    }

    let invalid = |message| io::Error::new(io::ErrorKind::InvalidData, message);
    let cubes: Vec<Facelets> = SCRAMBLES
        .iter()
        .map(|scramble| {
            let moves: MoveSequence = scramble
                .parse()
                .map_err(|_| invalid("benchmark scramble is invalid"))?;
            Facelets::solved(PuzzleKind::Cube3)
                .apply(&moves)
                .map_err(|_| invalid("benchmark scramble is invalid"))
        })
        .collect::<io::Result<_>>()?;
    let solver = Min2PhaseSolver::new();
    let options = SolveOptions::default().with_max_moves(MAX_LENGTH);
    let solve = |cube: &Facelets| {
        if parallel {
            solver.solve_parallel(cube, &options)
        } else {
            solver.solve(cube, &options)
        }
    };

    for cube in &cubes {
        validate_solution(cube, &solve(cube))?;
    }

    let mut durations = Vec::with_capacity(cubes.len() * iterations as usize);
    let mut total_probes = 0_u64;
    for _ in 0..iterations {
        for cube in &cubes {
            let started = Instant::now();
            let result = solve(cube);
            durations.push(started.elapsed());
            validate_solution(cube, &result)?;
            total_probes += result.metrics.probes.unwrap_or(0);
        }
    }

    durations.sort_unstable();
    let total = durations.len();
    let total_duration: Duration = durations.iter().sum();
    let total_seconds = total_duration.as_secs_f64();
    println!(
        "Validated {total} {} solves from {} fixed scrambles at a {MAX_LENGTH}-move limit after warm-up.",
        if parallel { "parallel" } else { "serial" },
        SCRAMBLES.len()
    );
    println!("Total solver time: {:.3} s", total_seconds);
    println!(
        "Average solve time: {:.3} ms",
        total_seconds * 1_000.0 / total as f64
    );
    println!(
        "Median solve time:  {:.3} ms",
        percentile(&durations, 50, 100).as_secs_f64() * 1_000.0
    );
    println!(
        "P95 solve time:     {:.3} ms",
        percentile(&durations, 95, 100).as_secs_f64() * 1_000.0
    );
    println!("Total probes:       {total_probes}");
    println!(
        "Probes per second:  {:.1}",
        total_probes as f64 / total_seconds
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::percentile;
    use std::time::Duration;

    #[test]
    fn percentile_uses_the_nearest_rank() {
        let values = [
            Duration::from_millis(1),
            Duration::from_millis(2),
            Duration::from_millis(3),
            Duration::from_millis(4),
        ];
        assert_eq!(percentile(&values, 50, 100), Duration::from_millis(2));
        assert_eq!(percentile(&values, 95, 100), Duration::from_millis(4));
    }
}
