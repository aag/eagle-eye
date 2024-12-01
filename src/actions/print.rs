use crate::actions::Action;
use notify::{Event, EventKind};

pub struct PrintAction;

impl PrintAction {
    pub fn new() -> PrintAction {
        PrintAction
    }

    pub fn event_kind_to_str(&self, kind: &EventKind) -> &'static str {
        match *kind {
            EventKind::Access(_) => "File or directory accessed",
            EventKind::Create(_) => "File or directory created",
            EventKind::Modify(_) => "File or directory modified",
            EventKind::Remove(_) => "File or directory removed",
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
        if event.paths.is_empty() {
            println!("No path for event");
            return Err("No path for event");
        }

        for path in event.paths.iter() {
            let message = self.event_kind_to_str(&event.kind);
            println!("{} on path {:?}", message, path);
        }

        Ok(())
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
        // Make sure new() works with no arguments
        let _ = PrintAction::new();
    }

    #[test]
    fn handle_change() {
        let event_kind = EventKind::Modify(event::ModifyKind::Any);
        let path_buf = PathBuf::from("/");
        let event = Event::new(event_kind).add_path(path_buf);

        let print = PrintAction::new();
        let result = print.handle_change(&event);

        // We can't capture the output, so just make sure the function
        // returns Ok.
        assert!(result.is_ok());
    }
}
