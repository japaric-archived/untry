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

## Known issues

- This tool won't replace the `try!`s in doc comments or the ones inside macros. You'll have to
    convert those manually.
- This tool will also replace user-defined `try!`s with `?`s. You'll have to manually undo those
    conversions.

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
