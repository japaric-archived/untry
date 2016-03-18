#![feature(question_mark)]

use std::borrow::Cow;
use std::env;
use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;

fn main() {
    for ref path in env::args_os().skip(1) {
        print!("{:?} ... ", path);

        match untry_file(path) {
            Ok(_) => println!("OK"),
            Err(e) => println!("error: {}", e),
        }
    }
}

fn untry_file<P>(path: P) -> io::Result<()>
    where P: AsRef<Path>
{
    fn inner(path: &Path) -> io::Result<()> {
        let input = &mut String::new();
        File::open(path)?.read_to_string(input)?;

        if input.find("try!(").is_none() {
            return Ok(());
        }

        OpenOptions::new()
            .write(true)
            .truncate(true)
            .open(path)?
            .write_all(untry_lines(input).as_bytes())
    }

    inner(path.as_ref())
}

fn untry_lines(text: &str) -> String {
    let mut out = String::with_capacity(text.len());

    for line in text.lines() {
        out.push_str(&untry_all(line));
        out.push('\n');
    }

    out
}

/// Converts all the `try!()`s in `line` into `?`s
fn untry_all(line: &str) -> Cow<str> {
    match untry_once(line) {
        Cow::Borrowed(line) => Cow::Borrowed(line),
        Cow::Owned(line) => {
            match untry_all(&line) {
                Cow::Borrowed(_) => Cow::Owned(line),
                Cow::Owned(line) => Cow::Owned(line),
            }
        }
    }
}

/// Converts the first/outer `try!()` of `line` into a `?`
fn untry_once(line: &str) -> Cow<str> {
    // ```
    // foo(try!(bar()));
    //                ^~ after
    //               ^~ end
    //          ^~ start
    //     ^~ before
    // ```
    if let Some(before) = line.find("try!(") {
        let start = before + "try!(".len();
        let mut end = None;
        let mut nparens = 0;

        for (offset, c) in line[start..].char_indices() {
            match c {
                '(' => nparens += 1,
                ')' if nparens == 0 => {
                    end = Some(start + offset);
                    break;
                }
                ')' => nparens -= 1,
                _ => {}
            }

        }

        if let Some(end) = end {
            let inside = start..end;
            let after = end + ")".len();
            let mut out = String::with_capacity(line.len());

            out.push_str(&line[..before]);
            out.push_str(&line[inside]);
            out.push('?');
            out.push_str(&line[after..]);

            Cow::from(out)
        } else {
            Cow::from(line)
        }
    } else {
        Cow::from(line)
    }
}
