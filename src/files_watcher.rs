extern crate inotify;
extern crate libc;

use inotify::INotify;
use inotify::wrapper::Event;
use inotify::wrapper::Watch;
use inotify::ffi::*;

use std::borrow::Borrow;
use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use actions::Action;

pub struct EventPath<'a> {
    pub event: &'a Event,
    pub path: &'a PathBuf
}

impl <'a>EventPath<'a> {
    fn new(path: &'a PathBuf, event: &'a Event) -> EventPath<'a> {
        EventPath {
            event: event,
            path: path
        }
    }
}

pub struct PathActions {
    pub path: PathBuf,
    pub actions: Vec<Box<Action>>
}

pub struct FilesWatcher {
    inotify: INotify,
    watches: HashMap<Watch, PathActions>
}

impl FilesWatcher {
    pub fn new() -> FilesWatcher {
        let inotify = INotify::init().ok().expect(
            "Fatal Error: Could not initialize inotify."
        );

        FilesWatcher { 
            inotify: inotify,
            watches: HashMap::new()
        }
    }

    pub fn add_file(&mut self, path: PathBuf, actions: Vec<Box<Action>>) {
        let watch_id = self.inotify.add_watch(&path, IN_MODIFY | IN_DELETE);

        if watch_id.is_ok() {
            let path_actions = PathActions { path: path, actions: actions };
            self.watches.insert(watch_id.unwrap(), path_actions);
        } else {
            println!("Error adding watch for file: {:?}", watch_id.err());
        }
    }

    pub fn wait_for_events(&mut self) -> io::Result<Vec<EventPath>> {
        let events = try!(self.inotify.wait_for_events());
        
        let mut event_paths: Vec<EventPath> = vec![];
        for event in events.iter() {
            let path_actions = self.watches.get(&event.wd);
            if path_actions.is_some() {
                event_paths.push(EventPath::new(
                    path_actions.unwrap().path.borrow(),
                    event
                ));
            } else {
                println!("Error: no path for watch: {:?}", event.wd);
            }
        }

        Ok(event_paths)
    }

    pub fn wait_and_execute(&mut self) -> io::Result<i32> {
        let events = try!(self.inotify.wait_for_events());
        let mut num_actions = 0;
        
        for event in events.iter() {
            let path_actions = self.watches.get(&event.wd);
            if path_actions.is_some() {
                let event_path = EventPath::new(
                    path_actions.unwrap().path.borrow(),
                    event
                );

                for action in &path_actions.unwrap().actions {
                    action.handle_change(&event_path);
                    num_actions += 1;
                }
            } else {
                println!("Error: no path for watch: {:?}", event.wd);
            }
        }

        Ok(num_actions)
    }

    pub fn close(&mut self) {
        self.inotify.close().ok().expect(
            "Fatal Error: Could not close inotify."
        );
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
    use actions::Action;

    #[test]
    fn watch_a_single_file() {
        let (path, mut file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();
        let actions: Vec<Box<Action + 'static>> = Vec::new();
        fw.add_file(path, actions);

        write_to(&mut file);

        {
            let event_paths = fw.wait_for_events().unwrap();
            assert_eq!(1, event_paths.len());
            for event_path in event_paths.iter() {
                assert_eq!(&filepath, event_path.path);
            }
        }

        for (_, path_actions) in &fw.watches {
            remove_temp_file(&path_actions.path);
        }
        fw.close();
    }


    #[test]
    fn watch_two_files() {
        let (path1, mut file1) = create_temp_file();
        let (path2, mut file2) = create_temp_file();

        let filepath1 = path1.clone();
        let filepath2 = path2.clone();
        let actions1: Vec<Box<Action + 'static>> = Vec::new();
        let actions2: Vec<Box<Action + 'static>> = Vec::new();

        let mut fw = FilesWatcher::new();
        fw.add_file(path1, actions1);
        fw.add_file(path2, actions2);

        write_to(&mut file1);
        write_to(&mut file2);

        {
            let event_paths = fw.wait_for_events().unwrap();
            assert_eq!(2, event_paths.len());
            for event_path in event_paths.iter() {
                assert!(&filepath1 == event_path.path || &filepath2 == event_path.path);
            }
        }

        for (_, path_actions) in &fw.watches {
            remove_temp_file(&path_actions.path);
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

