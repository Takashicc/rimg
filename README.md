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
$ ./target/release/rimg rename "~/test" -y
2 directories will be executed
There are no JPG files in test directory
Renaming JPG files in xxx directory
|############################################################| 24   /24    Renaming xxx
```

### Compress

Compress each directory directly under the specified directory.

This command will **NOT** look recursively.

<img src="/docs/images/compress-01.png"/>

<img src="/docs/images/compress-02.png"/>

```sh
$ ./target/release/rimg compress "~/test" -v -y
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
