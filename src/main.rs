#[global_allocator]
static GLOBAL: mimalloc::MiMalloc = mimalloc::MiMalloc;

use clap::{Parser};
use colored::Colorize;
use memchr::{memchr, memmem};
use memmap2::Mmap;
use rayon::prelude::*;
use std::fs::File;
use std::io::{self, BufRead, Write};
use std::path::{Path, PathBuf};
use walkdir::WalkDir;
use atty::Stream;

#[derive(Parser, Debug)]
#[command(name = "sss", version = "4.0.3", author = "Optimized by Google Advanced Compiler Expert", about = "A blazing fast tool to search for patterns in files or stdin.")]
struct Cli {
    /// 要搜索的模式字符串
    pattern: String,

    /// 要搜索的路径。如果省略，则根据是否有管道输入来决定是处理stdin还是搜索当前目录
    path: Option<PathBuf>,

    /// 按文件扩展名过滤 (例如 "rs", "cpp")
    #[arg(short = 't', long = "type")]
    file_ext: Option<String>,
}

// --- process_stdin 和 search_file 函数保持不变 ---

/// 高效处理标准输入流的函数
fn process_stdin(pattern: &str, finder: &memmem::Finder) {
    let stdin = io::stdin();
    let mut handle = stdin.lock();
    let mut line_num = 0;
    let mut buffer = String::new();

    while let Ok(bytes_read) = handle.read_line(&mut buffer) {
        if bytes_read == 0 {
            break;
        }
        line_num += 1;

        if finder.find(buffer.as_bytes()).is_some() {
            let mut last_match_end = 0;
            let stdout = io::stdout();
            let mut stdout_handle = stdout.lock();

            write!(stdout_handle, "{:>6}: ", line_num.to_string().cyan()).unwrap();
            
            let line_bytes = buffer.trim_end().as_bytes();
            for start in finder.find_iter(line_bytes) {
                stdout_handle.write_all(&line_bytes[last_match_end..start]).unwrap();
                write!(stdout_handle, "{}", pattern.red().bold()).unwrap();
                last_match_end = start + pattern.len();
            }
            stdout_handle.write_all(&line_bytes[last_match_end..]).unwrap();
            writeln!(stdout_handle).unwrap();
        }

        buffer.clear();
    }
}

/// 对单个文件进行搜索的核心逻辑
fn search_file(path: &Path, pattern: &str, finder: &memmem::Finder) {
    let file = match File::open(path) {
        Ok(f) => f,
        Err(_) => return,
    };
    let mmap: Mmap = match unsafe { memmap2::Mmap::map(&file) } {
        Ok(m) => m,
        Err(_) => return,
    };
    if memchr(b'\0', &mmap).is_some() {
        return;
    }
    let mut output_lines: Vec<String> = Vec::new();
    for (line_num, line) in mmap.split(|&b| b == b'\n').enumerate() {
        if finder.find(line).is_some() {
            if output_lines.is_empty() {
                output_lines.push(path.display().to_string().green().bold().to_string());
            }
            let mut highlighted_line = format!("{:>6}: ", (line_num + 1).to_string().cyan());
            let mut last_match_end = 0;
            for start in finder.find_iter(line) {
                if let Ok(s) = std::str::from_utf8(&line[last_match_end..start]) {
                    highlighted_line.push_str(s);
                }
                highlighted_line.push_str(&pattern.red().bold().to_string());
                last_match_end = start + pattern.len();
            }
            if let Ok(s) = std::str::from_utf8(&line[last_match_end..]) {
                highlighted_line.push_str(s);
            }
            output_lines.push(highlighted_line);
        }
    }
    if !output_lines.is_empty() {
        let stdout = io::stdout();
        let mut handle = stdout.lock();
        for line in output_lines {
            writeln!(handle, "{}", line).unwrap_or_default();
        }
    }
}

/// 将文件系统搜索逻辑封装成一个函数
fn search_path(path: &Path, pattern: &str, finder: &memmem::Finder, file_ext: Option<&str>) {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .filter(move |e| {
            file_ext.map_or(true, |ext| {
                e.path().extension().and_then(|s| s.to_str()) == Some(ext)
            })
        })
        .par_bridge()
        .for_each(|entry| {
            search_file(entry.path(), pattern, finder);
        });
}


// ######################################################
// #                主函数逻辑更新                        #
// ######################################################
fn main() {
    let cli = Cli::parse();
    let pattern = &cli.pattern;
    let file_ext = cli.file_ext.as_deref();
    let finder = memmem::Finder::new(pattern);

    // 检查是否有管道输入
    let is_pipe = !atty::is(Stream::Stdin);

    // 根据我们新的优先级规则进行逻辑分发
    match cli.path {
        // 1. 显式路径优先
        Some(path) => {
            search_path(&path, pattern, &finder, file_ext);
        }
        // 2. 没有提供路径
        None => {
            if is_pipe {
                // 2a. 如果有管道输入，则处理 stdin
                process_stdin(pattern, &finder);
            } else {
                // 2b. 如果没有管道输入，则默认搜索当前目录
                let current_dir = PathBuf::from(".");
                search_path(&current_dir, pattern, &finder, file_ext);
            }
        }
    }
}