// vimtcompile:cargo build

use std::{env::args, fs, process::exit};

use header_deper::{display_graph, walker::IncludeWalker};

fn help() {
    println!("Usage: {} <file> [search_paths]", env!("CARGO_BIN_NAME"));
}

fn main() {
    let mut filename = None;
    let mut search_paths = Vec::new();
    for arg in args().skip(1) {
        if let None = filename {
            filename = Some(arg);
        } else {
            search_paths.push(arg);
        }
    }

    if let Some(filename) = filename {
        if !fs::exists(&filename).unwrap_or(false) {
            eprintln!("File does not exist: {}", filename);
            exit(-1);
        }

        let mut walker = IncludeWalker::new();
        for path in search_paths {
            walker.append_dir(path);
        }
        walker.walk(filename);
        display_graph(&walker);
    } else {
        help();
        exit(-1);
    }
}

