mod cli;
mod input;
mod output;
mod transform;
mod types;

use clap::Parser;
use crossbeam::channel;
use std::{cmp::max, sync::Arc};

use cli::{Options, parse_options};
use input::read_input;
use output::get_results;
use transform::process_records;
use types::*;

fn main() {
    let options = Options::parse();
    let config = match parse_options(options) {
        Ok(config) => config,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let instructions = Arc::new(config.instructions);
    let reader_instructions = config.reader_instructions;

    let (record_sender, record_receiver) = channel::bounded::<Vec<Record>>(1024);
    let (result_sender, result_receiver) = channel::bounded::<ResultChunk>(1024);

    // Setting up our Reader worker
    let reader_sender = record_sender.clone();
    let reader_handle = std::thread::spawn(move || read_input(&reader_instructions, reader_sender));
    drop(record_sender);

    // Working out how much memory we need
    let worker_count = if std::env::var("SPLITBY_SINGLE_CORE").is_ok() {
        1
    } else {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    };

    // Setting up our main processing workers
    for _worker_index in 0..max(worker_count - 1, 1) {
        let worker_instructions = Arc::clone(&instructions);
        let worker_receiver = record_receiver.clone();
        let worker_sender = result_sender.clone();
        std::thread::spawn(move || {
            let _ = process_records(worker_instructions, worker_receiver, worker_sender)
                .map_err(|error| eprintln!("{error}"));
        });
    }
    drop(result_sender);

    let results_status = get_results(instructions, result_receiver);

    // Check if read_input thread encountered an I/O error
    if let Err(error) = reader_handle.join().unwrap() {
        eprintln!("{}", error);
        // Exit with code 2 for I/O errors
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }

    if let Err(error) = results_status {
        eprintln!("{}", error);
        // Exit with code 2 for I/O errors, code 1 for other errors
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }
}
