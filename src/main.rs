#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::Parser;
use colored::Colorize;
use crossbeam_channel::{bounded, Sender};
use ignore::{WalkBuilder, WalkState};
use memchr::{memchr, memrchr, memmem};
use std::fs::File;
use std::io::{self, BufRead, BufWriter, IsTerminal, Read, Write};
use std::path::{Path, PathBuf};
use std::thread;

#[derive(Parser, Debug)]
#[command(
    name = "sss",
    version = "5.0.0",
    about = "A blazing fast tool to search for patterns in files or stdin."
)]
struct Cli {
    pattern: String,
    path: Option<PathBuf>,
    #[arg(short = 't', long = "type")]
    file_ext: Option<String>,
}

#[inline]
fn write_line_number(buf: &mut Vec<u8>, line_num: usize, itoa_buf: &mut itoa::Buffer) {
    let s = itoa_buf.format(line_num);
    let padding = 6usize.saturating_sub(s.len());

    const SPACES: &[u8] = b"      ";
    buf.extend_from_slice(&SPACES[..padding]);
    buf.extend_from_slice(b"\x1b[36m");
    buf.extend_from_slice(s.as_bytes());
    buf.extend_from_slice(b":\x1b[0m ");
}

#[inline]
fn highlight_line(
    buf: &mut Vec<u8>,
    line: &[u8],
    pattern_len: usize,
    colored_pattern: &[u8],
    finder: &memmem::Finder,
) {
    let mut last = 0;
    for start in finder.find_iter(line) {
        buf.extend_from_slice(&line[last..start]);
        buf.extend_from_slice(colored_pattern);
        last = start + pattern_len;
    }
    buf.extend_from_slice(&line[last..]);
    buf.push(b'\n');
}

fn process_stdin(pattern_len: usize, colored_pattern: &[u8], finder: &memmem::Finder) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let stdout = io::stdout();
    let mut out = BufWriter::with_capacity(256 * 1024, stdout.lock());

    let mut buffer = Vec::with_capacity(1024);
    let mut line_num: usize = 0;
    let mut itoa_buf = itoa::Buffer::new();
    let mut line_buf: Vec<u8> = Vec::with_capacity(1024);

    loop {
        buffer.clear();
        match handle.read_until(b'\n', &mut buffer) {
            Ok(0) => break,
            Ok(_) => {}
            Err(_) => break,
        }
        line_num += 1;

        if finder.find(&buffer).is_some() {
            let line_bytes = if buffer.ends_with(&[b'\n']) {
                let len = buffer.len();
                if len > 1 && buffer[len - 2] == b'\r' {
                    &buffer[..len - 2]
                } else {
                    &buffer[..len - 1]
                }
            } else {
                &buffer[..]
            };

            line_buf.clear();
            write_line_number(&mut line_buf, line_num, &mut itoa_buf);
            highlight_line(&mut line_buf, line_bytes, pattern_len, colored_pattern, finder);
            out.write_all(&line_buf).unwrap();
        }
    }
    out.flush().unwrap();
}

