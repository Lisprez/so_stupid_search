extern crate colored;
extern crate isatty;
use colored::*;

use std::collections::VecDeque;
use std::env::args;
use std::ffi::OsString;
use std::fs;
use std::fs::File;
use std::io;
use std::io::prelude::*;
use std::path::{Path, PathBuf};
use isatty::{stdin_isatty};

fn is_text(file_path: &Path) -> bool {
    if let Ok(mut file_handle) = File::open(file_path) {
        let mut buffer = [0; 512];
        if let Ok(readed_size) = file_handle.read(&mut buffer) {
            if readed_size == 0 {
                return false;
            }
            let content = &buffer[0..readed_size];
            if readed_size >= 3 && content[0] == 0xEF && content[1] == 0xBB && content[2] == 0xBF {
                return true;
            }

            if readed_size >= 5 && "%PDF-".as_bytes() == &content[0..5] {
                return false;
            }

            let mut i = 0;
            while i < readed_size {
                if content[i] == '\0' as u8 {
                    return false;
                } else if (content[i] < 7 || content[i] > 14)
                    && (content[i] < 32 || content[i] > 127)
                {
                    if content[i] > 193 && content[i] < 224 && i + 1 < readed_size {
                        i += 1;
                        if content[i] > 127 && content[i] < 192 {
                            continue;
                        }
                    } else if content[i] > 223 && content[i] < 240 && i + 2 < readed_size {
                        i += 1;
                        if content[i] > 127
                            && content[i] < 192
                            && content[i + 1] > 127
                            && content[i + 1] < 192
                        {
                            i += 1;
                            continue;
                        }
                    }
                }

                i += 1;
            }
        }
        true
    } else {
        false
    }
}

fn call_back(de: &Path, pt: &String) {
    if is_text(de) {
        let mut switcher = false;
        let mut line_num = 0;
        let f = File::open(de).unwrap();
        let buf = io::BufReader::new(f);
        for line in io::BufRead::lines(buf) {
            line_num += 1;
            match line {
                Ok(ln) => {
                    if ln.as_str().contains(pt) {
                        if !switcher {
                            switcher = true;
                            if let Some(path_str) = de.to_str() {
                                println!("{}", path_str.green().bold());
                            }
                        }

                        if switcher {
                            let v: Vec<&str> = ln.as_str().split(pt).collect();
                            let v_len = v.len();
                            print!("{num:->6}: ", num=line_num);
                            for i in 1..v_len + 1 {
                                if i == v_len {
                                    println!("{}", &v[i - 1]);
                                } else {
                                    print!("{}", &v[i - 1]);
                                    print!("{}", pt.red().purple().magenta().bold());
                                }
                            }
                        }
                    } else {
                        ()
                    }
                }
                Err(_) => (),
            }
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if (args.len() == 2 && stdin_isatty()) || (args.len() != 2 && args.len() != 3 && args.len() != 5) {
        println!("usage:   sss pattern-string root-directory             在指定目录root-directory下面搜索pattern-string");
        println!("         sss -t file_ext pattern-string root-directory 在指定目录root-directory下面对扩展名是file_ext的文件搜索pattern-string");
        println!("         sss pattern-string root-directory -t file_ext 在指定目录root-directory下面对扩展名是file_ext的文件搜索pattern-string");
        println!("         command | sss pattern-string                  对命令command的输出进行pattern-string搜索");
        println!("version: 3.4.1");
        println!(r#"eg:      sss "func main(" ./src"#);
        println!(r#"eg:      sss -t go "func main(" ./src"#);
        println!(r#"eg:      sss "func main(" ./src -t cpp"#);
        println!(r#"eg:      somme_command | sss pattern-string"#);
        return;
    }

    if args.len() == 2 && !stdin_isatty() {
        let buf = io::BufReader::new(std::io::stdin());
        for line in io::BufRead::lines(buf) {
            match line {
                Ok(ln) => {
                    if ln.contains(args[1].as_str()) {
                        let v: Vec<&str> = ln.split(args[1].as_str()).collect();
                        let v_len = v.len();
                        for i in 1..v_len + 1 {
                            if i == v_len {
                                println!("{}", &v[i - 1]);
                            } else {
                                print!("{}", &v[i - 1]);
                                print!("{}", args[1].red().purple().magenta().bold());
                            }
                        }
                    } else {
                        ()
                    }
                }
                Err(_) => (),
            }
        }
        return
    }

    let mut queue: VecDeque<PathBuf> = VecDeque::new();
    if args.len() == 3 {
        let pattern_str = &args[1];
        let root_dir = &args[2];

        let pt = Path::new(&root_dir);
        let v = find_match(pt, &"".to_string(), pattern_str, &call_back);
        for element in v {
            queue.push_back(element);
        }
        while queue.len() != 0 {
            match queue.pop_front() {
                Some(path_buf) => {
                    for element in
                        find_match(path_buf.as_path(), &"".to_string(), pattern_str, &call_back)
                    {
                        queue.push_back(element);
                    }
                }
                None => (),
            }
        }
    } else {
        if args.contains(&"-t".to_string()) {
            if let Some(index) = args.iter().position(|&ref r| *r == "-t".to_string()) {
                if index == 1 {
                    let pattern_str = &args[3];
                    let root_dir = &args[4];
                    let pt = Path::new(&root_dir);
                    let v = find_match(pt, &args[2], pattern_str, &call_back);
                    for element in v {
                        queue.push_back(element);
                    }
                    while queue.len() != 0 {
                        match queue.pop_front() {
                            Some(path_buf) => {
                                for element in find_match(
                                    path_buf.as_path(),
                                    &args[2],
                                    pattern_str,
                                    &call_back,
                                ) {
                                    queue.push_back(element);
                                }
                            }
                            None => (),
                        }
                    }
                } else if index == 3 {
                    let pattern_str = &args[1];
                    let root_dir = &args[2];
                    let pt = Path::new(&root_dir);
                    let v = find_match(pt, &args[4], pattern_str, &call_back);
                    for element in v {
                        queue.push_back(element);
                    }
                    while queue.len() != 0 {
                        match queue.pop_front() {
                            Some(path_buf) => {
                                for element in find_match(
                                    path_buf.as_path(),
                                    &args[4],
                                    pattern_str,
                                    &call_back,
                                ) {
                                    queue.push_back(element);
                                }
                            }
                            None => (),
                        }
                    }
                } else {
                }
            }
        }
    }
}

fn find_match(
    root_dir: &Path,
    file_ext: &String,
    pattern_str: &String,
    cb: &dyn Fn(&Path, &String),
) -> Vec<PathBuf> {
    let mut vec: Vec<PathBuf> = vec![];

    match fs::read_dir(root_dir) {
        Ok(iterator_obj) => {
            for entry in iterator_obj {
                match entry {
                    Ok(ref dir_entry) => {
                        if !dir_entry.path().as_path().is_dir() {
                            if *file_ext == "".to_string() {
                                cb(dir_entry.path().as_path(), pattern_str);
                            } else {
                                if let Some(ext_name) = dir_entry.path().as_path().extension() {
                                    if OsString::from(file_ext) == ext_name {
                                        cb(dir_entry.path().as_path(), pattern_str);
                                    } else {
                                    }
                                } else {
                                }
                            }
                        } else {
                            vec.push(dir_entry.path());
                        }
                    }
                    Err(_) => (),
                }
            }
        }
        Err(_) => (),
    }
    vec
}
