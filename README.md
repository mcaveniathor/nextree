# nextree
[![Rust Report Card](https://rust-reportcard.xuri.me/badge/github.com/mcaveniathor/nextree)](https://rust-reportcard.xuri.me/report/github.com/mcaveniathor/nextree)
[![Crates.io](https://img.shields.io/crates/v/nextree)](https://crates.io/crates/nextree)
[![Crates.io](https://img.shields.io/crates/l/toboggan)](https://lbesson.mit-license.org)

Multithreaded command-line utility similar in function to tree, with logging and CSV output

## Usage
```
USAGE:
    nextree [OPTIONS] --path <PATH>

OPTIONS:
    -h, --help                 Print help information
    -o, --outfile <OUTFILE>    CSV file to output to [default: out.csv]
    -p, --path <PATH>          Root path whose children (files and directories) we want to index
```
