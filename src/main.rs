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
    let config = match get_instructions() {
        Ok(Some(config)) => Arc::new(config),
        Ok(None) => return,
        Err(error) => {
            eprintln!("{error}");
            std::process::exit(2);
        }
    };

    let (input_sender, input_receiver) = channel::bounded::<Vec<Record>>(1024);
    let (output_sender, output_receiver) = channel::bounded::<ResultChunk>(1024);

    let input_handle = {
        let config = Arc::clone(&config);
        std::thread::spawn(move || read_input(&config, input_sender))
    };

    let worker_count = if std::env::var("SPLITBY_SINGLE_CORE").is_ok() {
        1
    } else {
        std::thread::available_parallelism()
            .map(|count| count.get())
            .unwrap_or(1)
    };

    for _worker_index in 0..max(worker_count - 1, 1) {
        let worker_config = Arc::clone(&config);
        let worker_receiver = input_receiver.clone();
        let worker_sender = output_sender.clone();
        std::thread::spawn(move || {
            let _ = process_records(worker_config, worker_receiver, worker_sender)
                .map_err(|error| eprintln!("{error}"));
        });
    }
    drop(output_sender);

    let results_status = get_results(&config, output_receiver);

    if let Err(error) = input_handle.join().unwrap() {
        eprintln!("{}", error);
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }

    if let Err(error) = results_status {
        eprintln!("{}", error);
        let exit_code = if error.contains("failed to open") || error.contains("failed to create") {
            2
        } else {
            1
        };
        std::process::exit(exit_code);
    }
}
