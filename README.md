# Static FileZ

> **Build compressed archives for static files and serve them over HTTP**

[![Build Status](https://travis-ci.com/killercup/static-filez.svg?branch=master)](https://travis-ci.com/killercup/static-filez)

## What and Why

Say you want to store a huge number of very small files
that you are only viewing in a browser.
For example: You are using `rustdoc` to render the documentation of a library.
Without much work you'll end up with about 100k files that are about 10kB each.
As it turns out, this number of small files is very annoying for any kind of file system performance:
Best case: making copies/backups is slow.
Worst case: You're using an anti virus software and it takes ages.

Except for convenience when implementing software,
and people being used to having folders of files they can look into,
there is little reason to store these files individually.
Indeed, it will save much space and time to store files like these in compressed form in one continuous archive.
All that is needed to make this work is
some well-designed and discoverable software.

_static-filez_ is a prototype for that piece of software.

## Installation

For now, `cargo install --git https://github.com/killercup/static-filez`.

## Usage

1. Build an archive (and index) from a directory: `static-filez build target/doc/ ./docs.archive`
2. Start a HTTP server that serves the files in the archive: `static-filez serve -p 3000 docs.archive`
3. Open a browser and see your files: `http://127.0.0.1:3000/regex/`
   (`regex` is an example for a great documentation page you should read)

## Architecture

Currently, _static-filez_ will generate two files:
An `.index` file, and an `.archive` file.

The index is a specialized data structure
that maps paths to their content in the archive.

The archive file contains the (compressed) content of your files.
The server is implemented in a way that it can serve the compressed content directly,
with no need to ever look at the (potentially much larger) original decompressed data.
(This works by using the HTTP Content-Encoding header, if you are curios.)

You can read more about the structure of the files in
[this issue](https://github.com/killercup/static-filez/issues/1),
or, of course, the source.

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally
submitted for inclusion in the work by you, as defined in the Apache-2.0
license, shall be dual licensed as above, without any additional terms or
conditions.
