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

#[derive(Debug, RustcDecodable)]
struct Config {
    settings: Option<SettingsConfig>,
    watchers: Option<Vec<WatcherSettings>>,
}

#[derive(Debug, RustcDecodable)]
struct SettingsConfig {
    quiet: Option<bool>,
}

#[derive(Debug, RustcDecodable)]
struct WatcherSettings {
    action_type: String,
    execute: String,
    path: String,
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
    let mut fw = FilesWatcher::new();

    if !args.flag_config.is_empty() {
        let mut config_content = String::new();
        File::open(&args.flag_config).and_then(|mut f| {
            f.read_to_string(&mut config_content)
        }).unwrap();

        let config: Config = toml::decode_str(&config_content).unwrap();
        //println!("{:#?}", config);

        let settings = match config.settings {
            Some(settings) => settings,
            None => SettingsConfig { quiet: Some(false) },
        };

        let quiet_flag = match settings.quiet {
            Some(quiet_flag) => quiet_flag,
            None => false,
        };

        let watchers = match config.watchers {
            Some(watchers) => watchers,
            None => {
                println!("No watchers defined in config file");
                process::exit(1);
            }
        };

        // TODO: convert to for loop to handle multiple watchers
        if watchers.len() > 0 {
            let ref watcher = watchers[0];

            match watcher.action_type.as_ref() {
                "command" => {
                    let execute_string = watcher.execute.to_owned();
                    let command = CommandAction::new(execute_string, quiet_flag);
                    actions.push(Box::new(command));
                },
                _ => (),
            }

            let path_buf = PathBuf::from(watcher.path.to_owned());
            fw.add_file(path_buf, actions);
        }
    } else {
        if !args.flag_quiet {
            let print = PrintAction::new();
            actions.push(Box::new(print));
        }

        if !args.flag_execute.is_empty() {
            let command = CommandAction::new(args.flag_execute, args.flag_quiet);
            actions.push(Box::new(command));
        }

        let path_buf = PathBuf::from(args.flag_path);
        fw.add_file(path_buf, actions);
    }
    
    loop {
        let result = fw.wait_and_execute();

        match result {
            Ok(i) => println!("Executed {} action(s) successfully.", i),
            Err(_) => println!("Error executing some actions."),
        }
    }

    // Uncomment this as soon as we have a way of leaving the loop
    // fw.close();
}
