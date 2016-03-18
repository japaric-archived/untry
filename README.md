# `untry`

> Blindly convert `try!()` into `?`s

## Usage

```
$ untry $FILE1 $FILE2 ...
$FILE1 ... OK
$FILE2 ... OK
(...)
```

## Caveats

This tool is very simple so it doesn't cover all cases, in particular:

- It doesn't convert `try! {}`s (with braces)
- or multi-line `try!()`s
- it wrongly converts other macros like `fs_try!()`
- and has problem when parentheses appear in string literals: `try!(word(&mut self.s, ")"))` gets
    converted to `word(&mut self.s, ")"?);` instead of `word(&mut self.s, ")")?`

However, it does handle nested `try!()`s and appears to correctly convert like 70%+ of the usages in
the rust-lang/cargo repository.

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
