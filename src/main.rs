#[macro_use] extern crate prettytable;
use prettytable::Table;
use prettytable::row::Row;
use prettytable::cell::Cell;

extern crate term;

use std::env::args;
use std::io;
use std::fs::{File, DirEntry};
use std::fs;
use std::path::Path;


fn call_back(de: &DirEntry, btable: &mut Table, pt: &String) 
{
    let mut line_num = 0;
    let f = File::open(de.path()).unwrap();
    let buf = io::BufReader::new(f);
    for line in io::BufRead::lines(buf) 
    {
        line_num += 1;
        match line {
            Ok(ln) => {
                if ln.as_str().contains(pt) 
                {
                    let first_index = ln.as_str().find(pt).unwrap();
                    let pt_len = pt.len();
                    let tmp = ln.as_str();
                    let len = tmp.len();
                    let mut vec: Vec<Cell> = vec![];
                    vec.push(Cell::new(de.path().file_name().unwrap().to_str().unwrap()).style_spec("Fy"));
                    vec.push(Cell::new(line_num.to_string().as_str()).style_spec("Fr"));
                    if first_index > 5 
                    {

                        if len > (first_index + 3*pt_len) 
                        {
                            vec.push(Cell::new(&tmp[first_index - 5 .. first_index + 3*pt_len]).style_spec("Fg"));
                        } 
                        else 
                        {
                            vec.push(Cell::new(&tmp[first_index - 5 .. len-1]).style_spec("Fg"));
                        }
                    } 
                    else 
                    {
                        if len > (first_index + 3*pt_len) 
                        {
                            vec.push(Cell::new(&tmp[0 .. first_index + 3*pt_len]).style_spec("Fg"));
                        } 
                        else 
                        {
                            vec.push(Cell::new(&tmp[0 .. len - 1]).style_spec("Fg"));
                        }
                    }
                    btable.add_row(Row::new(vec));
                } 
                else 
                {
                    ()
                }
            },
            Err(_) => ()
        }
    }
}

fn main() 
{
    let name_prefix: Vec<&str> = vec!["c", "C", "cpp", "h", "hpp", "rs", "rb", "py", "java", "txt", "xml", "json", "js", "hs", "toml"];
    let mut table: Table = Table::new();
    table.add_row(Row::new(vec![Cell::new("file").style_spec("c"), 
                                Cell::new("line").style_spec("c"),
                                Cell::new("content").style_spec("c")]));
    let mut arg_iter = args();
    // 略过命令本身
    arg_iter.next();
    // panic if there is no one
    let pattern = arg_iter.next();
    let pattern_str = match pattern {
        Some(pattern_str) => pattern_str,
        None => {
            println!("Have not found the pattern string");
            return ();
        }
    };
    let pt = arg_iter.next().unwrap_or("./".to_string());
    let pt = Path::new(&pt);
    walk_through_dir(&pt, &pattern_str, &name_prefix, &mut table, &call_back).unwrap();
    if table.len() != 1 
    {
        table.printstd();
    }
}

fn walk_through_dir(dir: &Path,
                    pattern: &String,
                    name_prefix: &Vec<&str>,
                    btable: &mut Table,
                    cb: &Fn(&DirEntry, &mut Table, &String)) -> io::Result<()> 
{
    if try!(fs::metadata(dir)).is_dir() 
    {
        let dir_iter = fs::read_dir(dir);
        match dir_iter {
            Ok(dirs) => {
                for entry in dirs 
                {
                    match entry {
                        Ok(en) => {
                            if try!(fs::metadata(&en.path().as_path())).is_dir() 
                            {
                                try!(walk_through_dir(&en.path().as_path(), pattern, name_prefix, btable, cb));
                            } 
                            else 
                            {
                                match en.path().as_path().extension() 
                                {
                                    Some(extension_name) => {
                                        let extension_name_str = extension_name.to_str();
                                        match extension_name_str {
                                            Some(ref final_name) => {
                                                if name_prefix.contains(final_name) 
                                                {
                                                    cb(&en, btable, pattern);
                                                } 
                                                else 
                                                {
                                                    ()
                                                }
                                            },
                                            None => ()
                                        }
                                    },
                                    None => ()
                                }
                            }
                        },
                        Err(_) => ()
                    }
                }
            },
            Err(_) => (),
        }
    }
    Ok(())
}
