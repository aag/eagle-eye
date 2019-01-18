use actions::Action;
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
            command_line: command_line,
            quiet: quiet,
        }
    }

    pub fn get_command_line(&self, event: &Event) -> String {
        let path = match event.path {
            None => "",
            Some(ref path) => path.to_str().unwrap_or(""),
        };

        self.command_line.replace("{:p}", path)
    }

    pub fn get_command(&self, event: &Event) -> Command {
        let command_line = self.get_command_line(event);
        let mut cmd_pieces = command_line.split(" ");
        let mut command = Command::new(cmd_pieces.next().unwrap());
        for piece in cmd_pieces {
            command.arg(piece);
        }

        command
    }
}

impl Action for CommandAction {
    fn handle_change(&self, event: &Event) -> Result<(), ()> {
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
                Err(())
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
                        match write_result {
                            Err(x) => println!("Error: Unable to write to stderr: {}", x),
                            Ok(_) => {}
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

    use actions::Action;
    use notify::{Event, Op};
    use std::path::PathBuf;

    #[test]
    fn constructor() {
        let _ = CommandAction::new("date".to_string(), false);
    }

    #[test]
    fn handle_change_existing_command() {
        let o = Op::empty();
        let path_buf = PathBuf::from("/");

        let event = Event {
            path: Some(path_buf),
            op: Ok(o),
        };

        // Assume the "date" command exists on all platforms
        let command = CommandAction::new("date".to_string(), true);
        let result = command.handle_change(&event);

        // We can't capture the output, so just make sure the function
        // returns Ok.
        assert!(result.is_ok());
    }

    #[test]
    fn handle_change_missing_command() {
        let o = Op::empty();
        let path_buf = PathBuf::from("/");

        let event = Event {
            path: Some(path_buf),
            op: Ok(o),
        };

        // Assume this command does not exist
        let command = CommandAction::new("command_does_not_exist".to_string(), true);
        let result = command.handle_change(&event);

        assert!(result.is_err());
    }

}
