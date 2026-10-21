use min2phase::{from_moves, solve_with_minimum_time};
use std::env;
use std::fs::File;
use std::io::{self, stdout, BufRead, BufReader, BufWriter, Write};
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::sync::{mpsc, Arc};
use std::thread;
use std::time::{Duration, Instant};

const MAX_LENGTH: u8 = 19;
const DEFAULT_SOLVE_BUDGET: Duration = Duration::from_millis(100);
const SCRAMBLE_HEADER: &str =
    "competitionId,evntId,roundTypeId,group,isExtra,number,scramble,moves,solution";
const REPORT_HEADER: &str = "line,competitionId,eventId,roundTypeId,group,number,classification,expected_length,actual_length,probes,solve_micros,timed_out,scramble,expected_solution,actual_solution";

#[derive(Default)]
struct Summary {
    total: usize,
    exact: usize,
    equal_length: usize,
    shorter: usize,
    longer: usize,
    errors: usize,
    timed_out: usize,
    total_probes: u64,
    total_solve_micros: u128,
}

#[derive(Clone)]
struct Job {
    line_number: usize,
    competition_id: String,
    event_id: String,
    round_type_id: String,
    group: String,
    number: String,
    scramble: String,
    expected_solution: String,
    expected_length: usize,
}

struct Row {
    job_index: usize,
    classification: Classification,
    actual_length: usize,
    probes: u64,
    solve_micros: u128,
    actual_solution: String,
    timed_out: bool,
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

fn print_progress(completed: usize, total: usize, started: Instant) -> io::Result<()> {
    const WIDTH: usize = 30;

    let filled = completed * WIDTH / total;
    let elapsed_seconds = started.elapsed().as_secs_f64();
    let remaining_seconds = elapsed_seconds / completed as f64 * (total - completed) as f64;
    print!(
        "\r[{}{}] {completed}/{total} ({:5.1}%) elapsed {:6.1}s ETA {:6.1}s",
        "#".repeat(filled),
        "-".repeat(WIDTH - filled),
        completed as f64 / total as f64 * 100.0,
        elapsed_seconds,
        remaining_seconds,
    );
    stdout().flush()
}

fn read_jobs(input_path: &str, longer_only: bool) -> io::Result<Vec<Job>> {
    let input = BufReader::new(File::open(input_path)?);
    let mut lines = input.lines();
    let header = lines.next().transpose()?.unwrap_or_default();
    if header != SCRAMBLE_HEADER && header != REPORT_HEADER {
        return Err(io::Error::new(
            io::ErrorKind::InvalidData,
            "unexpected scramble CSV header",
        ));
    }

    let is_report = header == REPORT_HEADER;
    if longer_only && !is_report {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "the longer filter requires a bulk report input",
        ));
    }

    let mut jobs = Vec::new();
    for (index, line) in lines.enumerate() {
        let line_number = index + 2;
        let line = line?;
        let columns: Vec<_> = line.split(',').collect();
        let expected_columns = if is_report { 15 } else { 9 };
        if columns.len() != expected_columns {
            return Err(io::Error::new(
                io::ErrorKind::InvalidData,
                format!(
                    "line {line_number} has {} columns, expected {expected_columns}",
                    columns.len()
                ),
            ));
        }
        if longer_only && columns[6] != "longer" {
            continue;
        }

        let (competition_id, event_id, round_type_id, group, number, scramble, solution) =
            if is_report {
                (
                    columns[1],
                    columns[2],
                    columns[3],
                    columns[4],
                    columns[5],
                    columns[12],
                    columns[13],
                )
            } else {
                (
                    columns[0], columns[1], columns[2], columns[3], columns[5], columns[6],
                    columns[8],
                )
            };
        let expected_solution = normalize_moves(solution);
        jobs.push(Job {
            line_number,
            competition_id: competition_id.to_owned(),
            event_id: event_id.to_owned(),
            round_type_id: round_type_id.to_owned(),
            group: group.to_owned(),
            number: number.to_owned(),
            scramble: scramble.to_owned(),
            expected_length: move_count(&expected_solution),
            expected_solution,
        });
    }
    Ok(jobs)
}

