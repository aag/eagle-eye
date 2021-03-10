extern crate libc;
extern crate notify;

use notify::{Error, Event, RecommendedWatcher, Watcher};
use std::sync::mpsc::{channel, Receiver, RecvError};

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use actions::Action;

pub struct FilesWatcher {
    watcher: Box<RecommendedWatcher>,
    rx: Receiver<Event>,
    watches: HashMap<PathBuf, Vec<Box<dyn Action>>>,
}

impl FilesWatcher {
    pub fn new() -> FilesWatcher {
        let (tx, rx) = channel();
        let watcher: Result<RecommendedWatcher, Error> = Watcher::new(tx);

        FilesWatcher {
            watcher: Box::new(watcher.unwrap()),
            rx: rx,
            watches: HashMap::new(),
        }
    }

    // TODO: accept a Vec of paths
    pub fn add_file(&mut self, path: PathBuf, actions: Vec<Box<dyn Action>>) {
        let result = self.watcher.watch(&path);

        if result.is_ok() {
            println!("Watching file: {:?}", path);
            self.watches.insert(path, actions);
        } else {
            println!("Error adding watch for file: {:?}", result.err());
        }
    }

    pub fn wait_for_events(&mut self) -> Result<Event, RecvError> {
        self.rx.recv()
    }

    pub fn wait_and_execute(&mut self) -> Result<i32, io::Error> {
        let mut num_actions = 0;
        let event_result = self.rx.recv();

        match event_result {
            Err(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Error receiving event",
            )),
            Ok(event) => {
                match event.path {
                    None => println!("Warning: event has no path"),
                    Some(ref path) => {
                        let actions = self.watches.get(path);
                        if actions.is_some() {
                            for action in actions.unwrap() {
                                match action.handle_change(&event) {
                                    Ok(_) => num_actions += 1,
                                    Err(_) => {}
                                }
                            }
                        } else {
                            println!("Error: no actions found for path: {:?}", path.display());
                        }
                    }
                }

                Ok(num_actions)
            }
        }
    }
}

#[cfg(test)]
mod test {
    extern crate rand;

    use super::*;

    use self::rand::distributions::Alphanumeric;
    use self::rand::{thread_rng, Rng};
    use actions::print::PrintAction;
    use actions::Action;
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::io::Write;
    use std::path::Path;
    use std::path::PathBuf;

    #[test]
    fn watch_a_single_file() {
        let (path, mut file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();
        let actions: Vec<Box<dyn Action + 'static>> = Vec::new();
        fw.add_file(path, actions);

        write_to(&mut file);

        {
            let event = fw.wait_for_events().unwrap();
            match event.path {
                None => assert!(false, "Error: event has no path"),
                Some(event_path) => {
                    assert_eq!(filepath, event_path);
                }
            }
        }

        remove_temp_file(&filepath);
    }

    #[test]
    fn watch_two_files() {
        let (path1, mut file1) = create_temp_file();
        let (path2, mut file2) = create_temp_file();

        let filepath1 = path1.clone();
        let filepath2 = path2.clone();
        let actions1: Vec<Box<dyn Action + 'static>> = Vec::new();
        let actions2: Vec<Box<dyn Action + 'static>> = Vec::new();

        let mut fw = FilesWatcher::new();
        fw.add_file(path1, actions1);
        fw.add_file(path2, actions2);

        write_to(&mut file1);
        write_to(&mut file2);

        {
            let event = fw.wait_for_events().unwrap();
            match event.path {
                None => assert!(false, "Error: event has no path"),
                Some(event_path) => {
                    assert_eq!(filepath1, event_path);
                }
            }

            let event = fw.wait_for_events().unwrap();
            match event.path {
                None => assert!(false, "Error: event has no path"),
                Some(event_path) => {
                    assert_eq!(filepath2, event_path);
                }
            }
        }

        remove_temp_file(&filepath1);
        remove_temp_file(&filepath2);
    }

    #[test]
    fn watch_file_and_execute_no_actions() {
        let (path, mut file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();
        let actions: Vec<Box<dyn Action + 'static>> = Vec::new();
        fw.add_file(path, actions);

        write_to(&mut file);

        {
            let actions_executed = fw.wait_and_execute().unwrap();
            assert_eq!(0, actions_executed);
        }

        remove_temp_file(&filepath);
    }

    #[test]
    fn watch_file_and_execute_one_print_action() {
        let (path, mut file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();

        let print = PrintAction::new();
        let actions: Vec<Box<dyn Action + 'static>> = vec![Box::new(print)];
        fw.add_file(path, actions);

        write_to(&mut file);

        {
            let actions_executed = fw.wait_and_execute().unwrap();
            assert_eq!(1, actions_executed);
        }

        remove_temp_file(&filepath);
    }

    #[test]
    fn watch_file_and_execute_five_print_actions() {
        let (path, mut file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();

        let print1 = PrintAction::new();
        let print2 = PrintAction::new();
        let print3 = PrintAction::new();
        let print4 = PrintAction::new();
        let print5 = PrintAction::new();

        let actions: Vec<Box<dyn Action + 'static>> = vec![
            Box::new(print1),
            Box::new(print2),
            Box::new(print3),
            Box::new(print4),
            Box::new(print5),
        ];
        fw.add_file(path, actions);

        write_to(&mut file);

        {
            let actions_executed = fw.wait_and_execute().unwrap();
            assert_eq!(5, actions_executed);
        }

        remove_temp_file(&filepath);
    }

    fn create_temp_file() -> (PathBuf, File) {
        let rand_part: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let filename = "eagleeye-test-".to_string() + &rand_part;
        let path = temp_dir().join(filename);
        let file = File::create(&path)
            .unwrap_or_else(|error| panic!("Failed to create temporary file: {}", error));
        let path_buf = PathBuf::from(path);

        (path_buf, file)
    }

    fn remove_temp_file(path: &Path) {
        remove_file(path)
            .unwrap_or_else(|error| panic!("Failed to create temporary file: {}", error));
    }

    fn write_to(file: &mut File) {
        file.write_all(b"This should trigger an inotify event.")
            .unwrap_or_else(|error| panic!("Failed to write to file: {}", error));

        file.flush().unwrap();
    }
}