fn search_file(
    path: &Path,
    pattern_len: usize,
    colored_pattern: &[u8],
    finder: &memmem::Finder,
    tx: &Sender<Vec<u8>>,
) {
    let mut file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };

    let file_len = file.metadata().map(|m| m.len()).unwrap_or(0);
    if file_len == 0 {
        return;
    }

    const MMAP_THRESHOLD: u64 = 64 * 1024;

    let mut small_buf = Vec::new();
    let mmap_handle;

    let content: &[u8] = if file_len <= MMAP_THRESHOLD {
        small_buf.reserve(file_len as usize);
        if file.read_to_end(&mut small_buf).is_err() {
            return;
        }
        &small_buf
    } else {
        mmap_handle = match unsafe { memmap2::Mmap::map(&file) } {
            Ok(m) => m,
            Err(_) => return,
        };
        #[cfg(unix)]
        {
            let _ = mmap_handle.advise(memmap2::Advice::Sequential);
        }
        &mmap_handle
    };

    let check_len = content.len().min(1024);
    if memchr(b'\0', &content[..check_len]).is_some() {
        return;
    }

    let mut out_buffer: Vec<u8> = Vec::new();
    let mut file_name_printed = false;
    let mut search_pos: usize = 0;
    let mut line_count_pos: usize = 0;
    let mut line_num: usize = 1;
    let mut itoa_buf = itoa::Buffer::new();

    while search_pos < content.len() {
        let rel = match finder.find(&content[search_pos..]) {
            Some(p) => p,
            None => break,
        };
        let match_pos = search_pos + rel;

        let line_start = if search_pos < match_pos {
            match memrchr(b'\n', &content[search_pos..match_pos]) {
                Some(p) => search_pos + p + 1,
                None => search_pos,
            }
        } else {
            search_pos
        };

        let line_end = match memchr(b'\n', &content[match_pos..]) {
            Some(p) => match_pos + p,
            None => content.len(),
        };

        if line_start > line_count_pos {
            line_num += memchr::memchr_iter(b'\n', &content[line_count_pos..line_start]).count();
        }

        if !file_name_printed {
            out_buffer.reserve(4096);
            #[cfg(unix)]
            {
                use std::os::unix::ffi::OsStrExt;
                out_buffer.extend_from_slice(b"\x1b[1;32m");
                out_buffer.extend_from_slice(path.as_os_str().as_bytes());
                out_buffer.extend_from_slice(b"\x1b[0m\n");
            }
            #[cfg(not(unix))]
            {
                write!(&mut out_buffer, "\x1b[1;32m{}\x1b[0m\n", path.display()).unwrap();
            }
            file_name_printed = true;
        }

        write_line_number(&mut out_buffer, line_num, &mut itoa_buf);
        let line = &content[line_start..line_end];
        highlight_line(&mut out_buffer, line, pattern_len, colored_pattern, finder);

        search_pos = line_end + 1;
        line_count_pos = search_pos;
        line_num += 1;
    }

    if !out_buffer.is_empty() {
        let _ = tx.send(out_buffer);
    }
}

fn search_path(
    path: &Path,
    pattern_len: usize,
    colored_pattern: &'static [u8],
    finder_pattern: &'static str,
    file_ext: Option<&'static str>,
) {
    let (tx, rx) = bounded::<Vec<u8>>(1024);

    let printer_thread = thread::spawn(move || {
        let stdout = io::stdout();
        let mut handle = BufWriter::with_capacity(256 * 1024, stdout.lock());
        while let Ok(buf) = rx.recv() {
            handle.write_all(&buf).unwrap_or_default();
        }
        handle.flush().unwrap_or_default();
    });

    let builder = WalkBuilder::new(path);
    let tx_for_closure = tx.clone();

    builder.build_parallel().run(move || {
        let finder = memmem::Finder::new(finder_pattern);
        let thread_tx = tx_for_closure.clone();

        Box::new(move |result| {
            if let Ok(entry) = result {
                if entry.file_type().map_or(false, |ft| ft.is_file()) {
                    let path = entry.path();
                    let ext_match = file_ext.map_or(true, |ext| {
                        path.extension().and_then(|s| s.to_str()) == Some(ext)
                    });
                    if ext_match {
                        search_file(path, pattern_len, colored_pattern, &finder, &thread_tx);
                    }
                }
            }
            WalkState::Continue
        })
    });

    drop(tx);
    printer_thread.join().unwrap();
}

fn main() {
    let cli = Cli::parse();

    let pattern_static: &'static str = Box::leak(cli.pattern.into_boxed_str());
    let file_ext_static: Option<&'static str> =
        cli.file_ext.map(|s| Box::leak(s.into_boxed_str()) as &'static str);

    let colored_pattern: &'static [u8] =
        Box::leak(pattern_static.red().bold().to_string().into_boxed_str()).as_bytes();
    let pattern_len = pattern_static.len();

    let is_pipe = !io::stdin().is_terminal();

    match cli.path {
        Some(path) => {
            search_path(&path, pattern_len, colored_pattern, pattern_static, file_ext_static);
        }
        None => {
            if is_pipe {
                let finder = memmem::Finder::new(pattern_static);
                process_stdin(pattern_len, colored_pattern, &finder);
            } else {
                let current_dir = PathBuf::from(".");
                search_path(
                    &current_dir,
                    pattern_len,
                    colored_pattern,
                    pattern_static,
                    file_ext_static,
                );
            }
        }
    }
}