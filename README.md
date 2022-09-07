# nextree
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/mcaveniathor/nextree)](https://rust-reportcard.xuri.me/report/github.com/mcaveniathor/nextree)
[![Crates.io](https://img.shields.io/crates/v/nextree)](https://crates.io/crates/nextree)
[![Rust](https://github.com/mcaveniathor/nextree/actions/workflows/rust.yml/badge.svg)](https://github.com/mcaveniathor/nextree/actions/workflows/rust.yml)
[![Crates.io](https://img.shields.io/crates/l/toboggan)](https://lbesson.mit-license.org)

Nextree is a multithreaded (leveraging parallel iterators and threadpool from the [rayon](https://github.com/rayon-rs/rayon) crate) command-line utility similar in function to tree, with logging and CSV output

## Usage
Set the log level using the RUST_LOG environment variable, either by exporting it or by prepending it to the command.
RUST_LOG=OFF is recommended for maximum performance, or RUST_LOG=info for the most informative output.

```
USAGE:
    nextree [OPTIONS] --path <PATH>

OPTIONS:
    -h, --help                 Print help information
    -o, --outfile <OUTFILE>    CSV file to output to [default: out.csv]
    -p, --path <PATH>          Root path whose children (files and directories) we want to index
```

### Example
`RUST_LOG=OFF nextree -p /home -o ~/Documents/nextree_out.csv`
