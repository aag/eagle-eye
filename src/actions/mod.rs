pub mod print;
pub mod command;

use notify::Event;

pub trait Action {
    fn handle_change(&self, event: &Event);
}
