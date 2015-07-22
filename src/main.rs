extern crate inotify;

use actions::print::PrintAction;
use files_watcher::FilesWatcher;
use std::path::PathBuf;

pub mod files_watcher;
pub mod actions;


#[cfg_attr(test, allow(dead_code))]
fn main() {
    let print = PrintAction::new();
    let actions = vec![Box::new(print)];

    let mut fw = FilesWatcher::new();

    let path_buf = PathBuf::from("/tmp/testfile.txt");
    fw.add_file(path_buf, actions);

    fw.close();
}
