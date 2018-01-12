extern crate colored;
use colored::*;

use std::env::args;
use std::io;
use std::fs::File;
use std::fs;
use std::path::Path;
use std::io::prelude::*;

fn is_text(file_path: &Path) -> bool {
    if let Ok(mut file_handle) = File::open(file_path) {
        let mut buffer = [0;512];
        if let Ok(readed_size) = file_handle.read(&mut buffer) {
            if readed_size == 0 {
                return false;
            }
            let mut content = &buffer[0..readed_size];
            if readed_size >= 3 && content[0] == 0xEF && content[1] == 0xBB && content[2] == 0xBF {
                return true;
            }

            if readed_size >= 5 && "%PDF-".as_bytes() == &content[0..5] {
                return false;
            }

            let mut i = 0;
            while i <  readed_size {
                if content[i] == '\0' as u8 {
                    return false;
                } else if (content[i] < 7 || content[i] > 14) && (content[i] < 32 || content[i] > 127) {
                    if content[i] > 193 && content[i] < 224 && i + 1 < readed_size {
                        i += 1;
                        if content[i] > 127 && content[i] < 192 {
                            continue;
                        }
                    } else if content[i] > 223 && content[i] < 240 && i + 2 < readed_size {
                        i += 1;
                        if content[i] > 127 && content[i] < 192 && content[i+1] > 127 && content[i+1] < 192 {
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

                        if  switcher {
                            let v: Vec<&str> = ln.as_str().split(pt).collect();
                            let v_len = v.len();
                            print!("{}:", line_num);
                            for i in 1..v_len+1 { 
                                if i == v_len {
                                    println!("{}", &v[i-1]);
                                } else {
                                    print!("{}", &v[i-1]);
                                    print!("{}", pt.red().purple().magenta().bold());
                                }
                            }
                        }
                    } else {
                        ()
                    }
                },
                Err(_) => ()
            }
        }
    }
}

fn main() {
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        println!("usage: sf pattern-string root-directory");
        ()
    }
    let pattern_str = &args[1];
    let root_dir = &args[2];

    let pt = Path::new(&root_dir);
    walk_through_dir(&pt, &pattern_str, &call_back).unwrap();
}

fn walk_through_dir(dir: &Path,
                    pattern_str: &String,
                    cb: &Fn(&Path, &String)) -> io::Result<()> 
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            walk_through_dir(&path, pattern_str, cb)?;
        }
    } else {
        cb(&dir, pattern_str);
    }

    Ok(())
}
