extern crate syntex_syntax as syntax;

use std::fs::{File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::Path;
use std::rc::Rc;
use std::result;
use std::{env, fmt};

use syntax::ast;
use syntax::codemap::{CodeMap, Loc, Span};
use syntax::errors::emitter::ColorConfig;
use syntax::errors::{DiagnosticBuilder, Handler};
use syntax::parse::{self, token, ParseSess};
use syntax::visit::{self, Visitor};

fn main() {
    let files = env::args_os().skip(1);
    let n = files.len();

    if n == 0 {
        return;
    }

    let stderr = io::stderr();
    let stderr = &mut stderr.lock();
    println!("Processing {} file{}", n, if n != 1 { "s" } else { "" });
    for file in files {
        let file_ = file.to_string_lossy();

        match untry(&file) {
            Err(e) => println!("{}: {}", file_, e),
            Ok(warnings) => {
                if warnings.len() == 0 {
                    println!("{}: OK", file_);
                } else {
                    println!("{}: {} warnings", file_, warnings.len());

                    for warning in warnings {
                        writeln!(stderr, "{}:{}:{} warning: multi-line try!", warning.file.name,
                                 warning.line, warning.col.0).ok();
                    }
                }
            }
        }
    }
}

enum Error {
    Bug(String),
    Io(io::Error),
    Parse,
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Error::Bug(ref try) => write!(f, "BUG: couldn't parse the delimiters of: {}", try),
            Error::Io(ref e) => e.fmt(f),
            Error::Parse => f.write_str("parse error"),
        }
    }
}

impl<'a> From<DiagnosticBuilder<'a>> for Error {
    fn from(_: DiagnosticBuilder<'a>) -> Error {
        Error::Parse
    }
}

impl From<io::Error> for Error {
    fn from(e: io::Error) -> Error {
        Error::Io(e)
    }
}

type Result<T> = result::Result<T, Error>;

/// Replaces almost all(*) the `try!`s in `file` with `?`s
///
/// (*) This function won't replace the `try!`s that are in doc comments or inside other macros.
fn untry<P>(file: P) -> Result<Warnings>
    where P: AsRef<Path>
{
    untry_(file.as_ref())
}

fn untry_(path: &Path) -> Result<Warnings> {
    let name = path.display().to_string();
    let mut source = String::new();
    File::open(path)?.read_to_string(&mut source)?;
    let mut warnings = Warnings::default();

    let mut source_was_modified = false;
    loop {
        // NOTE fast path to avoid calling the rust parser
        if !source.contains("try!") {
            break;
        }

        let codemap = Rc::new(CodeMap::new());

        let tty_handler = Handler::with_tty_emitter(ColorConfig::Auto,
                                                    None,
                                                    true,
                                                    false,
                                                    codemap.clone());

        let parse_session = ParseSess::with_span_handler(tty_handler, codemap.clone());

        let krate = parse::parse_crate_from_source_str(name.clone(),
                                                            source.clone(),
                                                            vec![],
                                                            &parse_session)?;

        let visitor = &mut TryVisitor::new(&name, &codemap);
        visit::walk_mod(visitor, &krate.module);

        if visitor.spans.is_empty() {
            break;
        }

        // NOTE Our parser-based approach doesn't handle nested `try!`s; it only peels off the outer
        // `try!`s. To handle nested `try!`s, we simply reparse the rewritten source.
        let (rewritten_source, new_warnings) = visitor.rewrite(&source)?;
        source = rewritten_source;

        // We only care about the warnings from the first rewrite. Subsequent rewrites will yield
        // warnings that (a) are duplicates and (b) have the wrong span
        if !source_was_modified {
            warnings = new_warnings;
        }

        source_was_modified = true;
    }

    if source_was_modified {
        OpenOptions::new().write(true).truncate(true).open(path)?
                 .write_all(source.as_bytes())?;
    }

    Ok(warnings)
}

/// We mark `try!`s that span multiple lines as warnings, because the user may want to modify the
/// transformed source to e.g. re-adjust the alignment of function arguments.
type Warnings = Vec<Loc>;

/// Stores the span of all the `try!` macros
struct TryVisitor<'a> {
    codemap: &'a CodeMap,
    name: &'a str,
    spans: Vec<Span>,
}

impl<'s, 'v> Visitor<'v> for TryVisitor<'s> {
    fn visit_mac(&mut self, mac: &'v ast::Mac) {
        let segments = &mac.node.path.segments;

        if segments.len() == 1 &&
            segments[0].identifier == token::str_to_ident("try") &&
            // don't include spans that were found in child modules
            self.codemap.span_to_filename(mac.span) == self.name {
            self.spans.push(mac.span);
        }
    }
}

impl<'a> TryVisitor<'a> {
    fn new(file_name: &'a str, codemap: &'a CodeMap) -> Self {
        TryVisitor {
            codemap: codemap,
            name: file_name,
            spans: vec![],
        }
    }

    fn rewrite(&mut self, source: &str) -> Result<(String, Warnings)> {
        fn is_whitespace(c: char) -> bool {
            match c {
                ' ' | '\n' | '\t' => true,
                _ => false,
            }
        }

        // NOTE we don't have to worry about overlapping spans, because the parser "can't see"
        // nested `try!`s.
        self.spans.sort_by(|a, b| a.lo.cmp(&b.lo));

        let mut output = String::with_capacity(source.len());
        let mut warnings = Warnings::new();
        let mut last = 0;

        // Go from:
        //
        // let x = try! {
        // ^       ^      foo.bar()
        // last    lo     ^       ^ };
        //                start end ^
        //                          hi
        //
        // To:
        //
        // let x = foo.bar()?;
        for span in &self.spans {
            let offset = "try!".len();
            let lo = span.lo.0 as usize;
            let hi = span.hi.0 as usize;
            let (mut start, mut end) = (None, hi - 1);

            if source[lo..hi].contains('\n') {
                warnings.push(self.codemap.lookup_char_pos(span.lo));
            }

            output.push_str(&source[last..lo]);
            last = hi;

            // Look for the start of `try!` argument
            let mut found_delimiter = false;
            for (i, c) in source[lo + offset..hi].char_indices() {
                if is_whitespace(c) {
                    continue;
                } else if !found_delimiter && (c == '(' || c == '{') {
                    found_delimiter = true;
                } else if found_delimiter {
                    start = Some(lo + offset + i);
                    break;
                } else {
                    return Err(Error::Bug(source[lo..hi].to_owned()));
                }
            }

            // Look for the end of the `try!` argument
            for (i, c) in source[lo..hi].char_indices().rev().skip(1) {
                if is_whitespace(c) {
                    end = lo + i;
                    continue;
                } else {
                    break;
                }
            }

            if let Some(start) = start {
                output.push_str(&source[start..end]);
            } else {
                return Err(Error::Bug(source[lo..hi].to_owned()));
            }
            output.push('?');
        }

        output.push_str(&source[last..]);

        Ok((output, warnings))
    }
}
