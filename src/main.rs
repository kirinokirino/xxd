#![warn(clippy::nursery, clippy::pedantic)]
#![feature(array_chunks)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io;
use std::io::BufRead;

use confargenv::fusion;

struct Config {
    mode: Mode,
    input: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        let mut defaults = HashMap::new();
        defaults.insert("mode", "graphical");
        let conf = fusion(defaults, None);
        let mode = match conf.get("mode").unwrap().as_str() {
            "graphical" => Mode::Graphical,
            "hex" => Mode::Hex,
            "reverse" => Mode::Reverse,
            _ => unreachable!(),
        };
        Config { mode, input: None }
    }
}

enum Mode {
    Graphical,
    Hex,
    Reverse,
}

fn main() {
    let config = Config::new();
    let input = config.input.unwrap_or_else(|| {
        env::args()
            .nth(1)
            .expect("Path to the file as first cli argument.")
    });
    let file = File::open(input).expect("Existing file.");
    let mut reader = io::BufReader::new(file);
    read_contents(&mut reader, config.mode);
}

fn read_contents(reader: &mut io::BufReader<File>, mode: Mode) {
    let buffer = reader.fill_buf().unwrap();
    for (i, bytes) in buffer.array_chunks::<16>().enumerate() {
        match mode {
            Mode::Graphical => {
                let hex = format_hex(bytes, true);
                let text = format_text(bytes);
                println!("{i:0>8x}: {hex} {text}");
            }
            Mode::Hex => print!("{}", format_hex(bytes, false)),
            Mode::Reverse => print!("{}", parse_bytes(bytes)),
        }
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

fn format_hex(bytes: &[u8; 16], add_space: bool) -> String {
    let space = if add_space { " " } else { "" };
    bytes
        .map(|byte| format!("{byte:0>2x}{space}"))
        .into_iter()
        .collect()
}

fn parse_bytes(bytes: &[u8; 16]) -> String {
    bytes
        .array_chunks::<2>()
        .map(|[first, second]| {
            let first = u8::from_str_radix(&(*first as char).to_string(), 16)
                .expect("In reverse mode only [0-9a-f] chars are allowed");
            let second = u8::from_str_radix(&(*second as char).to_string(), 16)
                .expect("In reverse mode only [0-9a-f] chars are allowed");
            (first << 4 | second) as char
        })
        .collect()
}
