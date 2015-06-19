extern crate inotify;
extern crate libc;

use inotify::INotify;
use inotify::wrapper::Event;
use inotify::wrapper::Watch;
use inotify::ffi::*;

use std::collections::HashMap;
use std::path::PathBuf;

pub struct FilesWatcher {
    inotify: INotify,
    watches: HashMap<Watch, PathBuf>
}

impl FilesWatcher {
    pub fn new() -> FilesWatcher {
        let inotify = INotify::init().unwrap();

        FilesWatcher { 
            inotify: inotify,
            watches: HashMap::new()
        }
    }

    pub fn add_file(&mut self, path: PathBuf) {
        let watch_id = self.inotify.add_watch(&path, IN_MODIFY | IN_DELETE).unwrap();
        self.watches.insert(watch_id, path);
    }

    pub fn wait_for_events(&mut self) -> Vec<EventPath> {
        let events = self.inotify.wait_for_events().unwrap();
        
        let mut event_paths: Vec<EventPath> = vec![];
        for event in events.iter() {
            let path: PathBuf = self.watches.get(&event.wd).unwrap().clone();
            event_paths.push(EventPath::new(path, event.clone()));
        }

        event_paths
    }

    pub fn close(&mut self) {
        self.inotify.close().unwrap();
    }
}

pub struct EventPath {
    event: Event,
    path: PathBuf
}

impl EventPath {
    fn new(path: PathBuf, event: Event) -> EventPath {
        EventPath {
            event: event,
            path: path
        }
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
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();
        fw.add_file(path);

        write_to(&mut file);

        {
            let event_paths = fw.wait_for_events();
            assert_eq!(1, event_paths.len());
            for event_path in event_paths.iter() {
                assert_eq!(filepath, event_path.path);
            }
        }

        for path in fw.watches.values() {
            remove_temp_file(path);
        }
        fw.close();
    }


    #[test]
    fn watch_two_files() {
        let (path1, mut file1) = create_temp_file();
        let (path2, mut file2) = create_temp_file();

        let filepath1 = path1.clone();
        let filepath2 = path2.clone();

        let mut fw = FilesWatcher::new();
        fw.add_file(path1);
        fw.add_file(path2);

        write_to(&mut file1);
        write_to(&mut file2);

        {
            let event_paths = fw.wait_for_events();
            assert_eq!(2, event_paths.len());
            for event_path in event_paths.iter() {
                assert!(filepath1 == event_path.path || filepath2 == event_path.path);
            }
        }

        for path in fw.watches.values() {
            remove_temp_file(path);
        }
        fw.close();
    }

    fn create_temp_file() -> (PathBuf, File) {
        let rand_part: String = thread_rng()
            .gen_ascii_chars()
            .take(8)
            .collect();

        let filename = "eagleeye-test-".to_string() + &rand_part;
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

