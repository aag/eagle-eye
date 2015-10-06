use actions::Action;
use notify::Event;
use std::io::{self, Write};
use std::process::{Command, Stdio};

pub struct CommandAction {
    command_line: String
}

impl CommandAction {
    pub fn new(command_line: String) -> CommandAction {
        CommandAction {
            command_line: command_line
        }
    }
}

impl Action for CommandAction {
    fn handle_change(&self, _event: &Event) {
        let mut cmd_pieces = self.command_line.split(" ");
        let mut command = Command::new(cmd_pieces.next().unwrap());
        for piece in cmd_pieces {
            command.arg(piece);
        }

        let command_result = command
            .stdin(Stdio::inherit())
            .stdout(Stdio::inherit())
            .stderr(Stdio::inherit())
            .output();

        if command_result.is_err() {
            println!("Could not execute command: {:?}", self.command_line);
            return;
        }
        
        let output = command_result.unwrap();
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
}
