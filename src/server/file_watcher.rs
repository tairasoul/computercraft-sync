use std::{collections::HashSet, path::PathBuf, sync::Arc};

use notify::EventHandler;
use tokio::sync::broadcast::Sender;

#[allow(dead_code)]
pub struct FileWatcher {
	sender: Arc<Sender<FileChanged>>,
	known_files: HashSet<PathBuf>
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub enum FileChanged {
	Changed { path: PathBuf },
	Deleted { path: PathBuf }
}

impl FileWatcher {
	pub fn new(sender: Arc<Sender<FileChanged>>, known_files: HashSet<PathBuf>) -> Self {
		Self {
			sender,
			known_files
		}
	}
}

impl EventHandler for FileWatcher {
	fn handle_event(&mut self, event: notify::Result<notify::Event>) {
		let ev = event.unwrap();
		match ev.kind {
			notify::EventKind::Create(_) => {
				for file in ev.paths {
					let metadata = std::fs::metadata(&file).unwrap();
					if metadata.is_file() {
						self.known_files.insert(file.clone());
						let changed = FileChanged::Changed { path: file };
						self.sender.send(changed).unwrap();
					}
				}
			}
			notify::EventKind::Modify(_) => {
				for file in ev.paths {
					if self.known_files.contains(&file) {
						let changed = FileChanged::Changed { path: file };
						self.sender.send(changed).unwrap();
					}
				}
			}
			notify::EventKind::Remove(_) => {
				for file in ev.paths {
					if self.known_files.remove(&file) {
						let changed = FileChanged::Deleted { path: file };
						self.sender.send(changed).unwrap();
					}
				}
			}
			_ => {}
		}
	}
}