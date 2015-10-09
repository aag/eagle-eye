### Eagle Eye [![License](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)

Eagle Eye is a file watcher that waits for changes to files or directories,
then executes actions when they change. It is written in Rust and is
cross-platform. It works under Linux, Mac OS X, and MS Windows.

## Usage

After cloning the repository and building the software with `cargo build`, you
can run the `eagle` binary to watch a file or directory. For example, this
command will print out the date and time every time file.txt changes:

```
$ ./eagle --command "date" /tmp/file.txt
```

If you include `{:p}` in the command string, it will be replaced with the
path to the changed file or folder. For example, this command will call
`ls -l /tmp/file.txt` every time file.txt changes:

```
$ ./eagle --command "ls -l {:p}" /tmp/file.txt
```

You can get more information on usage by running `eagle -h`.

## License

Eagle Eye is licensed under the
[MIT License](http://opensource.org/licenses/MIT).  See the LICENSE file in
this directory for the full license text.

