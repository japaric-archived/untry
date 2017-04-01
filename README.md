# Status

This tool has been **DEPRECATED** in favor of [rustfmt]. To transform `try!`s
into `?`s using rustfmt, add the following setting to your rustfmt.toml:

``` toml
use_try_shorthand = true
```

[rustfmt]: https://github.com/rust-lang-nursery/rustfmt

-- @japaric, 2017-04-01

---

# `untry`

> Convert `try!`s into `?`s

## Usage

Pass the Rust files you want to modify as arguments. The files will be modified in place:

```
$ cat sinbad.rs
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    run().unwrap();
}

fn run() -> io::Result<()> {
    let wishes = &mut String::new();
    try!(try!(File::open("sesame")).read_to_string(wishes));
    try!(try!(File::create("genie")).write_all(wishes.as_bytes()));
    Ok(())
}

$ untry sinbad.rs
Processing 1 file
sinbad.rs: OK

$ cat sinbad.rs
use std::fs::File;
use std::io::{self, Read, Write};

fn main() {
    run().unwrap();
}

fn run() -> io::Result<()> {
    let wishes = &mut String::new();
    File::open("sesame")?.read_to_string(wishes)?;
    File::create("genie")?.write_all(wishes.as_bytes())?;
    Ok(())
}

```

For larger projects you probably want to use this in conjunction with `find`:

```
$ find -name '*.rs' -type f | xargs untry
```

## Notable examples

Automatic conversion of the [rust-lang/rust] repository.

[rust-lang/rust]: https://github.com/rust-lang/rust/pull/32390

## Known issues

- This tool won't replace the `try!`s in doc comments or the ones inside macros. You'll have to
    convert those manually.
- This tool will also replace user-defined `try!`s with `?`s. You'll have to manually undo those
    conversions.
- Converting multi-line `try!`s like the one shown may break alignment. `untry` doesn't attempt to
    be a formatting tool so it just informs the user about the issue by raising a warning.

```
// From
            try!(collect_tests_from_dir(config,
                                        base,
                                        &file_path,
                                        &relative_file_path,
                                        tests));

// To
            collect_tests_from_dir(config,
                                        base,
                                        &file_path,
                                        &relative_file_path,
                                        tests)?;
```

But you get warnings like these:

```
./src/librustc/middle/check_match.rs: OK
./src/librustc/middle/const_eval.rs: 2 warnings
./src/librustc/middle/const_eval.rs:829:28 warning: multiline try!
./src/librustc/middle/const_eval.rs:875:31 warning: multiline try!
./src/librustc/middle/dataflow.rs: 1 warnings
./src/librustc/middle/dataflow.rs:155:12 warning: multiline try!
./src/librustc/middle/def_id.rs: 2 warnings
./src/librustc/middle/def_id.rs:58:8 warning: multiline try!
./src/librustc/middle/def_id.rs:67:12 warning: multiline try!
./src/librustc/middle/expr_use_visitor.rs: OK
```

The spans get printed to stderr so the user can easily call `rustfmt` on the affected files:

```
$ find -name '*.rs' -type f | xargs untry 2>spans
./src/librustc/middle/check_match.rs: OK
./src/librustc/middle/const_eval.rs: 2 warnings
./src/librustc/middle/dataflow.rs: 1 warnings
./src/librustc/middle/def_id.rs: 2 warnings
./src/librustc/middle/expr_use_visitor.rs: OK

# possibly affected files
$ cat spans | cut -d':' -f1 | sort -u
./src/librustc/middle/const_eval.rs
./src/librustc/middle/dataflow.rs
./src/librustc/middle/def_id.rs

# reformat them
$ cat spans | cut -d':' -f1 | sort -u | xargs rustfmt
```

## License

Licensed under either of

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or
  http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the
work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any
additional terms or conditions.