fn solve_job(job_index: usize, job: &Job, budget: Duration) -> Row {
    let (actual_solution, probes, solve_micros, timed_out) = from_moves(&job.scramble)
        .map(|cube| {
            let started = Instant::now();
            let result = solve_with_minimum_time(&cube, MAX_LENGTH, budget, budget);
            (
                result.solution,
                result.probes,
                started.elapsed().as_micros(),
                result.timed_out,
            )
        })
        .unwrap_or_else(|| ("Error parse-scramble".to_owned(), 0, 0, false));
    let actual_solution = normalize_moves(&actual_solution);
    let (classification, actual_length) = classify(&job.expected_solution, &actual_solution);

    Row {
        job_index,
        classification,
        actual_length,
        probes,
        solve_micros,
        actual_solution,
        timed_out,
    }
}

fn solve_jobs(jobs: Arc<Vec<Job>>, workers: usize, budget: Duration) -> io::Result<Vec<Row>> {
    if jobs.is_empty() {
        return Ok(Vec::new());
    }

    let total = jobs.len();
    let workers = workers.clamp(1, total);
    let next = Arc::new(AtomicUsize::new(0));
    let (sender, receiver) = mpsc::channel();
    let started = Instant::now();
    let mut rows = Vec::with_capacity(total);

    thread::scope(|scope| -> io::Result<()> {
        for _ in 0..workers {
            let jobs = Arc::clone(&jobs);
            let next = Arc::clone(&next);
            let sender = sender.clone();
            scope.spawn(move || loop {
                let index = next.fetch_add(1, Ordering::Relaxed);
                if index >= jobs.len() {
                    break;
                }
                if sender.send(solve_job(index, &jobs[index], budget)).is_err() {
                    break;
                }
            });
        }
        drop(sender);

        for row in receiver {
            rows.push(row);
            let completed = rows.len();
            if completed % 10 == 0 || completed == total {
                print_progress(completed, total, started)?;
            }
        }
        Ok(())
    })?;
    println!();
    rows.sort_by_key(|row| row.job_index);
    Ok(rows)
}

fn requested_workers(argument: Option<String>) -> io::Result<usize> {
    match argument {
        Some(value) => value
            .parse::<usize>()
            .ok()
            .filter(|count| *count > 0)
            .ok_or_else(|| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "worker count must be a positive integer",
                )
            }),
        None => thread::available_parallelism()
            .map(usize::from)
            .map_err(io::Error::other),
    }
}

fn requested_job_limit(argument: Option<String>) -> io::Result<Option<usize>> {
    argument
        .map(|value| {
            if value == "-" {
                return Ok(None);
            }
            let limit = value
                .parse::<usize>()
                .ok()
                .filter(|count| *count > 0)
                .ok_or_else(|| {
                    io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "job limit must be a positive integer",
                    )
                })?;
            Ok(Some(limit))
        })
        .transpose()
        .map(Option::flatten)
}

fn requested_budget(argument: Option<String>) -> io::Result<Duration> {
    match argument {
        Some(value) => value
            .parse::<u64>()
            .map(Duration::from_millis)
            .map_err(|_| {
                io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "budget must be a non-negative number of milliseconds",
                )
            }),
        None => Ok(DEFAULT_SOLVE_BUDGET),
    }
}

