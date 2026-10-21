use min2phase::{apply_moves, from_moves, solve_with_stats};
use std::env;
use std::io;
use std::time::Instant;

const MAX_LENGTH: u8 = 20;
const SOLVED: &str = "UUUUUUUUURRRRRRRRRFFFFFFFFFDDDDDDDDDLLLLLLLLLBBBBBBBBB";
const SCRAMBLE: &str = "R U R' U' F2 D B2 L2 U2 R2 F' D2";

fn main() -> io::Result<()> {
    let iterations = match env::args().nth(1) {
        Some(value) => value.parse::<u32>().map_err(|_| {
            io::Error::new(
                io::ErrorKind::InvalidInput,
                "iterations must be a positive integer",
            )
        })?,
        None => 100,
    };
    if iterations == 0 {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "iterations must be a positive integer",
        ));
    }
    let cube = from_moves(SCRAMBLE).ok_or_else(|| {
        io::Error::new(io::ErrorKind::InvalidData, "benchmark scramble is invalid")
    })?;

    let started = Instant::now();
    let mut total_probes = 0;
    for _ in 0..iterations {
        let result = solve_with_stats(&cube, MAX_LENGTH);
        let solved = apply_moves(&cube, &result.solution).ok_or_else(|| {
            io::Error::new(io::ErrorKind::InvalidData, "solver returned invalid moves")
        })?;
        if solved != SOLVED {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                "solver result did not solve the cube",
            ));
        }
        total_probes += result.probes;
    }
    let elapsed = started.elapsed();

    println!("Validated {iterations} solutions at a {MAX_LENGTH}-move limit.");
    println!("Total time:    {:.3} s", elapsed.as_secs_f64());
    println!(
        "Average time:  {:.3} ms",
        elapsed.as_secs_f64() * 1_000.0 / iterations as f64
    );
    println!(
        "Average probes: {:.1}",
        total_probes as f64 / iterations as f64
    );

    Ok(())
}
