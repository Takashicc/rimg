# rimg

## Build

After installing Rust, run the below commands to build.

```sh
$ git clone https://github.com/Takashicc/rimg
$ cd rimg
$ cargo build --release
$ ./target/release/rimg --version
rimg 0.1.0
```

## Commands

### Rename

Rename files in each directory to sequential number.

```sh
$ ./target/release/rimg rename "~/test" -y
2 directories will be executed
There are no JPG files in test directory
Renaming JPG files in xxx directory
|############################################################| 24   /24    Renaming xxx
```
