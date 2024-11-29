extern crate clap;
extern crate notify;
#[macro_use]
extern crate serde_derive;
extern crate toml;

pub mod actions;
pub mod config;
pub mod files_watcher;

use std::path::PathBuf;
use std::process;

use crate::actions::command::CommandAction;
use crate::actions::print::PrintAction;
use crate::actions::Action;
use crate::config::SettingsConfig;
use crate::files_watcher::FilesWatcher;
use clap::Parser;

#[derive(Parser)]
#[command(version, about, long_about = None)]
struct Cli {
    /// Path to a TOML config file. This option is mutually exclusive to the
    /// -p and -e options.
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// A command to execute whenever a change happens. If the command contains
    /// one or more instances of {:p}, they will be replaced by the path to the
    /// changed file or folder. Requires also specifying the -p option.
    #[arg(short, long)]
    execute: Option<String>,

    /// Path to a file or directory to watch for changes.  Requires also specifying
    /// the -e option.
    #[arg(short, long, value_name = "PATH")]
    path: Option<PathBuf>,

    /// Do not print file change information.
    #[arg(short, long, default_value = "false")]
    quiet: bool,
}

// #[cfg_attr(test)
fn main() {
    let cli = Cli::parse();

    let mut actions: Vec<Box<dyn Action + 'static>> = vec![];
    let mut fw = FilesWatcher::new();

    if let Some(config_path) = cli.config.as_deref() {
        let config = match config::parse_file(config_path) {
            Some(config) => config,
            None => {
                println!("Error parsing config file. Exiting.");
                process::exit(1);
            }
        };

        let settings = match config.settings {
            Some(settings) => settings,
            None => SettingsConfig { quiet: Some(false) },
        };

        let quiet_flag = settings.quiet.unwrap_or(false);

        let watchers = match config.watchers {
            Some(watchers) => watchers,
            None => {
                println!("No watchers defined in config file");
                process::exit(1);
            }
        };

        // TODO: convert to for loop to handle multiple watchers
        if !watchers.is_empty() {
            let watcher = &watchers[0];

            if let "command" = watcher.action_type.as_ref() {
                let execute_string = watcher.execute.to_owned();
                let command = CommandAction::new(execute_string, quiet_flag);
                actions.push(Box::new(command));
            }

            let path_buf = PathBuf::from(watcher.path.to_owned());
            fw.add_file(path_buf, actions);
        }
    } else {
        let flag_quiet = cli.quiet;
        if !flag_quiet {
            let print = PrintAction::new();
            actions.push(Box::new(print));
        }

        if let Some(execute) = cli.execute.as_deref() {
            let command = CommandAction::new(execute.to_string(), flag_quiet);
            actions.push(Box::new(command));
        }

        if let Some(path) = cli.path.as_deref() {
            fw.add_file(path.to_path_buf(), actions);
        }
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
