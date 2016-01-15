extern crate docopt;
extern crate notify;
extern crate rustc_serialize;
extern crate toml;

use std::fs::File;
use std::io::Read;
use std::path::PathBuf;
use std::process;

use actions::Action;
use actions::print::PrintAction;
use actions::command::CommandAction;
use docopt::Docopt;
use files_watcher::FilesWatcher;
use toml::Value;

pub mod files_watcher;
pub mod actions;

const VERSION: &'static str = env!("CARGO_PKG_VERSION");

#[cfg_attr(rustfmt, rustfmt_skip)]
const USAGE: &'static str = "
Eagle Eye.

Usage:
  eagle [--quiet] [--config=<cfg_path>] [--execute=<cmd>] [--path=<path>]
  eagle (-h | --help)
  eagle --version

Options:
  -c --config=<cfg_path>  Path to a TOML config file. This option is mutually
                          exclusive to the -p and -e options.
  -e --execute=<cmd>      A command to execute whenever a change happens.
                          If the command contains one or more instances of
                          {:p}, they will be replaced by the path to the
                          changed file or folder. Requires also specifying
                          the -p option.
  -h --help               Show this screen.
  -p --path=<path>        Path to a file or directory to watch for changes.
                          Requires also specifying the -e option.
  -q --quiet              Do not print file change information.
  --version               Show version.
";

#[derive(Debug, RustcDecodable)]
struct Args {
    flag_path: String,
    flag_config: String,
    flag_execute: String,
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

    if !args.flag_config.is_empty() &&
        (!args.flag_execute.is_empty() || !args.flag_path.is_empty()) {
        println!("Error: you can specify a config file or a command to");
        println!("       execute and a path, but not both.");
        process::exit(1);
    }

    if args.flag_execute.is_empty() && !args.flag_path.is_empty() {
        println!("Error: you have specified a path to watch, but no command");
        println!("       to execute on changes. Use the -e option to specify");
        println!("       a command.");
        process::exit(1);
    }

    if !args.flag_execute.is_empty() && args.flag_path.is_empty() {
        println!("Error: you have specified a command to execute, but no");
        println!("       path to watch. Use the -p option to specify a path.");
        process::exit(1);
    }

    if args.flag_config.is_empty() && args.flag_execute.is_empty() {
        println!("Error: you must specify either a config file or a command");
        println!("       to execute and a path.");
        process::exit(1);
    }
    
    if !args.flag_quiet {
        let print = PrintAction::new();
        actions.push(Box::new(print));
    }

    if !args.flag_execute.is_empty() {
        let command = CommandAction::new(args.flag_execute, args.flag_quiet);
        actions.push(Box::new(command));
    }

    let mut fw = FilesWatcher::new();

    let path_buf = PathBuf::from(args.flag_path);
    fw.add_file(path_buf, actions);

    loop {
        let result = fw.wait_and_execute();

        match result {
            Ok(i) => println!("Executed {} action(s) successfully.", i),
            Err(_) => println!("Error executing some actions."),
        }
    }

     //Uncomment this as soon as we have a way of leaving the loop
     //fw.close();
}
