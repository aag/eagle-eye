use actions::Action;
use notify::Event;

pub struct PrintAction;

impl PrintAction {
    pub fn new() -> PrintAction {
        PrintAction
    }
}

impl Action for PrintAction {
    fn handle_change(&self, event: &Event) {
        match event.path {
            None => println!("No path for event"),
            Some(ref path) => println!("{} changed", path.to_str().unwrap())
        }
    }
}
