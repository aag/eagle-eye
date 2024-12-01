use crate::actions::Action;
use notify::Event;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub struct CommandAction {
    command_line: String,
    quiet: bool,
}

impl CommandAction {
    pub fn new(command_line: String, quiet: bool) -> CommandAction {
        CommandAction {
            command_line,
            quiet,
        }
    }

    pub fn get_command_line(&self, event: &Event) -> String {
        let path = if event.paths.is_empty() {
            ""
        } else {
            event.paths[0].to_str().unwrap_or("")
        };

        self.command_line.replace("{:p}", path)
    }

    pub fn get_command(&self, event: &Event) -> Command {
        let command_line = self.get_command_line(event);
        let mut cmd_pieces = command_line.split(' ');
        let mut command = Command::new(cmd_pieces.next().unwrap());
        for piece in cmd_pieces {
            command.arg(piece);
        }

        command
    }
}

impl Action for CommandAction {
    fn handle_change(&self, event: &Event) -> Result<(), &'static str> {
        let mut command = self.get_command(event);

        if !self.quiet {
            command
                .stdin(Stdio::inherit())
                .stdout(Stdio::inherit())
                .stderr(Stdio::inherit());
        }

        let command_result = command.output();

        match command_result {
            Err(_) => {
                println!("Could not execute command: {:?}", self.command_line);
                Err("Could not execute command")
            }
            Ok(output) => {
                if !self.quiet {
                    println!("{}", String::from_utf8_lossy(&output.stdout));

                    if !output.stderr.is_empty() {
                        let write_result = writeln!(
                            &mut io::stderr(),
                            "{}",
                            String::from_utf8_lossy(&output.stderr)
                        );
                        if let Err(x) = write_result {
                            println!("Error: Unable to write to stderr: {}", x);
                        }
                    }
                }

                Ok(())
            }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate rand;

    use super::*;

    use crate::actions::Action;
    use notify::{event, Event, EventKind};
    use std::path::PathBuf;

    #[test]
    fn constructor() {
        let _ = CommandAction::new("date".to_string(), false);
    }

    #[test]
    fn handle_change_existing_command() {
        let event_kind = EventKind::Modify(event::ModifyKind::Any);
        let path_buf = PathBuf::from("/");

        let event = Event::new(event_kind).add_path(path_buf);

        // Assume the "date" command exists on all platforms
        let command = CommandAction::new("date".to_string(), true);
        let result = command.handle_change(&event);

        // We can't capture the output, so just make sure the function
        // returns Ok.
        assert!(result.is_ok());
    }

    #[test]
    fn handle_change_missing_command() {
        let event_kind = EventKind::Modify(event::ModifyKind::Any);
        let path_buf = PathBuf::from("/");
        let event = Event::new(event_kind).add_path(path_buf);

        // Assume this command does not exist
        let command = CommandAction::new("command_does_not_exist".to_string(), true);
        let result = command.handle_change(&event);

        assert!(result.is_err());
    }
}