fn main() -> io::Result<()> {
    let mut arguments = env::args().skip(1);
    let input_path = arguments
        .next()
        .unwrap_or_else(|| "examples/scrambles.csv".to_owned());
    let report_path = arguments
        .next()
        .unwrap_or_else(|| "examples/scramble-report-19.csv".to_owned());
    let workers = requested_workers(arguments.next())?;
    let job_limit = requested_job_limit(arguments.next())?;
    let budget = requested_budget(arguments.next())?;
    let longer_only = arguments.next().is_some_and(|value| value == "longer");
    let mut jobs = read_jobs(&input_path, longer_only)?;
    if let Some(job_limit) = job_limit {
        jobs.truncate(job_limit);
    }
    let jobs = Arc::new(jobs);
    let rows = solve_jobs(Arc::clone(&jobs), workers, budget)?;

    let mut report = BufWriter::new(File::create(&report_path)?);
    writeln!(
        report,
        "line,competitionId,eventId,roundTypeId,group,number,classification,expected_length,actual_length,probes,solve_micros,timed_out,scramble,expected_solution,actual_solution"
    )?;

    let mut summary = Summary::default();
    for row in rows {
        let job = &jobs[row.job_index];
        summary.total += 1;
        summary.total_probes += row.probes;
        summary.total_solve_micros += row.solve_micros;

        match row.classification {
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
        if row.timed_out {
            summary.timed_out += 1;
        }

        writeln!(
            report,
            "{},{},{},{},{},{},{},{},{},{},{},{},{},{},{}",
            job.line_number,
            job.competition_id,
            job.event_id,
            job.round_type_id,
            job.group,
            job.number,
            row.classification.as_str(),
            job.expected_length,
            row.actual_length,
            row.probes,
            row.solve_micros,
            row.timed_out,
            job.scramble,
            job.expected_solution,
            row.actual_solution,
        )?;
    }
    report.flush()?;

    println!(
        "Bulk comparison (max solution length: {MAX_LENGTH}, minimum and maximum search time: {} ms)",
        budget.as_millis()
    );
    println!("Input:  {}", Path::new(&input_path).display());
    println!("Report: {}", Path::new(&report_path).display());
    println!("Workers: {}", workers);
    println!("Total:              {}", summary.total);
    println!("Exact move sequence: {}", summary.exact);
    println!("Same length:        {}", summary.equal_length);
    println!("Shorter:            {}", summary.shorter);
    println!("Longer:             {}", summary.longer);
    println!("Errors:             {}", summary.errors);
    println!("Timed out:          {}", summary.timed_out);
    println!("Total probes:       {}", summary.total_probes);
    println!(
        "Average probes:     {:.1}",
        summary.total_probes as f64 / summary.total as f64
    );
    println!(
        "Total solve time:   {:.3} s",
        summary.total_solve_micros as f64 / 1_000_000.0
    );
    println!(
        "Average solve time: {:.3} ms",
        summary.total_solve_micros as f64 / summary.total as f64 / 1_000.0
    );

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::{classify, normalize_moves, read_jobs, Classification, REPORT_HEADER};
    use std::fs;

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

    #[test]
    fn rejects_an_invalid_worker_count() {
        assert!(super::requested_workers(Some("0".to_owned())).is_err());
        assert!(super::requested_workers(Some("two".to_owned())).is_err());
    }

    #[test]
    fn rejects_an_invalid_job_limit() {
        assert!(super::requested_job_limit(Some("0".to_owned())).is_err());
        assert!(super::requested_job_limit(Some("two".to_owned())).is_err());
    }

    #[test]
    fn parses_budget_in_milliseconds() {
        assert_eq!(
            super::requested_budget(Some("200".to_owned())).expect("valid budget"),
            std::time::Duration::from_millis(200)
        );
    }

    #[test]
    fn filters_longer_rows_from_a_report() {
        let path =
            std::env::temp_dir().join(format!("min2phase-bulk-compare-{}.csv", std::process::id()));
        let contents = format!(
            "{REPORT_HEADER}\n2,event-a,333fm,f,A,1,longer,15,19,10,100,false,R U,R U,U R\n3,event-b,333fm,f,A,2,same-length,15,15,11,110,false,F U,F U,U F\n"
        );
        fs::write(&path, contents).expect("write report fixture");

        let jobs = read_jobs(path.to_str().expect("UTF-8 path"), true).expect("read report");
        fs::remove_file(path).expect("remove report fixture");

        assert_eq!(jobs.len(), 1);
        assert_eq!(jobs[0].scramble, "R U");
    }
}
