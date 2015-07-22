use actions::Action;
use files_watcher::EventPath;

pub struct PrintAction;

impl PrintAction {
    pub fn new() -> PrintAction {
        PrintAction
    }
}

impl Action for PrintAction {
    fn handle_change(&self, event_path: EventPath) {
        println!("{} changed", event_path.path.to_str().unwrap());
    }
}
