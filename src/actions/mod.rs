pub mod print;

use files_watcher::EventPath;

pub trait Action {
    fn handle_change(&self, event_path: EventPath);
}
