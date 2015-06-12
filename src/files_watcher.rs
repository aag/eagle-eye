extern crate inotify;
extern crate libc;

use inotify::INotify;
use inotify::wrapper::Event;
use inotify::wrapper::Watch;
use inotify::ffi::*;

use std::path::Path;


pub struct FilesWatcher {
    inotify: INotify
}

impl FilesWatcher {
    pub fn new() -> FilesWatcher {
        let inotify = INotify::init().unwrap();

        FilesWatcher { inotify: inotify }
    }

    pub fn add_file(&mut self, path: &Path) -> Watch {
        self.inotify.add_watch(path, IN_MODIFY | IN_DELETE).unwrap()
    }

    pub fn wait_for_events(&mut self) -> &[Event] {
        self.inotify.wait_for_events().unwrap()
    }

    pub fn close(&mut self) {
        self.inotify.close().unwrap();
    }
}

#[cfg(test)]
mod test {
    extern crate rand;

    use super::*;

    use self::rand::{thread_rng, Rng};
    use std::env::temp_dir;
    use std::fs::File;
    use std::fs::remove_file;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn watch_a_single_file() {
        let (path, mut file) = create_temp_file();

        let mut fw = FilesWatcher::new();
        let watch = fw.add_file(&path);

        write_to(&mut file);

        {
            let events = fw.wait_for_events();
            assert_eq!(1, events.len());
            for event in events.iter() {
                assert_eq!(watch, event.wd);
            }
        }

        fw.close();
        remove_temp_file(&path);
    }


    #[test]
    fn watch_two_files() {
        let (path1, mut file1) = create_temp_file();
        let (path2, mut file2) = create_temp_file();

        let mut fw = FilesWatcher::new();
        let watch1 = fw.add_file(&path1);
        let watch2 = fw.add_file(&path2);

        write_to(&mut file1);
        write_to(&mut file2);

        {
            let events = fw.wait_for_events();
            assert_eq!(2, events.len());
            for event in events.iter() {
                assert!(event.wd == watch1 || event.wd == watch2);
            }
        }

        fw.close();
        remove_temp_file(&path1);
        remove_temp_file(&path2);
    }

    fn create_temp_file() -> (PathBuf, File) {
        let rand_part: String = thread_rng()
            .gen_ascii_chars()
            .take(8)
            .collect();

        let filename = "eagle-test-".to_string() + &rand_part;
        let path = temp_dir().join(filename);
        let file = File::create(&path).unwrap_or_else(|error|
            panic!("Failed to create temporary file: {}", error)
        );
        let path_buf = PathBuf::from(path);

        (path_buf, file)
    }

    fn remove_temp_file(path: &Path) {
        remove_file(path).unwrap_or_else(|error|
            panic!("Failed to create temporary file: {}", error)
        );
    }

    fn write_to(file: &mut File) {
        file
            .write(b"This should trigger an inotify event.")
            .unwrap_or_else(|error|
                panic!("Failed to write to file: {}", error)
            );
    }

}

