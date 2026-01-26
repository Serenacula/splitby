mod get_largest_field_widths;

use crate::types::*;

use crossbeam::channel;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

fn read_record(
    reader: &mut Box<dyn BufRead>,
    buffer: &mut Vec<u8>,
    index: &mut usize,
    terminator: u8,
) -> Result<Option<Record>, String> {
    let bytes_read = reader.read_until(terminator, buffer).map_err(|error| {
        if terminator == b'\0' {
            format!("error while reading: {error}")
        } else {
            format!("{error}")
        }
    })?;
    if bytes_read == 0 {
        return Ok(None);
    }

    let has_terminator = buffer.last() == Some(&terminator);
    if has_terminator {
        buffer.pop();
        if &terminator == &b'\n' && buffer.last() == Some(&b'\r') {
            buffer.pop();
        }
    }

    let record_bytes = std::mem::take(buffer);
    let record = Record {
        index: *index,
        bytes: record_bytes,
        has_terminator,
        field_widths: None,
    };
    *index += 1;
    Ok(Some(record))
}

pub fn read_input(
    input_instructions: &InputInstructions,
    record_sender: channel::Sender<Vec<Record>>,
) -> Result<(), String> {
    let batch_byte_quota = std::env::var("SPLITBY_BATCH_QUOTA")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(128 * 1024);

    let mut reader: Box<dyn BufRead> = match input_instructions.input.as_ref() {
        Some(path) => {
            let file = File::open(path)
                .map_err(|error| format!("failed to open {}: {error}", path.display()))?;
            Box::new(BufReader::new(file))
        }
        None => {
            let stdin = io::stdin();
            Box::new(stdin.lock())
        }
    };
    let mut index: usize = 0;
    let mut batch: Vec<Record> = Vec::new();
    let mut batch_bytes: usize = 0;

    let flush_batch = |record_sender: &channel::Sender<Vec<Record>>,
                       batch: &mut Vec<Record>,
                       batch_bytes: &mut usize|
     -> Result<(), String> {
        if batch.is_empty() {
            return Ok(());
        }
        let pending_batch = std::mem::take(batch);
        *batch_bytes = 0;
        record_sender
            .send(pending_batch)
            .map_err(|error| format!("{error}"))?;
        Ok(())
    };

    let add_record_to_batch = |record: Record,
                               batch: &mut Vec<Record>,
                               batch_bytes: &mut usize,
                               batch_byte_quota: usize,
                               record_sender: &channel::Sender<Vec<Record>>|
     -> Result<(), String> {
        *batch_bytes = batch_bytes.saturating_add(record.bytes.len());
        batch.push(record);

        if *batch_bytes >= batch_byte_quota {
            flush_batch(record_sender, batch, batch_bytes)?;
        }
        Ok(())
    };

    // Handle align mode: read all records, scan widths, then stream
    if !matches!(input_instructions.align, Align::None)
        && input_instructions.input_mode == InputMode::PerLine
    {
        let mut all_records: Vec<Record> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();

        // Read all records into memory
        loop {
            match read_record(&mut reader, &mut buffer, &mut index, b'\n')? {
                Some(record) => all_records.push(record),
                None => break,
            }
        }

        // Scan field widths
        use crate::input::get_largest_field_widths::get_largest_field_widths;
        let max_widths = get_largest_field_widths(&all_records, input_instructions)?;

        // Attach field_widths to each record
        for record in &mut all_records {
            record.field_widths = Some(max_widths.clone());
        }

        // Stream buffered records in batches using existing batch variables
        for record in all_records {
            add_record_to_batch(
                record,
                &mut batch,
                &mut batch_bytes,
                batch_byte_quota,
                &record_sender,
            )?;
        }

        flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
        return Ok(());
    }

    // Normal streaming behavior
    match input_instructions.input_mode {
        InputMode::PerLine => {
            let mut buffer: Vec<u8> = Vec::new();
            loop {
                match read_record(&mut reader, &mut buffer, &mut index, b'\n')? {
                    Some(record) => {
                        add_record_to_batch(
                            record,
                            &mut batch,
                            &mut batch_bytes,
                            batch_byte_quota,
                            &record_sender,
                        )?;
                    }
                    None => {
                        flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                        return Ok(());
                    }
                }
            }
        }
        InputMode::ZeroTerminated => {
            let mut buffer: Vec<u8> = Vec::new();
            loop {
                match read_record(&mut reader, &mut buffer, &mut index, b'\0')? {
                    Some(record) => {
                        add_record_to_batch(
                            record,
                            &mut batch,
                            &mut batch_bytes,
                            batch_byte_quota,
                            &record_sender,
                        )?;
                    }
                    None => {
                        flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                        return Ok(());
                    }
                }
            }
        }
        InputMode::WholeString => {
            let mut buffer: Vec<u8> = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .map_err(|error| format!("{error}"))?;

            batch.push(Record {
                index,
                bytes: buffer,
                has_terminator: false,
                field_widths: None,
            });
            flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;

            Ok(())
        }
    }
}
