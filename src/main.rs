#![warn(clippy::nursery, clippy::pedantic)]
#![feature(array_chunks)]

use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;

fn main() {
    let path = env::args()
        .nth(1)
        .expect("Path to the file as first cli argument.");
    let file = File::open(path).expect("Existing file.");
    let mut reader = io::BufReader::new(file);
    read_contents(&mut reader);
}

pub fn read_contents(reader: &mut io::BufReader<File>) {
    let buffer = reader.fill_buf().unwrap();
    for (i, bytes) in buffer.array_chunks::<16>().enumerate() {
        let hex = format_hex(bytes);
        let text = format_text(bytes);
        println!("{i:0>8x}: {hex} {text}");
    }
    let bytes_read = buffer.len();
    reader.consume(bytes_read);
}

fn format_text(bytes: &[u8; 16]) -> String {
    bytes
        .map(|byte| {
            if byte.is_ascii_graphic() {
                byte as char
            } else {
                '.'
            }
        })
        .iter()
        .collect()
}

fn format_hex(bytes: &[u8; 16]) -> String {
    bytes
        .map(|byte| format!("{byte:0>2x} "))
        .into_iter()
        .collect()
}
