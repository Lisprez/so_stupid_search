extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

extern crate term;

use std::env::args;
use std::io;
use std::fs::File;
use std::fs;
use std::path::Path;


fn call_back(de: &Path, btable: &mut Table, pt: &String) {
    let mut line_num = 0;
    let f = File::open(de).unwrap();
    let buf = io::BufReader::new(f);
    for line in io::BufRead::lines(buf) {
        line_num += 1;
        match line {
            Ok(ln) => {
                if ln.as_str().contains(pt) {
                    let tmp = ln.as_str();
                    let mut vec: Vec<Cell> = vec![];
                    vec.push(Cell::new(de.file_name().unwrap().to_str().unwrap()).style_spec("Fy"));
                    vec.push(Cell::new(line_num.to_string().as_str()).style_spec("Fr"));
                    vec.push(Cell::new(&tmp).style_spec("Fg"));
                    btable.add_row(Row::new(vec));
                } else {
                    ()
                }
            },
            Err(_) => ()
        }
    }
}

fn main() {
    let name_prefix: Vec<&str> = vec!["c", "C", "cpp", "cxx", "CXX", "h", "hpp", "rs", "rb", "py", "java", "txt", "xml", "json", "js", "hs", "toml", "ini", "yml", "yaml"];
    let mut table: Table = Table::new();
    table.add_row(Row::new(vec![Cell::new("file").style_spec("c"), 
                                Cell::new("line").style_spec("c"),
                                Cell::new("content").style_spec("c")]));
    let args: Vec<String> = args().collect();
    if args.len() != 3 {
        println!("usage: sf pattern-string root-directory");
        ()
    }
    let pattern_str = &args[1];
    let root_dir = &args[2];

    let pt = Path::new(&root_dir);
    walk_through_dir(&pt, &pattern_str, &name_prefix, &mut table, &call_back);
    if table.len() != 1 {
        table.printstd();
    }
}

fn walk_through_dir(dir: &Path,
                    pattern_str: &String,
                    name_prefix: &Vec<&str>,
                    btable: &mut Table,
                    cb: &Fn(&Path, &mut Table, &String)) -> io::Result<()> 
{
    if dir.is_dir() {
        for entry in fs::read_dir(dir)? {
            let entry = entry?;
            let path = entry.path();
            walk_through_dir(&path, pattern_str, name_prefix, btable, cb)?;
        }
    } else {
        match dir.extension() {
            Some(extension_name) => {
                let extension_name_str = extension_name.to_str();
                match extension_name_str {
                    Some(ref final_name) => {
                        if name_prefix.contains(final_name) {
                            cb(&dir, btable, pattern_str);
                        } else {
                            ()
                        }
                    },
                    None => ()
                }
            },
            None => ()
        }
    }

    Ok(())
}
