#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use colored::Colorize;
use memchr::memmem;
use std::collections::VecDeque;
use std::env::args;
use std::fs::{self, File};
use std::io::{self, BufRead, Read};
use std::path::{Path, PathBuf};
use atty::Stream;


fn is_text(file_path: &Path) -> bool {
    let mut buffer = [0; 512];
    if let Ok(mut file_handle) = File::open(file_path) {
        if let Ok(read_size) = file_handle.read(&mut buffer) {
            if read_size == 0 {
                return false;
            }
            let content = &buffer[..read_size];

            // UTF-8 BOM check
            if read_size >= 3 && content.starts_with(&[0xEF, 0xBB, 0xBF]) {
                return true;
            }

            // PDF file check
            if read_size >= 5 && content.starts_with(b"%PDF-") {
                return false;
            }

            // Non-text character checks
            for &byte in content {
                if byte == 0 || ((byte < 7 || byte > 14) && (byte < 32 || byte > 127)) {
                    if !is_valid_utf8_byte(byte, &content) {
                        return false;
                    }
                }
            }
        }
    }
    true
}

fn is_valid_utf8_byte(byte: u8, content: &[u8]) -> bool {
    match byte {
        193..=223 if content.len() >= 2 => content[1] > 127 && content[1] < 192,
        224..=239 if content.len() >= 3 => content[1] > 127 && content[1] < 192 && content[2] > 127 && content[2] < 192,
        _ => true,
    }
}

fn call_back(file: &Path, pattern: &str) {
    if is_text(file) {
        let mut printed_header = false;
        let finder = memmem::Finder::new(pattern);
        let file_handle = File::open(file).unwrap();
        let reader = io::BufReader::new(file_handle);

        for (line_num, line) in reader.lines().enumerate() {
            if let Ok(ln) = line {
                if finder.find(ln.as_bytes()).is_some() {
                    if !printed_header {
                        if let Some(path_str) = file.to_str() {
                            println!("{}", path_str.green().bold());
                        }
                        printed_header = true;
                    }

                    let parts: Vec<&str> = ln.split(pattern).collect();
                    print!("{:>6}: ", line_num + 1);
                    for (i, part) in parts.iter().enumerate() {
                        if i > 0 {
                            print!("{}", pattern.red().bold());
                        }
                        print!("{}", part);
                    }
                    println!();
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if invalid_arguments(&args) {
        print_usage();
        return;
    }
    if args.len() == 2 && !atty::is(Stream::Stdin) {
        process_stdin(&args[1]);
        return;
    }

    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    if args.len() == 3 {
        let pattern = &args[1];
        let root_dir = Path::new(&args[2]);
        enqueue_files(root_dir, "", pattern, &mut queue);

        process_queue(&mut queue, pattern, "");
    } else if let (Some(ext), pos) = extract_extension(&args) {
        let pattern;
        let root_dir;
        if pos == 1 {
            pattern = &args[3];
            root_dir = Path::new(&args[4]);
        } else {
            pattern = &args[1];
            root_dir = Path::new(&args[2]);
        }
        enqueue_files(root_dir, &ext, pattern, &mut queue);

        process_queue(&mut queue, pattern, &ext);
    }
}

fn invalid_arguments(args: &[String]) -> bool {
    ((args.len() == 2) && atty::is(Stream::Stdin)) || (args.len() != 2 && args.len() != 3 && args.len() != 5)
}

fn print_usage() {
    println!("usage:   sss pattern-string root-directory");
    println!("         sss -t file_ext pattern-string root-directory");
    println!("         sss pattern-string root-directory -t file_ext");
    println!("         command | sss pattern-string");
    println!("version: 3.4.3");
    println!(r#"eg:      sss "func main(" ./src"#);
    println!(r#"eg:      sss -t go "func main(" ./src"#);
    println!(r#"eg:      sss "func main(" ./src -t cpp"#);
    println!(r#"eg:      some_command | sss "pattern-string""#);
}

fn process_stdin(pattern: &str) {
    let reader = io::BufReader::new(io::stdin());
    for line in reader.lines() {
        if let Ok(ln) = line {
            let parts: Vec<&str> = ln.split(pattern).collect();
            if parts.len() > 1 {
                for (i, part) in parts.iter().enumerate() {
                    if i > 0 {
                        print!("{}", pattern.red().bold());
                    }
                    print!("{}", part);
                }
                println!();
            }
        }
    }
}

fn enqueue_files(root_dir: &Path, file_ext: &str, pattern: &str, queue: &mut VecDeque<PathBuf>) {
    if let Ok(entries) = fs::read_dir(root_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                queue.push_back(path);
            } else if file_ext.is_empty() || path.extension().and_then(|ext| ext.to_str()) == Some(file_ext) {
                call_back(&path, pattern);
            }
        }
    }
}

fn process_queue(queue: &mut VecDeque<PathBuf>, pattern: &str, file_ext: &str) {
    while let Some(path) = queue.pop_front() {
        enqueue_files(&path, file_ext, pattern, queue);
    }
}

fn extract_extension(args: &[String]) -> (Option<String>, i32) {
    if args.contains(&"-t".to_string()) {
        if let Some(pos) = args.iter().position(|r| r == "-t") {
            if pos == 1 {
                return (Some(args[2].clone()), 1);
            } else if pos == 3 {
                return (Some(args[4].clone()), 3);
            }
        }
    }
    (None, -1)
}