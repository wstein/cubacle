use min2phase::{from_moves, solve};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader, BufWriter, Write};
use std::path::Path;

const MAX_LENGTH: u8 = 20;

#[derive(Default)]
struct Summary {
    total: usize,
    exact: usize,
    equal_length: usize,
    shorter: usize,
    longer: usize,
    errors: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum Classification {
    Exact,
    SameLength,
    Shorter,
    Longer,
    Error,
}

impl Classification {
    fn as_str(&self) -> &'static str {
        match self {
            Self::Exact => "exact",
            Self::SameLength => "same-length",
            Self::Shorter => "shorter",
            Self::Longer => "longer",
            Self::Error => "error",
        }
    }
}

fn normalize_moves(moves: &str) -> String {
    moves
        .split_whitespace()
        .map(|token| {
            let (face, suffix) = token.split_at(1);
            match suffix {
                "" | "1" => face.to_owned(),
                "2" => format!("{face}2"),
                "3" | "'" => format!("{face}'"),
                _ => token.to_owned(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

fn move_count(moves: &str) -> usize {
    moves.split_whitespace().count()
}

fn classify(expected_solution: &str, actual_solution: &str) -> (Classification, usize) {
    if actual_solution.starts_with("Error ") {
        return (Classification::Error, 0);
    }

    let expected_length = move_count(expected_solution);
    let actual_length = move_count(actual_solution);
    let classification = if actual_solution == expected_solution {
        Classification::Exact
    } else if actual_length == expected_length {
        Classification::SameLength
    } else if actual_length < expected_length {
        Classification::Shorter
    } else {
        Classification::Longer
    };
    (classification, actual_length)
}

fn main() -> io::Result<()> {
    let mut arguments = env::args().skip(1);
    let input_path = arguments
        .next()
        .unwrap_or_else(|| "examples/scrambles.csv".to_owned());
    let report_path = arguments
        .next()
        .unwrap_or_else(|| "examples/scramble-report-20.csv".to_owned());

    let input = BufReader::new(File::open(&input_path)?);
    let mut lines = input.lines();
    let header = lines.next().transpose()?.unwrap_or_default();
    if header != "competitionId,evntId,roundTypeId,group,isExtra,number,scramble,moves,solution" {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected scramble CSV header",
        ));
    }

    let mut report = BufWriter::new(File::create(&report_path)?);
    writeln!(
        report,
        "line,competitionId,eventId,roundTypeId,group,number,classification,expected_length,actual_length,scramble,expected_solution,actual_solution"
    )?;

    let mut summary = Summary::default();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let line = line?;
        let columns: Vec<_> = line.split(',').collect();
        if columns.len() != 9 {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "line {line_number} has {} columns, expected 9",
                    columns.len()
                ),
            ));
        }

        summary.total += 1;
        let scramble = columns[6];
        let expected_solution = normalize_moves(columns[8]);
        let actual_solution = from_moves(scramble)
            .map(|cube| solve(&cube, MAX_LENGTH))
            .unwrap_or_else(|| "Error parse-scramble".to_owned());
        let actual_solution = normalize_moves(&actual_solution);
        let expected_length = move_count(&expected_solution);
        let (classification, actual_length) = classify(&expected_solution, &actual_solution);

        match classification {
            Classification::Exact => {
                summary.exact += 1;
                summary.equal_length += 1;
            }
            Classification::SameLength => {
                summary.equal_length += 1;
            }
            Classification::Shorter => {
                summary.shorter += 1;
            }
            Classification::Longer => {
                summary.longer += 1;
            }
            Classification::Error => summary.errors += 1,
        }

        if classification != Classification::Exact {
            writeln!(
                report,
                "{line_number},{},{},{},{},{},{},{expected_length},{actual_length},{scramble},{expected_solution},{actual_solution}",
                columns[0], columns[1], columns[2], columns[3], columns[5],
                classification.as_str(),
            )?;
        }
    }
    report.flush()?;

    println!("Bulk comparison (max solution length: {MAX_LENGTH})");
    println!("Input:  {}", Path::new(&input_path).display());
    println!("Report: {}", Path::new(&report_path).display());
    println!("Total:              {}", summary.total);
    println!("Exact move sequence: {}", summary.exact);
    println!("Same length:        {}", summary.equal_length);
    println!("Shorter:            {}", summary.shorter);
    println!("Longer:             {}", summary.longer);
    println!("Errors:             {}", summary.errors);

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{classify, normalize_moves, Classification};

    #[test]
    fn normalizes_reference_notation() {
        assert_eq!(normalize_moves("R1 U2 F3"), "R U2 F'");
    }

    #[test]
    fn classifies_solution_lengths() {
        assert_eq!(classify("R U", "R U"), (Classification::Exact, 2));
        assert_eq!(classify("R U", "F B"), (Classification::SameLength, 2));
        assert_eq!(classify("R U", "F"), (Classification::Shorter, 1));
        assert_eq!(classify("R U", "F B L"), (Classification::Longer, 3));
        assert_eq!(classify("R U", "Error 8"), (Classification::Error, 0));
    }
}
