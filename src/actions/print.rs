use actions::Action;
use notify::Event;
use notify::op;
use notify::op::Op;

pub struct PrintAction;

impl PrintAction {
    pub fn new() -> PrintAction {
        PrintAction
    }

    pub fn flag_to_str(&self, flag: &Op) -> &'static str {
        match flag {
            &op::CHMOD => "Permissions or timestamps changed",
            &op::CREATE => "File or directory created",
            &op::REMOVE => "File or directory removed",
            &op::RENAME => "File or directory renamed",
            &op::WRITE => "File or diretory written to",
            _ => "Unknown change"
        }
    }
}

impl Action for PrintAction {
    fn handle_change(&self, event: &Event) -> Result<(), ()> {
        match event.path {
            None => {
                println!("No path for event");
                Err(())
            }
            Some(ref path) =>  {
                let message = match event.op {
                    Ok(op) => self.flag_to_str(&op),
                    Err(_) => "Unknown change"
                };

                let path_str = path.to_str().unwrap_or("Unknown path");

                println!("{} on path {:?}", message, path_str);

                Ok(())
            }
        }
    }
}
