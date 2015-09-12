pub mod print;

use notify::Event;

pub trait Action {
    fn handle_change(&self, event: &Event);
}
