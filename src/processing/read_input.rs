use crossbeam::channel;
use std::{
    fs::File,
    io::{self, BufRead, BufReader},
};

use crate::types::{InputMode, Record};
use crate::ReaderInstructions;

pub fn read_input(
    reader_instructions: &ReaderInstructions,
    record_sender: channel::Sender<Vec<Record>>,
) -> Result<(), String> {
    let batch_byte_quota = std::env::var("SPLITBY_BATCH_QUOTA")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .unwrap_or(128 * 1024);
    let mut reader: Box<dyn BufRead> = match reader_instructions.input.as_ref() {
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

    // Handle align mode: read all records, scan widths, then stream
    if reader_instructions.align && reader_instructions.input_mode == InputMode::PerLine {
        let mut all_records: Vec<Record> = Vec::new();
        let mut buffer: Vec<u8> = Vec::new();
        let mut index: usize = 0;

        // Read all records into memory
        loop {
            let bytes_read = reader
                .read_until(b'\n', &mut buffer)
                .map_err(|error| format!("{error}"))?;
            if bytes_read == 0 {
                break;
            }

            let has_terminator = buffer.last() == Some(&b'\n');
            if has_terminator {
                buffer.pop();
                if buffer.last() == Some(&b'\r') {
                    buffer.pop();
                }
            }

            let record_bytes = std::mem::take(&mut buffer);
            all_records.push(Record {
                index: index,
                bytes: record_bytes,
                has_terminator: has_terminator,
                field_widths: None,
            });

            index += 1;
        }

        // Scan field widths
        use crate::processing::scan_field_widths::scan_field_widths;
        let max_widths = scan_field_widths(&all_records, reader_instructions)?;

        // Attach field_widths to each record
        for record in &mut all_records {
            record.field_widths = Some(max_widths.clone());
        }

        // Stream buffered records in batches
        let batch_byte_quota = std::env::var("SPLITBY_BATCH_QUOTA")
            .ok()
            .and_then(|value| value.parse::<usize>().ok())
            .filter(|value| *value > 0)
            .unwrap_or(128 * 1024);
        let mut batch: Vec<Record> = Vec::new();
        let mut batch_bytes: usize = 0;

        for record in all_records {
            batch_bytes = batch_bytes.saturating_add(record.bytes.len());
            batch.push(record);

            if batch_bytes >= batch_byte_quota {
                record_sender
                    .send(std::mem::take(&mut batch))
                    .map_err(|error| format!("{error}"))?;
                batch_bytes = 0;
            }
        }

        if !batch.is_empty() {
            record_sender
                .send(batch)
                .map_err(|error| format!("{error}"))?;
        }

        return Ok(());
    }

    // Normal streaming behavior
    match reader_instructions.input_mode {
        InputMode::PerLine => {
            let mut buffer: Vec<u8> = Vec::new();
            loop {
                let bytes_read = reader
                    .read_until(b'\n', &mut buffer)
                    .map_err(|error| format!("{error}"))?;
                if bytes_read == 0 {
                    flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                    return Ok(());
                }

                let has_terminator = buffer.last() == Some(&b'\n');
                if has_terminator {
                    buffer.pop();
                    if buffer.last() == Some(&b'\r') {
                        buffer.pop();
                    }
                }

                let record_bytes = std::mem::take(&mut buffer);
                batch_bytes = batch_bytes.saturating_add(record_bytes.len());
                batch.push(Record {
                    index: index,
                    bytes: record_bytes,
                    has_terminator: has_terminator,
                    field_widths: None,
                });

                if batch_bytes >= batch_byte_quota {
                    flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                }

                index += 1;
            }
        }
        InputMode::ZeroTerminated => {
            let mut buffer: Vec<u8> = Vec::new();
            loop {
                let bytes_read = reader
                    .read_until(b'\0', &mut buffer)
                    .map_err(|error| format!("error while reading: {error}"))?;
                if bytes_read == 0 {
                    flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                    return Ok(());
                }

                let has_terminator = buffer.last() == Some(&b'\0');
                if has_terminator {
                    buffer.pop();
                }

                let record_bytes = std::mem::take(&mut buffer);
                batch_bytes = batch_bytes.saturating_add(record_bytes.len());
                batch.push(Record {
                    index: index,
                    bytes: record_bytes,
                    has_terminator: has_terminator,
                    field_widths: None,
                });

                if batch_bytes >= batch_byte_quota {
                    flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;
                }

                index += 1;
            }
        }
        InputMode::WholeString => {
            let mut buffer: Vec<u8> = Vec::new();
            reader
                .read_to_end(&mut buffer)
                .map_err(|error| format!("{error}"))?;

            batch.push(Record {
                index: index,
                bytes: buffer,
                has_terminator: false,
                field_widths: None,
            });
            flush_batch(&record_sender, &mut batch, &mut batch_bytes)?;

            Ok(())
        }
    }
}
