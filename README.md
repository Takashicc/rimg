# rimg

[![Build & Test](https://github.com/Takashicc/rimg/actions/workflows/ci.yml/badge.svg)](https://github.com/Takashicc/rimg/actions/workflows/ci.yml)

## Build

After installing Rust, run the below commands to build.

```sh
$ git clone https://github.com/Takashicc/rimg
$ cd rimg
$ cargo build --release
$ ./target/release/rimg --version
rimg 0.2.0
```

## Commands

### Rename

Rename files in each directory to sequential number.

This command will look recursively.

<img src="/docs/images/rename-01.png"/>

<img src="/docs/images/rename-02.png"/>

```sh
$ rimg rename -h
Rename files in each directory to sequential number

Usage: rimg rename [OPTIONS] <INPUT_DIR>

Arguments:
  <INPUT_DIR>  Target directory

Options:
  -d, --digit <DIGIT>            Number of digits for renaming [default: 4]
  -e, --extensions <EXTENSIONS>  Target file extension [default: jpg jpeg]
  -i, --initial <INITIAL>        Initial number [default: 1]
  -s, --step <STEP>              Number of steps to count each files [default: 1]
  -y, --yes                      Execute immediately or not
  -h, --help                     Print help information

$ rimg rename "~/test" -y
2 directories will be executed
There are no JPG files in test directory
Renaming JPG files in xxx directory
|############################################################| 24   /24    Renaming xxx
```

### Compress

Compress each directory directly under the specified directory.

This command will **NOT** look recursively.

Currently supports `rar` and `zip`.

<img src="/docs/images/compress-01.png"/>

<img src="/docs/images/compress-02.png"/>

```sh
$ rimg compress -h
Compress files in each directory

Usage: rimg compress [OPTIONS] <INPUT_DIR>

Arguments:
  <INPUT_DIR>  Input directory

Options:
  -o, --output-dir <OUTPUT_DIR>    Output directory
  -f, --format-type <FORMAT_TYPE>  Compress file format type [default: rar]
  -v, --validate                   Check the compressed file is not corrupted after the file was created
      --validate-only              Just check the compressed file is not corrupted
  -y, --yes                        Execute immediately or not
  -h, --help                       Print help information

$ rimg compress "~/test" -v -y
2 directories will be executed
  [00:00:03] [##############################]  (0.0s) Compressed xxx.rar!
Compression Result
# ----------------- #
| Total    ->     2 |
| Created  ->     2 |
| Error    ->     0 |
# ----------------- #
  [00:00:01] [##############################]  (0.0s) OK
Validation Result
# ----------------- #
| Total    ->     2 |
| Valid    ->     2 |
| Invalid  ->     0 |
# ----------------- #
```
