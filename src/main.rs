extern crate docopt;
extern crate notify;
extern crate rustc_serialize;

use actions::Action;
use actions::print::PrintAction;
use actions::command::CommandAction;
use docopt::Docopt;
use files_watcher::FilesWatcher;
use std::path::PathBuf;

pub mod files_watcher;
pub mod actions;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

const USAGE: &'static str = "
Eagle Eye.

Usage:
  eagle [--quiet] [--command=<cmd>] <path>
  eagle (-h | --help)
  eagle --version

Options:
  path                  Path to a file or directory to watch for changes.
  -c --command=<cmd>    A command to execute whenever a change happens.
  -h --help             Show this screen.
  -q --quiet            Do not print file change information.
  --version             Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    arg_path: String,
    flag_command: String,
    flag_help: bool,
    flag_quiet: bool,
    flag_version: bool,
}

#[cfg_attr(test, allow(dead_code))]
fn main() {
    let version_option = Some(VERSION.to_string());
    let args: Args = Docopt::new(USAGE)
                            .and_then(|d| {
                                d.help(true)
                                 .version(version_option)
                                 .decode()
                            })
                            .unwrap_or_else(|e| e.exit());

    let mut actions: Vec<Box<Action + 'static>> = vec![];

    if !args.flag_quiet {
        let print = PrintAction::new();
        actions.push(Box::new(print));
    }

    if !args.flag_command.is_empty() {
        let command = CommandAction::new(args.flag_command);
        actions.push(Box::new(command));
    }

    let mut fw = FilesWatcher::new();

    let path_buf = PathBuf::from(args.arg_path);
    fw.add_file(path_buf, actions);

    loop {
        let result = fw.wait_and_execute();

        match result {
            Ok(i) => println!("Executed {} action(s) successfully.", i),
            Err(_) => println!("Error executing some actions.")
        }
    }

    // Uncomment this as soon as we have a way of leaving the loop
    //fw.close();
}
