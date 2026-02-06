mod cli;
mod input;
mod output;
mod transform;
mod types;
mod utilities;

use input::read_input;
use output::get_results;
use transform::process_records;
use types::*;

use crossbeam::channel;
use std::{cmp::max, sync::Arc};

use crate::cli::get_instructions;

fn main() {
    let instructions = match get_instructions() {
        Ok(Some(instructions)) => instructions,
        Ok(None) => return,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let input_instructions = instructions.input_instructions;
    let transform_instructions = Arc::new(instructions.transform_instructions);
    let output_instructions = instructions.output_instructions;

    let (input_sender, input_receiver) = channel::bounded::<Vec<Record>>(1024);
    let (output_sender, output_receiver) = channel::bounded::<ResultChunk>(1024);

    // Setting up our Reader worker
    let input_handle = std::thread::spawn(move || read_input(&input_instructions, input_sender));

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
        let worker_instructions = Arc::clone(&transform_instructions);
        let worker_receiver = input_receiver.clone();
        let worker_sender = output_sender.clone();
        std::thread::spawn(move || {
            let _ = process_records(worker_instructions, worker_receiver, worker_sender)
                .map_err(|error| eprintln!("{error}"));
        });
    }
    drop(output_sender);

    let results_status = get_results(output_instructions, output_receiver);

    // Check if input thread encountered an I/O error
    if let Err(error) = input_handle.join().unwrap() {
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
