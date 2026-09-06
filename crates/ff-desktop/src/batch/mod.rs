#![allow(dead_code)]
pub mod cli;
pub mod input;
pub mod output;
pub mod return_code;
pub mod runner;
pub mod session;

use cli::BatchCliArgs;
use input::BatchInputSource;
use output::BatchOutputSink;
use return_code::BatchReturnCode;
use runner::{BatchOptions, BatchRunner};
use session::BatchSession;

/// Entry point called from `main.rs` when `--batch` is detected.
///
/// Builds the input source, output sink, session, and runner from the parsed
/// CLI args, executes the batch run, prints the final summary line to stderr,
/// and returns the exit code.
///
/// Validates: Requirement 1.1, 1.2, 1.6, 4.1, 4.2, 4.3, 5.5, 7.1-7.6
pub fn run_batch(args: BatchCliArgs) -> i32 {
    // Build input source (Req 1.1, 1.2, 1.6)
    let mut input = if args.input == "-" {
        BatchInputSource::from_reader(std::io::stdin())
    } else {
        match std::fs::File::open(&args.input) {
            Ok(f) => BatchInputSource::from_reader(f),
            Err(e) => {
                eprintln!("ffwb: cannot open batch input '{}': {}", args.input, e);
                return 12;
            }
        }
    };

    // Build output sink (Req 4.1, 4.2, 4.3, 4.7)
    let sink = match (&args.output, args.output_append) {
        (None, _) => BatchOutputSink::Stdout,
        (Some(path), false) => {
            // Truncate the file before the run (Req 4.2)
            if let Err(e) = std::fs::File::create(path) {
                eprintln!("ffwb: cannot create batch output '{}': {}", path, e);
                return 12;
            }
            BatchOutputSink::Append(path.clone())
        }
        (Some(path), true) => BatchOutputSink::Append(path.clone()),
    };

    // Build session (Req 7.1-7.6)
    let session = BatchSession::new(args.no_catalog, args.profile);

    // Build runner options (Req 4.4, 6.1, 6.2, 8.1)
    let options = BatchOptions {
        echo: args.echo,
        dry_run: args.dry_run,
        abort_policy: args.abort_policy,
    };

    let runner = BatchRunner::new(options);
    let brc: BatchReturnCode = runner.run(&mut input, &sink, &session);

    // Req 5.5: write final summary line to stderr
    eprintln!("FFWB BATCH RETURN CODE: {}", brc.as_i32());

    brc.as_i32()
}
