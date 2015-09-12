extern crate notify;

use actions::Action;
use actions::print::PrintAction;
use files_watcher::FilesWatcher;
use std::path::PathBuf;

pub mod files_watcher;
pub mod actions;


#[cfg_attr(test, allow(dead_code))]
fn main() {
    let print = PrintAction::new();
    let actions: Vec<Box<Action + 'static>> = vec![Box::new(print)];

    let mut fw = FilesWatcher::new();

    let path_buf = PathBuf::from("/tmp/testfile.txt");
    fw.add_file(path_buf, actions);

    loop {
        let result = fw.wait_and_execute();

        match result {
            Ok(i) => println!("Executed {} action(s) successfully.", i),
            Err(_) => println!("Error executing some actions.")
        }
    }

    // Uncomment this as soon as we have a way of leaving the loop
    //fw.close();
}
