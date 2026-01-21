use crossbeam::channel;
use std::sync::Arc;

mod process_bytes;
mod process_chars;
mod process_fields;
pub mod worker_utilities;

use self::process_bytes::process_bytes;
use self::process_chars::process_chars;
use self::process_fields::process_fields;
use crate::types::*;

pub fn process_records(
    instructions: Arc<Instructions>,
    record_receiver: channel::Receiver<Vec<Record>>,
    result_sender: channel::Sender<ResultChunk>,
) -> Result<(), String> {
    loop {
        let record_batch = match record_receiver.recv() {
            Ok(record_batch) => record_batch,
            Err(_) => return Ok(()),
        };

        if record_batch.is_empty() {
            continue;
        }

        let batch_start_index = record_batch[0].index;
        let mut batch_outputs: Vec<OutputRecord> = Vec::with_capacity(record_batch.len());

        for record in record_batch {
            let record_index = record.index;
            let has_terminator = record.has_terminator;

            let processed_result: Result<Vec<u8>, String> = match instructions.selection_mode {
                SelectionMode::Bytes => process_bytes(&instructions, record),
                SelectionMode::Chars => process_chars(&instructions, record),
                SelectionMode::Fields => {
                    let engine = instructions
                        .regex_engine
                        .as_ref()
                        .ok_or_else(|| "internal error: missing regex engine".to_string())?;
                    process_fields(&instructions, engine, record)
                }
            };

            match processed_result {
                Ok(bytes) => {
                    if instructions.strict_return && bytes.is_empty() {
                        let _ = result_sender.send(ResultChunk::Err {
                            index: record_index,
                            error: "strict return error: empty field".to_string(),
                        });
                        return Ok(());
                    }
                    batch_outputs.push(OutputRecord {
                        bytes,
                        has_terminator,
                    });
                }
                Err(error) => {
                    let _ = result_sender.send(ResultChunk::Err {
                        index: record_index,
                        error,
                    });
                    return Ok(());
                }
            }
        }

        result_sender
            .send(ResultChunk::Ok {
                start_index: batch_start_index,
                outputs: batch_outputs,
            })
            .map_err(|error| error.to_string())?;
    }
}
