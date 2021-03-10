### Eagle Eye [![Build Status](https://github.com/aag/eagle-eye/actions/workflows/rust.yml/badge.svg)](https://github.com/aag/eagle-eye/actions) [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Eagle Eye is a file watcher that waits for changes to files or directories,
then executes actions when they change. It is written in Rust and is
cross-platform. It works under Linux, Mac OS X, and MS Windows.

## Usage

After cloning the repository and building the software with `cargo build`, you
can run the `eagle` binary to watch a file or directory. For example, this
command will print out the date and time every time file.txt changes:

```
$ ./eagle --execute "date" --path=/tmp/file.txt
```

If you include `{:p}` in the command string, it will be replaced with the
path to the changed file or folder. For example, this command will call
`ls -l /tmp/file.txt` every time file.txt changes:

```
$ ./eagle --execute "ls -l {:p}" --path=/tmp/file.txt
```

You can get more information on usage by running `eagle -h`.

## Development

All of the lints and tests that are run during CI can be run locally with these commands:

```
$ cargo fmt --all
$ cargo check
$ cargo clippy --all-targets --all-features -- -D warnings
$ cargo test
```

## License

Eagle Eye is licensed under the
[MIT License](http://opensource.org/licenses/MIT).  See the LICENSE file in
this directory for the full license text.

