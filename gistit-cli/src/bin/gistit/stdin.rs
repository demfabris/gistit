use std::io::{stdin, BufRead};

use crate::prettyln;

const READ_LIMIT_BYTES: usize = 50_000;

pub fn read_to_end() -> String {
    let mut buf = String::new();
    let mut threshold = READ_LIMIT_BYTES;
    let stdin = stdin();
    let mut handle = stdin.lock();
    prettyln!("Edit mode, type Ctrl+D to quit");

    while let Ok(read) = handle.read_line(&mut buf) {
        if threshold == 0 || read == 0 {
            break;
        }
        threshold -= read;
    }

    buf
}
