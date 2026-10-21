use min2phase::{apply_moves, from_moves, solve_with_stats};
use std::env;
use std::io;
use std::time::{Duration, Instant};

const MAX_LENGTH: u8 = 20;
const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
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

fn validate_solution(cube: &str, solution: &str) -> io::Result<()> {
    let solved = apply_moves(cube, solution).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "solver returned invalid moves")
    })?;
    if solved != SOLVED {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "solver result did not solve the cube",
        ));
    }
    Ok(())
}

fn main() -> io::Result<()> {
    let iterations = match env::args().nth(1) {
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

    let cubes: Vec<_> = SCRAMBLES
        .iter()
        .map(|scramble| {
            from_moves(scramble).ok_or_else(|| {
                io::Error::new(io::ErrorKind::InvalidData, "benchmark scramble is invalid")
            })
        })
        .collect::<io::Result<_>>()?;

    for cube in &cubes {
        let result = solve_with_stats(cube, MAX_LENGTH);
        validate_solution(cube, &result.solution)?;
    }

    let mut durations = Vec::with_capacity(cubes.len() * iterations as usize);
    let mut total_probes = 0_u64;
    for _ in 0..iterations {
        for cube in &cubes {
            let started = Instant::now();
            let result = solve_with_stats(cube, MAX_LENGTH);
            durations.push(started.elapsed());
            validate_solution(cube, &result.solution)?;
            total_probes += result.probes;
        }
    }

    durations.sort_unstable();
    let total = durations.len();
    let total_duration: Duration = durations.iter().sum();
    let total_seconds = total_duration.as_secs_f64();
    println!(
        "Validated {total} solves from {} fixed scrambles at a {MAX_LENGTH}-move limit after warm-up.",
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
