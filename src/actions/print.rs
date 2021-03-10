use actions::Action;
use notify::op;
use notify::op::Op;
use notify::Event;

pub struct PrintAction;

impl PrintAction {
    pub fn new() -> PrintAction {
        PrintAction
    }

    pub fn flag_to_str(&self, flag: &Op) -> &'static str {
        match *flag {
            op::CHMOD => "Permissions or timestamps changed",
            op::CREATE => "File or directory created",
            op::REMOVE => "File or directory removed",
            op::RENAME => "File or directory renamed",
            op::WRITE => "File or diretory written to",
            _ => "Unknown change",
        }
    }
}

impl Default for PrintAction {
    fn default() -> Self {
        PrintAction::new()
    }
}

impl Action for PrintAction {
    fn handle_change(&self, event: &Event) -> Result<(), &'static str> {
        match event.path {
            None => {
                println!("No path for event");
                Err("No path for event")
            }
            Some(ref path) => {
                let message = match event.op {
                    Ok(op) => self.flag_to_str(&op),
                    Err(_) => "Unknown change",
                };

                let path_str = path.to_str().unwrap_or("Unknown path");

                println!("{} on path {:?}", message, path_str);

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
        // Make sure new() works with no arguments
        let _ = PrintAction::new();
    }

    #[test]
    fn handle_change() {
        let o = Op::empty();
        let path_buf = PathBuf::from("/");

        let event = Event {
            path: Some(path_buf),
            op: Ok(o),
        };

        let print = PrintAction::new();
        let result = print.handle_change(&event);

        // We can't capture the output, so just make sure the function
        // returns Ok.
        assert!(result.is_ok());
    }
}
