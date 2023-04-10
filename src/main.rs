#![warn(clippy::nursery, clippy::pedantic)]
#![feature(iter_array_chunks, array_chunks)]

use std::collections::HashMap;
use std::env;
use std::fs::File;
use std::io::BufRead;
use std::io::{self, Write};

use confargenv::fusion;
use log::LevelFilter;
use logger::BareLogger;

fn main() {
    //let _ = BareLogger::new(LevelFilter::max()).init();
    let config = Config::new();
    let input = config.input.unwrap_or_else(|| {
        env::args()
            .nth(1)
            .expect("Path to the file as first cli argument.")
    });
    read_path(input, config.mode);
}

fn read_path<S: AsRef<str>>(path: S, mode: Mode) {
    let file = File::open(path.as_ref()).expect("Existing file.");
    let mut reader = io::BufReader::new(file);
    read_contents(&mut reader, mode);
}

fn read_contents(reader: &mut io::BufReader<File>, mode: Mode) {
    let buffer = reader.fill_buf().unwrap();
    let mut line = 0;
    let iter = buffer.array_chunks::<16>();
    let remainder = iter.remainder();
    for bytes in iter {
        print_line(line, bytes, mode);
        line += 1;
    }
    print_line(line, remainder, mode);
    let bytes_read = buffer.len();
    reader.consume(bytes_read);
}

fn print_line(line: usize, bytes: &[u8], mode: Mode) {
    match mode {
        Mode::Graphical => {
            let hex = format_hex(bytes, true);
            let text = format_text(bytes);
            println!("{line:0>7x}0: {hex} {text}");
        }
        Mode::Hex => print!("{}", format_hex(bytes, false)),
        Mode::Reverse => {
            let mut out = io::stdout().lock();
            if let Err(err) = out.write_all(&parse_bytes(bytes)) {
                log::error!("{}", err);
            }
        }
    }
}

fn format_text(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|byte| {
            if byte.is_ascii_graphic() {
                *byte as char
            } else {
                '.'
            }
        })
        .collect()
}

fn format_hex(bytes: &[u8], add_space: bool) -> String {
    let pad_right = add_space;
    let space = if add_space { " " } else { "" };
    let mut result: String = bytes
        .iter()
        .map(|byte| format!("{byte:0>2x}{space}"))
        .into_iter()
        .collect();
    if pad_right {
        let hex_width = 16 * 3;
        if result.len() < hex_width {
            result.push_str(&" ".repeat(hex_width - result.len()));
        }
    }
    result
}

fn parse_bytes(bytes: &[u8]) -> Vec<u8> {
    let mut ignore_to_end_of_line = false;
    bytes
        .iter()
        .filter_map(|byte| {
            if *byte == b'\n' {
                ignore_to_end_of_line = false
            }
            if *byte == b';' || *byte == b'/' {
                ignore_to_end_of_line = true
            }
            if !ignore_to_end_of_line && byte.is_ascii_hexdigit() {
                Some(*byte as char)
            } else {
                None
            }
        })
        .array_chunks::<2>()
        .map(|[first, second]| {
            let hex_chars = format!("{}{}", first, second);
            let parsed = u8::from_str_radix(&hex_chars, 16)
                .expect("In reverse mode only [0-9a-fA-F] chars are allowed");
            parsed
        })
        .collect()
}

struct Config {
    mode: Mode,
    input: Option<String>,
}

impl Config {
    pub fn new() -> Config {
        let mut defaults = HashMap::new();
        defaults.insert("mode", "graphical");
        defaults.insert("input", "");
        let conf = fusion(defaults, None);
        let mode = match conf.get("mode").unwrap().as_str() {
            "graphical" => Mode::Graphical,
            "hex" => Mode::Hex,
            "reverse" => Mode::Reverse,
            _ => unreachable!(),
        };
        let input = conf.get("input").unwrap();
        let input = if input.is_empty() {
            None
        } else {
            Some(input.clone())
        };
        Config { mode, input }
    }
}

#[derive(Debug, Clone, Copy)]
enum Mode {
    Graphical,
    Hex,
    Reverse,
}

#[cfg(test)]
mod tests {
    use crate::*;

    #[test]
    fn test_reverse_mode() {
        static HEX_WITH_ANNOTATIONS: &'static [u8; 1381] =
            include_bytes!("../hex_with_annotations");
        static HEX: &'static [u8; 273] = include_bytes!("../hex");
        static EXECUTABLE: &'static [u8; 136] = include_bytes!("../executable");
        assert_eq!(EXECUTABLE, parse_bytes(HEX_WITH_ANNOTATIONS).as_slice());
        assert_eq!(EXECUTABLE, parse_bytes(HEX).as_slice());
    }
}
