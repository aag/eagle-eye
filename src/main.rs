extern crate inotify;

use files_watcher::FilesWatcher;

pub mod files_watcher;


#[cfg_attr(test, allow(dead_code))]
fn main() {
    let mut fw = FilesWatcher::new();
    fw.close();
}
