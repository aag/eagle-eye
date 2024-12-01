extern crate libc;
extern crate notify;

use notify::{Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::mpsc::{Receiver, RecvError};

use std::collections::HashMap;
use std::io;
use std::path::PathBuf;

use crate::actions::Action;

pub struct FilesWatcher {
    watcher: Box<RecommendedWatcher>,
    rx: Receiver<Result<Event, notify::Error>>,
    watches: HashMap<PathBuf, Vec<Box<dyn Action>>>,
}

impl Default for FilesWatcher {
    fn default() -> Self {
        Self::new()
    }
}

pub struct EventExecutionResult {
    pub num_actions: usize,
    pub was_file_changed: bool,
}

impl FilesWatcher {
    pub fn new() -> FilesWatcher {
        let (tx, rx) = std::sync::mpsc::channel();
        let watcher = notify::recommended_watcher(tx);

        FilesWatcher {
            watcher: Box::new(watcher.unwrap()),
            rx,
            watches: HashMap::new(),
        }
    }

    // TODO: accept a Vec of paths
    pub fn add_file(&mut self, path: PathBuf, actions: Vec<Box<dyn Action>>) {
        // TODO: Support running actions on all files in a directory tree.
        let result = self.watcher.watch(&path, RecursiveMode::NonRecursive);

        if result.is_ok() {
            println!("Watching file: {:?}", path);
            self.watches.insert(path, actions);
        } else {
            println!("Error adding watch for file: {:?}", result.err());
        }
    }

    pub fn wait_for_events(&mut self) -> Result<Result<Event, notify::Error>, RecvError> {
        self.rx.recv()
    }

    pub fn wait_and_execute(&mut self) -> Result<EventExecutionResult, io::Error> {
        let mut num_actions = 0;
        let event_result = self.rx.recv();

        match event_result {
            Err(_) => Err(io::Error::new(
                io::ErrorKind::Other,
                "Error receiving event",
            )),
            Ok(event) => match event {
                Err(_) => Err(io::Error::new(io::ErrorKind::Other, "Error in file event")),
                Ok(event) => {
                    if !is_file_changed_event(&event) {
                        return Ok(EventExecutionResult {
                            num_actions: 0,
                            was_file_changed: false,
                        });
                    }

                    if event.paths.is_empty() {
                        println!("Warning: event has no paths");
                        return Ok(EventExecutionResult {
                            num_actions: 0,
                            was_file_changed: true,
                        });
                    }

                    for path in event.paths.iter() {
                        let actions = self.watches.get(path);
                        if actions.is_some() {
                            for action in actions.unwrap() {
                                if action.handle_change(&event).is_ok() {
                                    num_actions += 1;
                                }
                            }
                        } else {
                            println!("Error: no actions found for path: {:?}", path.display());
                        }
                    }

                    Ok(EventExecutionResult {
                        num_actions,
                        was_file_changed: true,
                    })
                }
            },
        }
    }
}

/// Returns true if the event is for a file change. Just opening or accessing a file does not count.
fn is_file_changed_event(event: &Event) -> bool {
    event.kind.is_modify() || event.kind.is_create() || event.kind.is_remove()
}

#[cfg(test)]
mod test {
    extern crate rand;

    use super::*;

    use self::rand::distributions::Alphanumeric;
    use self::rand::{thread_rng, Rng};
    use crate::actions::print::PrintAction;
    use crate::actions::Action;
    use notify::{event, EventKind};
    use std::env::temp_dir;
    use std::fs::remove_file;
    use std::fs::File;
    use std::fs::OpenOptions;
    use std::io::Read;
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
            let event = fw.wait_for_events().unwrap().unwrap();
            if event.paths.is_empty() {
                panic!("Error: event has no paths");
            }

            assert_eq!(&filepath, event.paths.first().unwrap());
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
            let event = fw.wait_for_events().unwrap().unwrap();
            if event.paths.is_empty() {
                panic!("Error: event has no paths");
            }
            assert_eq!(&filepath1, event.paths.first().unwrap());

            let event = fw.wait_for_events().unwrap().unwrap();
            if event.paths.is_empty() {
                panic!("Error: event has no paths");
            }
            assert_eq!(&filepath2, event.paths.first().unwrap());
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
            assert_eq!(0, actions_executed.num_actions);
            assert!(actions_executed.was_file_changed);
        }

        remove_temp_file(&filepath);
    }

    #[test]
    fn watch_file_and_execute_no_file_change() {
        let (path, _file) = create_temp_file();
        let filepath = path.clone();

        let mut fw = FilesWatcher::new();

        let print = PrintAction::new();
        let actions: Vec<Box<dyn Action + 'static>> = vec![Box::new(print)];
        fw.add_file(path, actions);

        read_file(&filepath);

        {
            let execution_result = fw.wait_and_execute().unwrap();
            assert_eq!(0, execution_result.num_actions);
            assert!(!execution_result.was_file_changed);
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
            let execution_result = fw.wait_and_execute().unwrap();
            assert_eq!(1, execution_result.num_actions);
            assert!(execution_result.was_file_changed);
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
            let execution_result = fw.wait_and_execute().unwrap();
            assert_eq!(5, execution_result.num_actions);
            assert!(execution_result.was_file_changed);
        }

        remove_temp_file(&filepath);
    }

    #[test]
    fn is_file_changed_event_read_access() {
        assert!(!is_file_changed_event(&Event::new(EventKind::Access(
            event::AccessKind::Read
        ))));
    }

    #[test]
    fn is_file_changed_event_modified() {
        assert!(is_file_changed_event(&Event::new(EventKind::Modify(
            event::ModifyKind::Any
        ))));
    }

    fn create_temp_file() -> (PathBuf, File) {
        let rand_part: String = thread_rng()
            .sample_iter(&Alphanumeric)
            .take(8)
            .map(char::from)
            .collect();

        let filename = "eagleeye-test-".to_string() + &rand_part;
        let path = temp_dir().join(filename);
        // let file = File::create(&path)
        //     .unwrap_or_else(|error| panic!("Failed to create temporary file: {}", error));

        let file = OpenOptions::new()
            .read(true)
            .write(true)
            .create(true)
            .truncate(true)
            .open(&path)
            .unwrap_or_else(|error| panic!("Failed to create temporary file: {}", error));

        (path, file)
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

    fn read_file(path: &Path) {
        let mut data = vec![];
        let mut file = File::open(path).unwrap();
        file.read_to_end(&mut data).unwrap();
    }
}
